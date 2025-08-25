//! 音频文件处理模块
//! 
//! 处理音频文件的打开、信息提取和时长计算。

use std::{fs::File, path::Path, collections::HashMap};
use symphonia::core::{
    codecs::CODEC_TYPE_NULL,
    formats::{FormatOptions, Track},
    io::MediaSourceStream,
    meta::{MetadataOptions, Value},
    probe::Hint,
};
use symphonia::default;

use crate::error::{PlayerError, Result};
use crate::config::audio::{MAX_ESTIMATION_PACKETS, DEFAULT_SAMPLE_RATE};

/// 封面图片信息
#[derive(Debug, Clone)]
pub struct CoverArt {
    /// 图片数据
    pub data: Vec<u8>,
    /// MIME类型 (如 "image/jpeg", "image/png")
    pub mime_type: String,
    /// 描述
    pub description: Option<String>,
}

/// 音频元数据信息
#[derive(Debug, Clone, Default)]
pub struct AudioMetadata {
    /// 艺术家
    pub artist: Option<String>,
    /// 专辑
    pub album: Option<String>,
    /// 标题
    pub title: Option<String>,
    /// 年份
    pub year: Option<String>,
    /// 流派
    pub genre: Option<String>,
    /// 音轨号
    pub track_number: Option<String>,
    /// 专辑艺术家
    pub album_artist: Option<String>,
    /// 作曲家
    pub composer: Option<String>,
    /// 评论
    pub comment: Option<String>,
    /// 封面图片
    pub cover_art: Option<CoverArt>,
    /// 内嵌歌词信息
    pub embedded_lyrics: Vec<EmbeddedLyrics>,
    /// 其他标签
    pub other_tags: HashMap<String, String>,
}

/// 内嵌歌词信息
#[derive(Debug, Clone)]
pub struct EmbeddedLyrics {
    /// 歌词内容
    pub content: String,
    /// 语言代码 (ISO 639-2)
    pub language: Option<String>,
    /// 歌词描述/标题
    pub description: Option<String>,
    /// 歌词类型
    pub lyrics_type: LyricsType,
}

/// 歌词类型
#[derive(Debug, Clone, PartialEq)]
pub enum LyricsType {
    /// 非同步歌词 (USLT)
    Unsynchronized,
    /// 同步歌词 (SYLT) 
    Synchronized,
    /// LRC格式歌词
    Lrc,
    /// 其他格式
    Other(String),
}

impl AudioMetadata {
    /// 从symphonia metadata创建
    pub fn from_symphonia_metadata(metadata: &symphonia::core::meta::MetadataRevision) -> Self {
        let mut audio_metadata = AudioMetadata::default();
        
        // 首先处理视觉数据（封面）
        for visual in metadata.visuals() {
            // 检查usage字段来确定这是封面图片
            if let Some(usage) = &visual.usage {
                use symphonia::core::meta::StandardVisualKey;
                if matches!(usage, StandardVisualKey::FrontCover | StandardVisualKey::BackCover) {
                    // 只有在还没有封面的时候才设置，优先使用前封面
                    if audio_metadata.cover_art.is_none() || matches!(usage, StandardVisualKey::FrontCover) {
                        let mime_type = if visual.media_type.is_empty() {
                            detect_image_format(&visual.data)
                        } else {
                            visual.media_type.clone()
                        };
                        
                        if !mime_type.is_empty() {
                            audio_metadata.cover_art = Some(CoverArt {
                                data: visual.data.to_vec(),
                                mime_type,
                                description: None, // Visual结构体可能没有description字段
                            });
                        }
                    }
                }
            } else {
                // 如果没有usage字段，尝试将其作为封面（通常是第一个视觉数据）
                if audio_metadata.cover_art.is_none() {
                    let mime_type = if visual.media_type.is_empty() {
                        detect_image_format(&visual.data)
                    } else {
                        visual.media_type.clone()
                    };
                    
                    if !mime_type.is_empty() {
                        audio_metadata.cover_art = Some(CoverArt {
                            data: visual.data.to_vec(),
                            mime_type,
                            description: None,
                        });
                    }
                }
            }
        }
        
        // 然后处理文本标签
        for tag in metadata.tags() {
            let value = match &tag.value {
                Value::String(s) => s.clone(),
                Value::Binary(_) => continue, // 跳过二进制数据，视觉数据已经在上面处理了
                _ => tag.value.to_string(),
            };
            
            // 首先检查歌词相关的标签
            if let Some(lyrics) = Self::extract_lyrics_from_tag(&tag.key, &value) {
                audio_metadata.embedded_lyrics.push(lyrics);
                continue;
            }
            
            // 使用std_key进行标准化匹配
            if let Some(std_key) = &tag.std_key {
                use symphonia::core::meta::StandardTagKey;
                
                match std_key {
                    StandardTagKey::Artist => audio_metadata.artist = Some(value),
                    StandardTagKey::Album => audio_metadata.album = Some(value),
                    StandardTagKey::TrackTitle => audio_metadata.title = Some(value),
                    StandardTagKey::Date => audio_metadata.year = Some(value),
                    StandardTagKey::Genre => audio_metadata.genre = Some(value),
                    StandardTagKey::TrackNumber => audio_metadata.track_number = Some(value),
                    StandardTagKey::AlbumArtist => audio_metadata.album_artist = Some(value),
                    StandardTagKey::Composer => audio_metadata.composer = Some(value),
                    StandardTagKey::Comment => audio_metadata.comment = Some(value),
                    _ => {
                        // 对于其他标准键，使用其调试字符串表示作为键名
                        audio_metadata.other_tags.insert(format!("{:?}", std_key), value);
                    }
                }
            } else {
                // 如果没有std_key，回退到使用原始key
                let key = &tag.key;
                audio_metadata.other_tags.insert(key.clone(), value);
            }
        }
        
        audio_metadata
    }

