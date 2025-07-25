//! 音频流处理模块
//! 
//! 处理音频流的创建和数据填充。

use cpal::{
    traits::DeviceTrait,
    Device, Sample, SampleFormat, SizedSample, Stream, StreamConfig,
    FromSample,
};

use crate::error::{PlayerError, Result};
use super::playback::AudioBuffer;

/// 创建音频流
/// 
/// # 参数
/// * `device` - 音频设备
/// * `config` - 流配置
/// * `sample_format` - 样本格式
/// * `buffer` - 音频缓冲区
/// * `channels` - 声道数
/// 
/// # 返回
/// 成功时返回音频流，失败时返回错误
pub fn create_audio_stream(
    device: &Device,
    config: &StreamConfig,
    sample_format: SampleFormat,
    buffer: AudioBuffer,
    channels: usize,
) -> Result<Stream> {
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

/// 创建特定类型的音频流
/// 
/// # 参数
/// * `device` - 音频设备
/// * `config` - 流配置
/// * `buffer` - 音频缓冲区
/// * `channels` - 源音频声道数
/// 
/// # 返回
/// 成功时返回音频流，失败时返回错误
pub fn create_stream<T>(
    device: &Device,
    config: &StreamConfig,
    buffer: AudioBuffer,
    channels: usize,
) -> Result<Stream>
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

/// 填充音频缓冲区
/// 
/// # 参数
/// * `data` - 输出数据缓冲区
/// * `buffer` - 音频缓冲区
/// * `output_channels` - 输出声道数
/// * `source_channels` - 源音频声道数
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
            for _i in 0..source_channels {
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