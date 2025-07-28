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
pub fn lyrics_view(file_path: &str, is_playing: bool, current_time: f64, lyrics: &Option<crate::lyrics::Lyrics>, window_height: f32) -> Element<'static, Message> {
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
    
    // 显示歌词内容 - 动态行数显示，当前行居中
    if let Some(ref lyrics_data) = lyrics {
        if lyrics_data.has_lyrics() {
            // 动态计算显示行数 - 基于窗口高度和内容
            let total_lyrics_count = lyrics_data.lines.len();
            let display_lines = calculate_optimal_display_lines(total_lyrics_count, window_height);
            let center_line = display_lines / 2; // 动态中心位置
            
            // 获取当前歌词行索引
            let current_line_index = lyrics_data.get_current_line_index(current_time);
            
            // 计算显示范围 - 让当前行尽量居中
            let (start_index, visible_count) = if let Some(current_idx) = current_line_index {
                // 计算显示窗口的起始位置，让当前行居中
                let ideal_start = if current_idx >= center_line {
                    current_idx - center_line
                } else {
                    0
                };
                
                // 确保不超出歌词总数
                let available_lyrics = lyrics_data.lines.len();
                let actual_start = if ideal_start + display_lines > available_lyrics {
                    if available_lyrics > display_lines {
                        available_lyrics - display_lines
                    } else {
                        0
                    }
                } else {
                    ideal_start
                };
                
                let visible_count = (available_lyrics - actual_start).min(display_lines);
                (actual_start, visible_count)
            } else {
                // 如果没有当前行，显示前面的歌词
                let visible_count = lyrics_data.lines.len().min(display_lines);
                (0, visible_count)
            };
            
            // 如果歌词总数少于显示行数，添加前置空行来保持居中效果
            let total_lyrics = lyrics_data.lines.len();
            let (pre_empty_lines, post_empty_lines) = if total_lyrics < display_lines {
                let empty_lines = display_lines - total_lyrics;
                let pre_lines = empty_lines / 2;
                let post_lines = empty_lines - pre_lines;
                (pre_lines, post_lines)
            } else {
                (0, 0)
            };
            
            // 添加前置空行
            for _ in 0..pre_empty_lines {
                lyrics_elements.push(
                    text("")
                        .size(16)
                        .align_x(iced::alignment::Horizontal::Center)
                        .into()
                );
            }
            
            // 创建实际歌词显示行
            for i in 0..visible_count {
                let lyrics_index = start_index + i;
                
                if lyrics_index < lyrics_data.lines.len() {
                    let line = &lyrics_data.lines[lyrics_index];
                    let is_current = current_line_index == Some(lyrics_index);
                    let is_upcoming = current_line_index.map_or(false, |current| lyrics_index == current + 1);
                    
                    // 创建歌词文本
                    let lyric_text = if line.text.trim().is_empty() {
                        "♪".to_string() // 空行显示音符
                    } else {
                        line.text.clone()
                    };
                    
                    // 根据状态设置样式
                    let text_element = if is_current && is_playing {
                        // 当前播放行 - 高亮显示，居中对齐
                        text(format!("▶ {}", lyric_text))
                            .size(18)
                            .align_x(iced::alignment::Horizontal::Center)
                            .style(|theme: &iced::Theme| {
                                let palette = theme.extended_palette();
                                iced::widget::text::Style {
                                    color: Some(palette.primary.strong.color),
                                }
                            })
                    } else if is_upcoming && is_playing {
                        // 下一行 - 稍微突出显示
                        text(lyric_text)
                            .size(16)
                            .align_x(iced::alignment::Horizontal::Center)
                            .style(|theme: &iced::Theme| {
                                let palette = theme.extended_palette();
                                iced::widget::text::Style {
                                    color: Some(palette.secondary.base.color),
                                }
                            })
                    } else if current_line_index.map_or(false, |current| lyrics_index <= current) {
                        // 已播放的行 - 淡化显示
                        text(lyric_text)
                            .size(14)
                            .align_x(iced::alignment::Horizontal::Center)
                            .style(|theme: &iced::Theme| {
                                let palette = theme.extended_palette();
                                iced::widget::text::Style {
                                    color: Some(iced::Color {
                                        a: 0.4,
                                        ..palette.background.weak.text
                                    }),
                                }
                            })
                    } else {
                        // 未播放的行 - 正常显示但稍微淡一些
                        text(lyric_text)
                            .size(14)
                            .align_x(iced::alignment::Horizontal::Center)
                            .style(|theme: &iced::Theme| {
                                let palette = theme.extended_palette();
                                iced::widget::text::Style {
                                    color: Some(iced::Color {
                                        a: 0.7,
                                        ..palette.background.weak.text
                                    }),
                                }
                            })
                    };
                    
                    lyrics_elements.push(text_element.into());
                }
            }
            
            // 添加后置空行
            for _ in 0..post_empty_lines {
                lyrics_elements.push(
                    text("")
                        .size(16)
                        .align_x(iced::alignment::Horizontal::Center)
                        .into()
                );
            }
            
            // 如果没有当前行且正在播放，在底部显示提示
            if current_line_index.is_none() && is_playing {
                lyrics_elements.push(text("").into());
                lyrics_elements.push(
                    text("♪ 音乐开始了... ♪")
                        .size(14)
                        .align_x(iced::alignment::Horizontal::Center)
                        .into()
                );
            }
            
        } else {
            // 歌词文件存在但没有歌词内容
            lyrics_elements.push(
                text("歌词文件已加载，但没有找到歌词内容")
                    .align_x(iced::alignment::Horizontal::Center)
                    .into()
            );
        }
    } else {
        // 没有歌词文件
        if is_playing {
            lyrics_elements.push(
                text("♪ 正在播放中... ♪")
                    .size(16)
                    .align_x(iced::alignment::Horizontal::Center)
                    .into()
            );
            lyrics_elements.push(text("").into());
            lyrics_elements.push(
                text("暂无歌词文件")
                    .align_x(iced::alignment::Horizontal::Center)
                    .into()
            );
            lyrics_elements.push(text("").into());
            lyrics_elements.push(
                text(format!("当前时间: {}", format_duration(current_time)))
                    .size(12)
                    .align_x(iced::alignment::Horizontal::Center)
                    .into()
            );
        } else {
            lyrics_elements.push(
                text("♪ 歌词显示 ♪")
                    .size(16)
                    .align_x(iced::alignment::Horizontal::Center)
                    .into()
            );
            lyrics_elements.push(text("").into());
            lyrics_elements.push(
                text("暂停播放中")
                    .align_x(iced::alignment::Horizontal::Center)
                    .into()
            );
        }
        
        lyrics_elements.push(text("").into());
        lyrics_elements.push(text("提示：").size(12).into());
        lyrics_elements.push(text("• 将 .lrc 歌词文件放在音频文件同目录下").size(11).into());
        lyrics_elements.push(text("• 歌词文件名需与音频文件名相同").size(11).into());
        lyrics_elements.push(text("• 支持时间同步的LRC格式歌词").size(11).into());
    }
    
    // 创建高度自适应的歌词显示区域，不使用滚动条
    column(lyrics_elements)
        .spacing(12)  // 增加行间距使视觉更舒适
        .width(Length::Fill)
        .height(Length::Shrink)  // 高度自适应内容
        .padding(20)  // 添加内边距
        .into()
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

