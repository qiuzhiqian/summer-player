//! 音频播放控制模块
//! 
//! 处理音频播放状态管理和播放控制命令。

use std::{
    collections::VecDeque,
    sync::{Arc, Mutex, atomic::{AtomicBool, AtomicU64, Ordering}},
    thread,
    time::Duration,
};
use tokio::sync::mpsc;
use cpal::traits::StreamTrait;
use symphonia::core::audio::Signal;
use symphonia::core::formats::{SeekMode, SeekTo};

use crate::error::{PlayerError, Result};
use crate::config::{BUFFER_CAPACITY_THRESHOLD, BUFFER_WRITE_DELAY};
use super::{AudioFile, create_decoder, setup_audio_device, create_audio_stream};

/// 播放控制命令
#[derive(Debug, Clone)]
pub enum PlaybackCommand {
    /// 暂停播放
    Pause,
    /// 恢复播放
    Resume,
    /// 停止播放
    Stop,
    /// 跳转到指定时间（秒）
    Seek(f64),
}

/// 播放状态
#[derive(Debug, Clone)]
pub struct PlaybackState {
    /// 是否正在播放
    pub is_playing: bool,
    /// 是否已暂停
    pub is_paused: bool,
    /// 当前播放时间（秒）
    pub current_time: f64,
    /// 总时长（秒）
    pub total_duration: f64,
    /// 当前样本位置
    pub current_samples: u64,
    /// 样本率
    pub sample_rate: u32,
}

impl Default for PlaybackState {
    fn default() -> Self {
        Self {
            is_playing: false,
            is_paused: false,
            current_time: 0.0,
            total_duration: 0.0,
            current_samples: 0,
            sample_rate: 0,
        }
    }
}

/// 音频缓冲区类型
pub type AudioBuffer = Arc<Mutex<VecDeque<f32>>>;

/// 启动音频播放会话
/// 
/// # 参数
/// * `file_path` - 音频文件路径
/// 
/// # 返回
/// 返回命令发送器和播放任务句柄
pub async fn start_audio_playback(
    file_path: String
) -> (mpsc::UnboundedSender<PlaybackCommand>, tokio::task::JoinHandle<()>) {
    let (command_sender, command_receiver) = mpsc::unbounded_channel();
    
    let handle = tokio::task::spawn_blocking(move || {
        tokio::runtime::Handle::current().block_on(async move {
            if let Err(e) = run_audio_playback_with_control(&file_path, None, command_receiver).await {
                eprintln!("Audio playback error: {}", e);
            }
        })
    });
    
    (command_sender, handle)
}

/// 启动音频播放会话（带状态更新）
/// 
/// # 参数
/// * `file_path` - 音频文件路径
/// * `state_sender` - 播放状态发送器
/// 
/// # 返回
/// 返回命令发送器和播放任务句柄
pub async fn start_audio_playback_with_state(
    file_path: String,
    state_sender: mpsc::UnboundedSender<PlaybackState>,
) -> (mpsc::UnboundedSender<PlaybackCommand>, tokio::task::JoinHandle<()>) {
    let (command_sender, command_receiver) = mpsc::unbounded_channel();
    
    let handle = tokio::task::spawn_blocking(move || {
        tokio::runtime::Handle::current().block_on(async move {
            if let Err(e) = run_audio_playback_with_control_and_state(&file_path, None, command_receiver, state_sender).await {
                eprintln!("Audio playback error: {}", e);
            }
        })
    });
    
    (command_sender, handle)
}