    /// 从标签中提取歌词信息
    fn extract_lyrics_from_tag(key: &str, value: &str) -> Option<EmbeddedLyrics> {
        // 转换为大写以便匹配
        let key_upper = key.to_uppercase();
        
        match key_upper.as_str() {
            // ID3v2.3/2.4 非同步歌词 (USLT)
            "USLT" => Some(EmbeddedLyrics {
                content: value.to_string(),
                language: None, // TODO: 解析语言信息
                description: None,
                lyrics_type: LyricsType::Unsynchronized,
            }),
            
            // ID3v2.3/2.4 同步歌词 (SYLT)
            "SYLT" => Some(EmbeddedLyrics {
                content: value.to_string(),
                language: None,
                description: None,
                lyrics_type: LyricsType::Synchronized,
            }),
            
            // 常见的歌词标签变体
            "LYRIC" => Some(EmbeddedLyrics {
                content: value.to_string(),
                language: None,
                description: None,
                lyrics_type: LyricsType::Unsynchronized,
            }),
            
            // iTunes/MP4 歌词标签
            "©LYR" | "LYR" => Some(EmbeddedLyrics {
                content: value.to_string(),
                language: None,
                description: Some("iTunes Lyrics".to_string()),
                lyrics_type: LyricsType::Unsynchronized,
            }),
            
            // FLAC/Vorbis 歌词标签
            "LYRICS" | "UNSYNCEDLYRICS" => Some(EmbeddedLyrics {
                content: value.to_string(),
                language: None,
                description: None,
                lyrics_type: LyricsType::Unsynchronized,
            }),
            
            // 检查是否包含LRC格式内容
            _ if Self::is_lrc_content(value) => Some(EmbeddedLyrics {
                content: value.to_string(),
                language: None,
                description: Some(format!("LRC from {}", key)),
                lyrics_type: LyricsType::Lrc,
            }),
            
            _ => None,
        }
    }
    
    /// 检查内容是否是LRC格式
    fn is_lrc_content(content: &str) -> bool {
        // 简单检查：是否包含LRC时间标签格式 [mm:ss.xx]
        content.contains("[") && 
        content.contains("]") && 
        content.lines().any(|line| {
            line.trim_start().starts_with("[") && 
            line.contains(":") && 
            line.contains("]")
        })
    }
}

/// 检测图片格式
fn detect_image_format(data: &[u8]) -> String {
    if data.len() < 8 {
        return String::new();
    }
    
    // JPEG格式检测 (FF D8 FF)
    if data.len() >= 3 && data[0] == 0xFF && data[1] == 0xD8 && data[2] == 0xFF {
        return "image/jpeg".to_string();
    }
    
    // PNG格式检测 (89 50 4E 47 0D 0A 1A 0A)
    if data.len() >= 8 && 
       data[0] == 0x89 && data[1] == 0x50 && data[2] == 0x4E && data[3] == 0x47 &&
       data[4] == 0x0D && data[5] == 0x0A && data[6] == 0x1A && data[7] == 0x0A {
        return "image/png".to_string();
    }
    
    // GIF格式检测 (47 49 46 38)
    if data.len() >= 4 && 
       data[0] == 0x47 && data[1] == 0x49 && data[2] == 0x46 && data[3] == 0x38 {
        return "image/gif".to_string();
    }
    
    // BMP格式检测 (42 4D)
    if data.len() >= 2 && data[0] == 0x42 && data[1] == 0x4D {
        return "image/bmp".to_string();
    }
    
    // WebP格式检测 (52 49 46 46 ... 57 45 42 50)
    if data.len() >= 12 && 
       data[0] == 0x52 && data[1] == 0x49 && data[2] == 0x46 && data[3] == 0x46 &&
       data[8] == 0x57 && data[9] == 0x45 && data[10] == 0x42 && data[11] == 0x50 {
        return "image/webp".to_string();
    }
    
    String::new()
}

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
    /// 元数据信息
    pub metadata: AudioMetadata,
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
            metadata: AudioMetadata::default(),
        }
    }
    
    /// 从音频轨道和文件路径创建音频信息（包含时长估算）
    pub fn from_track_with_file_path(track: &Track, file_path: &str) -> Self {
        let mut info = Self::from_track(track);
        
        // 如果标准方法无法获取时长或时长为0，尝试通过解析文件来估算
        // 这对于m4s等流媒体片段文件特别有用
        //if info.duration.is_none() || info.duration == Some(0.0) {
        // 调用estimate_audio_duration_by_parsing太耗时了，只会导致整个open操作对会有很大的延迟
        // 因此通过调用open来创建AudioFile可以允许duration就是为none或者0的情况，后续因该通过异步的方式来调用estimate_audio_duration_by_parsing
        //    info.duration = estimate_audio_duration_by_parsing(file_path);
        //}
        
        info
    }
    
    /// 从音频轨道和元数据创建音频信息
    pub fn from_track_with_metadata(track: &Track, file_path: &str, metadata: AudioMetadata) -> Self {
        let mut info = Self::from_track_with_file_path(track, file_path);
        info.metadata = metadata;
        info
    }
}

