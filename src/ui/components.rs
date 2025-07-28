//! UI组件模块
//! 
//! 包含可重用的UI组件。

use iced::{
    widget::{button, column, row, text, progress_bar, scrollable, Space},
    Element, Length,
};

use crate::audio::{AudioInfo, PlaybackState};
use crate::playlist::Playlist;
use crate::utils::format_duration;
use super::Message;

/// 视图类型枚举
#[derive(Debug, Clone, PartialEq, Default)]
pub enum ViewType {
    /// 播放列表视图
    #[default]
    Playlist,
    /// 歌词显示视图
    Lyrics,
}

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

/// 创建视图切换按钮
/// 
/// # 参数
/// * `current_view` - 当前视图类型
/// 
/// # 返回
/// 视图切换按钮UI元素
pub fn view_toggle_button(current_view: &ViewType) -> Element<'static, Message> {
    let button_text = match current_view {
        ViewType::Playlist => "切换到歌词",
        ViewType::Lyrics => "切换到播放列表",
    };
    
    button(button_text)
        .on_press(Message::ToggleView)
        .into()
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
            
            let song_name = if is_current {
                format!("▶ {}", item.name)
            } else {
                format!("  {}", item.name)
            };
            
            let duration_text = item.duration.map_or("未知时长".to_string(), |d| format_duration(d));
            
            let row_content = row![
                text(song_name).width(Length::FillPortion(3)),
                text(duration_text).width(Length::FillPortion(1)).align_x(iced::alignment::Horizontal::Right),
            ].spacing(10);
            
            let btn = button(row_content)
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

/// 创建歌词显示组件
/// 
/// # 参数
/// * `file_path` - 当前文件路径
/// * `is_playing` - 是否正在播放
/// * `current_time` - 当前播放时间
/// * `lyrics` - 当前歌词数据
/// 
/// # 返回
/// 歌词显示UI元素
pub fn lyrics_view(file_path: &str, is_playing: bool, current_time: f64, lyrics: &Option<crate::lyrics::Lyrics>) -> Element<'static, Message> {
    if file_path.is_empty() {
        return column![
            text("歌词显示").size(16),
            text("请选择音频文件"),
        ].spacing(10).into();
    }
    
    // 创建歌词内容
    let mut lyrics_elements = Vec::<Element<Message>>::new();
    
    // 添加标题，包含歌曲信息
    if let Some(ref lyrics_data) = lyrics {
        let title = if let Some(ref title) = lyrics_data.metadata.title {
            title.clone()
        } else {
            // 从文件路径提取文件名
            std::path::Path::new(file_path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("未知歌曲")
                .to_string()
        };
        
        lyrics_elements.push(text(title).size(18).into());
        
        if let Some(ref artist) = lyrics_data.metadata.artist {
            lyrics_elements.push(text(format!("演唱: {}", artist)).size(14).into());
        }
        
        lyrics_elements.push(text("").into()); // 空行
    } else {
        lyrics_elements.push(text("歌词显示").size(16).into());
    }
    
    // 显示歌词内容
    if let Some(ref lyrics_data) = lyrics {
        if lyrics_data.has_lyrics() {
            // 获取当前歌词行索引
            let current_line_index = lyrics_data.get_current_line_index(current_time);
            
            // 显示歌词行
            for (index, line) in lyrics_data.lines.iter().enumerate() {
                let is_current = current_line_index == Some(index);
                let is_upcoming = current_line_index.map_or(false, |current| index == current + 1);
                
                // 创建歌词文本
                let lyric_text = if line.text.trim().is_empty() {
                    "♪".to_string() // 空行显示音符
                } else {
                    line.text.clone()
                };
                
                // 根据状态设置样式
                let text_element = if is_current && is_playing {
                    // 当前播放行 - 高亮显示
                    text(format!("▶ {}", lyric_text))
                        .size(16)
                        .style(|theme: &iced::Theme| {
                            let palette = theme.extended_palette();
                            iced::widget::text::Style {
                                color: Some(palette.primary.strong.color),
                            }
                        })
                } else if is_upcoming && is_playing {
                    // 下一行 - 稍微突出显示
                    text(format!("  {}", lyric_text))
                        .size(15)
                        .style(|theme: &iced::Theme| {
                            let palette = theme.extended_palette();
                            iced::widget::text::Style {
                                color: Some(palette.secondary.base.color),
                            }
                        })
                } else if current_line_index.map_or(false, |current| index <= current) {
                    // 已播放的行 - 淡化显示
                    text(format!("  {}", lyric_text))
                        .size(14)
                        .style(|theme: &iced::Theme| {
                            let palette = theme.extended_palette();
                            iced::widget::text::Style {
                                color: Some(iced::Color {
                                    a: 0.6,
                                    ..palette.background.weak.text
                                }),
                            }
                        })
                } else {
                    // 未播放的行 - 正常显示
                    text(format!("  {}", lyric_text)).size(14)
                };
                
                lyrics_elements.push(text_element.into());
            }
            
            // 如果没有当前行，显示播放状态
            if current_line_index.is_none() && is_playing {
                lyrics_elements.push(text("").into());
                lyrics_elements.push(text("♪ 音乐开始了... ♪").size(14).into());
            }
            
        } else {
            // 歌词文件存在但没有歌词内容
            lyrics_elements.push(text("歌词文件已加载，但没有找到歌词内容").into());
            lyrics_elements.push(text("").into());
        }
    } else {
        // 没有歌词文件
        if is_playing {
            lyrics_elements.push(text("♪ 正在播放中... ♪").into());
            lyrics_elements.push(text("").into());
            lyrics_elements.push(text("暂无歌词文件").into());
            lyrics_elements.push(text("").into());
            lyrics_elements.push(text(format!("当前时间: {}", format_duration(current_time))).into());
        } else {
            lyrics_elements.push(text("♪ 歌词显示 ♪").into());
            lyrics_elements.push(text("").into());
            lyrics_elements.push(text("暂停播放中").into());
        }
        
        lyrics_elements.push(text("").into());
        lyrics_elements.push(text("提示：").into());
        lyrics_elements.push(text("• 将 .lrc 歌词文件放在音频文件同目录下").into());
        lyrics_elements.push(text("• 歌词文件名需与音频文件名相同").into());
        lyrics_elements.push(text("• 支持时间同步的LRC格式歌词").into());
    }
    
    // 创建可滚动的歌词显示
    let lyrics_column = column(lyrics_elements).spacing(8).width(Length::Fill);
    
    // 如果有歌词且正在播放，尝试自动滚动到当前行
    let scrollable_lyrics = if lyrics.as_ref().map_or(false, |l| l.has_lyrics()) && is_playing {
        if let Some(_current_index) = lyrics.as_ref().unwrap().get_current_line_index(current_time) {
            // TODO: 实现自动滚动到当前行的功能
            // let scroll_offset = (current_index as f32 * 32.0).max(0.0); // 假设每行约32px高度
            scrollable(lyrics_column)
                .height(Length::Fill)
                .width(Length::Fill)
        } else {
            scrollable(lyrics_column)
                .height(Length::Fill)
                .width(Length::Fill)
        }
    } else {
        scrollable(lyrics_column)
            .height(Length::Fill)
            .width(Length::Fill)
    };
    
    scrollable_lyrics.into()
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