/// 音频播放控制函数
/// 
/// # 参数
/// * `file_path` - 音频文件路径  
/// * `device_index` - 音频设备索引
/// * `command_receiver` - 命令接收器
pub async fn run_audio_playback_with_control(
    file_path: &str,
    device_index: Option<usize>,
    mut command_receiver: mpsc::UnboundedReceiver<PlaybackCommand>,
) -> Result<()> {
    let audio_file = AudioFile::open(file_path)?;
    let decoder = create_decoder(&audio_file.track)?;
    let (device, config, sample_format) = setup_audio_device(
        device_index, 
        audio_file.info.sample_rate, 
        audio_file.info.channels
    )?;
    
    let buffer_size = calculate_buffer_size(audio_file.info.sample_rate, audio_file.info.channels);
    let audio_buffer: AudioBuffer = Arc::new(Mutex::new(VecDeque::with_capacity(buffer_size)));
    
    let stream = create_audio_stream(&device, &config, sample_format, audio_buffer.clone(), audio_file.info.channels)?;
    
    let _is_playing = Arc::new(AtomicBool::new(true));
    let is_paused = Arc::new(AtomicBool::new(false));
    let should_stop = Arc::new(AtomicBool::new(false));
    
    // 添加跳转目标时间的原子变量（以秒为单位，乘以1000存储为毫秒以保持精度）
    let seek_target_ms = Arc::new(AtomicU64::new(u64::MAX)); // u64::MAX 表示没有跳转请求
    
    // 创建音频流的暂停/恢复控制
    let _stream_is_paused = is_paused.clone();
    
    stream.play().map_err(|e| PlayerError::PlaybackError(e.to_string()))?;
    
    let playback_thread = {
        let audio_buffer = audio_buffer.clone();
        let should_stop = should_stop.clone();
        let is_paused = is_paused.clone();
        let seek_target_ms = seek_target_ms.clone();
        
        thread::spawn(move || {
            let _ = run_playback_loop(audio_file, decoder, audio_buffer, should_stop, is_paused, seek_target_ms);
        })
    };
    
    // 处理播放控制命令
    while let Some(command) = command_receiver.recv().await {
        match command {
            PlaybackCommand::Pause => {
                is_paused.store(true, Ordering::Relaxed);
                // 暂停音频流
                if let Err(e) = stream.pause() {
                    eprintln!("Failed to pause stream: {}", e);
                }
            }
            PlaybackCommand::Resume => {
                is_paused.store(false, Ordering::Relaxed);
                // 恢复音频流
                if let Err(e) = stream.play() {
                    eprintln!("Failed to resume stream: {}", e);
                }
            }
            PlaybackCommand::Stop => {
                should_stop.store(true, Ordering::Relaxed);
                break;
            }
            PlaybackCommand::Seek(target_time) => {
                // 将跳转目标时间转换为毫秒并存储
                let target_ms = (target_time * 1000.0) as u64;
                seek_target_ms.store(target_ms, Ordering::Relaxed);
                println!("Seek request: {:.2}s", target_time);
            }
        }
    }
    
    should_stop.store(true, Ordering::Relaxed);
    playback_thread.join().unwrap();
    
    Ok(())
}

