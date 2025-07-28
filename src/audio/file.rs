//! 音频文件处理模块
//! 
//! 处理音频文件的打开、信息提取和时长计算。

use std::{fs::File, path::Path};
use symphonia::core::{
    codecs::CODEC_TYPE_NULL,
    formats::{FormatOptions, Track},
    io::MediaSourceStream,
    meta::MetadataOptions,
    probe::Hint,
};
use symphonia::default;

use crate::error::{PlayerError, Result};
use crate::config::audio::{MAX_ESTIMATION_PACKETS, DEFAULT_SAMPLE_RATE};

/// 音频文件信息
#[derive(Debug, Clone)]
pub struct AudioInfo {
    /// 声道数
    pub channels: usize,
    /// 采样率
    pub sample_rate: u32,
    /// 时长（秒）
    pub duration: Option<f64>,
    /// 每样本位数
    pub bits_per_sample: Option<u32>,
}

impl AudioInfo {
    /// 从音频轨道创建音频信息
    pub fn from_track(track: &Track) -> Self {
        let channels = track.codec_params.channels
            .map(|c| c.count())
            .unwrap_or(2);
        let sample_rate = track.codec_params.sample_rate
            .unwrap_or(DEFAULT_SAMPLE_RATE);
        let duration = calculate_audio_duration(track, sample_rate);
        let bits_per_sample = track.codec_params.bits_per_sample;

        Self {
            channels,
            sample_rate,
            duration,
            bits_per_sample,
        }
    }
    
    /// 从音频轨道和文件路径创建音频信息（包含时长估算）
    pub fn from_track_with_file_path(track: &Track, file_path: &str) -> Self {
        let mut info = Self::from_track(track);
        
        // 如果标准方法无法获取时长或时长为0，尝试通过解析文件来估算
        // 这对于m4s等流媒体片段文件特别有用
        if info.duration.is_none() || info.duration == Some(0.0) {
            info.duration = estimate_audio_duration_by_parsing(file_path);
        }
        
        info
    }
}

/// 音频文件结构体
pub struct AudioFile {
    /// 探测结果
    pub probed: symphonia::core::probe::ProbeResult,
    /// 音频轨道
    pub track: Track,
    /// 轨道ID
    pub track_id: u32,
    /// 音频信息
    pub info: AudioInfo,
}

impl AudioFile {
    /// 打开音频文件
    /// 
    /// # 参数
    /// * `file_path` - 音频文件路径
    /// 
    /// # 返回
    /// 成功时返回AudioFile实例，失败时返回错误
    pub fn open(file_path: &str) -> Result<Self> {
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
        let info = AudioInfo::from_track_with_file_path(&track, file_path);
        
        Ok(Self {
            probed,
            track,
            track_id,
            info,
        })
    }
    
    /// 获取音频文件信息（不创建完整的AudioFile实例）
    /// 
    /// # 参数
    /// * `file_path` - 音频文件路径
    /// 
    /// # 返回
    /// 成功时返回AudioInfo，失败时返回错误
    pub fn get_info(file_path: &str) -> Result<AudioInfo> {
        let audio_file = Self::open(file_path)?;
        Ok(audio_file.info)
    }
}

/// 创建文件提示
fn create_hint(file_path: &str) -> Hint {
    let mut hint = Hint::new();
    if let Some(extension) = Path::new(file_path).extension() {
        if let Some(ext_str) = extension.to_str() {
            hint.with_extension(ext_str);
        }
    }
    hint
}

/// 计算音频时长
fn calculate_audio_duration(track: &Track, sample_rate: u32) -> Option<f64> {
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

/// 通过解析整个文件来估算时长的函数
fn estimate_audio_duration_by_parsing(file_path: &str) -> Option<f64> {
    use symphonia::core::{
        codecs::DecoderOptions,
        formats::FormatOptions,
        meta::MetadataOptions,
    };
    
    // 尝试打开文件并计算样本数量
    let file = File::open(file_path).ok()?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    let hint = create_hint(file_path);
    
    let mut probed = default::get_probe()
        .format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default())
        .ok()?;
    
    let track = probed.format.tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)?
        .clone();
    
    let sample_rate = track.codec_params.sample_rate.unwrap_or(DEFAULT_SAMPLE_RATE);
    let mut decoder = default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .ok()?;
    
    let mut total_frames = 0u64;
    let mut packet_count = 0u64;
    
    // 尝试解析整个文件来获得准确时长
    // 对于m4s这样的文件，完整解析是值得的
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_hint() {
        let _hint = create_hint("test.mp3");
        // 测试hint是否正确创建，只要不panic就说明成功
        // 由于Hint的内部状态无法直接访问，我们只测试创建过程
    }
} 