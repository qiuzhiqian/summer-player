//! UI组件模块
//! 
//! 包含可重用的UI组件。

use iced::{
    widget::{button, column, row, text, progress_bar, scrollable, Space, container},
    Element, Length, Border, Shadow, Background, Color,
    alignment::{Horizontal, Vertical},
    theme::Theme,
    border::Radius,
};
use iced::advanced::text::Shaping;

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



/// 现代化卡片容器样式
fn card_style() -> fn(&Theme) -> container::Style {
    |theme: &Theme| {
        let palette = theme.extended_palette();
        container::Style {
            background: Some(Background::Color(palette.background.base.color)),
            border: Border {
                radius: Radius::from(12.0),
                width: 1.0,
                color: palette.background.strong.color,
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.1),
                offset: iced::Vector::new(0.0, 2.0),
                blur_radius: 8.0,
            },
            text_color: Some(palette.background.base.text),
        }
    }
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
    let content = if let Some(info) = audio_info {
        let file_name = std::path::Path::new(file_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("未知文件")
            .to_string();
        
        // 创建音频信息列
        let mut audio_info_column = column![
            info_row("🎵", "声道", &format!("{}", info.channels)),
            info_row("📡", "采样率", &format!("{} Hz", info.sample_rate)),
            info_row("⏱️", "时长", &if let Some(duration) = info.duration {
                format_duration(duration)
            } else {
                "未知".to_string()
            }),
        ].spacing(8);
        
        // 如果有比特深度信息，添加它
        if let Some(bits) = info.bits_per_sample {
            audio_info_column = audio_info_column.push(
                info_row("🎚️", "位深", &format!("{} bit", bits))
            );
        }
        
        // 创建元数据信息列
        let mut metadata_column = column![].spacing(8);
        
        // 添加元数据信息
        if let Some(title) = &info.metadata.title {
            metadata_column = metadata_column.push(
                info_row("🎤", "标题", &title.clone())
            );
        }
        
        if let Some(artist) = &info.metadata.artist {
            metadata_column = metadata_column.push(
                info_row("🎨", "艺术家", &artist.clone())
            );
        }
        
        if let Some(album) = &info.metadata.album {
            metadata_column = metadata_column.push(
                info_row("💿", "专辑", &album.clone())
            );
        }
        
        if let Some(year) = &info.metadata.year {
            metadata_column = metadata_column.push(
                info_row("📅", "年份", &year.clone())
            );
        }
        
        if let Some(genre) = &info.metadata.genre {
            metadata_column = metadata_column.push(
                info_row("🎭", "流派", &genre.clone())
            );
        }
        
        if let Some(track_number) = &info.metadata.track_number {
            metadata_column = metadata_column.push(
                info_row("🔢", "音轨", &track_number.clone())
            );
        }
        
        if let Some(composer) = &info.metadata.composer {
            metadata_column = metadata_column.push(
                info_row("✍️", "作曲", &composer.clone())
            );
        }
        
        // 如果没有元数据，显示文件名
        let display_title = info.metadata.title.clone()
            .unwrap_or(file_name);
        
        {
            let mut main_column = column![
                // 显示标题（优先使用元数据中的标题）
                text(display_title)
                    .size(16)
                    .style(|theme: &Theme| {
                        let palette = theme.extended_palette();
                        text::Style {
                            color: Some(palette.primary.base.color),
                        }
                    }),
                
                // 技术信息部分
                text("技术信息")
                    .size(14)
                    .style(|theme: &Theme| {
                        let palette = theme.extended_palette();
                        text::Style {
                            color: Some(Color {
                                a: 0.8,
                                ..palette.background.base.text
                            }),
                        }
                    }),
                audio_info_column,
            ].spacing(12);
            
            // 如果有元数据信息，添加元数据部分
            if info.metadata.title.is_some() || info.metadata.artist.is_some() || 
               info.metadata.album.is_some() || info.metadata.year.is_some() ||
               info.metadata.genre.is_some() || info.metadata.track_number.is_some() ||
               info.metadata.composer.is_some() {
                main_column = main_column.push(
                    column![
                        text("元数据信息")
                            .size(14)
                            .style(|theme: &Theme| {
                                let palette = theme.extended_palette();
                                text::Style {
                                    color: Some(Color {
                                        a: 0.8,
                                        ..palette.background.base.text
                                    }),
                                }
                            }),
                        metadata_column,
                    ].spacing(8)
                );
            }
            
            main_column
        }
    } else {
        column![
            text("🎼")
                .size(32)
                .align_x(Horizontal::Center)
                .shaping(Shaping::Advanced),
            text("未选择文件")
                .size(14)
                .align_x(Horizontal::Center)
                .style(|theme: &Theme| {
                    let palette = theme.extended_palette();
                    text::Style {
                        color: Some(Color {
                            a: 0.7,
                            ..palette.background.base.text
                        }),
                    }
                }),
        ].spacing(8).align_x(Horizontal::Center)
    };

    container(content)
        .style(card_style())
        .padding(16)
        .width(Length::Fill)
        .into()
}