/// 音频播放控制函数（带状态更新）
/// 
/// # 参数
/// * `file_path` - 音频文件路径  
/// * `device_index` - 音频设备索引
/// * `command_receiver` - 命令接收器
/// * `state_sender` - 播放状态发送器
pub async fn run_audio_playback_with_control_and_state(
    file_path: &str,
    device_index: Option<usize>,
    mut command_receiver: mpsc::UnboundedReceiver<PlaybackCommand>,
    state_sender: mpsc::UnboundedSender<PlaybackState>,
) -> Result<()> {
    let audio_file = AudioFile::open(file_path)?;
    let decoder = create_decoder(&audio_file.track)?;
    let (device, config, sample_format) = setup_audio_device(
        device_index, 
        audio_file.info.sample_rate, 
        audio_file.info.channels
    )?;
    
    let buffer_size = calculate_buffer_size(audio_file.info.sample_rate, audio_file.info.channels);
    let audio_buffer: AudioBuffer = Arc::new(Mutex::new(VecDeque::with_capacity(buffer_size)));
    
    let stream = create_audio_stream(&device, &config, sample_format, audio_buffer.clone(), audio_file.info.channels)?;
    
    let _is_playing = Arc::new(AtomicBool::new(true));
    let is_paused = Arc::new(AtomicBool::new(false));
    let should_stop = Arc::new(AtomicBool::new(false));
    
    // 添加跳转目标时间的原子变量（以秒为单位，乘以1000存储为毫秒以保持精度）
    let seek_target_ms = Arc::new(AtomicU64::new(u64::MAX)); // u64::MAX 表示没有跳转请求
    
    // 用于跟踪当前播放位置的原子变量
    let current_samples = Arc::new(AtomicU64::new(0));
    
    // 创建音频流的暂停/恢复控制
    let _stream_is_paused = is_paused.clone();
    
    stream.play().map_err(|e| PlayerError::PlaybackError(e.to_string()))?;
    
    // 提取播放会话所需的信息
    let audio_sample_rate = audio_file.info.sample_rate;
    let total_duration = audio_file.info.duration.unwrap_or(0.0);
    
    // 创建初始播放状态
    let initial_state = PlaybackState {
        is_playing: true,
        is_paused: false,
        current_time: 0.0,
        total_duration,
        current_samples: 0,
        sample_rate: audio_sample_rate,
    };
    
    // 发送初始状态
    let _ = state_sender.send(initial_state);
    
    let playback_thread = {
        let audio_buffer = audio_buffer.clone();
        let should_stop = should_stop.clone();
        let is_paused = is_paused.clone();
        let seek_target_ms = seek_target_ms.clone();
        let current_samples = current_samples.clone();
        let state_sender = state_sender.clone();
        
        thread::spawn(move || {
            let _ = run_playback_loop_with_state(
                audio_file, 
                decoder, 
                audio_buffer, 
                should_stop, 
                is_paused, 
                seek_target_ms,
                current_samples,
                state_sender,
                audio_sample_rate,
                total_duration,
            );
        })
    };
    
    // 处理播放控制命令
    while let Some(command) = command_receiver.recv().await {
        match command {
            PlaybackCommand::Pause => {
                is_paused.store(true, Ordering::Relaxed);
                // 暂停音频流
                if let Err(e) = stream.pause() {
                    eprintln!("Failed to pause stream: {}", e);
                }
                
                // 发送暂停状态更新
                let current_pos = current_samples.load(Ordering::Relaxed);
                let current_time = current_pos as f64 / audio_sample_rate as f64;
                let state = PlaybackState {
                    is_playing: false,
                    is_paused: true,
                    current_time,
                    total_duration,
                    current_samples: current_pos,
                    sample_rate: audio_sample_rate,
                };
                let _ = state_sender.send(state);
            }
            PlaybackCommand::Resume => {
                is_paused.store(false, Ordering::Relaxed);
                // 恢复音频流
                if let Err(e) = stream.play() {
                    eprintln!("Failed to resume stream: {}", e);
                }
                
                // 发送恢复状态更新
                let current_pos = current_samples.load(Ordering::Relaxed);
                let current_time = current_pos as f64 / audio_sample_rate as f64;
                let state = PlaybackState {
                    is_playing: true,
                    is_paused: false,
                    current_time,
                    total_duration,
                    current_samples: current_pos,
                    sample_rate: audio_sample_rate,
                };
                let _ = state_sender.send(state);
            }
            PlaybackCommand::Stop => {
                should_stop.store(true, Ordering::Relaxed);
                break;
            }
            PlaybackCommand::Seek(target_time) => {
                // 将跳转目标时间转换为毫秒并存储
                let target_ms = (target_time * 1000.0) as u64;
                seek_target_ms.store(target_ms, Ordering::Relaxed);
                println!("Seek request: {:.2}s", target_time);
            }
        }
    }
    
    should_stop.store(true, Ordering::Relaxed);
    playback_thread.join().unwrap();
    
    Ok(())
}

