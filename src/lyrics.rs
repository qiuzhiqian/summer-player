use std::fs;
use std::path::Path;
use std::collections::BTreeMap;
use crate::error::Result;
use crate::audio::file::{EmbeddedLyrics, LyricsType};

/// 歌词行结构
#[derive(Debug, Clone)]
pub struct LyricLine {
    /// 时间戳（秒）
    pub timestamp: f64,
    /// 歌词文本
    pub text: String,
}

/// 歌词结构
#[derive(Debug, Clone, Default)]
pub struct Lyrics {
    /// 歌词行列表（按时间排序）
    pub lines: Vec<LyricLine>,
    /// 元数据
    pub metadata: LyricsMetadata,
}

/// 歌词元数据
#[derive(Debug, Clone, Default)]
pub struct LyricsMetadata {
    /// 标题
    pub title: Option<String>,
    /// 艺术家
    pub artist: Option<String>,
    /// 专辑
    pub album: Option<String>,
    /// 制作者
    pub by: Option<String>,
    /// 偏移量（毫秒）
    pub offset: i32,
}

impl Lyrics {
    /// 从LRC文件路径加载歌词
    pub fn from_lrc_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        Self::from_lrc_content(&content)
    }

    /// 从LRC内容字符串解析歌词
    pub fn from_lrc_content(content: &str) -> Result<Self> {
        let mut lyrics = Lyrics::default();
        let mut temp_lines: BTreeMap<i64, String> = BTreeMap::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // 解析元数据标签
            if let Some(metadata) = parse_metadata_tag(line) {
                match metadata.0.as_str() {
                    "ti" => lyrics.metadata.title = Some(metadata.1),
                    "ar" => lyrics.metadata.artist = Some(metadata.1),
                    "al" => lyrics.metadata.album = Some(metadata.1),
                    "by" => lyrics.metadata.by = Some(metadata.1),
                    "offset" => {
                        if let Ok(offset) = metadata.1.parse::<i32>() {
                            lyrics.metadata.offset = offset;
                        }
                    }
                    _ => {} // 忽略未知标签
                }
                continue;
            }

            // 解析时间标签和歌词
            if let Some((timestamps, text)) = parse_lyric_line(line) {
                for timestamp in timestamps {
                    // 应用偏移量
                    let adjusted_timestamp = timestamp + (lyrics.metadata.offset as i64);
                    temp_lines.insert(adjusted_timestamp, text.clone());
                }
            }
        }

        // 转换为LyricLine并排序
        lyrics.lines = temp_lines
            .into_iter()
            .map(|(timestamp_ms, text)| LyricLine {
                timestamp: timestamp_ms as f64 / 1000.0, // 转换为秒
                text,
            })
            .collect();

        Ok(lyrics)
    }

    /// 从内嵌歌词创建歌词对象
    pub fn from_embedded_lyrics(embedded_lyrics: &[EmbeddedLyrics]) -> Result<Self> {
        // 优先级：LRC > 同步歌词 > 非同步歌词
        let selected_lyrics = embedded_lyrics
            .iter()
            .min_by_key(|lyrics| match lyrics.lyrics_type {
                LyricsType::Lrc => 0,
                LyricsType::Synchronized => 1,
                LyricsType::Unsynchronized => 2,
                LyricsType::Other(_) => 3,
            });

        if let Some(lyrics) = selected_lyrics {
            match lyrics.lyrics_type {
                LyricsType::Lrc => {
                    // 使用现有的LRC解析功能
                    Self::from_lrc_content(&lyrics.content)
                }
                LyricsType::Synchronized => {
                    // TODO: 实现同步歌词解析
                    Self::from_synchronized_lyrics(&lyrics.content)
                }
                LyricsType::Unsynchronized | LyricsType::Other(_) => {
                    // 创建简单的非时间同步歌词
                    Self::from_plain_text(&lyrics.content)
                }
            }
        } else {
            Ok(Lyrics::default())
        }
    }

    /// 从纯文本创建歌词（非时间同步）
    pub fn from_plain_text(content: &str) -> Result<Self> {
        let mut lyrics = Lyrics::default();
        
        let lines: Vec<&str> = content.lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect();

        // 为每行分配虚拟时间戳（每行间隔3秒）
        lyrics.lines = lines
            .into_iter()
            .enumerate()
            .map(|(index, text)| LyricLine {
                timestamp: (index as f64) * 3.0, // 3秒间隔
                text: text.to_string(),
            })
            .collect();

        Ok(lyrics)
    }

    /// 从同步歌词内容解析（ID3v2 SYLT格式）
    fn from_synchronized_lyrics(content: &str) -> Result<Self> {
        // TODO: 实现ID3v2 SYLT格式的解析
        // 这是一个简化的实现，实际的SYLT格式更复杂
        // 暂时回退到纯文本处理
        Self::from_plain_text(content)
    }

    /// 尝试从音频文件路径加载内嵌歌词
    pub fn try_load_embedded<P: AsRef<Path>>(audio_path: P) -> Result<Option<Self>> {
        use crate::audio::file::AudioFile;
        
        let audio_file = AudioFile::open(audio_path.as_ref().to_str().unwrap())?;
        let embedded_lyrics = &audio_file.info.metadata.embedded_lyrics;
        
        if embedded_lyrics.is_empty() {
            Ok(None)
        } else {
            let lyrics = Self::from_embedded_lyrics(embedded_lyrics)?;
            Ok(Some(lyrics))
        }
    }

    /// 智能歌词加载：优先外部LRC文件，回退到内嵌歌词
    pub fn smart_load<P: AsRef<Path>>(audio_path: P) -> Result<Self> {
        let audio_path = audio_path.as_ref();
        
        // 1. 尝试加载同名LRC文件
        if let Some(lrc_path) = audio_path.with_extension("lrc").to_str() {
            if Path::new(lrc_path).exists() {
                if let Ok(lyrics) = Self::from_lrc_file(lrc_path) {
                    return Ok(lyrics);
                }
            }
        }
        
        // 2. 尝试加载内嵌歌词
        if let Ok(Some(lyrics)) = Self::try_load_embedded(audio_path) {
            return Ok(lyrics);
        }
        
        // 3. 返回空歌词
        Ok(Lyrics::default())
    }

    /// 根据当前时间获取当前歌词行索引
    pub fn get_current_line_index(&self, current_time: f64) -> Option<usize> {
        if self.lines.is_empty() {
            return None;
        }

        // 找到最后一个时间戳小于等于当前时间的歌词行
        for (index, line) in self.lines.iter().enumerate().rev() {
            if line.timestamp <= current_time {
                return Some(index);
            }
        }

        // 如果没有找到，检查是否在第一行之前
        if current_time < self.lines[0].timestamp {
            return None;
        }

        Some(0)
    }

    /// 获取在指定时间范围内的歌词行
    pub fn get_lines_in_range(&self, start_time: f64, end_time: f64) -> Vec<(usize, &LyricLine)> {
        self.lines
            .iter()
            .enumerate()
            .filter(|(_, line)| line.timestamp >= start_time && line.timestamp <= end_time)
            .collect()
    }

    /// 检查是否有歌词
    pub fn has_lyrics(&self) -> bool {
        !self.lines.is_empty()
    }

    /// 获取歌词总时长（最后一行的时间戳）
    pub fn duration(&self) -> Option<f64> {
        self.lines.last().map(|line| line.timestamp)
    }
}

