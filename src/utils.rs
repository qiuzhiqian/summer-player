//! 实用工具模块
//! 
//! 包含播放器中使用的通用辅助函数。

use std::path::Path;

/// 格式化时长显示
/// 
/// # 参数
/// * `seconds` - 时长（秒）
/// 
/// # 返回
/// 格式化的时长字符串（如 "03:45" 或 "1:23:45"）
pub fn format_duration(seconds: f64) -> String {
    let total_seconds = seconds as u64;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    
    if hours > 0 {
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    } else {
        format!("{:02}:{:02}", minutes, seconds)
    }
}

/// 检查文件是否为M3U播放列表
/// 
/// # 参数
/// * `file_path` - 文件路径
/// 
/// # 返回
/// 如果是M3U播放列表文件则返回true
pub fn is_m3u_playlist(file_path: &str) -> bool {
    if let Some(extension) = Path::new(file_path).extension() {
        if let Some(ext_str) = extension.to_str() {
            let ext_lower = ext_str.to_lowercase();
            return ext_lower == "m3u" || ext_lower == "m3u8";
        }
    }
    false
}

/// 检查文件是否为支持的音频格式
/// 
/// # 参数
/// * `file_path` - 文件路径
/// 
/// # 返回
/// 如果是支持的音频文件则返回true
pub fn is_supported_audio_file(file_path: &str) -> bool {
    const SUPPORTED_EXTENSIONS: &[&str] = &[
        "mp3", "flac", "wav", "ogg", "aac", "m4a", "m4s", "wma", "opus"
    ];
    
    if let Some(extension) = Path::new(file_path).extension() {
        if let Some(ext_str) = extension.to_str() {
            let ext_lower = ext_str.to_lowercase();
            return SUPPORTED_EXTENSIONS.contains(&ext_lower.as_str());
        }
    }
    false
}

/// 从文件路径提取文件名
/// 
/// # 参数
/// * `path` - 文件路径
/// 
/// # 返回
/// 文件名，如果无法提取则返回"Unknown"
pub fn extract_filename(path: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Unknown")
        .to_string()
}

/// 从文件路径提取文件名（不含扩展名）
/// 
/// # 参数
/// * `path` - 文件路径
/// 
/// # 返回
/// 不含扩展名的文件名
pub fn extract_filename_without_extension(path: &str) -> String {
    Path::new(path)
        .file_stem()
        .and_then(|n| n.to_str())
        .unwrap_or("Unknown")
        .to_string()
}

/// 规范化文件路径
/// 
/// # 参数
/// * `path` - 原始路径
/// * `base_dir` - 基准目录（用于解析相对路径）
/// 
/// # 返回
/// 规范化的绝对路径
pub fn normalize_path(path: &str, base_dir: Option<&Path>) -> String {
    let path_obj = Path::new(path);
    
    if path_obj.is_absolute() {
        path.to_string()
    } else if let Some(base) = base_dir {
        base.join(path).to_string_lossy().to_string()
    } else {
        path.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(0.0), "00:00");
        assert_eq!(format_duration(65.0), "01:05");
        assert_eq!(format_duration(3661.0), "01:01:01");
    }

    #[test]
    fn test_is_m3u_playlist() {
        assert!(is_m3u_playlist("test.m3u"));
        assert!(is_m3u_playlist("test.M3U"));
        assert!(is_m3u_playlist("test.m3u8"));
        assert!(!is_m3u_playlist("test.mp3"));
        assert!(!is_m3u_playlist("test.txt"));
    }

    #[test]
    fn test_is_supported_audio_file() {
        assert!(is_supported_audio_file("test.mp3"));
        assert!(is_supported_audio_file("test.FLAC"));
        assert!(is_supported_audio_file("test.m4a"));
        assert!(!is_supported_audio_file("test.txt"));
        assert!(!is_supported_audio_file("test.m3u"));
    }

    #[test]
    fn test_extract_filename() {
        assert_eq!(extract_filename("/path/to/file.mp3"), "file.mp3");
        assert_eq!(extract_filename("file.mp3"), "file.mp3");
        assert_eq!(extract_filename(""), "Unknown");
    }
} 