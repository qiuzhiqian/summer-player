//! 音频设备管理模块
//! 
//! 处理音频设备的列举、选择和配置。

use cpal::{
    traits::{DeviceTrait, HostTrait},
    Device, SampleFormat, StreamConfig,
};

use crate::error::{PlayerError, Result};

/// 列出所有可用的音频输出设备
pub fn list_audio_devices() {
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

/// 设置音频设备
/// 
/// # 参数
/// * `device_index` - 设备索引，None表示使用默认设备
/// * `sample_rate` - 所需采样率
/// * `source_channels` - 源音频声道数
/// 
/// # 返回
/// 返回设备、配置和样本格式
pub fn setup_audio_device(
    device_index: Option<usize>, 
    sample_rate: u32, 
    source_channels: usize
) -> Result<(Device, StreamConfig, SampleFormat)> {
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