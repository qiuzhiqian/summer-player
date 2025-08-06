//! 配置模块
//! 
//! 定义应用程序的各种配置常量和配置结构。

/// 默认缓冲区倍数
pub const DEFAULT_BUFFER_MULTIPLIER: usize = 2;

/// 缓冲区容量阈值
pub const BUFFER_CAPACITY_THRESHOLD: usize = 1000;

/// 缓冲区写入延迟（毫秒）
pub const BUFFER_WRITE_DELAY: u64 = 10;

/// 字体配置
pub mod fonts {
    /// 中文字体
    pub const CHINESE_FONT: &str = "SimHei";
    
    /// Emoji 字体文件路径
    pub const EMOJI_FONT_PATH: &str = "fonts/NotoColorEmoji.ttf";
    
    /// Emoji 字体名称
    pub const EMOJI_FONT_NAME: &str = "NotoColorEmoji";
}

/// UI常量
pub mod ui {
    /// 主面板宽度
    pub const MAIN_PANEL_WIDTH: f32 = 220.0; // 增加宽度以适应新设计
    
    /// 播放列表面板高度
    pub const PLAYLIST_HEIGHT: f32 = 320.0; // 略微增加高度
    
    /// 窗口最小宽度
    pub const MIN_WINDOW_WIDTH: f32 = 900.0; // 增加最小宽度
    
    /// 窗口最小高度
    pub const MIN_WINDOW_HEIGHT: f32 = 650.0; // 增加最小高度
    
    /// 默认窗口宽度
    pub const DEFAULT_WINDOW_WIDTH: f32 = 1200.0; // 增加默认宽度以展示更好的视觉效果
    
    /// 默认窗口高度
    pub const DEFAULT_WINDOW_HEIGHT: f32 = 800.0; // 增加默认高度
    
    /// 进度更新间隔（毫秒）
    pub const PROGRESS_UPDATE_INTERVAL: u64 = 100;
    
    /// 标准间距
    pub const STANDARD_SPACING: f32 = 16.0;
    
    /// 大间距
    pub const LARGE_SPACING: f32 = 24.0;
    
    /// 标准内边距
    pub const STANDARD_PADDING: f32 = 16.0;
    
    /// 大内边距
    pub const LARGE_PADDING: f32 = 24.0;
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
    /// 自动播放下一首
    pub auto_next: bool,
    /// 启用播放列表循环
    pub loop_playlist: bool,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            device_index: None,
            auto_next: true,
            loop_playlist: false,
        }
    }
} 