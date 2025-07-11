use std::{fs::File, path::Path, sync::Arc, time::Duration};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::collections::VecDeque;
use std::thread;
use std::fmt;

use clap::Parser;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample, SampleFormat, SizedSample};

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
const DECODE_PROGRESS_INTERVAL: usize = 100;
const PLAYBACK_POLL_INTERVAL: u64 = 100;
const PLAYBACK_FINISH_DELAY: u64 = 500;
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

// 音频信息结构体
#[derive(Debug, Clone)]
struct AudioInfo {
    channels: usize,
    sample_rate: u32,
    duration: Option<f64>,
    bits_per_sample: Option<u32>,
    n_frames: Option<u64>,
}

impl AudioInfo {
    fn new(track: &symphonia::core::formats::Track) -> Self {
        let channels = track.codec_params.channels.map(|c| c.count()).unwrap_or(2);
        let sample_rate = track.codec_params.sample_rate.unwrap_or(44100);
        let duration = calculate_audio_duration(track, sample_rate);
        let bits_per_sample = track.codec_params.bits_per_sample;
        let n_frames = track.codec_params.n_frames;

        Self {
            channels,
            sample_rate,
            duration,
            bits_per_sample,
            n_frames,
        }
    }

    fn display_info(&self) {
        if let Some(duration) = self.duration {
            println!("Duration: {}", format_duration(duration));
        } else {
            println!("Duration: Unknown");
        }
        
        println!("Audio info: {} channels, {} Hz", self.channels, self.sample_rate);
        
        if let Some(bits_per_sample) = self.bits_per_sample {
            println!("Bits per sample: {}", bits_per_sample);
        }
        
        if let Some(frames) = self.n_frames {
            println!("Total frames: {}", frames);
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
        // 验证文件是否存在
        if !Path::new(file_path).exists() {
            return Err(PlayerError::FileNotFound(file_path.to_string()));
        }

        // 打开音频文件
        let file = File::open(file_path)
            .map_err(|e| PlayerError::FileNotFound(format!("{}: {}", file_path, e)))?;
        let mss = MediaSourceStream::new(Box::new(file), Default::default());
        
        // 创建提示信息
        let hint = create_hint(file_path);
        
        // 探测音频格式
        let probed = default::get_probe()
            .format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default())
            .map_err(|e| PlayerError::UnsupportedFormat(format!("{}: {}", file_path, e)))?;
        
        // 获取音频轨道
        let track = probed
            .format
            .tracks()
            .iter()
            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
            .ok_or_else(|| PlayerError::UnsupportedFormat("No supported audio tracks found".to_string()))?
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

    fn display_metadata(&mut self) {
        if let Some(metadata) = self.probed.metadata.get() {
            if let Some(current) = metadata.current() {
                println!("Metadata:");
                for tag in current.tags() {
                    if let Some(std_key) = tag.std_key {
                        println!("  {:?}: {:?}", std_key, tag.value);
                    } else {
                        println!("  {}: {:?}", tag.key, tag.value);
                    }
                }
            }
        }
    }
}

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

// 音频缓冲区类型
type AudioBuffer = Arc<Mutex<VecDeque<f32>>>;

fn main() {
    let cli = Cli::parse();

    if cli.list_devices {
        list_audio_devices();
        return;
    }

    let file_path = match cli.file {
        Some(path) => path,
        None => {
            eprintln!("Please provide a file path or use --list-devices to see available devices");
            std::process::exit(1);
        }
    };

    let result = if cli.info {
        get_audio_info(&file_path)
    } else {
        play_audio(&file_path, cli.device)
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn create_hint(file_path: &str) -> Hint {
    let mut hint = Hint::new();
    if let Some(ext) = Path::new(file_path).extension() {
        if let Some(ext_str) = ext.to_str() {
            hint.with_extension(ext_str);
        }
    }
    hint
}

fn get_audio_info(file_path: &str) -> Result<(), PlayerError> {
    let mut audio_file = AudioFile::open(file_path)?;
    
    println!("File: {}", file_path);
    audio_file.display_metadata();
    audio_file.info.display_info();
    
    Ok(())
}

fn play_audio(file_path: &str, device_index: Option<usize>) -> Result<(), PlayerError> {
    let mut audio_file = AudioFile::open(file_path)?;
    
    println!("Playing: {}", file_path);
    audio_file.display_metadata();
    audio_file.info.display_info();
    
    // 创建解码器
    let decoder = create_decoder(&audio_file.track)?;
    
    // 获取音频设备和配置
    let (device, config, sample_format) = setup_audio_device(device_index, audio_file.info.sample_rate, audio_file.info.channels)?;
    
    // 创建音频缓冲区
    let buffer_size = calculate_buffer_size(audio_file.info.sample_rate, audio_file.info.channels);
    let audio_buffer: AudioBuffer = Arc::new(Mutex::new(VecDeque::with_capacity(buffer_size)));
    
    // 创建音频流
    let stream = create_audio_stream(&device, &config, sample_format, Arc::clone(&audio_buffer), audio_file.info.channels)?;
    
    // 开始播放
    stream.play().map_err(|e| PlayerError::PlaybackError(e.to_string()))?;
    
    // 执行播放循环
    run_playback_loop(audio_file, decoder, audio_buffer)?;
    
    println!("Playback finished successfully");
    Ok(())
}

fn create_decoder(track: &symphonia::core::formats::Track) -> Result<Box<dyn symphonia::core::codecs::Decoder>, PlayerError> {
    default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|e| PlayerError::DecodingError(e.to_string()))
}

fn setup_audio_device(device_index: Option<usize>, sample_rate: u32, source_channels: usize) -> Result<(cpal::Device, cpal::StreamConfig, SampleFormat), PlayerError> {
    let host = cpal::default_host();
    let audio_device = if let Some(index) = device_index {
        let mut devices = host.output_devices()
            .map_err(|e| PlayerError::AudioDeviceError(e.to_string()))?;
        devices.nth(index).ok_or(PlayerError::AudioDeviceError("Device index out of range".to_string()))?
    } else {
        host.default_output_device().ok_or(PlayerError::AudioDeviceError("No default audio device".to_string()))?
    };

    println!("Using audio device: {}", audio_device.name().unwrap_or("Unknown".to_string()));

    let supported_configs = audio_device.supported_output_configs()
        .map_err(|e| PlayerError::AudioDeviceError(e.to_string()))?;
    
    // 收集所有支持的配置
    let mut configs: Vec<_> = supported_configs.collect();
    
    if configs.is_empty() {
        return Err(PlayerError::AudioDeviceError("No supported output config".to_string()));
    }
    
    // 选择最佳配置的策略：
    // 1. 优先选择与源音频通道数完全匹配的配置
    // 2. 如果没有完全匹配的，选择通道数大于等于源通道数的配置（优先选择最接近的）
    // 3. 如果都不满足，选择通道数最多的配置
    
    // 策略1：寻找完全匹配的通道数
    if let Some(exact_match) = configs.iter().find(|config| config.channels() as usize == source_channels) {
        let sample_format = exact_match.sample_format();
        let mut config = exact_match.with_sample_rate(cpal::SampleRate(sample_rate));
        
        // 如果设备不支持文件的采样率，使用设备的默认采样率
        if sample_rate < exact_match.min_sample_rate().0 || sample_rate > exact_match.max_sample_rate().0 {
            config = exact_match.with_max_sample_rate();
            println!("Using device sample rate: {} Hz", config.sample_rate().0);
        }
        
        println!("Found exact channel match: {} channels", exact_match.channels());
        return Ok((audio_device, config.into(), sample_format));
    }
    
    // 策略2：寻找通道数大于等于源通道数的配置（选择最接近的）
    let mut suitable_configs: Vec<_> = configs.iter()
        .filter(|config| config.channels() as usize >= source_channels)
        .collect();
    
    if !suitable_configs.is_empty() {
        // 按通道数排序，选择最接近的
        suitable_configs.sort_by_key(|config| config.channels());
        let best_config = suitable_configs[0];
        
        let sample_format = best_config.sample_format();
        let mut config = best_config.with_sample_rate(cpal::SampleRate(sample_rate));
        
        if sample_rate < best_config.min_sample_rate().0 || sample_rate > best_config.max_sample_rate().0 {
            config = best_config.with_max_sample_rate();
            println!("Using device sample rate: {} Hz", config.sample_rate().0);
        }
        
        println!("Selected {} channels for {} channel audio", best_config.channels(), source_channels);
        return Ok((audio_device, config.into(), sample_format));
    }
    
    // 策略3：如果都不满足，选择通道数最多的配置（并警告用户）
    configs.sort_by_key(|config| std::cmp::Reverse(config.channels()));
    let fallback_config = &configs[0];
    
    let sample_format = fallback_config.sample_format();
    let mut config = fallback_config.with_sample_rate(cpal::SampleRate(sample_rate));
    
    if sample_rate < fallback_config.min_sample_rate().0 || sample_rate > fallback_config.max_sample_rate().0 {
        config = fallback_config.with_max_sample_rate();
        println!("Using device sample rate: {} Hz", config.sample_rate().0);
    }
    
    println!("Warning: Using {} channels for {} channel audio (may cause downmixing)", 
             fallback_config.channels(), source_channels);
    
    Ok((audio_device, config.into(), sample_format))
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
        SampleFormat::U8 => create_stream::<u8>(device, config, buffer, channels),
        SampleFormat::I8 => create_stream::<i8>(device, config, buffer, channels),
        SampleFormat::I32 => create_stream::<i32>(device, config, buffer, channels),
        SampleFormat::U32 => create_stream::<u32>(device, config, buffer, channels),
        SampleFormat::F64 => create_stream::<f64>(device, config, buffer, channels),
        SampleFormat::I64 => create_stream::<i64>(device, config, buffer, channels),
        SampleFormat::U64 => create_stream::<u64>(device, config, buffer, channels),
        _ => Err(PlayerError::UnsupportedFormat(format!("Unsupported sample format: {:?}", sample_format))),
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
    
    println!("Stream config - Channels: {}, Sample rate: {}", output_channels, config.sample_rate.0);
    
    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            fill_audio_buffer(data, &buffer, output_channels, channels);
        },
        |err| {
            eprintln!("Stream error: {}", err);
        },
        None,
    ).map_err(|e| PlayerError::PlaybackError(e.to_string()))?;
    
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
    let mut buffer = buffer.lock().unwrap();
    
