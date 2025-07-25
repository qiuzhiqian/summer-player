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
    widget::{button, column, container, row, text, progress_bar, slider, Space, scrollable},
    executor, Font,
    window::Event as WindowEvent,
    time,
    Settings,
    Theme,
    Element,
    Length,
    Subscription,
    Task,
    event::{self, Event},
    window,
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

// 字体配置常量
const CHINESE_FONT: &str = "Noto Sans CJK SC";
const EMOJI_FONT: &str = "Noto Color Emoji";
const DEFAULT_FONT: &str = "DejaVu Sans";

// 自定义错误类型
#[derive(Debug)]
enum PlayerError {
    FileNotFound(String),
    UnsupportedFormat(String),
    AudioDeviceError(String),
    DecodingError(String),
    PlaybackError(String),
    PlaylistError(String),
}

impl fmt::Display for PlayerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlayerError::FileNotFound(msg) => write!(f, "File not found: {}", msg),
            PlayerError::UnsupportedFormat(msg) => write!(f, "Unsupported format: {}", msg),
            PlayerError::AudioDeviceError(msg) => write!(f, "Audio device error: {}", msg),
            PlayerError::DecodingError(msg) => write!(f, "Decoding error: {}", msg),
            PlayerError::PlaybackError(msg) => write!(f, "Playback error: {}", msg),
            PlayerError::PlaylistError(msg) => write!(f, "Playlist error: {}", msg),
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
    
    fn new_with_file_path(track: &symphonia::core::formats::Track, file_path: &str) -> Self {
        let channels = track.codec_params.channels.map(|c| c.count()).unwrap_or(2);
        let sample_rate = track.codec_params.sample_rate.unwrap_or(44100);
        let mut duration = calculate_audio_duration(track, sample_rate);
        let bits_per_sample = track.codec_params.bits_per_sample;

        // 如果标准方法无法获取时长或时长为0，尝试通过解析文件来估算
        // 这对于m4s等流媒体片段文件特别有用
        if duration.is_none() || duration == Some(0.0) {
            duration = estimate_audio_duration_by_parsing(file_path);
        }

        Self {
            channels,
            sample_rate,
            duration,
            bits_per_sample,
        }
    }
}

// 播放列表项结构体
#[derive(Debug, Clone)]
struct PlaylistItem {
    path: String,
    name: String,
    duration: Option<f64>,
}

impl PlaylistItem {
    fn new(path: String) -> Self {
        let name = Path::new(&path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();
        
        Self {
            path,
            name,
            duration: None,
        }
    }
    
    fn with_duration(mut self, duration: Option<f64>) -> Self {
        self.duration = duration;
        self
    }
}

// 播放列表结构体
#[derive(Debug, Clone, Default)]
struct Playlist {
    items: Vec<PlaylistItem>,
    current_index: Option<usize>,
}

impl Playlist {
    fn new() -> Self {
        Self {
            items: Vec::new(),
            current_index: None,
        }
    }
    
    fn add_item(&mut self, item: PlaylistItem) {
        self.items.push(item);
    }
    
    fn current_item(&self) -> Option<&PlaylistItem> {
        self.current_index.and_then(|i| self.items.get(i))
    }
    
    fn next_item(&mut self) -> Option<&PlaylistItem> {
        if self.items.is_empty() {
            return None;
        }
        
        let next_index = match self.current_index {
            Some(current) => {
                if current + 1 < self.items.len() {
                    Some(current + 1)
                } else {
                    None // 播放列表结束
                }
            }
            None => Some(0), // 开始播放
        };
        
        self.current_index = next_index;
        self.current_item()
    }
    
    fn previous_item(&mut self) -> Option<&PlaylistItem> {
        if self.items.is_empty() {
            return None;
        }
        
        let prev_index = match self.current_index {
            Some(current) => {
                if current > 0 {
                    Some(current - 1)
                } else {
                    None // 已经是第一首
                }
            }
            None => Some(0), // 开始播放
        };
        
        self.current_index = prev_index;
        self.current_item()
    }
    
