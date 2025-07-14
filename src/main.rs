use std::{fs::File, path::Path, sync::Arc, time::Duration};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::collections::VecDeque;
use std::thread;
use std::fmt;

use clap::Parser;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample, SampleFormat, SizedSample};

use iced::{
    widget::{button, column, container, row, text, progress_bar, slider, Space},
    Application, Command, Element, Length, Settings, Theme,
    executor,
};

use symphonia::core::audio::{AudioBufferRef, Signal};
use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::default;

// 常量定义
const DEFAULT_BUFFER_MULTIPLIER: usize = 2;
const BUFFER_CAPACITY_THRESHOLD: usize = 1000;
const BUFFER_WRITE_DELAY: u64 = 1;

// 自定义错误类型
#[derive(Debug)]
enum PlayerError {
    FileNotFound(String),
    UnsupportedFormat(String),
    AudioDeviceError(String),
    DecodingError(String),
    PlaybackError(String),
}

impl fmt::Display for PlayerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlayerError::FileNotFound(msg) => write!(f, "File not found: {}", msg),
            PlayerError::UnsupportedFormat(msg) => write!(f, "Unsupported format: {}", msg),
            PlayerError::AudioDeviceError(msg) => write!(f, "Audio device error: {}", msg),
            PlayerError::DecodingError(msg) => write!(f, "Decoding error: {}", msg),
            PlayerError::PlaybackError(msg) => write!(f, "Playback error: {}", msg),
        }
    }
}

impl std::error::Error for PlayerError {}

// 播放控制命令
#[derive(Debug, Clone)]
enum PlaybackCommand {
    Pause,
    Resume,
    Stop,
    SetVolume(f32),
}

// 播放状态
#[derive(Debug, Clone)]
struct PlaybackState {
    is_playing: bool,
    is_paused: bool,
    current_time: f64,
    total_duration: f64,
    volume: f32,
}

impl Default for PlaybackState {
    fn default() -> Self {
        Self {
            is_playing: false,
            is_paused: false,
            current_time: 0.0,
            total_duration: 0.0,
            volume: 1.0,
        }
    }
}

// 音频信息结构体
#[derive(Debug, Clone)]
struct AudioInfo {
    channels: usize,
    sample_rate: u32,
    duration: Option<f64>,
    bits_per_sample: Option<u32>,
}

impl AudioInfo {
    fn new(track: &symphonia::core::formats::Track) -> Self {
        let channels = track.codec_params.channels.map(|c| c.count()).unwrap_or(2);
        let sample_rate = track.codec_params.sample_rate.unwrap_or(44100);
        let duration = calculate_audio_duration(track, sample_rate);
        let bits_per_sample = track.codec_params.bits_per_sample;

        Self {
            channels,
            sample_rate,
            duration,
            bits_per_sample,
        }
    }
}

// 音频文件结构体
struct AudioFile {
    probed: symphonia::core::probe::ProbeResult,
    track: symphonia::core::formats::Track,
    track_id: u32,
    info: AudioInfo,
}

impl AudioFile {
    fn open(file_path: &str) -> Result<Self, PlayerError> {
        if !Path::new(file_path).exists() {
            return Err(PlayerError::FileNotFound(file_path.to_string()));
        }

        let file = File::open(file_path)
            .map_err(|e| PlayerError::FileNotFound(format!("{}: {}", file_path, e)))?;
        let mss = MediaSourceStream::new(Box::new(file), Default::default());
        
        let hint = create_hint(file_path);
        
        let probed = default::get_probe()
            .format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default())
            .map_err(|e| PlayerError::UnsupportedFormat(format!("{}: {}", file_path, e)))?;
        
        let track = probed
            .format
            .tracks()
            .iter()
            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
            .ok_or_else(|| PlayerError::UnsupportedFormat("No valid audio track found".to_string()))?
            .clone();
        
        let track_id = track.id;
        let info = AudioInfo::new(&track);
        
        Ok(Self {
            probed,
            track,
            track_id,
            info,
        })
    }
}

