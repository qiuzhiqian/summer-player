use std::{fs::File, path::Path, sync::Arc, time::Duration};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::collections::VecDeque;
use std::thread;

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

    // 验证文件是否存在
    if !Path::new(&file_path).exists() {
        eprintln!("File not found: {}", file_path);
        std::process::exit(1);
    }

    if cli.info {
        // 只显示音频信息而不播放
        match get_audio_info(&file_path) {
            Ok(_) => {},
            Err(e) => {
                eprintln!("Error reading audio file: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        println!("Playing: {}", file_path);
        
        // 加载并播放音频文件
        match play_audio(&file_path, cli.device) {
            Ok(_) => println!("Playback finished successfully"),
            Err(e) => {
                eprintln!("Error during playback: {}", e);
                std::process::exit(1);
            }
        }
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

fn play_audio(file_path: &str, device_index: Option<usize>) -> Result<(), Box<dyn std::error::Error>> {
    // 打开音频文件
    let file = File::open(file_path)?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    
    // 创建提示信息
    let mut hint = Hint::new();
    if let Some(ext) = Path::new(file_path).extension() {
        if let Some(ext_str) = ext.to_str() {
            hint.with_extension(ext_str);
        }
    }
    
    // 探测音频格式
    let mut probed = default::get_probe()
        .format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default())?;
    
    // 显示元数据
    if let Some(metadata) = probed.metadata.get() {
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
    
    // 获取音频轨道
    let track = probed
        .format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or("No supported audio tracks found")?;
    
    let track_id = track.id;
    
    // 获取音频参数
    let channels = track.codec_params.channels.map(|c| c.count()).unwrap_or(2);
    let sample_rate = track.codec_params.sample_rate.unwrap_or(44100);
    
    // 计算并显示音频时长
    if let Some(duration) = calculate_audio_duration(track, sample_rate) {
        println!("Duration: {}", format_duration(duration));
    }
    
    println!("Audio info: {} channels, {} Hz", channels, sample_rate);
    
    // 创建解码器
    let mut decoder = default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())?;
    
    // 获取音频设备
    let host = cpal::default_host();
    let audio_device = if let Some(index) = device_index {
        let mut devices = host.output_devices()?;
        devices.nth(index).ok_or("Device index out of range")?
    } else {
        host.default_output_device().ok_or("No default audio device")?
    };
    
    println!("Using audio device: {}", audio_device.name().unwrap_or("Unknown".to_string()));
    
    // 获取设备支持的配置
    let mut supported_configs = audio_device.supported_output_configs()?;
    let supported_config = supported_configs
        .next()
        .ok_or("No supported output config")?;
    
    // 创建音频流配置，尽量匹配音频文件的参数
    let mut config = supported_config.with_sample_rate(cpal::SampleRate(sample_rate));
    
    // 如果设备不支持文件的采样率，使用设备的默认采样率
    if sample_rate < supported_config.min_sample_rate().0 || sample_rate > supported_config.max_sample_rate().0 {
        config = supported_config.with_max_sample_rate();
        println!("Using device sample rate: {} Hz", config.sample_rate().0);
    }
    
    // 创建音频缓冲区 (存储约1秒的音频数据)
    let buffer_size = sample_rate as usize * channels * 2;
    let audio_buffer: AudioBuffer = Arc::new(Mutex::new(VecDeque::with_capacity(buffer_size)));
    
    // 创建用于控制播放的共享状态
    let is_playing = Arc::new(AtomicBool::new(true));
    let is_finished = Arc::new(AtomicBool::new(false));
    
    // 克隆用于不同线程的引用
    let buffer_clone = Arc::clone(&audio_buffer);
    let is_finished_clone = Arc::clone(&is_finished);
    
    // 创建音频流
    let stream = match config.sample_format() {
        SampleFormat::F32 => create_stream::<f32>(&audio_device, &config.into(), buffer_clone, channels as usize)?,
        SampleFormat::I16 => create_stream::<i16>(&audio_device, &config.into(), buffer_clone, channels as usize)?,
        SampleFormat::U16 => create_stream::<u16>(&audio_device, &config.into(), buffer_clone, channels as usize)?,
        SampleFormat::U8 => create_stream::<u8>(&audio_device, &config.into(), buffer_clone, channels as usize)?,
        SampleFormat::I8 => create_stream::<i8>(&audio_device, &config.into(), buffer_clone, channels as usize)?,
        SampleFormat::I32 => create_stream::<i32>(&audio_device, &config.into(), buffer_clone, channels as usize)?,
        SampleFormat::U32 => create_stream::<u32>(&audio_device, &config.into(), buffer_clone, channels as usize)?,
        SampleFormat::F64 => create_stream::<f64>(&audio_device, &config.into(), buffer_clone, channels as usize)?,
        SampleFormat::I64 => create_stream::<i64>(&audio_device, &config.into(), buffer_clone, channels as usize)?,
        SampleFormat::U64 => create_stream::<u64>(&audio_device, &config.into(), buffer_clone, channels as usize)?,
        _ => return Err(format!("Unsupported sample format: {:?}", &config.sample_format()).into()),
    };
    
    // 开始播放
    stream.play()?;
    
    // 在单独的线程中进行解码
    let decode_thread = {
        let buffer = Arc::clone(&audio_buffer);
        let is_playing = Arc::clone(&is_playing);
        let is_finished: Arc<AtomicBool> = Arc::clone(&is_finished);
        
        thread::spawn(move || {
            let mut packet_count = 0;
            
            // 解码循环
            loop {
                if !is_playing.load(Ordering::Relaxed) {
                    break;
                }
                
                match probed.format.next_packet() {
                    Ok(packet) => {
                        // 只处理选定轨道的数据包
                        if packet.track_id() != track_id {
                            continue;
                        }
                        
                        match decoder.decode(&packet) {
                            Ok(decoded) => {
                                // 将解码后的音频数据写入缓冲区
                                if let Err(e) = write_audio_buffer(&buffer, &decoded, channels) {
                                    eprintln!("Error writing to buffer: {}", e);
                                    break;
                                }
                                
                                packet_count += 1;
                                if packet_count % 100 == 0 {
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
        thread::sleep(Duration::from_millis(100));
        
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
    thread::sleep(Duration::from_millis(500));
    
    // 停止播放
    is_playing.store(false, Ordering::Relaxed);
    decode_thread.join().unwrap();
    drop(stream);
    
    Ok(())
}

fn get_audio_info(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // 打开音频文件
    let file = File::open(file_path)?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    
    // 创建提示信息
    let mut hint = Hint::new();
    if let Some(ext) = Path::new(file_path).extension() {
        if let Some(ext_str) = ext.to_str() {
            hint.with_extension(ext_str);
        }
    }
    
    // 探测音频格式
    let mut probed = default::get_probe()
        .format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default())?;
    
    println!("File: {}", file_path);
    
    // 显示元数据
    if let Some(metadata) = probed.metadata.get() {
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
    
    // 获取音频轨道
    let track = probed
        .format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or("No supported audio tracks found")?;
    
    // 获取音频参数
    let channels = track.codec_params.channels.map(|c| c.count()).unwrap_or(2);
    let sample_rate = track.codec_params.sample_rate.unwrap_or(44100);
    
    // 计算并显示音频时长
    if let Some(duration) = calculate_audio_duration(track, sample_rate) {
        println!("Duration: {}", format_duration(duration));
    } else {
        println!("Duration: Unknown");
    }
    
    println!("Audio info: {} channels, {} Hz", channels, sample_rate);
    
    if let Some(bits_per_sample) = track.codec_params.bits_per_sample {
        println!("Bits per sample: {}", bits_per_sample);
    }
    
    if let Some(frames) = track.codec_params.n_frames {
        println!("Total frames: {}", frames);
    }
    
    Ok(())
}

fn write_audio_buffer(buffer: &AudioBuffer, decoded: &AudioBufferRef, target_channels: usize) -> Result<(), Box<dyn std::error::Error>> {
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
                    if buffer.len() < buffer.capacity() - 1000 {
                        buffer.push_back(sample);
                        break;
                    }
                }
                thread::sleep(Duration::from_millis(1));
            }
        }
    }
    
    Ok(())
}

fn create_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    buffer: AudioBuffer,
    channels: usize,
) -> Result<cpal::Stream, Box<dyn std::error::Error>>
where
    T: Sample + SizedSample + FromSample<f32> + Send + 'static,
{
    let sample_rate = config.sample_rate.0;
    let output_channels = config.channels as usize;
    
    println!("Stream config - Channels: {}, Sample rate: {}", output_channels, sample_rate);
    
    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            let mut buffer = buffer.lock().unwrap();
            
            // 填充输出缓冲区
            for frame in data.chunks_mut(output_channels) {
                for (i, sample) in frame.iter_mut().enumerate() {
                    if let Some(audio_sample) = buffer.pop_front() {
                        *sample = T::from_sample(audio_sample);
                    } else {
                        // 缓冲区为空，输出静音
                        *sample = T::from_sample(0.0);
                    }
                    
                    // 如果音频源的通道数少于输出通道数，重复使用最后一个通道
                    if i >= channels && channels > 0 {
                        // 从缓冲区获取最后一个样本重复使用
                        if let Some(last_sample) = buffer.pop_front() {
                            *sample = T::from_sample(last_sample);
                        }
                    }
                }
            }
        },
        |err| {
            eprintln!("Stream error: {}", err);
        },
        None,
    )?;
    
    Ok(stream)
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