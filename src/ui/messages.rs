//! UI消息定义模块
//! 
//! 定义应用程序中使用的所有UI消息类型。

use iced::event::Event;
use tokio::sync::mpsc;

use crate::audio::{PlaybackCommand, PlaybackState};

/// 应用程序消息类型
#[derive(Debug, Clone)]
pub enum Message {
    /// 播放/暂停切换
    PlayPause,
    /// 打开文件对话框
    OpenFile,
    /// 文件选择完成
    FileSelected(Option<String>),
    /// 播放列表项目选择
    PlaylistItemSelected(usize),
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
    /// 动画更新
    AnimationTick,
    /// 窗口大小变化
    WindowResized(f32, f32),
    /// 进度条变化（值为0.0-1.0的比例）
    ProgressChanged(f32),
} 