// 音频缓冲区类型
type AudioBuffer = Arc<Mutex<VecDeque<f32>>>;

// iced应用程序消息
#[derive(Debug, Clone)]
enum Message {
    PlayPause,
    Stop,
    VolumeChanged(f32),
    OpenFile,
    FileSelected(Option<String>),
    Tick,
    PlaybackStateUpdate(PlaybackState),
    AudioSessionStarted(tokio::sync::mpsc::UnboundedSender<PlaybackCommand>),
}

// iced应用程序状态
struct PlayerApp {
    playback_state: PlaybackState,
    audio_info: Option<AudioInfo>,
    file_path: String,
    is_playing: bool,
    command_sender: Option<tokio::sync::mpsc::UnboundedSender<PlaybackCommand>>,
    audio_handle: Option<tokio::task::JoinHandle<()>>,
}

impl Application for PlayerApp {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = String;

    fn new(file_path: String) -> (Self, Command<Message>) {
        let mut app = Self {
            playback_state: PlaybackState::default(),
            audio_info: None,
            file_path: file_path.clone(),
            is_playing: false,
            command_sender: None,
            audio_handle: None,
        };
        
        // 尝试加载音频文件信息
        if !file_path.is_empty() {
            if let Ok(audio_file) = AudioFile::open(&file_path) {
                app.audio_info = Some(audio_file.info.clone());
                app.playback_state.total_duration = audio_file.info.duration.unwrap_or(0.0);
            }
        }
        
        (app, Command::none())
    }

