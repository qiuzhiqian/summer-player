//! 音频处理模块
//! 
//! 处理音频文件解码、播放控制和音频设备管理。

pub mod file;
pub mod device;
pub mod playback;
pub mod decoder;
pub mod stream;

// 重新导出主要类型
pub use file::{AudioFile, AudioInfo};
pub use device::{list_audio_devices, setup_audio_device};
pub use playback::{
    PlaybackState, PlaybackCommand, AudioBuffer,
    start_audio_playback, run_audio_playback_with_control
};
pub use decoder::create_decoder;
pub use stream::{create_audio_stream, create_stream}; 