/// 创建信息行
fn info_row(icon: &'static str, label: &'static str, value: &str) -> Element<'static, Message> {
    row![
        text(icon).size(14).shaping(Shaping::Advanced),
        text(format!("{}: {}", label, value))
            .size(12)
            .style(|theme: &Theme| {
                let palette = theme.extended_palette();
                text::Style {
                    color: Some(Color {
                        a: 0.8,
                        ..palette.background.base.text
                    }),
                }
            }),
    ]
    .spacing(8)
    .align_y(Vertical::Center)
    .into()
}

/// 创建播放控制按钮组
/// 
/// # 返回
/// 控制按钮UI元素
pub fn control_buttons_view() -> Element<'static, Message> {
    container(
        row![
            // 上一首
            button(
                container(text("⏮").size(18).shaping(Shaping::Advanced))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(Horizontal::Center)
                    .align_y(Vertical::Center)
            )
            .style(|theme: &Theme, status| {
                let palette = theme.extended_palette();
                match status {
                    button::Status::Active => button::Style {
                        background: Some(Background::Color(Color {
                            a: 0.8,
                            ..palette.secondary.base.color
                        })),
                        text_color: Color::WHITE,
                        border: Border {
                            radius: Radius::from(24.0),
                            width: 0.0,
                            color: Color::TRANSPARENT,
                        },
                        shadow: Shadow {
                            color: Color::from_rgba(0.0, 0.0, 0.0, 0.2),
                            offset: iced::Vector::new(0.0, 3.0),
                            blur_radius: 6.0,
                        },
                    },
                    button::Status::Hovered => button::Style {
                        background: Some(Background::Color(palette.secondary.strong.color)),
                        text_color: Color::WHITE,
                        border: Border {
                            radius: Radius::from(24.0),
                            width: 0.0,
                            color: Color::TRANSPARENT,
                        },
                        shadow: Shadow {
                            color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
                            offset: iced::Vector::new(0.0, 6.0),
                            blur_radius: 12.0,
                        },
                    },
                    button::Status::Pressed => button::Style {
                        background: Some(Background::Color(palette.secondary.weak.color)),
                        text_color: Color::WHITE,
                        border: Border {
                            radius: Radius::from(24.0),
                            width: 0.0,
                            color: Color::TRANSPARENT,
                        },
                        shadow: Shadow {
                            color: Color::from_rgba(0.0, 0.0, 0.0, 0.15),
                            offset: iced::Vector::new(0.0, 1.0),
                            blur_radius: 3.0,
                        },
                    },
                    _ => button::Style::default(),
                }
            })
            .width(Length::Fixed(48.0))
            .height(Length::Fixed(48.0))
            .on_press(Message::PreviousTrack),
            
            // 播放/暂停 - 主要按钮，更大更突出
            button(
                container(text("⏯").size(24).shaping(Shaping::Advanced))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(Horizontal::Center)
                    .align_y(Vertical::Center)
            )
            .style(|theme: &Theme, status| {
                let palette = theme.extended_palette();
                match status {
                    button::Status::Active => button::Style {
                        background: Some(Background::Color(palette.primary.strong.color)),
                        text_color: Color::WHITE,
                        border: Border {
                            radius: Radius::from(32.0),
                            width: 2.0,
                            color: Color {
                                a: 0.3,
                                ..Color::WHITE
                            },
                        },
                        shadow: Shadow {
                            color: Color::from_rgba(0.0, 0.0, 0.0, 0.25),
                            offset: iced::Vector::new(0.0, 4.0),
                            blur_radius: 12.0,
                        },
                    },
                    button::Status::Hovered => button::Style {
                        background: Some(Background::Color(Color {
                            r: palette.primary.strong.color.r * 1.1,
                            g: palette.primary.strong.color.g * 1.1,
                            b: palette.primary.strong.color.b * 1.1,
                            a: 1.0,
                        })),
                        text_color: Color::WHITE,
                        border: Border {
                            radius: Radius::from(32.0),
                            width: 2.0,
                            color: Color {
                                a: 0.5,
                                ..Color::WHITE
                            },
                        },
                        shadow: Shadow {
                            color: Color::from_rgba(0.0, 0.0, 0.0, 0.35),
                            offset: iced::Vector::new(0.0, 6.0),
                            blur_radius: 16.0,
                        },
                    },
                    button::Status::Pressed => button::Style {
                        background: Some(Background::Color(palette.primary.base.color)),
                        text_color: Color::WHITE,
                        border: Border {
                            radius: Radius::from(32.0),
                            width: 2.0,
                            color: Color {
                                a: 0.2,
                                ..Color::WHITE
                            },
                        },
                        shadow: Shadow {
                            color: Color::from_rgba(0.0, 0.0, 0.0, 0.2),
                            offset: iced::Vector::new(0.0, 2.0),
                            blur_radius: 6.0,
                        },
                    },
                    _ => button::Style::default(),
                }
            })
            .width(Length::Fixed(64.0))
            .height(Length::Fixed(64.0))
            .on_press(Message::PlayPause),
            
            // 停止
            button(
                container(text("⏹").size(18).shaping(Shaping::Advanced))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(Horizontal::Center)
                    .align_y(Vertical::Center)
            )
            .style(|theme: &Theme, status| {
                let palette = theme.extended_palette();
                match status {
                    button::Status::Active => button::Style {
                        background: Some(Background::Color(Color {
                            a: 0.8,
                            ..palette.danger.base.color
                        })),
                        text_color: Color::WHITE,
                        border: Border {
                            radius: Radius::from(24.0),
                            width: 0.0,
                            color: Color::TRANSPARENT,
                        },
                        shadow: Shadow {
                            color: Color::from_rgba(0.0, 0.0, 0.0, 0.2),
                            offset: iced::Vector::new(0.0, 3.0),
                            blur_radius: 6.0,
                        },
                    },
                    button::Status::Hovered => button::Style {
                        background: Some(Background::Color(palette.danger.strong.color)),
                        text_color: Color::WHITE,
                        border: Border {
                            radius: Radius::from(24.0),
                            width: 0.0,
                            color: Color::TRANSPARENT,
                        },
                        shadow: Shadow {
                            color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
                            offset: iced::Vector::new(0.0, 6.0),
                            blur_radius: 12.0,
                        },
                    },
                    button::Status::Pressed => button::Style {
                        background: Some(Background::Color(palette.danger.weak.color)),
                        text_color: Color::WHITE,
                        border: Border {
                            radius: Radius::from(24.0),
                            width: 0.0,
                            color: Color::TRANSPARENT,
                        },
                        shadow: Shadow {
                            color: Color::from_rgba(0.0, 0.0, 0.0, 0.15),
                            offset: iced::Vector::new(0.0, 1.0),
                            blur_radius: 3.0,
                        },
                    },
                    _ => button::Style::default(),
                }
            })
            .width(Length::Fixed(48.0))
            .height(Length::Fixed(48.0))
            .on_press(Message::Stop),
            
            // 下一首
            button(
                container(text("⏭").size(18).shaping(Shaping::Advanced))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(Horizontal::Center)
                    .align_y(Vertical::Center)
            )
            .style(|theme: &Theme, status| {
                let palette = theme.extended_palette();
                match status {
                    button::Status::Active => button::Style {
                        background: Some(Background::Color(Color {
                            a: 0.8,
                            ..palette.secondary.base.color
                        })),
                        text_color: Color::WHITE,
                        border: Border {
                            radius: Radius::from(24.0),
                            width: 0.0,
                            color: Color::TRANSPARENT,
                        },
                        shadow: Shadow {
                            color: Color::from_rgba(0.0, 0.0, 0.0, 0.2),
                            offset: iced::Vector::new(0.0, 3.0),
                            blur_radius: 6.0,
                        },
                    },
                    button::Status::Hovered => button::Style {
                        background: Some(Background::Color(palette.secondary.strong.color)),
                        text_color: Color::WHITE,
                        border: Border {
                            radius: Radius::from(24.0),
                            width: 0.0,
                            color: Color::TRANSPARENT,
                        },
                        shadow: Shadow {
                            color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
                            offset: iced::Vector::new(0.0, 6.0),
                            blur_radius: 12.0,
                        },
                    },
                    button::Status::Pressed => button::Style {
                        background: Some(Background::Color(palette.secondary.weak.color)),
                        text_color: Color::WHITE,
                        border: Border {
                            radius: Radius::from(24.0),
                            width: 0.0,
                            color: Color::TRANSPARENT,
                        },
                        shadow: Shadow {
                            color: Color::from_rgba(0.0, 0.0, 0.0, 0.15),
                            offset: iced::Vector::new(0.0, 1.0),
                            blur_radius: 3.0,
                        },
                    },
                    _ => button::Style::default(),
                }
            })
            .width(Length::Fixed(48.0))
            .height(Length::Fixed(48.0))
            .on_press(Message::NextTrack),
        ]
        .spacing(16)
        .align_y(Vertical::Center)
    )
    .style(card_style())
    .padding(20)
    .width(Length::Fill)
    .into()
}