    fn title(&self) -> String {
        "Rust Audio Player".to_string()
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::PlayPause => {
                if self.file_path.is_empty() {
                    return Command::none();
                }
                
                if self.command_sender.is_none() && !self.is_playing {
                    // 启动新的音频播放会话
                    return Command::perform(
                        start_audio_playback(self.file_path.clone()),
                        |(sender, _handle)| {
                            Message::AudioSessionStarted(sender)
                        }
                    );
                } else if let Some(sender) = &self.command_sender {
                    // 发送播放/暂停命令
                    let command = if self.is_playing {
                        PlaybackCommand::Pause
                    } else {
                        PlaybackCommand::Resume
                    };
                    let _ = sender.send(command);
                }
                
                Command::none()
            }
            Message::Stop => {
                if let Some(sender) = &self.command_sender {
                    let _ = sender.send(PlaybackCommand::Stop);
                }
                self.is_playing = false;
                self.playback_state.is_playing = false;
                self.playback_state.is_paused = false;
                self.playback_state.current_time = 0.0;
                self.command_sender = None;
                Command::none()
            }
            Message::VolumeChanged(volume) => {
                self.playback_state.volume = volume;
                if let Some(sender) = &self.command_sender {
                    let _ = sender.send(PlaybackCommand::SetVolume(volume));
                }
                Command::none()
            }
            Message::OpenFile => {
                Command::perform(open_file_dialog(), Message::FileSelected)
            }
            Message::FileSelected(file_path) => {
                if let Some(path) = file_path {
                    self.file_path = path.clone();
                    self.is_playing = false;
                    self.playback_state = PlaybackState::default();
                    
                    // 尝试加载新的音频文件信息
                    if let Ok(audio_file) = AudioFile::open(&path) {
                        self.audio_info = Some(audio_file.info.clone());
                        self.playback_state.total_duration = audio_file.info.duration.unwrap_or(0.0);
                    } else {
                        self.audio_info = None;
                    }
                }
                Command::none()
            }
            Message::Tick => {
                // 更新播放时间
                if self.is_playing && self.playback_state.total_duration > 0.0 {
                    self.playback_state.current_time += 0.1;
                    if self.playback_state.current_time >= self.playback_state.total_duration {
                        self.playback_state.current_time = self.playback_state.total_duration;
                        self.is_playing = false;
                        self.playback_state.is_playing = false;
                        self.command_sender = None;
                    }
                }
                Command::none()
            }
            Message::PlaybackStateUpdate(state) => {
                self.playback_state = state.clone();
                self.is_playing = state.is_playing && !state.is_paused;
                Command::none()
            }
            Message::AudioSessionStarted(sender) => {
                self.command_sender = Some(sender);
                self.is_playing = true;
                self.playback_state.is_playing = true;
                self.playback_state.is_paused = false;
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let play_button_text = if self.is_playing {
            "⏸ Pause"
        } else {
            "▶ Play"
        };

        let file_name = if self.file_path.is_empty() {
            "No file loaded".to_string()
        } else {
            Path::new(&self.file_path)
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
        };

        let progress = if self.playback_state.total_duration > 0.0 {
            (self.playback_state.current_time / self.playback_state.total_duration).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let time_text = format!(
            "{} / {}",
            format_duration(self.playback_state.current_time),
            format_duration(self.playback_state.total_duration)
        );

        let audio_info_text = if let Some(info) = &self.audio_info {
            format!(
                "{} channels, {} Hz{}",
                info.channels,
                info.sample_rate,
                if let Some(bits) = info.bits_per_sample {
                    format!(", {} bits", bits)
                } else {
                    String::new()
                }
            )
        } else {
            "No audio info available".to_string()
        };

        let content = column![
            // 文件信息
            text(&file_name).size(20),
            text(&audio_info_text).size(14),
            Space::with_height(20),
            
            // 时间和进度条
            text(&time_text).size(16),
            progress_bar(0.0..=1.0, progress as f32).width(Length::Fill),
            Space::with_height(20),
            
            // 控制按钮
            row![
                button("📁 Open File").on_press(Message::OpenFile),
                Space::with_width(20),
                button(play_button_text).on_press(Message::PlayPause),
                Space::with_width(20),
                button("⏹ Stop").on_press(Message::Stop),
            ]
            .spacing(10),
            
            Space::with_height(20),
            
            // 音量控制
            row![
                text("Volume:").size(14),
                slider(0.0..=1.0, self.playback_state.volume, Message::VolumeChanged)
                    .width(Length::Fill),
                text(format!("{:.0}%", self.playback_state.volume * 100.0)).size(14),
            ]
            .spacing(10)
            .align_items(iced::Alignment::Center),
        ]
        .padding(20)
        .spacing(10)
        .align_items(iced::Alignment::Center);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        if self.is_playing {
            iced::time::every(Duration::from_millis(100)).map(|_| Message::Tick)
        } else {
            iced::Subscription::none()
        }
    }
}

// CLI参数
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long, help = "List available audio output devices")]
    list_devices: bool,

    #[arg(short, long, help = "Select output device by index")]
    device: Option<usize>,

    #[arg(short, long, help = "Show audio file information and duration without playing")]
    info: bool,

    #[arg(help = "Path to audio file")]
    file: Option<String>,
}

fn main() {
    let args = Cli::parse();
    
    if args.list_devices {
        list_audio_devices();
        return;
    }
    
    let file_path = args.file.unwrap_or_default();
    
    if args.info {
        if let Err(e) = get_audio_info(&file_path) {
            eprintln!("Error: {}", e);
        }
        return;
    }
    
    let settings = Settings {
        window: iced::window::Settings {
            size: iced::Size::new(600.0, 400.0),
            resizable: true,
            ..Default::default()
        },
        flags: file_path,
        ..Default::default()
    };
    
    PlayerApp::run(settings).unwrap();
}

async fn open_file_dialog() -> Option<String> {
    let file = rfd::AsyncFileDialog::new()
        .add_filter("Audio Files", &["mp3", "flac", "wav", "ogg", "aac", "m4a"])
        .add_filter("All Files", &["*"])
        .pick_file()
        .await;
    
    file.map(|f| f.path().to_string_lossy().to_string())
}

fn create_hint(file_path: &str) -> Hint {
    let mut hint = Hint::new();
    if let Some(extension) = Path::new(file_path).extension() {
        if let Some(ext_str) = extension.to_str() {
            hint.with_extension(ext_str);
        }
    }
    hint
}