/// 计算最佳歌词显示行数
/// 
/// # 参数
/// * `total_lyrics_count` - 歌词总行数
/// * `window_height` - 当前窗口高度
/// 
/// # 返回
/// 最佳显示行数
fn calculate_optimal_display_lines(total_lyrics_count: usize, window_height: f32) -> usize {
    // 基于实际窗口高度和歌词总数的动态策略
    
    // 1. 根据窗口高度计算可用空间
    let title_and_metadata_height = 80.0;  // 标题和艺术家信息
    let toggle_button_height = 40.0;       // 切换按钮
    let progress_bar_height = 60.0;        // 进度条区域
    let padding_and_spacing = 60.0;        // 内边距和间距
    
    let available_height = window_height 
        - title_and_metadata_height 
        - toggle_button_height 
        - progress_bar_height 
        - padding_and_spacing;
    
    // 2. 根据可用高度计算行数
    let line_height = 24.0; // 每行预估高度（字体 + 行间距）
    let calculated_lines = (available_height / line_height) as usize;
    
    // 3. 基于歌词数量调整策略
    let content_based_lines = if total_lyrics_count <= 7 {
        9  // 歌词很少时，固定显示9行保持居中
    } else {
        // 根据歌词数量和计算出的行数取较小值
        calculated_lines.min(total_lyrics_count + 4) // 允许前后各2行的上下文
    };
    
    // 4. 确保在合理范围内，并优先保持奇数（有助于居中）
    let final_lines = content_based_lines.max(5).min(21);
    if final_lines % 2 == 0 {
        final_lines + 1
    } else {
        final_lines
    }
}

 