/// 创建文件操作按钮组
/// 
/// # 返回
/// 文件操作按钮UI元素
pub fn file_controls_view() -> Element<'static, Message> {
    container(
        button(
            row![
                container(text("📁").size(16).shaping(Shaping::Advanced))
                    .style(|theme: &Theme| {
                        let palette = theme.extended_palette();
                        container::Style {
                            background: Some(Background::Color(Color {
                                a: 0.1,
                                ..palette.primary.base.color
                            })),
                            border: Border {
                                radius: Radius::from(6.0),
                                width: 0.0,
                                color: Color::TRANSPARENT,
                            },
                            shadow: Shadow::default(),
                            text_color: Some(palette.primary.base.color),
                        }
                    })
                    .padding(8),
                text("打开文件").size(14).style(|theme: &Theme| {
                    let palette = theme.extended_palette();
                    text::Style {
                        color: Some(palette.primary.base.text),
                    }
                })
            ].spacing(12).align_y(Vertical::Center)
        )
        .style(|theme: &Theme, status| {
            let palette = theme.extended_palette();
            match status {
                button::Status::Active => button::Style {
                    background: Some(Background::Color(Color {
                        a: 0.05,
                        ..palette.primary.base.color
                    })),
                    text_color: palette.primary.base.text,
                    border: Border {
                        radius: Radius::from(12.0),
                        width: 1.0,
                        color: palette.primary.weak.color,
                    },
                    shadow: Shadow {
                        color: Color::from_rgba(0.0, 0.0, 0.0, 0.08),
                        offset: iced::Vector::new(0.0, 2.0),
                        blur_radius: 4.0,
                    },
                },
                button::Status::Hovered => button::Style {
                    background: Some(Background::Color(Color {
                        a: 0.1,
                        ..palette.primary.base.color
                    })),
                    text_color: palette.primary.strong.text,
                    border: Border {
                        radius: Radius::from(12.0),
                        width: 1.0,
                        color: palette.primary.base.color,
                    },
                    shadow: Shadow {
                        color: Color::from_rgba(0.0, 0.0, 0.0, 0.15),
                        offset: iced::Vector::new(0.0, 4.0),
                        blur_radius: 8.0,
                    },
                },
                button::Status::Pressed => button::Style {
                    background: Some(Background::Color(Color {
                        a: 0.15,
                        ..palette.primary.base.color
                    })),
                    text_color: palette.primary.strong.text,
                    border: Border {
                        radius: Radius::from(12.0),
                        width: 1.0,
                        color: palette.primary.base.color,
                    },
                    shadow: Shadow {
                        color: Color::from_rgba(0.0, 0.0, 0.0, 0.1),
                        offset: iced::Vector::new(0.0, 1.0),
                        blur_radius: 2.0,
                    },
                },
                button::Status::Disabled => button::Style {
                    background: Some(Background::Color(palette.background.weak.color)),
                    text_color: palette.background.weak.text,
                    border: Border {
                        radius: Radius::from(12.0),
                        width: 1.0,
                        color: palette.background.strong.color,
                    },
                    shadow: Shadow::default(),
                },
            }
        })
        .padding([16, 20])
        .width(Length::Fill)
        .on_press(Message::OpenFile)
    )
    .width(Length::Fill)
    .into()
}

