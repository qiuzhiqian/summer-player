//! 音频播放控制模块
//! 
//! 处理音频播放状态管理和播放控制命令。

use std::{
    collections::VecDeque,
    sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}},
    thread,
    time::Duration,
};
use tokio::sync::mpsc;
use cpal::traits::StreamTrait;
use symphonia::core::audio::Signal;

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
}

impl Default for PlaybackState {
    fn default() -> Self {
        Self {
            is_playing: false,
            is_paused: false,
            current_time: 0.0,
            total_duration: 0.0,
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
    
    let is_playing = Arc::new(AtomicBool::new(true));
    let is_paused = Arc::new(AtomicBool::new(false));
    let should_stop = Arc::new(AtomicBool::new(false));
    
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
) -> Result<()> {
    let mut format = audio_file.probed.format;
    let track_id = audio_file.track_id;
    let target_channels = audio_file.info.channels;
    
    loop {
        if should_stop.load(Ordering::Relaxed) {
            break;
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
    use symphonia::core::audio::{AudioBuffer, AudioBufferRef};
    
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