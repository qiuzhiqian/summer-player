//! UI消息定义模块
//! 
//! 定义应用程序中使用的所有UI消息类型。

use iced::event::Event;
use tokio::sync::mpsc;

use crate::audio::{PlaybackCommand, PlaybackState};
use super::components::PageType;

/// 应用程序消息类型
#[derive(Debug, Clone)]
pub enum Message {
    /// 播放/暂停切换
    PlayPause,
    /// 打开文件对话框（音频文件多选，播放列表单选，有验证逻辑）
    OpenFile,
    /// 多个音频文件选择完成
    MultipleAudioFilesSelected(Vec<String>),
    /// 播放列表项目选择
    PlaylistItemSelected(usize),
    /// 播放列表卡片选中切换（用于视觉选中效果）
    PlaylistCardToggled(String),
    /// 播放列表卡片的更多菜单按钮被点击（占位，后续实现）
    PlaylistCardMoreClicked(String),

    /// 开始重命名指定播放列表
    PlaylistCardActionRenameStart(String),
    /// 重命名输入内容变化
    PlaylistCardRenameNameChanged(String),
    /// 确认重命名
    PlaylistCardRenameConfirm,
    /// 取消重命名
    PlaylistCardRenameCancel,
    /// 删除播放列表
    PlaylistCardActionDelete(String),
    /// 为播放列表添加音乐（打开文件对话框）
    PlaylistCardActionAddMusic(String),
    /// 添加音乐选择完成（携带播放列表路径和所选文件）
    PlaylistAddMusicFilesSelected(String, Vec<String>),

    /// 开始创建播放列表（显示输入框）
    StartCreatePlaylist,
    /// 创建播放列表名称变化
    CreatePlaylistNameChanged(String),
    /// 确认创建播放列表
    ConfirmCreatePlaylist,
    /// 取消创建播放列表
    CancelCreatePlaylist,

    /// 下一首
    NextTrack,
    /// 上一首
    PreviousTrack,
    /// 定时器触发（用于更新进度）
    Tick,
    /// 播放状态更新
    PlaybackStateUpdate(PlaybackState),
    /// 音频会话启动
    AudioSessionStarted(mpsc::UnboundedSender<PlaybackCommand>),
    /// 系统事件
    EventOccurred(Event),
    /// 切换播放列表/歌词显示视图
    ToggleView,
    /// 窗口大小变化
    WindowResized(f32, f32),
    /// 进度条变化（值为0.0-1.0的比例）
    ProgressChanged(f32),
    /// 切换主题
    ToggleTheme,
    /// 页面切换
    PageChanged(PageType),
    /// 播放模式切换
    TogglePlayMode,
    /// 配置更新
    ConfigUpdate,
    /// 语言切换
    LanguageChanged(String),
    /// 配置重置
    ResetConfig,
    /// AudioFile 后台加载完成（文件路径，加载是否成功）
    AudioFileLoaded(String, bool),
    /// 异步估算时长完成（文件路径，估算的时长）
    AudioDurationEstimated(String, Option<f64>),
} 