/// 解析元数据标签，如 [ti:Title]
fn parse_metadata_tag(line: &str) -> Option<(String, String)> {
    if !line.starts_with('[') || !line.ends_with(']') {
        return None;
    }

    let content = &line[1..line.len() - 1];
    if let Some(colon_pos) = content.find(':') {
        let tag = content[..colon_pos].to_string();
        let value = content[colon_pos + 1..].to_string();
        
        // 检查是否是时间标签
        if tag.chars().all(|c| c.is_ascii_digit() || c == '.' || c == ':') {
            return None;
        }
        
        Some((tag, value))
    } else {
        None
    }
}

/// 解析歌词行，提取时间标签和文本
/// 返回 (时间戳列表（毫秒）, 歌词文本)
fn parse_lyric_line(line: &str) -> Option<(Vec<i64>, String)> {
    let mut timestamps = Vec::new();
    let mut current_pos = 0;

    // 提取所有时间标签
    while let Some(start) = line[current_pos..].find('[') {
        let start = current_pos + start;
        if let Some(end) = line[start..].find(']') {
            let end = start + end;
            let time_str = &line[start + 1..end];
            
            if let Some(timestamp_ms) = parse_timestamp(time_str) {
                timestamps.push(timestamp_ms);
                current_pos = end + 1;
            } else {
                // 如果不是有效的时间戳，跳过这个标签
                current_pos = end + 1;
            }
        } else {
            break;
        }
    }

    if timestamps.is_empty() {
        return None;
    }

    // 提取歌词文本（去除所有时间标签后的内容）
    let text = extract_text_after_timestamps(line);

    Some((timestamps, text))
}