fn get_audio_info(file_path: &str) -> Result<(), PlayerError> {
    let audio_file = AudioFile::open(file_path)?;
    let info = &audio_file.info;
    
    if let Some(duration) = info.duration {
        println!("Duration: {}", format_duration(duration));
    } else {
        println!("Duration: Unknown");
    }
    
    println!("Audio info: {} channels, {} Hz", info.channels, info.sample_rate);
    
    if let Some(bits_per_sample) = info.bits_per_sample {
        println!("Bits per sample: {}", bits_per_sample);
    }
    
    Ok(())
}

fn calculate_audio_duration(track: &symphonia::core::formats::Track, sample_rate: u32) -> Option<f64> {
    track.codec_params.n_frames.map(|frames| {
        frames as f64 / sample_rate as f64
    })
}

fn format_duration(seconds: f64) -> String {
    let total_seconds = seconds as u64;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    
    if hours > 0 {
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    } else {
        format!("{:02}:{:02}", minutes, seconds)
    }
}

// 启动音频播放会话
async fn start_audio_playback(file_path: String) -> (tokio::sync::mpsc::UnboundedSender<PlaybackCommand>, tokio::task::JoinHandle<()>) {
    let (command_sender, command_receiver) = tokio::sync::mpsc::unbounded_channel();
    
    let handle = tokio::task::spawn_blocking(move || {
        tokio::runtime::Handle::current().block_on(async move {
            if let Err(e) = run_audio_playback_with_control(&file_path, None, command_receiver).await {
                eprintln!("Audio playback error: {}", e);
            }
        })
    });
    
    (command_sender, handle)
}

// 音频播放控制函数
async fn run_audio_playback_with_control(
    file_path: &str,
    device_index: Option<usize>,
    mut command_receiver: tokio::sync::mpsc::UnboundedReceiver<PlaybackCommand>,
) -> Result<(), PlayerError> {
    let audio_file = AudioFile::open(file_path)?;
    let decoder = create_decoder(&audio_file.track)?;
    let (device, config, sample_format) = setup_audio_device(device_index, audio_file.info.sample_rate, audio_file.info.channels)?;
    
    let buffer_size = calculate_buffer_size(audio_file.info.sample_rate, audio_file.info.channels);
    let audio_buffer: AudioBuffer = Arc::new(Mutex::new(VecDeque::with_capacity(buffer_size)));
    
    let stream = create_audio_stream(&device, &config, sample_format, audio_buffer.clone(), audio_file.info.channels)?;
    
    let is_playing = Arc::new(AtomicBool::new(true));
    let is_paused = Arc::new(AtomicBool::new(false));
    let should_stop = Arc::new(AtomicBool::new(false));
    let current_volume = Arc::new(Mutex::new(1.0f32));
    
    stream.play().map_err(|e| PlayerError::PlaybackError(e.to_string()))?;
    
    let playback_thread = {
        let audio_buffer = audio_buffer.clone();
        let should_stop = should_stop.clone();
        
        thread::spawn(move || {
            let _ = run_playback_loop(audio_file, decoder, audio_buffer, should_stop);
        })
    };
    
    // 处理播放控制命令
    while let Some(command) = command_receiver.recv().await {
        match command {
            PlaybackCommand::Pause => {
                is_paused.store(true, Ordering::Relaxed);
            }
            PlaybackCommand::Resume => {
                is_paused.store(false, Ordering::Relaxed);
            }
            PlaybackCommand::Stop => {
                should_stop.store(true, Ordering::Relaxed);
                break;
            }
            PlaybackCommand::SetVolume(volume) => {
                let mut current_vol = current_volume.lock().unwrap();
                *current_vol = volume.clamp(0.0, 1.0);
            }
        }
    }
    
    should_stop.store(true, Ordering::Relaxed);
    playback_thread.join().unwrap();
    
    Ok(())
}

fn create_decoder(track: &symphonia::core::formats::Track) -> Result<Box<dyn symphonia::core::codecs::Decoder>, PlayerError> {
    default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|e| PlayerError::DecodingError(e.to_string()))
}

