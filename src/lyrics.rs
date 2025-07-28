use std::fs;
use std::path::Path;
use std::collections::BTreeMap;
use crate::error::Result;

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

/// 加载音频文件对应的歌词
pub fn load_lyrics_for_audio<P: AsRef<Path>>(audio_file_path: P) -> Result<Option<Lyrics>> {
    if let Some(lrc_path) = find_lyrics_file(audio_file_path) {
        match Lyrics::from_lrc_file(&lrc_path) {
            Ok(lyrics) => Ok(Some(lyrics)),
            Err(e) => {
                eprintln!("Failed to load lyrics from {}: {}", lrc_path, e);
                Ok(None)
            }
        }
    } else {
        Ok(None)
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
        let result = parse_lyric_line("[00:12.34]Hello World");
        assert!(result.is_some());
        let (timestamps, text) = result.unwrap();
        assert_eq!(timestamps, vec![12340]);
        assert_eq!(text, "Hello World");

        let result = parse_lyric_line("[00:12.34][00:15.67]Multiple timestamps");
        assert!(result.is_some());
        let (timestamps, text) = result.unwrap();
        assert_eq!(timestamps, vec![12340, 15670]);
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
} 