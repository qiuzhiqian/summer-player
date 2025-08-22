//! # Summer Audio Player Library
//! 
//! 一个现代化的夏日主题音频播放器库，支持多种音频格式和播放列表功能。

pub mod error;
pub mod audio;
pub mod playlist;
pub mod lyrics;
pub mod ui;
pub mod config;
pub mod utils;

// 重新导出主要的公共类型
pub use error::{PlayerError, Result};
pub use audio::{AudioFile, AudioInfo, PlaybackState, PlaybackCommand};
pub use playlist::{Playlist};
pub use ui::PlayerApp; 

rust_i18n::i18n!("locales");