//! 配置模块
//! 
//! 定义播放器的配置常量和选项。

/// 默认缓冲区倍数
pub const DEFAULT_BUFFER_MULTIPLIER: usize = 2;

/// 缓冲区容量阈值
pub const BUFFER_CAPACITY_THRESHOLD: usize = 1000;

/// 缓冲区写入延迟（毫秒）
pub const BUFFER_WRITE_DELAY: u64 = 1;

/// 字体配置
pub mod fonts {
    /// 中文字体
    pub const CHINESE_FONT: &str = "Noto Sans CJK SC";
    
    /// 表情字体
    pub const EMOJI_FONT: &str = "Noto Color Emoji";
    
    /// 默认字体
    pub const DEFAULT_FONT: &str = "DejaVu Sans";
}

/// UI配置
pub mod ui {
    /// 主面板宽度
    pub const MAIN_PANEL_WIDTH: f32 = 300.0;
    
    /// 进度更新间隔（毫秒）
    pub const PROGRESS_UPDATE_INTERVAL: u64 = 100;
    
    /// 音量调节步长
    pub const VOLUME_STEP: f32 = 0.05;
}

/// 音频配置
pub mod audio {
    /// 最大估算包数
    pub const MAX_ESTIMATION_PACKETS: u64 = 100000;
    
    /// 默认采样率
    pub const DEFAULT_SAMPLE_RATE: u32 = 44100;
    
    /// 默认声道数
    pub const DEFAULT_CHANNELS: usize = 2;
}

/// 播放器配置结构
#[derive(Debug, Clone)]
pub struct PlayerConfig {
    /// 音频设备索引
    pub device_index: Option<usize>,
    /// 默认音量 (0.0 - 1.0)
    pub default_volume: f32,
    /// 自动播放下一首
    pub auto_next: bool,
    /// 启用播放列表循环
    pub loop_playlist: bool,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            device_index: None,
            default_volume: 1.0,
            auto_next: true,
            loop_playlist: false,
        }
    }
} 