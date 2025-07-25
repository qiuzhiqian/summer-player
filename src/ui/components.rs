//! UI组件模块
//! 
//! 包含可重用的UI组件。

use iced::{
    widget::{button, column, row, text, progress_bar, scrollable, Space},
    Element, Length,
};

use crate::audio::{AudioInfo, PlaybackState};
use crate::playlist::{Playlist, PlaylistItem};
use crate::utils::format_duration;
use super::Message;

/// 创建文件信息显示组件
/// 
/// # 参数
/// * `audio_info` - 音频信息
/// * `file_path` - 文件路径
/// 
/// # 返回
/// 文件信息UI元素
pub fn file_info_view(audio_info: Option<&AudioInfo>, file_path: &str) -> Element<'static, Message> {
    if let Some(info) = audio_info {
        column![
            text(format!("文件: {}", file_path)),
            text(format!("声道: {}", info.channels)),
            text(format!("采样率: {} Hz", info.sample_rate)),
            text(format!("时长: {}", 
                if let Some(duration) = info.duration {
                    format_duration(duration)
                } else {
                    "未知".to_string()
                }
            )),
        ].spacing(10).into()
    } else {
        column![
            text("未选择文件"),
        ].spacing(10).into()
    }
}

/// 创建播放控制按钮组
/// 
/// # 返回
/// 控制按钮UI元素
pub fn control_buttons_view() -> Element<'static, Message> {
    row![
        button("播放/暂停").on_press(Message::PlayPause),
        button("停止").on_press(Message::Stop),
        button("上一首").on_press(Message::PreviousTrack),
        button("下一首").on_press(Message::NextTrack),
    ].spacing(10).into()
}

/// 创建文件操作按钮组
/// 
/// # 返回
/// 文件操作按钮UI元素
pub fn file_controls_view() -> Element<'static, Message> {
    row![
        button("打开文件").on_press(Message::OpenFile),
    ].spacing(10).into()
}

/// 创建播放进度显示组件
/// 
/// # 参数
/// * `playback_state` - 播放状态
/// 
/// # 返回
/// 进度显示UI元素
pub fn progress_view(playback_state: &PlaybackState) -> Element<'static, Message> {
    if playback_state.total_duration > 0.0 {
        let progress_value = (playback_state.current_time / playback_state.total_duration) as f32;
        column![
            progress_bar(0.0..=1.0, progress_value),
            text(format!("{} / {}", 
                format_duration(playback_state.current_time),
                format_duration(playback_state.total_duration)
            )),
        ].spacing(5).into()
    } else {
        column![
            progress_bar(0.0..=1.0, 0.0),
            text("0:00 / 0:00"),
        ].spacing(5).into()
    }
}

/// 创建播放状态显示组件
/// 
/// # 参数
/// * `is_playing` - 是否正在播放
/// 
/// # 返回
/// 状态显示UI元素
pub fn status_view(is_playing: bool) -> Element<'static, Message> {
    text(format!("状态: {}", 
        if is_playing { "播放中" } else { "已停止" }
    )).into()
}

/// 创建播放列表视图组件
/// 
/// # 参数
/// * `playlist` - 播放列表
/// * `playlist_loaded` - 是否已加载播放列表
/// * `is_playing` - 是否正在播放
/// 
/// # 返回
/// 播放列表UI元素
pub fn playlist_view(
    playlist: &Playlist, 
    playlist_loaded: bool, 
    is_playing: bool
) -> Element<'static, Message> {
    if playlist_loaded {
        let playlist_items: Vec<Element<Message>> = playlist.items().iter().enumerate().map(|(index, item)| {
            let is_current = playlist.current_index() == Some(index);
            let is_playing_current = is_current && is_playing;
            
            let item_text = if is_current {
                format!("▶ {} ({})", 
                    item.name,
                    item.duration.map_or("未知时长".to_string(), |d| format_duration(d))
                )
            } else {
                format!("  {} ({})", 
                    item.name,
                    item.duration.map_or("未知时长".to_string(), |d| format_duration(d))
                )
            };
            
            let btn = button(text(item_text))
                .on_press(Message::PlaylistItemSelected(index))
                .width(Length::Fill);
            
            // 为当前播放的项目添加样式
            if is_playing_current {
                btn.style(|theme: &iced::Theme, status| {
                    let palette = theme.extended_palette();
                    match status {
                        iced::widget::button::Status::Active => iced::widget::button::Style {
                            background: Some(iced::Background::Color(palette.primary.weak.color)),
                            text_color: palette.primary.strong.text,
                            border: iced::Border::default(),
                            shadow: iced::Shadow::default(),
                        },
                        iced::widget::button::Status::Hovered => iced::widget::button::Style {
                            background: Some(iced::Background::Color(palette.primary.base.color)),
                            text_color: palette.primary.strong.text,
                            border: iced::Border::default(),
                            shadow: iced::Shadow::default(),
                        },
                        _ => iced::widget::button::Style::default(),
                    }
                }).into()
            } else if is_current {
                btn.style(|theme: &iced::Theme, status| {
                    let palette = theme.extended_palette();
                    match status {
                        iced::widget::button::Status::Active => iced::widget::button::Style {
                            background: Some(iced::Background::Color(palette.secondary.weak.color)),
                            text_color: palette.secondary.strong.text,
                            border: iced::Border::default(),
                            shadow: iced::Shadow::default(),
                        },
                        iced::widget::button::Status::Hovered => iced::widget::button::Style {
                            background: Some(iced::Background::Color(palette.secondary.base.color)),
                            text_color: palette.secondary.strong.text,
                            border: iced::Border::default(),
                            shadow: iced::Shadow::default(),
                        },
                        _ => iced::widget::button::Style::default(),
                    }
                }).into()
            } else {
                btn.into()
            }
        }).collect();
        
        column![
            text(format!("播放列表 ({} 首歌曲)", playlist.len())).size(16),
            scrollable(
                column(playlist_items).spacing(5).width(Length::Fill)
            ).height(Length::Fill).width(Length::Fill),
        ].spacing(10).into()
    } else {
        column![
            text("未加载播放列表"),
        ].spacing(10).into()
    }
}

/// 创建应用程序标题
/// 
/// # 返回
/// 标题UI元素
pub fn title_view() -> Element<'static, Message> {
    text("音频播放器").size(24).into()
}

/// 创建空白填充组件
/// 
/// # 返回
/// 空白填充UI元素
pub fn spacer() -> Element<'static, Message> {
    Space::new(Length::Fill, Length::Fill).into()
} 