    fn set_current_index(&mut self, index: usize) -> Option<&PlaylistItem> {
        if index < self.items.len() {
            self.current_index = Some(index);
            self.current_item()
        } else {
            None
        }
    }
    
    fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
    
    fn len(&self) -> usize {
        self.items.len()
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
        let info = AudioInfo::new_with_file_path(&track, file_path);
        
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
    PlaylistItemSelected(usize),
    NextTrack,
    PreviousTrack,
    Tick,
    PlaybackStateUpdate(PlaybackState),
    AudioSessionStarted(tokio::sync::mpsc::UnboundedSender<PlaybackCommand>),
    EventOccurred(Event),
}

// iced应用程序状态
#[derive(Default)]
struct PlayerApp {
    playback_state: PlaybackState,
    audio_info: Option<AudioInfo>,
    file_path: String,
    is_playing: bool,
    command_sender: Option<tokio::sync::mpsc::UnboundedSender<PlaybackCommand>>,
    audio_handle: Option<tokio::task::JoinHandle<()>>,
    playlist: Playlist,
    playlist_loaded: bool,
}

impl PlayerApp {

    fn title(&self) -> String {
        "音频播放器".to_string()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::PlayPause => {
                if self.file_path.is_empty() {
                    return Task::none();
                }
                
                // 检查是否需要启动新的播放会话
                // 情况1：没有command_sender且没有在播放
                // 情况2：歌曲已播放完毕（到达或超过总时长）
                let should_start_new_session = self.command_sender.is_none() && !self.is_playing ||
                    (self.playback_state.total_duration > 0.0 && 
                     self.playback_state.current_time >= self.playback_state.total_duration);
                
                if should_start_new_session {
                    // 清理旧的状态
                    if let Some(sender) = &self.command_sender {
                        let _ = sender.send(PlaybackCommand::Stop);
                    }
                    self.command_sender = None;
                    self.is_playing = false;
                    self.playback_state.is_playing = false;
                    self.playback_state.is_paused = false;
                    self.playback_state.current_time = 0.0;
                    
                    // 启动新的音频播放会话
                    return Task::perform(
                        start_audio_playback(self.file_path.clone()),
                        |(sender, handle)| {
                            Message::AudioSessionStarted(sender)
                        }
                    );
                }
                
                if let Some(sender) = &self.command_sender {
                    if self.is_playing {
                        let _ = sender.send(PlaybackCommand::Pause);
                    } else {
                        let _ = sender.send(PlaybackCommand::Resume);
                    }
                }
                Task::none()
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
                Task::none()
            }
            Message::VolumeChanged(volume) => {
                self.playback_state.volume = volume;
                if let Some(sender) = &self.command_sender {
                    let _ = sender.send(PlaybackCommand::SetVolume(volume));
                }
                Task::none()
            }
            Message::OpenFile => {
                Task::perform(open_file_dialog(), Message::FileSelected)
            }
            Message::FileSelected(file_path) => {
                if let Some(path) = file_path {
                    // 检查是否为M3U播放列表
                    if is_m3u_playlist(&path) {
                        match parse_m3u_playlist(&path) {
                            Ok(playlist) => {
                                self.playlist = playlist;
                                self.playlist_loaded = true;
                                // 自动播放第一首歌曲
                                if let Some(first_item) = self.playlist.set_current_index(0) {
                                    self.file_path = first_item.path.clone();
                                    if let Ok(audio_file) = AudioFile::open(&first_item.path) {
                                        self.audio_info = Some(audio_file.info.clone());
                                        self.playback_state.total_duration = audio_file.info.duration.unwrap_or(0.0);
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to parse playlist: {}", e);
                            }
                        }
                                } else {
                // 普通音频文件 - 创建单文件播放列表
                self.file_path = path.clone();
                
                // 创建包含单个文件的播放列表
                let mut single_file_playlist = Playlist::new();
                let mut playlist_item = PlaylistItem::new(path.clone());
                
                // 获取音频信息并设置时长
                if let Ok(audio_file) = AudioFile::open(&path) {
                    self.audio_info = Some(audio_file.info.clone());
                    self.playback_state.total_duration = audio_file.info.duration.unwrap_or(0.0);
                    playlist_item = playlist_item.with_duration(audio_file.info.duration);
                }
                
                single_file_playlist.add_item(playlist_item);
                single_file_playlist.set_current_index(0);
                
                self.playlist = single_file_playlist;
                self.playlist_loaded = true;
            }
                }
                Task::none()
            }
            Message::PlaylistItemSelected(index) => {
                if self.playlist_loaded {
                    if let Some(item) = self.playlist.set_current_index(index) {
                        self.file_path = item.path.clone();
                        if let Ok(audio_file) = AudioFile::open(&item.path) {
                            self.audio_info = Some(audio_file.info.clone());
                            self.playback_state.total_duration = audio_file.info.duration.unwrap_or(0.0);
                        }
                        // 停止当前播放并开始播放新选择的歌曲
                        if let Some(sender) = &self.command_sender {
                            let _ = sender.send(PlaybackCommand::Stop);
                        }
                        self.is_playing = false;
                        self.playback_state.is_playing = false;
                        self.playback_state.is_paused = false;
                        self.playback_state.current_time = 0.0;
                        self.command_sender = None;
                    }
                }
                Task::none()
            }
            Message::NextTrack => {
                if self.playlist_loaded {
                    if let Some(next_item) = self.playlist.next_item() {
                        self.file_path = next_item.path.clone();
                        if let Ok(audio_file) = AudioFile::open(&next_item.path) {
                            self.audio_info = Some(audio_file.info.clone());
                            self.playback_state.total_duration = audio_file.info.duration.unwrap_or(0.0);
                        }
                        // 停止当前播放并开始播放下一首
                        if let Some(sender) = &self.command_sender {
                            let _ = sender.send(PlaybackCommand::Stop);
                        }
                        self.is_playing = false;
                        self.playback_state.is_playing = false;
                        self.playback_state.is_paused = false;
                        self.playback_state.current_time = 0.0;
                        self.command_sender = None;
                    }
                }
                Task::none()
            }
            Message::PreviousTrack => {
                if self.playlist_loaded {
                    if let Some(prev_item) = self.playlist.previous_item() {
                        self.file_path = prev_item.path.clone();
                        if let Ok(audio_file) = AudioFile::open(&prev_item.path) {
                            self.audio_info = Some(audio_file.info.clone());
                            self.playback_state.total_duration = audio_file.info.duration.unwrap_or(0.0);
                        }
                        // 停止当前播放并开始播放上一首
                        if let Some(sender) = &self.command_sender {
                            let _ = sender.send(PlaybackCommand::Stop);
                        }
                        self.is_playing = false;
                        self.playback_state.is_playing = false;
                        self.playback_state.is_paused = false;
                        self.playback_state.current_time = 0.0;
                        self.command_sender = None;
                    }
                }
                Task::none()
            }
            Message::Tick => {
                // 更新播放进度
                if self.is_playing {
                    self.playback_state.current_time += 0.1;
                    if self.playback_state.total_duration > 0.0 && 
                       self.playback_state.current_time >= self.playback_state.total_duration {
                        // 歌曲播放完毕，清理状态
                        self.playback_state.current_time = self.playback_state.total_duration;
                        self.is_playing = false;
                        self.playback_state.is_playing = false;
                        self.playback_state.is_paused = false;
                        
                        // 清理command_sender，为下次播放做准备
                        if let Some(sender) = &self.command_sender {
                            let _ = sender.send(PlaybackCommand::Stop);
                        }
                        self.command_sender = None;
                        
                        // 如果是播放列表模式，自动播放下一首
                        if self.playlist_loaded {
                            if let Some(next_item) = self.playlist.next_item() {
                                self.file_path = next_item.path.clone();
                                if let Ok(audio_file) = AudioFile::open(&next_item.path) {
                                    self.audio_info = Some(audio_file.info.clone());
                                    self.playback_state.total_duration = audio_file.info.duration.unwrap_or(0.0);
                                    self.playback_state.current_time = 0.0;
                                    
                                    // 自动开始播放下一首
                                    return Task::perform(
                                        start_audio_playback(next_item.path.clone()),
                                        |(sender, handle)| {
                                            Message::AudioSessionStarted(sender)
                                        }
                                    );
                                }
                            }
                        }
                    }
                }
                Task::none()
            }
            Message::PlaybackStateUpdate(state) => {
                self.playback_state = state.clone();
                self.is_playing = state.is_playing && !state.is_paused;
                Task::none()
            }
            Message::AudioSessionStarted(sender) => {
                self.command_sender = Some(sender);
                self.is_playing = true;
                self.playback_state.is_playing = true;
                self.playback_state.is_paused = false;
                Task::none()
            }
            Message::EventOccurred(event) => {
                if let Event::Window(window::Event::Closed) = event {
                    if let Some(sender) = &self.command_sender {
                        let _ = sender.send(PlaybackCommand::Stop);
                        // 给一点时间让停止命令生效
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    }
                    
                    // 清理状态
                    self.is_playing = false;
                    self.playback_state.is_playing = false;
                    self.playback_state.is_paused = false;
                    self.command_sender = None;
                    self.audio_handle = None;

                    Task::none()
                } else {
                    Task::none()
                }
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let title = text("音频播放器").size(24);
        
        let file_info = if let Some(info) = &self.audio_info {
            column![
                text(format!("文件: {}", self.file_path)),
                text(format!("声道: {}", info.channels)),
                text(format!("采样率: {} Hz", info.sample_rate)),
                text(format!("时长: {}", 
                    if let Some(duration) = info.duration {
                        format_duration(duration)
                    } else {
                        "未知".to_string()
                    }
                )),
            ].spacing(10)
        } else {
            column![
                text("未选择文件"),
            ].spacing(10)
        };
        
        let controls = row![
            button("播放/暂停").on_press(Message::PlayPause),
            button("停止").on_press(Message::Stop),
            button("上一首").on_press(Message::PreviousTrack),
            button("下一首").on_press(Message::NextTrack),
        ].spacing(10);
        
        let file_controls = row![
            button("打开文件").on_press(Message::OpenFile),
        ].spacing(10);
        
        let progress = if self.playback_state.total_duration > 0.0 {
            let progress_value = (self.playback_state.current_time / self.playback_state.total_duration) as f32;
            column![
                progress_bar(0.0..=1.0, progress_value),
                text(format!("{} / {}", 
                    format_duration(self.playback_state.current_time),
                    format_duration(self.playback_state.total_duration)
                )),
            ].spacing(5)
        } else {
            column![
                progress_bar(0.0..=1.0, 0.0),
                text("0:00 / 0:00"),
            ].spacing(5)
        };
        
        let volume_control = column![
            text(format!("音量: {:.0}%", self.playback_state.volume * 100.0)),
            slider(0.0..=1.0, self.playback_state.volume, Message::VolumeChanged)
                .width(Length::Fill),
        ].spacing(5);
        
        let status = text(format!("状态: {}", 
            if self.is_playing { "播放中" } else { "已停止" }
        ));
        
        // 播放列表显示
        let playlist_view = if self.playlist_loaded {
            let playlist_items: Vec<Element<Message>> = self.playlist.items.iter().enumerate().map(|(index, item)| {
                let is_current = self.playlist.current_index == Some(index);
                let is_playing = is_current && self.is_playing;
                
                let item_text = if is_current {
                    format!("▶ {} ({})", 
                        item.name,
                        item.duration.map_or("未知时长".to_string(), |d| format_duration(d))
                    )
                } else {
                    format!("  {} ({})", 
                        item.name,
                        item.duration.map_or("未知时长".to_string(), |d| format_duration(d))
                    )
                };
                
                let btn = button(text(item_text))
                    .on_press(Message::PlaylistItemSelected(index))
                    .width(Length::Fill);
                
                // 为当前播放的项目添加样式
                if is_playing {
                    btn.style(|theme: &iced::Theme, status| {
                        let palette = theme.extended_palette();
                        match status {
                            iced::widget::button::Status::Active => iced::widget::button::Style {
                                background: Some(iced::Background::Color(palette.primary.weak.color)),
                                text_color: palette.primary.strong.text,
                                border: iced::Border::default(),
                                shadow: iced::Shadow::default(),
                            },
                            iced::widget::button::Status::Hovered => iced::widget::button::Style {
                                background: Some(iced::Background::Color(palette.primary.base.color)),
                                text_color: palette.primary.strong.text,
                                border: iced::Border::default(),
                                shadow: iced::Shadow::default(),
                            },
                            _ => iced::widget::button::Style::default(),
                        }
                    }).into()
                } else if is_current {
                    btn.style(|theme: &iced::Theme, status| {
                        let palette = theme.extended_palette();
                        match status {
                            iced::widget::button::Status::Active => iced::widget::button::Style {
                                background: Some(iced::Background::Color(palette.secondary.weak.color)),
                                text_color: palette.secondary.strong.text,
                                border: iced::Border::default(),
                                shadow: iced::Shadow::default(),
                            },
                            iced::widget::button::Status::Hovered => iced::widget::button::Style {
                                background: Some(iced::Background::Color(palette.secondary.base.color)),
                                text_color: palette.secondary.strong.text,
                                border: iced::Border::default(),
                                shadow: iced::Shadow::default(),
                            },
                            _ => iced::widget::button::Style::default(),
                        }
                    }).into()
                } else {
                    btn.into()
                }
            }).collect();
            
            column![
                text(format!("播放列表 ({} 首歌曲)", self.playlist.len())).size(16),
                scrollable(
                    column(playlist_items).spacing(5).width(Length::Fill)
                ).height(Length::Fill).width(Length::Fill),
            ].spacing(10)
        } else {
            column![
                text("未加载播放列表"),
            ].spacing(10)
        };
        
        column![
            row![
                // 左侧控制面板
                column![
                    title,
                    file_info,
                    file_controls,
                    controls,
                    volume_control,
                    status,
                    Space::new(Length::Fill, Length::Fill),
                ].spacing(10).width(Length::Fixed(300.0)),
                
                // 右侧播放列表
                playlist_view.width(Length::Fill),
            ].spacing(10),
            
            // 底部进度条，横跨整个宽度
            progress.width(Length::Fill),
        ]
        .spacing(10)
        .padding(10)
        .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            time::every(Duration::from_millis(100)).map(|_| Message::Tick),
            event::listen().map(Message::EventOccurred)
        ])
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
    
    iced::application("player", PlayerApp::update, PlayerApp::view)
        .subscription(PlayerApp::subscription)
        .default_font(Font::with_name("Noto Sans CJK SC"))
        .run().unwrap();
}

async fn open_file_dialog() -> Option<String> {
    let file = rfd::AsyncFileDialog::new()
        .add_filter("Audio Files", &["mp3", "flac", "wav", "ogg", "aac", "m4a", "m4s"])
        .add_filter("Playlist Files", &["m3u", "m3u8"])
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

// M3U播放列表解析函数
fn parse_m3u_playlist(file_path: &str) -> Result<Playlist, PlayerError> {
    use std::fs;
    use std::io::{BufRead, BufReader};
    
    let file = fs::File::open(file_path)
        .map_err(|e| PlayerError::PlaylistError(format!("Failed to open playlist file: {}", e)))?;
    
    let reader = BufReader::new(file);
    let mut playlist = Playlist::new();
    let playlist_dir = Path::new(file_path).parent()
        .ok_or_else(|| PlayerError::PlaylistError("Invalid playlist path".to_string()))?;
    
    for (line_num, line) in reader.lines().enumerate() {
        let line = line.map_err(|e| PlayerError::PlaylistError(format!("Failed to read line {}: {}", line_num + 1, e)))?;
        let line = line.trim();
        
        // 跳过空行和注释行
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        
        // 处理文件路径
        let file_path = if Path::new(line).is_absolute() {
            line.to_string()
        } else {
            // 相对路径，相对于播放列表文件的位置
            playlist_dir.join(line).to_string_lossy().to_string()
        };
        
        // 检查文件是否存在
        if !Path::new(&file_path).exists() {
            eprintln!("Warning: File not found: {}", file_path);
            continue;
        }
        
        // 创建播放列表项
        let mut item = PlaylistItem::new(file_path.clone());
        
        // 尝试获取音频时长信息
        if let Ok(audio_file) = AudioFile::open(&file_path) {
            item = item.with_duration(audio_file.info.duration);
        }
        
        playlist.add_item(item);
    }
    
    if playlist.is_empty() {
        return Err(PlayerError::PlaylistError("No valid files found in playlist".to_string()));
    }
    
    Ok(playlist)
}

// 检查文件是否为M3U播放列表
fn is_m3u_playlist(file_path: &str) -> bool {
    if let Some(extension) = Path::new(file_path).extension() {
        if let Some(ext_str) = extension.to_str() {
            return ext_str.to_lowercase() == "m3u" || ext_str.to_lowercase() == "m3u8";
        }
    }
    false
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
    // 方法1: 尝试从n_frames获取时长
    if let Some(frames) = track.codec_params.n_frames {
        if frames > 0 {
            return Some(frames as f64 / sample_rate as f64);
        }
    }
    
    // 方法2: 尝试从time_base和start_ts计算时长
    // 这对于m4s等流媒体格式特别重要
    if let Some(time_base) = track.codec_params.time_base {
        let start_ts = track.codec_params.start_ts;
        // start_ts在某些格式中可能表示duration_ts
        if start_ts > 0 {
            let duration_seconds = start_ts as f64 * time_base.numer as f64 / time_base.denom as f64;
            if duration_seconds > 0.0 {
                return Some(duration_seconds);
            }
        }
    }
    
    None
}

// 通过解析整个文件来估算时长的函数
fn estimate_audio_duration_by_parsing(file_path: &str) -> Option<f64> {
    use std::fs::File;
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::probe::Hint;
    use symphonia::core::formats::FormatOptions;
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::codecs::DecoderOptions;
    use symphonia::default;
    
    // 尝试打开文件并计算样本数量
    let file = File::open(file_path).ok()?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    let hint = {
        let mut hint = Hint::new();
        if let Some(extension) = Path::new(file_path).extension() {
            if let Some(ext_str) = extension.to_str() {
                hint.with_extension(ext_str);
            }
        }
        hint
    };
    
    let mut probed = default::get_probe()
        .format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default())
        .ok()?;
    
    let track = probed.format.tracks()
        .iter()
        .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)?
        .clone();
    
    let sample_rate = track.codec_params.sample_rate.unwrap_or(44100);
    let mut decoder = default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        
        .ok()?;
    
    let mut total_frames = 0u64;
    let mut packet_count = 0u64;
    
    // 尝试解析整个文件来获得准确时长
    // 对于m4s这样的文件，完整解析是值得的
    const MAX_ESTIMATION_PACKETS: u64 = 100000; // 大幅增加到100000包
    
    loop {
        if packet_count >= MAX_ESTIMATION_PACKETS {
            break;
        }
        
        let packet = match probed.format.next_packet() {
            Ok(packet) => packet,
            Err(_) => break,
        };
        
        if packet.track_id() != track.id {
            continue;
        }
        
        packet_count += 1;
        
        if let Ok(decoded) = decoder.decode(&packet) {
            total_frames += decoded.frames() as u64;
        }
    }
    
    if total_frames > 0 && packet_count > 0 {
        // 直接计算解析的音频时长
        let parsed_duration = total_frames as f64 / sample_rate as f64;
        
        // 如果我们解析到了文件末尾（没有达到包数限制），那么这就是准确的时长
        if packet_count < MAX_ESTIMATION_PACKETS {
            return Some(parsed_duration);
        }
        
        // 否则，我们需要估算总时长
        if let Ok(metadata) = std::fs::metadata(file_path) {
            let file_size = metadata.len();
            
            // 基于解析的包数比例来估算
            let avg_packet_size = file_size as f64 / packet_count as f64;
            let estimated_total_packets = file_size as f64 / avg_packet_size;
            let estimated_duration = parsed_duration * (estimated_total_packets / packet_count as f64);
            
            if estimated_duration > 0.0 && estimated_duration < 86400.0 {
                return Some(estimated_duration);
            }
        }
        
        // 备用方法：直接返回解析的时长
        return Some(parsed_duration);
    }
    
    None
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