/// 播放循环
fn run_playback_loop(
    audio_file: AudioFile,
    mut decoder: Box<dyn symphonia::core::codecs::Decoder>,
    audio_buffer: AudioBuffer,
    should_stop: Arc<AtomicBool>,
    is_paused: Arc<AtomicBool>,
    seek_target_ms: Arc<AtomicU64>,
) -> Result<()> {
    let mut format = audio_file.probed.format;
    let track_id = audio_file.track_id;
    let target_channels = audio_file.info.channels;
    let sample_rate = audio_file.info.sample_rate;
    
    // 跟踪当前播放位置（以样本数计算）
    let mut _current_samples: u64 = 0;
    
    loop {
        if should_stop.load(Ordering::Relaxed) {
            break;
        }
        
        // 检查是否有跳转请求
        let seek_target = seek_target_ms.load(Ordering::Relaxed);
        if seek_target != u64::MAX {
            // 执行跳转操作
            if let Err(e) = perform_seek(&mut format, &mut decoder, seek_target, sample_rate, track_id, &audio_buffer) {
                eprintln!("Seek failed: {}", e);
            } else {
                // 跳转成功，更新当前样本位置
                _current_samples = (seek_target * sample_rate as u64) / 1000;
                println!("Seek completed to {:.2}s", seek_target as f64 / 1000.0);
            }
            // 清除跳转请求
            seek_target_ms.store(u64::MAX, Ordering::Relaxed);
        }
        
        // 如果暂停，等待一小段时间后重新检查
        if is_paused.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_millis(10));
            continue;
        }
        
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(symphonia::core::errors::Error::IoError(ref e)) 
                if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                break;
            }
            Err(e) => return Err(PlayerError::DecodingError(e.to_string())),
        };
        
        if packet.track_id() != track_id {
            continue;
        }
        
        let decoded = decoder.decode(&packet)
            .map_err(|e| PlayerError::DecodingError(e.to_string()))?;
        
        // 更新当前样本位置
        _current_samples += decoded.frames() as u64;
        
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

/// 播放循环（带状态更新）
fn run_playback_loop_with_state(
    audio_file: AudioFile,
    mut decoder: Box<dyn symphonia::core::codecs::Decoder>,
    audio_buffer: AudioBuffer,
    should_stop: Arc<AtomicBool>,
    is_paused: Arc<AtomicBool>,
    seek_target_ms: Arc<AtomicU64>,
    current_samples_atomic: Arc<AtomicU64>,
    state_sender: mpsc::UnboundedSender<PlaybackState>,
    _sample_rate: u32,
    total_duration: f64,
) -> Result<()> {
    let mut format = audio_file.probed.format;
    let track_id = audio_file.track_id;
    let target_channels = audio_file.info.channels;
    let audio_sample_rate = audio_file.info.sample_rate;
    
    // 跟踪当前播放位置（以样本数计算）
    let mut _current_samples: u64 = 0;
    
    loop {
        if should_stop.load(Ordering::Relaxed) {
            break;
        }
        
        // 检查是否有跳转请求
        let seek_target = seek_target_ms.load(Ordering::Relaxed);
        if seek_target != u64::MAX {
            // 执行跳转操作
            if let Err(e) = perform_seek(&mut format, &mut decoder, seek_target, audio_sample_rate, track_id, &audio_buffer) {
                eprintln!("Seek failed: {}", e);
            } else {
                // 跳转成功，更新当前样本位置
                _current_samples = (seek_target * audio_sample_rate as u64) / 1000;
                current_samples_atomic.store(_current_samples, Ordering::Relaxed);
                println!("Seek completed to {:.2}s", seek_target as f64 / 1000.0);
            }
            // 清除跳转请求
            seek_target_ms.store(u64::MAX, Ordering::Relaxed);
        }
        
        // 如果暂停，等待一小段时间后重新检查
        if is_paused.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_millis(10));
            continue;
        }
        
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(symphonia::core::errors::Error::IoError(ref e)) 
                if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                break;
            }
            Err(e) => return Err(PlayerError::DecodingError(e.to_string())),
        };
        
        if packet.track_id() != track_id {
            continue;
        }
        
        let decoded = decoder.decode(&packet)
            .map_err(|e| PlayerError::DecodingError(e.to_string()))?;
        
        // 更新当前样本位置
        _current_samples += decoded.frames() as u64;
        current_samples_atomic.store(_current_samples, Ordering::Relaxed);
        
        write_audio_buffer(&audio_buffer, &decoded, target_channels)?;
        
        // 控制缓冲区大小
        {
            let buffer = audio_buffer.lock().unwrap();
            if buffer.len() > BUFFER_CAPACITY_THRESHOLD {
                drop(buffer);
                thread::sleep(Duration::from_millis(BUFFER_WRITE_DELAY));
            }
        }

        // 每隔一定数量的帧发送状态更新（避免过于频繁的更新）
        if _current_samples % (audio_sample_rate as u64 / 10) == 0 { // 每100ms更新一次
            let current_time = _current_samples as f64 / audio_sample_rate as f64;
            let state = PlaybackState {
                is_playing: !is_paused.load(Ordering::Relaxed),
                is_paused: is_paused.load(Ordering::Relaxed),
                current_time,
                total_duration,
                current_samples: _current_samples,
                sample_rate: audio_sample_rate,
            };
            let _ = state_sender.send(state);
        }
    }
    
    Ok(())
}