fn setup_audio_device(device_index: Option<usize>, sample_rate: u32, source_channels: usize) -> Result<(cpal::Device, cpal::StreamConfig, SampleFormat), PlayerError> {
    let host = cpal::default_host();
    let device = if let Some(index) = device_index {
        let devices: Vec<_> = host.output_devices()
            .map_err(|e| PlayerError::AudioDeviceError(e.to_string()))?
            .collect();
        
        if index >= devices.len() {
            return Err(PlayerError::AudioDeviceError(format!("Device index {} out of range", index)));
        }
        
        devices[index].clone()
    } else {
        host.default_output_device()
            .ok_or_else(|| PlayerError::AudioDeviceError("No default output device available".to_string()))?
    };

    println!("Using audio device: {}", device.name().unwrap_or("Unknown".to_string()));
    
    let supported_configs: Vec<_> = device.supported_output_configs()
        .map_err(|e| PlayerError::AudioDeviceError(e.to_string()))?
        .collect();
    
    if supported_configs.is_empty() {
        return Err(PlayerError::AudioDeviceError("No supported output configurations available".to_string()));
    }
    
    // 策略1: 寻找完全匹配的通道数配置
    if let Some(exact_match) = supported_configs.iter().find(|config| config.channels() as usize == source_channels) {
        let sample_format = exact_match.sample_format();
        let mut config = exact_match.with_sample_rate(cpal::SampleRate(sample_rate));
        
        // 如果设备不支持文件的采样率，使用设备支持的最接近采样率
        if sample_rate < exact_match.min_sample_rate().0 || sample_rate > exact_match.max_sample_rate().0 {
            // 选择最接近的采样率
            let closest_rate = if sample_rate < exact_match.min_sample_rate().0 {
                exact_match.min_sample_rate().0
            } else if sample_rate > exact_match.max_sample_rate().0 {
                exact_match.max_sample_rate().0
            } else {
                sample_rate
            };
            config = exact_match.with_sample_rate(cpal::SampleRate(closest_rate));
            println!("Using closest supported sample rate: {} Hz (requested: {} Hz)", closest_rate, sample_rate);
        } else {
            println!("Using exact sample rate match: {} Hz", sample_rate);
        }
        
        println!("Found exact channel match: {} channels, format: {:?}", exact_match.channels(), sample_format);
        return Ok((device, config.into(), sample_format));
    }
    
    // 策略2: 寻找通道数大于等于源通道数的配置（选择最接近的）
    let mut suitable_configs: Vec<_> = supported_configs.iter()
        .filter(|config| config.channels() as usize >= source_channels)
        .collect();
    
    if !suitable_configs.is_empty() {
        // 按通道数排序，选择最接近的
        suitable_configs.sort_by_key(|config| config.channels());
        let best_config = suitable_configs[0];
        
        let sample_format = best_config.sample_format();
        let mut config = best_config.with_sample_rate(cpal::SampleRate(sample_rate));
        
        if sample_rate < best_config.min_sample_rate().0 || sample_rate > best_config.max_sample_rate().0 {
            let closest_rate = if sample_rate < best_config.min_sample_rate().0 {
                best_config.min_sample_rate().0
            } else {
                best_config.max_sample_rate().0
            };
            config = best_config.with_sample_rate(cpal::SampleRate(closest_rate));
            println!("Using closest supported sample rate: {} Hz (requested: {} Hz)", closest_rate, sample_rate);
        }
        
        println!("Selected {} channels for {} channel audio, format: {:?}", best_config.channels(), source_channels, sample_format);
        return Ok((device, config.into(), sample_format));
    }
    
    // 策略3: 如果都不满足，选择质量最高的配置（更多通道、更高采样率、更好格式）
    let mut configs_with_score: Vec<_> = supported_configs.iter()
        .map(|config| {
            let format_score = match config.sample_format() {
                SampleFormat::F32 => 6,
                SampleFormat::F64 => 5,
                SampleFormat::I32 => 4,
                SampleFormat::I16 => 3,
                SampleFormat::U16 => 2,
                _ => 1,
            };
            
            let channel_score = config.channels() as i32;
            let rate_score = config.max_sample_rate().0 as i32 / 1000; // 归一化采样率分数
            
            let total_score = format_score * 100 + channel_score * 10 + rate_score;
            (config, total_score)
        })
        .collect();
    
    configs_with_score.sort_by_key(|(_, score)| std::cmp::Reverse(*score));
    let (fallback_config, _) = configs_with_score[0];
    
    let sample_format = fallback_config.sample_format();
    let mut config = fallback_config.with_sample_rate(cpal::SampleRate(sample_rate));
    
    if sample_rate < fallback_config.min_sample_rate().0 || sample_rate > fallback_config.max_sample_rate().0 {
        config = fallback_config.with_max_sample_rate();
        println!("Using device maximum sample rate: {} Hz (requested: {} Hz)", config.sample_rate().0, sample_rate);
    }
    
    println!("Warning: Using fallback configuration - {} channels for {} channel audio, format: {:?}", 
             fallback_config.channels(), source_channels, sample_format);
    println!("This may cause audio quality issues. Consider using a different audio device.");
    
    Ok((device, config.into(), sample_format))
}