/// 解析时间戳字符串，如 "00:12.34" 或 "01:23.456"
/// 返回毫秒数
fn parse_timestamp(time_str: &str) -> Option<i64> {
    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() != 2 {
        return None;
    }

    let minutes: i64 = parts[0].parse().ok()?;
    let seconds_str = parts[1];

    // 处理秒和毫秒
    let (seconds, milliseconds) = if let Some(dot_pos) = seconds_str.find('.') {
        let seconds: i64 = seconds_str[..dot_pos].parse().ok()?;
        let ms_str = &seconds_str[dot_pos + 1..];
        
        // 将毫秒字符串补齐或截断到3位
        let ms_str = if ms_str.len() >= 3 {
            &ms_str[..3]
        } else {
            ms_str
        };
        
        let milliseconds: i64 = match ms_str.len() {
            1 => ms_str.parse::<i64>().ok()? * 100,
            2 => ms_str.parse::<i64>().ok()? * 10,
            3 => ms_str.parse().ok()?,
            _ => 0,
        };
        
        (seconds, milliseconds)
    } else {
        (seconds_str.parse().ok()?, 0)
    };

    Some(minutes * 60 * 1000 + seconds * 1000 + milliseconds)
}

/// 提取时间标签后的文本内容
fn extract_text_after_timestamps(line: &str) -> String {
    let mut result = String::new();
    let mut in_bracket = false;
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '[' => {
                in_bracket = true;
            }
            ']' => {
                in_bracket = false;
            }
            _ => {
                if !in_bracket {
                    result.push(ch);
                }
            }
        }
    }

    result.trim().to_string()
}

/// 根据音频文件路径查找对应的LRC歌词文件
pub fn find_lyrics_file<P: AsRef<Path>>(audio_file_path: P) -> Option<String> {
    let audio_path = audio_file_path.as_ref();
    let parent_dir = audio_path.parent()?;
    let file_stem = audio_path.file_stem()?.to_str()?;
    
    let lrc_path = parent_dir.join(format!("{}.lrc", file_stem));
    
    if lrc_path.exists() {
        lrc_path.to_str().map(|s| s.to_string())
    } else {
        None
    }
}