/// 执行音频跳转操作
fn perform_seek(
    format: &mut Box<dyn symphonia::core::formats::FormatReader>,
    decoder: &mut Box<dyn symphonia::core::codecs::Decoder>,
    target_time_ms: u64,
    sample_rate: u32,
    track_id: u32,
    audio_buffer: &AudioBuffer,
) -> Result<()> {
    // 清空音频缓冲区
    {
        let mut buffer = audio_buffer.lock().unwrap();
        buffer.clear();
    }
    
    // 计算目标时间戳（以timebase为单位）
    // 对于大多数格式，timebase通常是样本率
    let target_ts = (target_time_ms * sample_rate as u64) / 1000;
    
    // 执行跳转
    match format.seek(SeekMode::Accurate, SeekTo::TimeStamp { ts: target_ts, track_id }) {
        Ok(seeked_to) => {
            let actual_seconds = seeked_to.actual_ts as f64 / sample_rate as f64;
            println!("Seeked to: {:.2}s (requested: {:.2}s)", 
                actual_seconds,
                target_time_ms as f64 / 1000.0);
            
            // 重置解码器状态
            decoder.reset();
            
            // 预加载一些数据到缓冲区
            for _ in 0..5 { // 预加载几个包
                if let Ok(packet) = format.next_packet() {
                    if packet.track_id() == track_id {
                        if let Ok(decoded) = decoder.decode(&packet) {
                            let _ = write_audio_buffer(audio_buffer, &decoded, decoded.spec().channels.count());
                        }
                    }
                }
            }
            
            Ok(())
        }
        Err(e) => {
            Err(PlayerError::PlaybackError(format!("Seek failed: {}", e)))
        }
    }
}

/// 计算缓冲区大小
fn calculate_buffer_size(sample_rate: u32, channels: usize) -> usize {
    use crate::config::DEFAULT_BUFFER_MULTIPLIER;
    sample_rate as usize * channels * DEFAULT_BUFFER_MULTIPLIER
}

/// 写入音频缓冲区
fn write_audio_buffer(
    buffer: &AudioBuffer, 
    decoded: &symphonia::core::audio::AudioBufferRef, 
    target_channels: usize
) -> Result<()> {
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

/// 转换音频样本
fn convert_samples_any(input: &symphonia::core::audio::AudioBufferRef<'_>, output: &mut [Vec<f32>]) {
    use symphonia::core::audio::AudioBufferRef;
    
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

/// 转换特定类型的音频样本
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