fn calculate_buffer_size(sample_rate: u32, channels: usize) -> usize {
    sample_rate as usize * channels * DEFAULT_BUFFER_MULTIPLIER
}

fn create_audio_stream(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    sample_format: SampleFormat,
    buffer: AudioBuffer,
    channels: usize,
) -> Result<cpal::Stream, PlayerError> {
    match sample_format {
        SampleFormat::F32 => create_stream::<f32>(device, config, buffer, channels),
        SampleFormat::I16 => create_stream::<i16>(device, config, buffer, channels),
        SampleFormat::U16 => create_stream::<u16>(device, config, buffer, channels),
        SampleFormat::I8 => create_stream::<i8>(device, config, buffer, channels),
        SampleFormat::U8 => create_stream::<u8>(device, config, buffer, channels),
        SampleFormat::I32 => create_stream::<i32>(device, config, buffer, channels),
        SampleFormat::U32 => create_stream::<u32>(device, config, buffer, channels),
        SampleFormat::F64 => create_stream::<f64>(device, config, buffer, channels),
        _ => Err(PlayerError::AudioDeviceError(format!("Unsupported sample format: {:?}", sample_format))),
    }
}

fn create_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    buffer: AudioBuffer,
    channels: usize,
) -> Result<cpal::Stream, PlayerError>
where
    T: Sample + SizedSample + FromSample<f32> + Send + 'static,
{
    let output_channels = config.channels as usize;
    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            fill_audio_buffer(data, &buffer, output_channels, channels);
        },
        |err| {
            eprintln!("Audio stream error: {}", err);
        },
        None,
    ).map_err(|e| PlayerError::AudioDeviceError(e.to_string()))?;
    
    Ok(stream)
}

