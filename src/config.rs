//! 配置模块
//! 
//! 定义应用程序的各种配置常量和配置结构，支持TOML格式的持久化配置。

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 默认缓冲区倍数
pub const DEFAULT_BUFFER_MULTIPLIER: usize = 2;

/// 缓冲区容量阈值
pub const BUFFER_CAPACITY_THRESHOLD: usize = 1000;

/// 缓冲区写入延迟（毫秒）
pub const BUFFER_WRITE_DELAY: u64 = 10;

/// 字体配置
pub mod fonts {
    /// 获取适合当前平台的中文字体
    pub fn get_chinese_font() -> &'static str {
        #[cfg(target_os = "windows")]
        {
            // Windows系统优先使用微软雅黑，fallback到SimHei
            "Microsoft YaHei"
        }
        #[cfg(target_os = "macos")]
        {
            // macOS系统使用PingFang SC
            "PingFang SC"
        }
        #[cfg(target_os = "linux")]
        {
            // Linux系统使用Noto Sans CJK SC，fallback到思源黑体
            "Noto Sans CJK SC"
        }
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            // 其他系统使用通用回退
            "sans-serif"
        }
    }
    
    /// Windows字体回退选项
    pub const WINDOWS_FONT_FALLBACKS: &[&str] = &[
        "Microsoft YaHei",
        "Microsoft YaHei UI",
        "SimHei",
        "SimSun",
        "NSimSun",
        "FangSong",
        "KaiTi",
        "sans-serif"
    ];
    
    /// macOS字体回退选项
    pub const MACOS_FONT_FALLBACKS: &[&str] = &[
        "PingFang SC",
        "Hiragino Sans GB",
        "Heiti SC",
        "STHeiti",
        "sans-serif"
    ];
    
    /// Linux字体回退选项
    pub const LINUX_FONT_FALLBACKS: &[&str] = &[
        "Noto Sans CJK SC",
        "Source Han Sans SC",
        "WenQuanYi Micro Hei",
        "DejaVu Sans",
        "Liberation Sans",
        "sans-serif"
    ];
    
    /// 中文字体（保持向后兼容）
    pub const CHINESE_FONT: &str = "Microsoft YaHei";
    
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

// ============================================================================
// TOML 配置结构
// ============================================================================

/// 主题类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ThemeVariant {
    Light,
    Dark,
}

impl Default for ThemeVariant {
    fn default() -> Self {
        Self::Light
    }
}

/// 播放模式
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PlayModeConfig {
    ListLoop,
    SingleLoop,
    Random,
}

impl Default for PlayModeConfig {
    fn default() -> Self {
        Self::ListLoop
    }
}

/// 窗口配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    /// 窗口宽度
    pub width: f32,
    /// 窗口高度
    pub height: f32,
    /// 是否最大化
    pub maximized: bool,
    /// 窗口位置 (x, y)
    pub position: Option<(i32, i32)>,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            width: ui::DEFAULT_WINDOW_WIDTH,
            height: ui::DEFAULT_WINDOW_HEIGHT,
            maximized: false,
            position: None,
        }
    }
}

/// 界面配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIConfig {
    /// 当前主题
    pub theme: ThemeVariant,
    /// 界面语言
    pub language: String,
    /// 当前页面类型
    pub current_page: String,
    /// 当前视图类型
    pub current_view: String,
    /// 左侧面板宽度
    pub left_panel_width: f32,
}

impl Default for UIConfig {
    fn default() -> Self {
        Self {
            theme: ThemeVariant::Light,
            language: "en".to_string(),
            current_page: "Home".to_string(),
            current_view: "Playlist".to_string(),
            left_panel_width: ui::MAIN_PANEL_WIDTH,
        }
    }
}

/// 播放器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerConfig {
    /// 音频设备索引
    pub device_index: Option<usize>,
    /// 自动播放下一首
    pub auto_next: bool,
    /// 启用播放列表循环
    pub loop_playlist: bool,
    /// 播放模式
    pub play_mode: PlayModeConfig,
    /// 音量（0.0 - 1.0）
    pub volume: f64,
    /// 最后播放的文件路径
    pub last_file_path: Option<String>,
    /// 最后播放的播放列表路径
    pub last_playlist_path: Option<String>,
    /// 记住播放位置
    pub remember_position: bool,
    /// 最后播放位置（秒）
    pub last_position: f64,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            device_index: None,
            auto_next: true,
            loop_playlist: false,
            play_mode: PlayModeConfig::ListLoop,
            volume: 0.8,
            last_file_path: None,
            last_playlist_path: None,
            remember_position: true,
            last_position: 0.0,
        }
    }
}

/// 歌词配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LyricsConfig {
    /// 是否启用歌词显示
    pub enabled: bool,
    /// 歌词字体大小
    pub font_size: u16,
    /// 歌词显示行数
    pub display_lines: usize,
    /// 是否自动搜索歌词
    pub auto_search: bool,
    /// 歌词颜色主题
    pub color_theme: String,
}

impl Default for LyricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            font_size: 16,
            display_lines: 7,
            auto_search: true,
            color_theme: "default".to_string(),
        }
    }
}

/// 应用程序配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// 配置文件版本
    pub version: String,
    /// 窗口配置
    pub window: WindowConfig,
    /// 界面配置
    pub ui: UIConfig,
    /// 播放器配置
    pub player: PlayerConfig,
    /// 歌词配置
    pub lyrics: LyricsConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            window: WindowConfig::default(),
            ui: UIConfig::default(),
            player: PlayerConfig::default(),
            lyrics: LyricsConfig::default(),
        }
    }
}

// ============================================================================
// 配置管理功能
// ============================================================================