    // 填充输出缓冲区
    for frame in data.chunks_mut(output_channels) {
        if output_channels >= source_channels {
            // 情况1：输出通道数 >= 源通道数（upmix 或直接映射）
            let mut frame_samples = Vec::with_capacity(output_channels);
            
            // 获取源声道数据
            for i in 0..output_channels {
                if i < source_channels {
                    // 从缓冲区获取对应声道的样本
                    if let Some(audio_sample) = buffer.pop_front() {
                        frame_samples.push(audio_sample);
                    } else {
                        // 缓冲区为空，使用静音
                        frame_samples.push(0.0);
                    }
                } else if source_channels > 0 {
                    // 如果输出声道数多于源声道数，重复使用最后一个声道
                    let last_channel_idx = (i % source_channels).min(frame_samples.len() - 1);
                    frame_samples.push(frame_samples[last_channel_idx]);
                } else {
                    // 无源声道数据，使用静音
                    frame_samples.push(0.0);
                }
            }
            
            // 将处理好的样本写入输出
            for (sample, &audio_sample) in frame.iter_mut().zip(frame_samples.iter()) {
                *sample = T::from_sample(audio_sample);
            }
        } else {
            // 情况2：输出通道数 < 源通道数（downmix）
            // 先获取所有源通道的数据
            let mut source_samples = Vec::with_capacity(source_channels);
            for _ in 0..source_channels {
                if let Some(audio_sample) = buffer.pop_front() {
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
                } else {
                    // 其他情况：分布式映射
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
) -> Result<(), PlayerError> {
    let is_playing = Arc::new(AtomicBool::new(true));
    let is_finished = Arc::new(AtomicBool::new(false));

    let is_finished_clone = Arc::clone(&is_finished);

    let decode_thread = {
        let buffer = Arc::clone(&audio_buffer);
        let is_playing = Arc::clone(&is_playing);
        let is_finished = Arc::clone(&is_finished);
        
        thread::spawn(move || {
            let mut packet_count = 0;
            
            // 解码循环
            loop {
                if !is_playing.load(Ordering::Relaxed) {
                    break;
                }
                
                match audio_file.probed.format.next_packet() {
                    Ok(packet) => {
                        // 只处理选定轨道的数据包
                        if packet.track_id() != audio_file.track_id {
                            continue;
                        }
                        
                        match decoder.decode(&packet) {
                            Ok(decoded) => {
                                // 将解码后的音频数据写入缓冲区
                                if let Err(e) = write_audio_buffer(&buffer, &decoded, audio_file.info.channels) {
                                    eprintln!("Error writing to buffer: {}", e);
                                    break;
                                }
                                
                                packet_count += 1;
                                if packet_count % DECODE_PROGRESS_INTERVAL == 0 {
                                    print!(".");
                                    std::io::Write::flush(&mut std::io::stdout()).unwrap();
                                }
                            }
                            Err(e) => {
                                eprintln!("Decode error: {}", e);
                                break;
                            }
                        }
                    }
                    Err(_) => {
                        // 到达文件末尾
                        println!("\nFinished decoding audio file");
                        is_finished.store(true, Ordering::Relaxed);
                        break;
                    }
                }
            }
        })
    };

    // 等待解码完成或用户中断
    while !is_finished_clone.load(Ordering::Relaxed) {
        thread::sleep(Duration::from_millis(PLAYBACK_POLL_INTERVAL));
        
        // 检查缓冲区是否为空（播放完成）
        if is_finished_clone.load(Ordering::Relaxed) {
            let buffer_empty = {
                let buffer = audio_buffer.lock().unwrap();
                buffer.is_empty()
            };
            
            if buffer_empty {
                break;
            }
        }
    }
    
    // 等待一段时间让剩余音频播放完
    thread::sleep(Duration::from_millis(PLAYBACK_FINISH_DELAY));
    
    // 停止播放
    is_playing.store(false, Ordering::Relaxed);
    decode_thread.join().unwrap();
    
    Ok(())
}

fn write_audio_buffer(buffer: &AudioBuffer, decoded: &AudioBufferRef, target_channels: usize) -> Result<(), PlayerError> {
    // 创建输出缓冲区
    let mut output = vec![Vec::new(); target_channels];
    
    // 获取解码后的音频数据
    convert_samples_any(decoded, &mut output);
    
    // 写入缓冲区
    // 需要交错处理多通道数据
    if output.is_empty() {
        return Ok(());
    }
    
    let frame_count = output[0].len();
    for frame_idx in 0..frame_count {
        for channel_idx in 0..target_channels {
            let sample = if channel_idx < output.len() {
                output[channel_idx][frame_idx]
            } else {
                // 如果通道不足，使用第一个通道的数据
                output[0][frame_idx]
            };
            
            // 如果缓冲区太满，等待一下
            loop {
                {
                    let mut buffer = buffer.lock().unwrap();
                    if buffer.len() < buffer.capacity() - BUFFER_CAPACITY_THRESHOLD {
                        buffer.push_back(sample);
                        break;
                    }
                }
                thread::sleep(Duration::from_millis(BUFFER_WRITE_DELAY));
            }
        }
    }
    
    Ok(())
}

fn convert_samples_any(input: &AudioBufferRef<'_>, output: &mut [Vec<f32>]) {
    match input {
        AudioBufferRef::U8(input) => convert_samples(input, output),
        AudioBufferRef::U16(input) => convert_samples(input, output),
        AudioBufferRef::U24(input) => convert_samples(input, output),
        AudioBufferRef::U32(input) => convert_samples(input, output),
        AudioBufferRef::S8(input) => convert_samples(input, output),
        AudioBufferRef::S16(input) => convert_samples(input, output),
        AudioBufferRef::S24(input) => convert_samples(input, output),
        AudioBufferRef::S32(input) => convert_samples(input, output),
        AudioBufferRef::F32(input) => convert_samples(input, output),
        AudioBufferRef::F64(input) => convert_samples(input, output),
    }
}

fn convert_samples<S>(input: &symphonia::core::audio::AudioBuffer<S>, output: &mut [Vec<f32>])
where
    S: symphonia::core::sample::Sample + symphonia::core::conv::IntoSample<f32>,
{
    for (c, dst) in output.iter_mut().enumerate() {
        let src = input.chan(c);
        dst.extend(src.iter().map(|&s| s.into_sample()));
    }
}

fn calculate_audio_duration(track: &symphonia::core::formats::Track, sample_rate: u32) -> Option<f64> {
    // 尝试从轨道的时间基准和帧数计算时长
    if let Some(n_frames) = track.codec_params.n_frames {
        if let Some(time_base) = track.codec_params.time_base {
            // 使用时间基准更准确地计算时长
            let duration_seconds = n_frames as f64 * time_base.numer as f64 / time_base.denom as f64;
            return Some(duration_seconds);
        } else {
            // 如果没有时间基准，使用采样率计算
            return Some(n_frames as f64 / sample_rate as f64);
        }
    }
    
    // 如果没有帧数信息，尝试从其他源计算
    // 这里可以添加更多的时长计算方法
    
    None
}

fn format_duration(seconds: f64) -> String {
    let total_seconds = seconds as u64;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let secs = total_seconds % 60;
    let milliseconds = ((seconds - total_seconds as f64) * 1000.0) as u64;
    
    if hours > 0 {
        format!("{:02}:{:02}:{:02}.{:03}", hours, minutes, secs, milliseconds)
    } else {
        format!("{:02}:{:02}.{:03}", minutes, secs, milliseconds)
    }
}

fn list_audio_devices() {
    let host = cpal::default_host();
    
    println!("Available audio output devices:");
    
    match host.output_devices() {
        Ok(devices) => {
            for (i, device) in devices.enumerate() {
                let name = device.name().unwrap_or_else(|_| "Unnamed Device".into());
                println!("  {}: {}", i, name);
                
                // 显示设备支持的配置
                if let Ok(mut configs) = device.supported_output_configs() {
                    if let Some(config) = configs.next() {
                        println!("    - Sample rate: {} - {}", config.min_sample_rate().0, config.max_sample_rate().0);
                        println!("    - Channels: {}", config.channels());
                        println!("    - Format: {:?}", config.sample_format());
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Error listing devices: {}", e);
        }
    }
    
    // 显示默认设备
    if let Some(device) = host.default_output_device() {    
        let name = device.name().unwrap_or_else(|_| "Unnamed Device".into());
        println!("\nDefault output device: {}", name);
    }
}