fn fill_audio_buffer<T>(
    data: &mut [T],
    buffer: &AudioBuffer,
    output_channels: usize,
    source_channels: usize,
) where
    T: Sample + FromSample<f32>,
{
    let mut audio_buffer = buffer.lock().unwrap();
    
    for frame in data.chunks_mut(output_channels) {
        if output_channels >= source_channels {
            // 输出通道数 >= 源通道数 (upmix 或直接映射)
            let mut frame_samples = Vec::with_capacity(output_channels);
            
            // 获取源声道数据
            for i in 0..source_channels {
                if let Some(audio_sample) = audio_buffer.pop_front() {
                    frame_samples.push(audio_sample);
                } else {
                    // 缓冲区为空，使用静音
                    frame_samples.push(0.0);
                }
            }
            
            // 填充输出声道
            for (i, sample) in frame.iter_mut().enumerate() {
                let audio_value = if i < source_channels {
                    // 直接映射源声道
                    frame_samples[i]
                } else if source_channels == 1 {
                    // 单声道到多声道：重复单声道信号
                    frame_samples[0]
                } else if source_channels == 2 && output_channels > 2 {
                    // 立体声到多声道：映射逻辑
                    match i {
                        0 | 2 => frame_samples[0], // 左声道 -> 前左、后左
                        1 | 3 => frame_samples[1], // 右声道 -> 前右、后右
                        4 => (frame_samples[0] + frame_samples[1]) * 0.5, // 中置：混合左右
                        5 => (frame_samples[0] + frame_samples[1]) * 0.3, // 低音炮：混合左右（降低音量）
                        _ => frame_samples[i % source_channels], // 其他：循环映射
                    }
                } else {
                    // 其他情况：循环映射
                    frame_samples[i % source_channels]
                };
                
                *sample = T::from_sample(audio_value);
            }
        } else {
            // 输出通道数 < 源通道数 (downmix)
            // 先获取所有源通道的数据
            let mut source_samples = Vec::with_capacity(source_channels);
            for _ in 0..source_channels {
                if let Some(audio_sample) = audio_buffer.pop_front() {
                    source_samples.push(audio_sample);
                } else {
                    source_samples.push(0.0);
                }
            }
            
            // 下混音到输出通道
            for (i, sample) in frame.iter_mut().enumerate() {
                let mixed_sample = if output_channels == 1 && source_channels == 2 {
                    // 立体声到单声道：平均左右声道
                    (source_samples[0] + source_samples[1]) * 0.5
                } else if output_channels == 1 {
                    // 多声道到单声道：平均所有声道
                    source_samples.iter().sum::<f32>() / source_channels as f32
                } else if output_channels == 2 && source_channels > 2 {
                    // 多声道到立体声
                    match i {
                        0 => { // 左声道：混合左相关声道
                            let mut left_mix = source_samples[0]; // 前左
                            if source_channels > 2 { left_mix += source_samples[2] * 0.7; } // 后左
                            if source_channels > 4 { left_mix += source_samples[4] * 0.5; } // 中置
                            if source_channels > 5 { left_mix += source_samples[5] * 0.3; } // 低音炮
                            left_mix.min(1.0).max(-1.0) // 限制幅度
                        }
                        1 => { // 右声道：混合右相关声道
                            let mut right_mix = source_samples[1]; // 前右
                            if source_channels > 3 { right_mix += source_samples[3] * 0.7; } // 后右
                            if source_channels > 4 { right_mix += source_samples[4] * 0.5; } // 中置
                            if source_channels > 5 { right_mix += source_samples[5] * 0.3; } // 低音炮
                            right_mix.min(1.0).max(-1.0) // 限制幅度
                        }
                        _ => source_samples[i % source_channels]
                    }
                } else {
                    // 其他情况：简单映射
                    let source_idx = (i * source_channels) / output_channels;
                    source_samples[source_idx.min(source_samples.len() - 1)]
                };
                
                *sample = T::from_sample(mixed_sample);
            }
        }
    }
}

fn run_playback_loop(
    mut audio_file: AudioFile,
    mut decoder: Box<dyn symphonia::core::codecs::Decoder>,
    audio_buffer: AudioBuffer,
    should_stop: Arc<AtomicBool>,
) -> Result<(), PlayerError> {
    let mut format = audio_file.probed.format;
    let track_id = audio_file.track_id;
    let target_channels = audio_file.info.channels;
    
    loop {
        if should_stop.load(Ordering::Relaxed) {
            break;
        }
        
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(symphonia::core::errors::Error::IoError(ref e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                break;
            }
            Err(e) => return Err(PlayerError::DecodingError(e.to_string())),
        };
        
        if packet.track_id() != track_id {
            continue;
        }
        
        let decoded = decoder.decode(&packet)
            .map_err(|e| PlayerError::DecodingError(e.to_string()))?;
        
        write_audio_buffer(&audio_buffer, &decoded, target_channels)?;
        
        // 控制缓冲区大小
        {
            let buffer = audio_buffer.lock().unwrap();
            if buffer.len() > BUFFER_CAPACITY_THRESHOLD {
                drop(buffer);
                thread::sleep(Duration::from_millis(BUFFER_WRITE_DELAY));
            }
        }
    }
    
    Ok(())
}