/// 创建视图切换按钮
/// 
/// # 参数
/// * `current_view` - 当前视图类型
/// 
/// # 返回
/// 视图切换按钮UI元素
pub fn view_toggle_button(current_view: &ViewType) -> Element<'static, Message> {
    let (icon, text_content, subtitle) = match current_view {
        ViewType::Playlist => ("🎵", "切换到歌词", "查看歌词同步"),
        ViewType::Lyrics => ("📋", "切换到播放列表", "浏览音乐库"),
    };
    
    let is_playlist = matches!(current_view, ViewType::Playlist);
    
    container(
        button(
            row![
                container(text(icon).size(18).shaping(Shaping::Advanced))
                    .style(move |theme: &Theme| {
                        let palette = theme.extended_palette();
                        let color = if is_playlist {
                            palette.success.base.color
                        } else {
                            palette.secondary.base.color
                        };
                        container::Style {
                            background: Some(Background::Color(Color {
                                a: 0.15,
                                ..color
                            })),
                            border: Border {
                                radius: Radius::from(8.0),
                                width: 0.0,
                                color: Color::TRANSPARENT,
                            },
                            shadow: Shadow::default(),
                            text_color: Some(color),
                        }
                    })
                    .padding(10),
                column![
                    text(text_content)
                        .size(14)
                        .style(|theme: &Theme| {
                            let palette = theme.extended_palette();
                            text::Style {
                                color: Some(palette.background.base.text),
                            }
                        }),
                    text(subtitle)
                    .size(11)
                    .style(|theme: &Theme| {
                        let palette = theme.extended_palette();
                        text::Style {
                            color: Some(Color {
                                a: 0.6,
                                ..palette.background.base.text
                            }),
                        }
                    }),
                ].spacing(2)
            ].spacing(12).align_y(Vertical::Center)
        )
        .style(|theme: &Theme, status| {
            let palette = theme.extended_palette();
            match status {
                button::Status::Active => button::Style {
                    background: Some(Background::Color(Color {
                        a: 0.03,
                        ..palette.primary.base.color
                    })),
                    text_color: palette.background.base.text,
                    border: Border {
                        radius: Radius::from(12.0),
                        width: 1.0,
                        color: Color {
                            a: 0.1,
                            ..palette.primary.base.color
                        },
                    },
                    shadow: Shadow {
                        color: Color::from_rgba(0.0, 0.0, 0.0, 0.05),
                        offset: iced::Vector::new(0.0, 1.0),
                        blur_radius: 3.0,
                    },
                },
                button::Status::Hovered => button::Style {
                    background: Some(Background::Color(Color {
                        a: 0.08,
                        ..palette.primary.base.color
                    })),
                    text_color: palette.background.base.text,
                    border: Border {
                        radius: Radius::from(12.0),
                        width: 1.0,
                        color: Color {
                            a: 0.2,
                            ..palette.primary.base.color
                        },
                    },
                    shadow: Shadow {
                        color: Color::from_rgba(0.0, 0.0, 0.0, 0.1),
                        offset: iced::Vector::new(0.0, 3.0),
                        blur_radius: 6.0,
                    },
                },
                button::Status::Pressed => button::Style {
                    background: Some(Background::Color(Color {
                        a: 0.12,
                        ..palette.primary.base.color
                    })),
                    text_color: palette.background.base.text,
                    border: Border {
                        radius: Radius::from(12.0),
                        width: 1.0,
                        color: Color {
                            a: 0.3,
                            ..palette.primary.base.color
                        },
                    },
                    shadow: Shadow {
                        color: Color::from_rgba(0.0, 0.0, 0.0, 0.05),
                        offset: iced::Vector::new(0.0, 1.0),
                        blur_radius: 2.0,
                    },
                },
                button::Status::Disabled => button::Style {
                    background: Some(Background::Color(palette.background.weak.color)),
                    text_color: palette.background.weak.text,
                    border: Border {
                        radius: Radius::from(12.0),
                        width: 1.0,
                        color: palette.background.strong.color,
                    },
                    shadow: Shadow::default(),
                },
            }
        })
        .padding([16, 20])
        .width(Length::Fill)
        .on_press(Message::ToggleView)
    )
    .width(Length::Fill)
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
    let progress_value = if playback_state.total_duration > 0.0 {
        (playback_state.current_time / playback_state.total_duration) as f32
    } else {
        0.0
    };
    
    let current_time_text = format_duration(playback_state.current_time);
    let total_time_text = format_duration(playback_state.total_duration);
    
    container(
        column![
            // 时间显示
            row![
                text(current_time_text)
                    .size(12)
                    .style(|theme: &Theme| {
                        let palette = theme.extended_palette();
                        text::Style {
                            color: Some(palette.primary.base.color),
                        }
                    }),
                Space::new(Length::Fill, Length::Shrink),
                text(total_time_text)
                    .size(12)
                    .style(|theme: &Theme| {
                        let palette = theme.extended_palette();
                        text::Style {
                            color: Some(Color {
                                a: 0.7,
                                ..palette.background.base.text
                            }),
                        }
                    }),
            ],
            
            // 进度条
            container(
                progress_bar(0.0..=1.0, progress_value)
                    .height(Length::Fixed(6.0))
                    .style(|theme: &Theme| {
                        let palette = theme.extended_palette();
                        progress_bar::Style {
                            background: Background::Color(Color {
                                a: 0.3,
                                ..palette.background.strong.color
                            }),
                            bar: Background::Color(palette.primary.strong.color),
                            border: Border {
                                radius: Radius::from(3.0),
                                width: 0.0,
                                color: Color::TRANSPARENT,
                            },
                        }
                    })
            )
            .style(|theme: &Theme| {
                let palette = theme.extended_palette();
                container::Style {
                    background: Some(Background::Color(Color {
                        a: 0.05,
                        ..palette.primary.base.color
                    })),
                    border: Border {
                        radius: Radius::from(6.0),
                        width: 0.0,
                        color: Color::TRANSPARENT,
                    },
                    shadow: Shadow {
                        color: Color::from_rgba(0.0, 0.0, 0.0, 0.05),
                        offset: iced::Vector::new(0.0, 1.0),
                        blur_radius: 2.0,
                    },
                    text_color: None,
                }
            })
            .padding(2),
        ].spacing(8)
    )
    .style(card_style())
    .padding(16)
    .width(Length::Fill)
    .into()
}