/// 加载音频文件对应的歌词（智能加载：外部LRC优先，回退到内嵌歌词）
pub fn load_lyrics_for_audio<P: AsRef<Path>>(audio_file_path: P) -> Result<Option<Lyrics>> {
    match Lyrics::smart_load(&audio_file_path) {
        Ok(lyrics) => {
            if lyrics.has_lyrics() {
                Ok(Some(lyrics))
            } else {
                Ok(None)
            }
        }
        Err(e) => {
            eprintln!("Failed to load lyrics for {}: {}", audio_file_path.as_ref().display(), e);
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_timestamp() {
        assert_eq!(parse_timestamp("00:12.34"), Some(12340));
        assert_eq!(parse_timestamp("01:23.456"), Some(83456));
        assert_eq!(parse_timestamp("02:30"), Some(150000));
        assert_eq!(parse_timestamp("invalid"), None);
    }

    #[test]
    fn test_parse_lyric_line() {
        let (timestamps, text) = parse_lyric_line("[00:12.34]Hello world").unwrap();
        assert_eq!(timestamps, vec![12340]);
        assert_eq!(text, "Hello world");

        let (timestamps, text) = parse_lyric_line("[00:12.34][00:56.78]Multiple timestamps").unwrap();
        assert_eq!(timestamps, vec![12340, 56780]);
        assert_eq!(text, "Multiple timestamps");
    }

    #[test]
    fn test_parse_metadata_tag() {
        assert_eq!(
            parse_metadata_tag("[ti:Song Title]"),
            Some(("ti".to_string(), "Song Title".to_string()))
        );
        assert_eq!(
            parse_metadata_tag("[ar:Artist Name]"),
            Some(("ar".to_string(), "Artist Name".to_string()))
        );
        assert_eq!(parse_metadata_tag("[00:12.34]"), None);
    }

    #[test]
    fn test_extract_text_after_timestamps() {
        assert_eq!(
            extract_text_after_timestamps("[00:12.34]Hello World"),
            "Hello World"
        );
        assert_eq!(
            extract_text_after_timestamps("[00:12.34][00:15.67]Multiple timestamps"),
            "Multiple timestamps"
        );
    }

    #[test]
    fn test_from_embedded_lyrics() {
        use crate::audio::file::{EmbeddedLyrics, LyricsType};
        
        // 测试LRC格式的内嵌歌词
        let lrc_lyrics = EmbeddedLyrics {
            content: "[00:12.34]First line\n[00:15.78]Second line".to_string(),
            language: Some("en".to_string()),
            description: Some("Test LRC".to_string()),
            lyrics_type: LyricsType::Lrc,
        };
        
        let lyrics = Lyrics::from_embedded_lyrics(&[lrc_lyrics]).unwrap();
        assert_eq!(lyrics.lines.len(), 2);
        assert_eq!(lyrics.lines[0].text, "First line");
        assert_eq!(lyrics.lines[0].timestamp, 12.34);
    }

    #[test]
    fn test_from_plain_text() {
        let content = "Line 1\nLine 2\nLine 3";
        let lyrics = Lyrics::from_plain_text(content).unwrap();
        
        assert_eq!(lyrics.lines.len(), 3);
        assert_eq!(lyrics.lines[0].text, "Line 1");
        assert_eq!(lyrics.lines[0].timestamp, 0.0);
        assert_eq!(lyrics.lines[1].text, "Line 2");
        assert_eq!(lyrics.lines[1].timestamp, 3.0);
        assert_eq!(lyrics.lines[2].text, "Line 3");
        assert_eq!(lyrics.lines[2].timestamp, 6.0);
    }

    #[test]
    fn test_embedded_lyrics_priority() {
        use crate::audio::file::{EmbeddedLyrics, LyricsType};
        
        let unsync_lyrics = EmbeddedLyrics {
            content: "Unsync lyrics".to_string(),
            language: None,
            description: None,
            lyrics_type: LyricsType::Unsynchronized,
        };
        
        let lrc_lyrics = EmbeddedLyrics {
            content: "[00:12.34]LRC lyrics".to_string(),
            language: None,
            description: None,
            lyrics_type: LyricsType::Lrc,
        };
        
        // LRC应该有更高优先级
        let lyrics = Lyrics::from_embedded_lyrics(&[unsync_lyrics, lrc_lyrics]).unwrap();
        assert_eq!(lyrics.lines.len(), 1);
        assert_eq!(lyrics.lines[0].text, "LRC lyrics");
    }

    #[test]
    fn test_is_lrc_content() {
        use crate::audio::file::AudioMetadata;
        
        assert!(AudioMetadata::is_lrc_content("[00:12.34]Test line"));
        assert!(AudioMetadata::is_lrc_content("[00:12.34]First\n[00:15.67]Second"));
        assert!(!AudioMetadata::is_lrc_content("Plain text without timestamps"));
        assert!(!AudioMetadata::is_lrc_content("Some [text] but not timestamps"));
    }
} 