/// 音频文件结构体
#[derive(Debug, Clone)]
pub struct AudioFile {
    /// 文件路径
    pub file_path: String,
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
        
        let mut probed = default::get_probe()
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
        // 提取元数据 - 尝试多种方式获取元数据
        let metadata = {
            // 方法1: 从probed.metadata获取
            if let Some(metadata_obj) = probed.metadata.get() {
                if let Some(metadata_rev) = metadata_obj.current() {
                    AudioMetadata::from_symphonia_metadata(metadata_rev)
                } else {
                    AudioMetadata::default()
                }
            } else {
            // 方法2: 从format reader获取元数据
                 let format_metadata = probed.format.metadata();
                 if let Some(metadata_rev) = format_metadata.current() {
                     AudioMetadata::from_symphonia_metadata(metadata_rev)
                 } else {
                     AudioMetadata::default()
                 }
            }
        };
        // 添加时间，来测试打开一个文件的耗时
        let start_time = std::time::Instant::now();
        let info = AudioInfo::from_track_with_metadata(&track, file_path, metadata);
        let end_time = std::time::Instant::now();
        println!("open file: {} cost: {:?}", file_path, end_time.duration_since(start_time));
        
        Ok(Self {
            file_path: file_path.to_string(),
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
    
    /// 创建播放所需的探测结果和轨道信息
    /// 
    /// # 返回
    /// 成功时返回(ProbeResult, Track)，失败时返回错误
    pub fn create_playback_context(&self) -> Result<(symphonia::core::probe::ProbeResult, Track)> {
        if !Path::new(&self.file_path).exists() {
            return Err(PlayerError::FileNotFound(self.file_path.clone()));
        }

        let file = File::open(&self.file_path)
            .map_err(|e| PlayerError::FileNotFound(format!("{}: {}", self.file_path, e)))?;
        
        let mss = MediaSourceStream::new(Box::new(file), Default::default());
        let hint = create_hint(&self.file_path);
        
        let probed = default::get_probe()
            .format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default())
            .map_err(|e| PlayerError::UnsupportedFormat(format!("{}: {}", self.file_path, e)))?;
        
        let track = probed
            .format
            .tracks()
            .iter()
            .find(|t| t.id == self.track_id)
            .ok_or_else(|| PlayerError::UnsupportedFormat("Track not found".to_string()))?
            .clone();
        
        Ok((probed, track))
    }

    /// 加载当前音频文件对应的歌词
    /// 
    /// 先查找内嵌歌词，如果没有找到，再查找同名的外部LRC文件
    /// 
    /// # 返回
    /// 成功时返回歌词Option，失败时返回错误
    pub fn load_lyrics(&self) -> Result<Option<crate::lyrics::Lyrics>> {
        use crate::lyrics::Lyrics;
        
        // 1. 优先尝试加载外部LRC文件
        if let Some(lrc_path) = Path::new(&self.file_path).with_extension("lrc").to_str() {
            if Path::new(lrc_path).exists() {
                if let Ok(lyrics) = Lyrics::from_lrc_file(lrc_path) {
                    return Ok(Some(lyrics));
                }
            }
        }
        
        // 2. 回退到内嵌歌词
        if let Ok(Some(lyrics)) = Lyrics::try_load_embedded_from_file(self) {
            return Ok(Some(lyrics));
        }
        
        // 3. 没有找到歌词
        Ok(None)
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

/// 对外暴露：通过解析整个文件来估算时长
///
/// 注意：这是一个耗时操作，应在后台异步执行。
pub fn estimate_duration_by_parsing(file_path: &str) -> Option<f64> {
    estimate_audio_duration_by_parsing(file_path)
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