/// 创建播放状态显示组件
/// 
/// # 参数
/// * `is_playing` - 是否正在播放
/// 
/// # 返回
/// 状态显示UI元素
pub fn status_view(is_playing: bool) -> Element<'static, Message> {
    let (icon, status_text) = if is_playing {
        ("🎵", "播放中")
    } else {
        ("⏸", "已停止")
    };
    
    container(
        row![
            text(icon).size(16).shaping(Shaping::Advanced),
            text(status_text)
                .size(14)
                .style(move |theme: &Theme| {
                    let palette = theme.extended_palette();
                    text::Style {
                        color: Some(if is_playing {
                            palette.success.base.color
                        } else {
                            palette.background.base.text
                        }),
                    }
                }),
        ].spacing(8).align_y(Vertical::Center)
    )
    .style(card_style())
    .padding(12)
    .width(Length::Fill)
    .into()
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
            
            let (icon, song_name) = if is_current {
                if is_playing_current {
                    ("🎵", item.name.clone())
                } else {
                    ("⏸", item.name.clone())
                }
            } else {
                ("🎼", item.name.clone())
            };
            
            let duration_text = item.duration.map_or("--:--".to_string(), |d| format_duration(d));
            
            let content = container(
                row![
                    text(icon).size(14).shaping(Shaping::Advanced),
                    text(song_name)
                        .shaping(Shaping::Advanced)
                        .width(Length::FillPortion(4))
                        .style(move |theme: &Theme| {
                            let palette = theme.extended_palette();
                            text::Style {
                                color: Some(if is_current {
                                    palette.primary.base.color
                                } else {
                                    palette.background.base.text
                                }),
                            }
                        }),
                    text(duration_text)
                        .width(Length::FillPortion(1))
                        .size(12)
                        .align_x(Horizontal::Right)
                        .style(|theme: &Theme| {
                            let palette = theme.extended_palette();
                            text::Style {
                                color: Some(Color {
                                    a: 0.7,
                                    ..palette.background.base.text
                                }),
                            }
                        }),
                ].spacing(12).align_y(Vertical::Center)
            )
            .padding([8, 12])
            .width(Length::Fill);
            
            let btn = button(content)
                .on_press(Message::PlaylistItemSelected(index))
                .width(Length::Fill)
                .style(move |theme: &Theme, status| {
                    let palette = theme.extended_palette();
                    
                    if is_playing_current {
                        match status {
                            button::Status::Active => button::Style {
                                background: Some(Background::Color(palette.primary.weak.color)),
                                text_color: palette.primary.strong.text,
                                border: Border {
                                    radius: Radius::from(8.0),
                                    width: 0.0,
                                    color: Color::TRANSPARENT,
                                },
                                shadow: Shadow {
                                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.1),
                                    offset: iced::Vector::new(0.0, 1.0),
                                    blur_radius: 3.0,
                                },
                            },
                            button::Status::Hovered => button::Style {
                                background: Some(Background::Color(palette.primary.base.color)),
                                text_color: palette.primary.base.text,
                                border: Border {
                                    radius: Radius::from(8.0),
                                    width: 0.0,
                                    color: Color::TRANSPARENT,
                                },
                                shadow: Shadow {
                                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.15),
                                    offset: iced::Vector::new(0.0, 2.0),
                                    blur_radius: 4.0,
                                },
                            },
                            _ => button::Style::default(),
                        }
                    } else if is_current {
                        match status {
                            button::Status::Active => button::Style {
                                background: Some(Background::Color(palette.secondary.weak.color)),
                                text_color: palette.secondary.strong.text,
                                border: Border {
                                    radius: Radius::from(8.0),
                                    width: 0.0,
                                    color: Color::TRANSPARENT,
                                },
                                shadow: Shadow {
                                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.05),
                                    offset: iced::Vector::new(0.0, 1.0),
                                    blur_radius: 2.0,
                                },
                            },
                            button::Status::Hovered => button::Style {
                                background: Some(Background::Color(palette.secondary.base.color)),
                                text_color: palette.secondary.base.text,
                                border: Border {
                                    radius: Radius::from(8.0),
                                    width: 0.0,
                                    color: Color::TRANSPARENT,
                                },
                                shadow: Shadow {
                                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.1),
                                    offset: iced::Vector::new(0.0, 2.0),
                                    blur_radius: 4.0,
                                },
                            },
                            _ => button::Style::default(),
                        }
                    } else {
                        match status {
                            button::Status::Hovered => button::Style {
                                background: Some(Background::Color(palette.background.strong.color)),
                                text_color: palette.background.strong.text,
                                border: Border {
                                    radius: Radius::from(8.0),
                                    width: 0.0,
                                    color: Color::TRANSPARENT,
                                },
                                shadow: Shadow {
                                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.05),
                                    offset: iced::Vector::new(0.0, 1.0),
                                    blur_radius: 2.0,
                                },
                            },
                            _ => button::Style {
                                background: Some(Background::Color(Color::TRANSPARENT)),
                                text_color: palette.background.base.text,
                                border: Border {
                                    radius: Radius::from(8.0),
                                    width: 0.0,
                                    color: Color::TRANSPARENT,
                                },
                                shadow: Shadow::default(),
                            },
                        }
                    }
                });
            
            btn.into()
        }).collect();
        
        container(
            column![
                // 播放列表标题
                row![
                    text("📋").size(18).shaping(Shaping::Advanced),
                    text(format!("播放列表 ({} 首歌曲)", playlist.len()))
                        .size(16)
                        .style(|theme: &Theme| {
                            let palette = theme.extended_palette();
                            text::Style {
                                color: Some(palette.primary.base.color),
                            }
                        }),
                ].spacing(8).align_y(Vertical::Center),
                
                // 播放列表项目
                scrollable(
                    column(playlist_items).spacing(4).padding([8, 0])
                ).height(Length::Fill).width(Length::Fill),
            ].spacing(16)
        )
        .style(card_style())
        .padding(16)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    } else {
        container(
            column![
                text("📂").size(48).align_x(Horizontal::Center).shaping(Shaping::Advanced),
                text("未加载播放列表")
                    .size(16)
                    .align_x(Horizontal::Center)
                    .style(|theme: &Theme| {
                        let palette = theme.extended_palette();
                        text::Style {
                            color: Some(Color {
                                a: 0.7,
                                ..palette.background.base.text
                            }),
                        }
                    }),
                text("点击「打开文件」开始")
                    .size(12)
                    .align_x(Horizontal::Center)
                    .style(|theme: &Theme| {
                        let palette = theme.extended_palette();
                        text::Style {
                            color: Some(Color {
                                a: 0.5,
                                ..palette.background.base.text
                            }),
                        }
                    }),
            ].spacing(12).align_x(Horizontal::Center)
        )
        .style(card_style())
        .padding(32)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
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
        return container(
            column![
                text("🎵").size(48).align_x(Horizontal::Center).shaping(Shaping::Advanced),
                text("歌词显示")
                    .size(20)
                    .align_x(Horizontal::Center)
                    .style(|theme: &Theme| {
                        let palette = theme.extended_palette();
                        text::Style {
                            color: Some(palette.primary.base.color),
                        }
                    }),
                text("请选择音频文件")
                    .size(14)
                    .align_x(Horizontal::Center)
                    .style(|theme: &Theme| {
                        let palette = theme.extended_palette();
                        text::Style {
                            color: Some(Color {
                                a: 0.7,
                                ..palette.background.base.text
                            }),
                        }
                    }),
            ].spacing(16).align_x(Horizontal::Center)
        )
        .style(card_style())
        .padding(32)
        .width(Length::Fill)
        .height(Length::Fill)
        .into();
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
        
        lyrics_elements.push(
            text(title)
                .size(20)
                .align_x(Horizontal::Center)
                .style(|theme: &Theme| {
                    let palette = theme.extended_palette();
                    text::Style {
                        color: Some(palette.primary.base.color),
                    }
                })
                .into()
        );
        
        if let Some(ref artist) = lyrics_data.metadata.artist {
            lyrics_elements.push(
                text(format!("🎤 {}", artist))
                    .size(14)
                    .align_x(Horizontal::Center)
                    .shaping(Shaping::Advanced)
                    .style(|theme: &Theme| {
                        let palette = theme.extended_palette();
                        text::Style {
                            color: Some(Color {
                                a: 0.8,
                                ..palette.background.base.text
                            }),
                        }
                    })
                    .into()
            );
        }
        
        lyrics_elements.push(text("").into()); // 空行
    } else {
        lyrics_elements.push(
            text("🎵 歌词显示")
                .size(18)
                .align_x(Horizontal::Center)
                .shaping(Shaping::Advanced)
                .style(|theme: &Theme| {
                    let palette = theme.extended_palette();
                    text::Style {
                        color: Some(palette.primary.base.color),
                    }
                })
                .into()
        );
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
                        .align_x(Horizontal::Center)
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
                    let text_element: Element<Message> = if is_current && is_playing {
                        // 当前播放行 - 高亮显示，居中对齐
                        container(
                            text(format!("▶ {}", lyric_text))
                                .size(18)
                                .align_x(Horizontal::Center)
                                .shaping(Shaping::Advanced)
                                .style(|theme: &Theme| {
                                    let palette = theme.extended_palette();
                                    text::Style {
                                        color: Some(palette.primary.strong.color),
                                    }
                                })
                        )
                        .style(|theme: &Theme| {
                            let palette = theme.extended_palette();
                            container::Style {
                                background: Some(Background::Color(Color {
                                    a: 0.1,
                                    ..palette.primary.base.color
                                })),
                                border: Border {
                                    radius: Radius::from(8.0),
                                    width: 0.0,
                                    color: Color::TRANSPARENT,
                                },
                                shadow: Shadow::default(),
                                text_color: None,
                            }
                        })
                        .padding([8, 16])
                        .width(Length::Fill)
                        .into()
                    } else if is_upcoming && is_playing {
                        // 下一行 - 稍微突出显示
                        text(lyric_text)
                            .size(16)
                            .align_x(Horizontal::Center)
                            .shaping(Shaping::Advanced)
                            .style(|theme: &Theme| {
                                let palette = theme.extended_palette();
                                text::Style {
                                    color: Some(palette.secondary.base.color),
                                }
                            })
                            .into()
                    } else if current_line_index.map_or(false, |current| lyrics_index <= current) {
                        // 已播放的行 - 淡化显示
                        text(lyric_text)
                            .size(14)
                            .align_x(Horizontal::Center)
                            .shaping(Shaping::Advanced)
                            .style(|theme: &Theme| {
                                let palette = theme.extended_palette();
                                text::Style {
                                    color: Some(Color {
                                        a: 0.4,
                                        ..palette.background.weak.text
                                    }),
                                }
                            })
                            .into()
                    } else {
                        // 未播放的行 - 正常显示但稍微淡一些
                        text(lyric_text)
                            .size(14)
                            .align_x(Horizontal::Center)
                            .shaping(Shaping::Advanced)
                            .style(|theme: &Theme| {
                                let palette = theme.extended_palette();
                                text::Style {
                                    color: Some(Color {
                                        a: 0.7,
                                        ..palette.background.weak.text
                                    }),
                                }
                            })
                            .into()
                    };
                    
                    lyrics_elements.push(text_element.into());
                }
            }
            
            // 添加后置空行
            for _ in 0..post_empty_lines {
                lyrics_elements.push(
                    text("")
                        .size(16)
                        .align_x(Horizontal::Center)
                        .into()
                );
            }
            
            // 如果没有当前行且正在播放，在底部显示提示
            if current_line_index.is_none() && is_playing {
                lyrics_elements.push(text("").into());
                lyrics_elements.push(
                    text("♪ 音乐开始了... ♪")
                        .size(14)
                        .align_x(Horizontal::Center)
                        .shaping(Shaping::Advanced)
                        .style(|theme: &Theme| {
                            let palette = theme.extended_palette();
                            text::Style {
                                color: Some(palette.primary.base.color),
                            }
                        })
                        .into()
                );
            }
            
        } else {
            // 歌词文件存在但没有歌词内容
            lyrics_elements.push(
                text("⚠️ 歌词文件已加载，但没有找到歌词内容")
                    .align_x(Horizontal::Center)
                    .shaping(Shaping::Advanced)
                    .style(|theme: &Theme| {
                        let palette = theme.extended_palette();
                        text::Style {
                            color: Some(Color {
                                a: 0.7,
                                ..palette.background.base.text
                            }),
                        }
                    })
                    .into()
            );
        }
    } else {
        // 没有歌词文件
        if is_playing {
            lyrics_elements.push(
                text("♪ 正在播放中... ♪")
                    .size(18)
                    .align_x(Horizontal::Center)
                    .shaping(Shaping::Advanced)
                    .style(|theme: &Theme| {
                        let palette = theme.extended_palette();
                        text::Style {
                            color: Some(palette.primary.base.color),
                        }
                    })
                    .into()
            );
            lyrics_elements.push(text("").into());
            lyrics_elements.push(
                text("🎵 暂无歌词文件")
                    .align_x(Horizontal::Center)
                    .shaping(Shaping::Advanced)
                    .style(|theme: &Theme| {
                        let palette = theme.extended_palette();
                        text::Style {
                            color: Some(Color {
                                a: 0.7,
                                ..palette.background.base.text
                            }),
                        }
                    })
                    .into()
            );
            lyrics_elements.push(text("").into());
            lyrics_elements.push(
                text(format!("⏱️ {}", format_duration(current_time)))
                    .size(12)
                    .align_x(Horizontal::Center)
                    .shaping(Shaping::Advanced)
                    .style(|theme: &Theme| {
                        let palette = theme.extended_palette();
                        text::Style {
                            color: Some(Color {
                                a: 0.6,
                                ..palette.background.base.text
                            }),
                        }
                    })
                    .into()
            );
        } else {
            lyrics_elements.push(
                text("♪ 歌词显示 ♪")
                    .size(18)
                    .align_x(Horizontal::Center)
                    .shaping(Shaping::Advanced)
                    .style(|theme: &Theme| {
                        let palette = theme.extended_palette();
                        text::Style {
                            color: Some(palette.primary.base.color),
                        }
                    })
                    .into()
            );
            lyrics_elements.push(text("").into());
            lyrics_elements.push(
                text("⏸️ 暂停播放中")
                    .align_x(Horizontal::Center)
                    .shaping(Shaping::Advanced)
                    .style(|theme: &Theme| {
                        let palette = theme.extended_palette();
                        text::Style {
                            color: Some(Color {
                                a: 0.7,
                                ..palette.background.base.text
                            }),
                        }
                    })
                    .into()
            );
        }
        
        lyrics_elements.push(text("").into());
        lyrics_elements.push(
            container(
                column![
                    text("💡 使用提示")
                        .size(14)
                        .shaping(Shaping::Advanced)
                        .style(|theme: &Theme| {
                            let palette = theme.extended_palette();
                            text::Style {
                                color: Some(palette.primary.base.color),
                            }
                        }),
                    text("📁 将 .lrc 歌词文件放在音频文件同目录下").size(11).shaping(Shaping::Advanced),
                    text("📝 歌词文件名需与音频文件名相同").size(11).shaping(Shaping::Advanced),
                    text("🕐 支持时间同步的LRC格式歌词").size(11).shaping(Shaping::Advanced),
                ].spacing(6)
            )
            .style(|theme: &Theme| {
                let palette = theme.extended_palette();
                container::Style {
                    background: Some(Background::Color(Color {
                        a: 0.05,
                        ..palette.primary.base.color
                    })),
                    border: Border {
                        radius: Radius::from(8.0),
                        width: 1.0,
                        color: Color {
                            a: 0.2,
                            ..palette.primary.base.color
                        },
                    },
                    shadow: Shadow::default(),
                    text_color: Some(Color {
                        a: 0.8,
                        ..palette.background.base.text
                    }),
                }
            })
            .padding(12)
            .into()
        );
    }
    
    // 创建高度自适应的歌词显示区域，不使用滚动条
    container(
        column(lyrics_elements)
            .spacing(16)  // 增加行间距使视觉更舒适
            .width(Length::Fill)
            .align_x(Horizontal::Center)
    )
    .style(card_style())
    .padding(24)  // 增加内边距
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

/// 创建应用程序标题
/// 
/// # 返回
/// 标题UI元素
pub fn title_view() -> Element<'static, Message> {
    container(
        row![
            text("🎵").size(24).shaping(Shaping::Advanced),
            text("音频播放器")
                .size(20)
                .style(|theme: &Theme| {
                    let palette = theme.extended_palette();
                    text::Style {
                        color: Some(palette.primary.base.color),
                    }
                })
        ].spacing(8).align_y(Vertical::Center)
    )
    .style(card_style())
    .padding(16)
    .width(Length::Fill)
    .into()
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
    let title_and_metadata_height = 120.0;  // 标题和艺术家信息
    let toggle_button_height = 50.0;        // 切换按钮
    let progress_bar_height = 80.0;         // 进度条区域
    let padding_and_spacing = 100.0;        // 内边距和间距
    
    let available_height = window_height 
        - title_and_metadata_height 
        - toggle_button_height 
        - progress_bar_height 
        - padding_and_spacing;
    
    // 2. 根据可用高度计算行数
    let line_height = 28.0; // 每行预估高度（字体 + 行间距）
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

 