//! 错误处理模块
//! 
//! 定义了播放器中使用的所有错误类型。

use std::fmt;

/// 播放器错误类型
#[derive(Debug)]
pub enum PlayerError {
    /// 文件未找到错误
    FileNotFound(String),
    /// 不支持的音频格式
    UnsupportedFormat(String),
    /// 音频设备错误
    AudioDeviceError(String),
    /// 音频解码错误
    DecodingError(String),
    /// 播放错误
    PlaybackError(String),
    /// 播放列表错误
    PlaylistError(String),
    /// IO错误
    IoError(std::io::Error),
    /// 其他错误
    Other(String),
}

impl fmt::Display for PlayerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlayerError::FileNotFound(msg) => write!(f, "File not found: {}", msg),
            PlayerError::UnsupportedFormat(msg) => write!(f, "Unsupported format: {}", msg),
            PlayerError::AudioDeviceError(msg) => write!(f, "Audio device error: {}", msg),
            PlayerError::DecodingError(msg) => write!(f, "Decoding error: {}", msg),
            PlayerError::PlaybackError(msg) => write!(f, "Playback error: {}", msg),
            PlayerError::PlaylistError(msg) => write!(f, "Playlist error: {}", msg),
            PlayerError::IoError(err) => write!(f, "IO error: {}", err),
            PlayerError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for PlayerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            PlayerError::IoError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for PlayerError {
    fn from(err: std::io::Error) -> Self {
        PlayerError::IoError(err)
    }
}

/// 播放器结果类型
pub type Result<T> = std::result::Result<T, PlayerError>; 