impl AppConfig {
    /// 获取配置文件路径
    pub fn config_file_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let config_dir = dirs::config_dir()
            .ok_or("无法获取配置目录")?
            .join("summer-player");

        // 确保配置目录存在
        std::fs::create_dir_all(&config_dir)?;

        Ok(config_dir.join("config.toml"))
    }

    /// 从文件加载配置
    pub fn load() -> Self {
        match Self::load_from_file() {
            Ok(config) => {
                // 验证和更新配置版本
                let mut config = config;
                let current_version = env!("CARGO_PKG_VERSION");
                if config.version != current_version {
                    config.version = current_version.to_string();
                    // 可以在这里添加版本迁移逻辑
                    let _ = config.save();
                }
                config
            }
            Err(e) => {
                eprintln!("加载配置文件失败: {}, 使用默认配置", e);
                let default_config = Self::default();
                let _ = default_config.save(); // 保存默认配置
                default_config
            }
        }
    }

    /// 加载配置并返回是否从文件加载的标志
    /// 返回 (配置, 是否从实际配置文件加载)
    pub fn load_with_source() -> (Self, bool) {
        match Self::load_from_file() {
            Ok(config) => {
                // 验证和更新配置版本
                let mut config = config;
                let current_version = env!("CARGO_PKG_VERSION");
                if config.version != current_version {
                    config.version = current_version.to_string();
                    // 可以在这里添加版本迁移逻辑
                    let _ = config.save();
                }
                (config, true) // 从文件加载
            }
            Err(_) => {
                // 配置文件不存在或损坏，使用默认配置但不立即保存
                (Self::default(), false) // 使用默认值
            }
        }
    }

    /// 从文件加载配置（内部方法）
    fn load_from_file() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Self::config_file_path()?;
        
        if !config_path.exists() {
            return Err("配置文件不存在".into());
        }

        let content = std::fs::read_to_string(&config_path)?;
        let config: AppConfig = toml::from_str(&content)?;
        
        Ok(config)
    }

    /// 保存配置到文件
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = Self::config_file_path()?;
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;
        Ok(())
    }

    /// 安全保存配置（忽略错误）
    pub fn save_safe(&self) {
        if let Err(e) = self.save() {
            eprintln!("保存配置文件失败: {}", e);
        }
    }

    /// 重置为默认配置
    pub fn reset() -> Self {
        let default_config = Self::default();
        let _ = default_config.save();
        default_config
    }

    /// 获取配置文件路径（用于用户查看）
    pub fn get_config_path_string() -> String {
        Self::config_file_path()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| "无法获取配置路径".to_string())
    }
}

// ============================================================================
// 兼容性和转换
// ============================================================================

/// 从UI组件的枚举转换到配置枚举
impl From<crate::ui::components::PlayMode> for PlayModeConfig {
    fn from(mode: crate::ui::components::PlayMode) -> Self {
        match mode {
            crate::ui::components::PlayMode::ListLoop => PlayModeConfig::ListLoop,
            crate::ui::components::PlayMode::SingleLoop => PlayModeConfig::SingleLoop,
            crate::ui::components::PlayMode::Random => PlayModeConfig::Random,
        }
    }
}

/// 从配置枚举转换到UI组件的枚举
impl Into<crate::ui::components::PlayMode> for PlayModeConfig {
    fn into(self) -> crate::ui::components::PlayMode {
        match self {
            PlayModeConfig::ListLoop => crate::ui::components::PlayMode::ListLoop,
            PlayModeConfig::SingleLoop => crate::ui::components::PlayMode::SingleLoop,
            PlayModeConfig::Random => crate::ui::components::PlayMode::Random,
        }
    }
}

/// 从UI主题枚举转换到配置枚举
impl From<crate::ui::theme::AppThemeVariant> for ThemeVariant {
    fn from(theme: crate::ui::theme::AppThemeVariant) -> Self {
        match theme {
            crate::ui::theme::AppThemeVariant::Light => ThemeVariant::Light,
            crate::ui::theme::AppThemeVariant::Dark => ThemeVariant::Dark,
        }
    }
}

/// 从配置枚举转换到UI主题枚举
impl Into<crate::ui::theme::AppThemeVariant> for ThemeVariant {
    fn into(self) -> crate::ui::theme::AppThemeVariant {
        match self {
            ThemeVariant::Light => crate::ui::theme::AppThemeVariant::Light,
            ThemeVariant::Dark => crate::ui::theme::AppThemeVariant::Dark,
        }
    }
}

/// 从UI页面类型转换到字符串
impl From<crate::ui::components::PageType> for String {
    fn from(page: crate::ui::components::PageType) -> Self {
        match page {
            crate::ui::components::PageType::Home => "Home".to_string(),
            crate::ui::components::PageType::Settings => "Settings".to_string(),
        }
    }
}

/// 从字符串转换到UI页面类型
impl From<String> for crate::ui::components::PageType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "Settings" => crate::ui::components::PageType::Settings,
            _ => crate::ui::components::PageType::Home,
        }
    }
}

/// 从UI视图类型转换到字符串
impl From<crate::ui::components::ViewType> for String {
    fn from(view: crate::ui::components::ViewType) -> Self {
        match view {
            crate::ui::components::ViewType::Playlist => "Playlist".to_string(),
            crate::ui::components::ViewType::Lyrics => "Lyrics".to_string(),
        }
    }
}

/// 从字符串转换到UI视图类型
impl From<String> for crate::ui::components::ViewType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "Lyrics" => crate::ui::components::ViewType::Lyrics,
            _ => crate::ui::components::ViewType::Playlist,
        }
    }
} 