fn write_audio_buffer(buffer: &AudioBuffer, decoded: &AudioBufferRef, target_channels: usize) -> Result<(), PlayerError> {
    let mut interleaved = Vec::new();
    let mut channel_data = vec![Vec::new(); target_channels];
    
    convert_samples_any(decoded, &mut channel_data);
    
    let frame_count = channel_data[0].len();
    interleaved.reserve(frame_count * target_channels);
    
    for frame_idx in 0..frame_count {
        for channel_idx in 0..target_channels {
            if channel_idx < channel_data.len() {
                interleaved.push(channel_data[channel_idx][frame_idx]);
            } else {
                interleaved.push(0.0);
            }
        }
    }
    
    let mut audio_buffer = buffer.lock().unwrap();
    audio_buffer.extend(interleaved);
    
    Ok(())
}

fn convert_samples_any(input: &AudioBufferRef<'_>, output: &mut [Vec<f32>]) {
    use symphonia::core::audio::AudioBuffer;
    
    match input {
        AudioBufferRef::U8(buf) => convert_samples(buf, output),
        AudioBufferRef::U16(buf) => convert_samples(buf, output),
        AudioBufferRef::U24(buf) => convert_samples(buf, output),
        AudioBufferRef::U32(buf) => convert_samples(buf, output),
        AudioBufferRef::S8(buf) => convert_samples(buf, output),
        AudioBufferRef::S16(buf) => convert_samples(buf, output),
        AudioBufferRef::S24(buf) => convert_samples(buf, output),
        AudioBufferRef::S32(buf) => convert_samples(buf, output),
        AudioBufferRef::F32(buf) => convert_samples(buf, output),
        AudioBufferRef::F64(buf) => convert_samples(buf, output),
    }
}

fn convert_samples<S>(input: &symphonia::core::audio::AudioBuffer<S>, output: &mut [Vec<f32>])
where
    S: symphonia::core::sample::Sample + symphonia::core::conv::IntoSample<f32>,
{
    for (c, dst) in output.iter_mut().enumerate() {
        if c < input.spec().channels.count() {
            let src = input.chan(c);
            dst.extend(src.iter().map(|&s| s.into_sample()));
        }
    }
}

fn list_audio_devices() {
    let host = cpal::default_host();
    
    println!("Available audio output devices:");
    
    match host.output_devices() {
        Ok(devices) => {
            for (index, device) in devices.enumerate() {
                let name = device.name().unwrap_or_else(|_| "Unknown Device".to_string());
                println!("\n  Device {}: {}", index, name);
                
                // 显示设备支持的配置
                match device.supported_output_configs() {
                    Ok(configs) => {
                        println!("    Supported configurations:");
                        for (config_idx, config) in configs.enumerate() {
                            println!("      Config {}: {} channels, {}-{} Hz, {:?}", 
                                config_idx,
                                config.channels(),
                                config.min_sample_rate().0,
                                config.max_sample_rate().0,
                                config.sample_format()
                            );
                        }
                    }
                    Err(e) => println!("    Error getting configs: {}", e),
                }
                
                // 显示默认配置
                match device.default_output_config() {
                    Ok(default_config) => {
                        println!("    Default config: {} channels, {} Hz, {:?}",
                            default_config.channels(),
                            default_config.sample_rate().0,
                            default_config.sample_format()
                        );
                    }
                    Err(e) => println!("    Error getting default config: {}", e),
                }
            }
            
            // 显示默认设备
            if let Some(default_device) = host.default_output_device() {
                let default_name = default_device.name().unwrap_or_else(|_| "Unknown Device".to_string());
                println!("\n  Default output device: {}", default_name);
            }
        }
        Err(e) => println!("Error listing devices: {}", e),
    }
    
    println!("\nTip: Use --device <index> to select a specific device for better audio quality.");
}