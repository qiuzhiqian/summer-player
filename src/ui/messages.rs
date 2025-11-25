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
    /// 播放列表卡片的更多菜单按钮被点击
    PlaylistCardMoreClicked(String),
    /// 播放列表卡片的重命名操作开始
    PlaylistCardActionRenameStart(String),
    /// 重命名输入内容变化
    PlaylistCardRenameNameChanged(String),
    /// 重命名确认
    PlaylistCardRenameConfirm,
    /// 重命名取消
    PlaylistCardRenameCancel,
    /// 播放列表卡片的删除操作
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
    /// 窗口大小变化
    WindowResized(f32, f32),
    /// 进度条变化（值为0.0-1.0的比例）
    ProgressChanged(f32),
    /// 切换主题
    ToggleTheme,
    /// 切换主页/歌词页面
    ToggleHomeLyrics,
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
    
    /// 歌曲项菜单相关消息
    /// 展开歌曲项菜单
    ExpandSongItemMenu(usize),
    /// 关闭歌曲项菜单
    DismissSongItemMenu(usize),
    /// 查看歌曲详情
    SongItemActionDetails(usize),
    /// 编辑歌曲标签
    SongItemActionEditTags(usize),
    /// 移除歌曲
    SongItemActionRemove(usize),
    /// ID3标签字段变化
    Id3TagFieldChanged {
        field: Id3TagField,
        value: String,
    },
    /// 确认ID3标签修改
    ConfirmId3TagChanges,
    /// 从Id3Tag页面返回
    ReturnFromId3Tag,
    /// 从Lyrics页面返回
    ReturnFromLyrics,
    /// 展开菜单
    ExpandMenu(String),
    /// 关闭菜单
    DismissMenu(String),
    
    /// 模态窗口相关消息
    /// 显示模态窗口
    ShowModal(ModalType),
    /// 隐藏模态窗口
    HideModal,
    /// 模态窗口输入字段变化
    ModalInputChanged {
        field_type: String,
        field: String,
        value: String,
    },
    /// 显示播放列表重命名模态窗口
    ShowPlaylistRenameModal(String),
    /// 播放列表重命名模态窗口
    PlaylistCardRenameModal(String, String),
    // 歌曲详情模态窗口
    //ShowSongDetailsModal(usize),
    // 编辑标签模态窗口
    //ShowEditTagsModal(usize),
}

/// ID3标签字段枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Id3TagField {
    Title,
    Album,
    Artist,
    Year,
    TrackNumber,
    Genre,
}

/// 模态窗口类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModalType {
    None,
    PlaylistRename,
    SongDetails,
    EditTags,
}

/// 模态窗口数据
#[derive(Debug, Clone, Default)]
pub struct ModalData {
    /// 播放列表重命名数据
    pub playlist_rename_data: PlaylistRenameData,
    /// 歌曲详情数据
    pub song_details_data: SongDetailsData,
    /// 编辑标签数据
    pub edit_tags_data: EditTagsData,
}

/// 播放列表重命名数据
#[derive(Debug, Clone)]
pub struct PlaylistRenameData {
    /// 播放列表路径
    pub playlist_path: String,
    /// 新名称
    pub new_name: String,
}

impl Default for PlaylistRenameData {
    fn default() -> Self {
        Self {
            playlist_path: String::new(),
            new_name: String::new(),
        }
    }
}

/// 歌曲详情数据
#[derive(Debug, Clone)]
pub struct SongDetailsData {
    /// 歌曲标题
    pub title: String,
    /// 时长
    pub duration: Option<f64>,
    /// 文件大小
    pub file_size: String,
    /// 文件路径
    pub file_path: String,
    /// 文件格式
    pub format: String,
    /// 比特率
    pub bitrate: String,
    /// 采样率
    pub sample_rate: String,
}

impl Default for SongDetailsData {
    fn default() -> Self {
        Self {
            title: String::new(),
            duration: None,
            file_size: String::new(),
            file_path: String::new(),
            format: String::new(),
            bitrate: String::new(),
            sample_rate: String::new(),
        }
    }
}

/// 编辑标签数据
#[derive(Debug, Clone)]
pub struct EditTagsData {
    /// 文件路径
    pub file_path: String,
    /// 歌曲标题
    pub title: String,
    /// 专辑名称
    pub album: String,
    /// 艺术家
    pub artist: String,
    /// 年代
    pub year: String,
    /// 音轨号
    pub track_number: String,
    /// 音乐流派
    pub genre: String,
}

impl Default for EditTagsData {
    fn default() -> Self {
        Self {
            file_path: String::new(),
            title: String::new(),
            album: String::new(),
            artist: String::new(),
            year: String::new(),
            track_number: String::new(),
            genre: String::new(),
        }
    }
}
