//! 主题样式模块
//! 
//! 包含应用程序所有UI组件的统一样式设置。

use iced::{
    widget::{button, text, slider, container},
    Border, Shadow, Background, Color,
    theme::Theme,
    border::Radius,
};

/// 应用程序主题样式集合
pub struct AppTheme;

impl AppTheme {
    /// 创建现代化卡片容器样式
    pub fn card_container() -> fn(&Theme) -> container::Style {
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

    /// 创建信息卡片容器样式
    pub fn info_card_container() -> fn(&Theme) -> container::Style {
        |theme: &Theme| {
            let palette = theme.extended_palette();
            container::Style {
                background: Some(Background::Color(Color {
                    a: 0.02,
                    ..palette.primary.base.color
                })),
                border: Border {
                    radius: Radius::from(8.0),
                    width: 1.0,
                    color: Color {
                        a: 0.1,
                        ..palette.primary.base.color
                    },
                },
                shadow: Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.03),
                    offset: iced::Vector::new(0.0, 1.0),
                    blur_radius: 2.0,
                },
                text_color: Some(palette.background.base.text),
            }
        }
    }

    /// 创建圆形播放按钮样式
    pub fn play_button() -> fn(&Theme, button::Status) -> button::Style {
        |theme: &Theme, status| {
            let palette = theme.extended_palette();
            match status {
                button::Status::Active => button::Style {
                    background: Some(Background::Color(palette.primary.strong.color)),
                    text_color: Color::WHITE,
                    border: Border {
                        radius: Radius::from(26.0),
                        width: 2.0,
                        color: Color {
                            a: 0.3,
                            ..Color::WHITE
                        },
                    },
                    shadow: Shadow {
                        color: Color::from_rgba(0.0, 0.0, 0.0, 0.25),
                        offset: iced::Vector::new(0.0, 3.0),
                        blur_radius: 8.0,
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
                        radius: Radius::from(26.0),
                        width: 2.0,
                        color: Color {
                            a: 0.5,
                            ..Color::WHITE
                        },
                    },
                    shadow: Shadow {
                        color: Color::from_rgba(0.0, 0.0, 0.0, 0.35),
                        offset: iced::Vector::new(0.0, 5.0),
                        blur_radius: 12.0,
                    },
                },
                button::Status::Pressed => button::Style {
                    background: Some(Background::Color(Color {
                        r: palette.primary.strong.color.r * 0.9,
                        g: palette.primary.strong.color.g * 0.9,
                        b: palette.primary.strong.color.b * 0.9,
                        a: 1.0,
                    })),
                    text_color: Color::WHITE,
                    border: Border {
                        radius: Radius::from(26.0),
                        width: 2.0,
                        color: Color {
                            a: 0.7,
                            ..Color::WHITE
                        },
                    },
                    shadow: Shadow {
                        color: Color::from_rgba(0.0, 0.0, 0.0, 0.2),
                        offset: iced::Vector::new(0.0, 2.0),
                        blur_radius: 4.0,
                    },
                },
                button::Status::Disabled => button::Style {
                    background: Some(Background::Color(palette.background.weak.color)),
                    text_color: palette.background.weak.text,
                    border: Border {
                        radius: Radius::from(26.0),
                        width: 1.0,
                        color: palette.background.strong.color,
                    },
                    shadow: Shadow::default(),
                },
            }
        }
    }

    /// 创建控制按钮样式（停止、上一首、下一首等）
    pub fn control_button() -> fn(&Theme, button::Status) -> button::Style {
        |theme: &Theme, status| {
            let palette = theme.extended_palette();
            match status {
                button::Status::Active => button::Style {
                    background: Some(Background::Color(Color {
                        a: 0.8,
                        ..palette.secondary.base.color
                    })),
                    text_color: Color::WHITE,
                    border: Border {
                        radius: Radius::from(20.0),
                        width: 0.0,
                        color: Color::TRANSPARENT,
                    },
                    shadow: Shadow {
                        color: Color::from_rgba(0.0, 0.0, 0.0, 0.2),
                        offset: iced::Vector::new(0.0, 2.0),
                        blur_radius: 4.0,
                    },
                },
                button::Status::Hovered => button::Style {
                    background: Some(Background::Color(palette.secondary.strong.color)),
                    text_color: Color::WHITE,
                    border: Border {
                        radius: Radius::from(20.0),
                        width: 0.0,
                        color: Color::TRANSPARENT,
                    },
                    shadow: Shadow {
                        color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
                        offset: iced::Vector::new(0.0, 4.0),
                        blur_radius: 8.0,
                    },
                },
                button::Status::Pressed => button::Style {
                    background: Some(Background::Color(Color {
                        r: palette.secondary.strong.color.r * 0.9,
                        g: palette.secondary.strong.color.g * 0.9,
                        b: palette.secondary.strong.color.b * 0.9,
                        a: 1.0,
                    })),
                    text_color: Color::WHITE,
                    border: Border {
                        radius: Radius::from(20.0),
                        width: 0.0,
                        color: Color::TRANSPARENT,
                    },
                    shadow: Shadow {
                        color: Color::from_rgba(0.0, 0.0, 0.0, 0.15),
                        offset: iced::Vector::new(0.0, 1.0),
                        blur_radius: 2.0,
                    },
                },
                button::Status::Disabled => button::Style {
                    background: Some(Background::Color(palette.background.weak.color)),
                    text_color: palette.background.weak.text,
                    border: Border {
                        radius: Radius::from(20.0),
                        width: 1.0,
                        color: palette.background.strong.color,
                    },
                    shadow: Shadow::default(),
                },
            }
        }
    }

    /// 创建文件打开按钮样式
    pub fn file_button() -> fn(&Theme, button::Status) -> button::Style {
        |theme: &Theme, status| {
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
        }
    }

    /// 创建视图切换按钮样式
    pub fn view_toggle_button() -> fn(&Theme, button::Status) -> button::Style {
        |theme: &Theme, status| {
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
        }
    }

    /// 创建播放列表项按钮样式
    pub fn playlist_item_button(is_playing_current: bool, is_current: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
        move |theme: &Theme, status| {
            let palette = theme.extended_palette();
            
            if is_playing_current {
                // 正在播放的当前项目
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
                // 当前选中的项目
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
                // 普通项目
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
        }
    }

    /// 创建进度滑块样式
    pub fn progress_slider() -> fn(&Theme, slider::Status) -> slider::Style {
        |theme: &Theme, _status| {
            let palette = theme.extended_palette();
            slider::Style {
                rail: slider::Rail {
                    backgrounds: (
                        Background::Color(palette.primary.strong.color),
                        Background::Color(Color {
                            a: 0.3,
                            ..palette.background.strong.color
                        }),
                    ),
                    width: 6.0,
                    border: Border {
                        radius: Radius::from(3.0),
                        width: 0.0,
                        color: Color::TRANSPARENT,
                    },
                },
                handle: slider::Handle {
                    shape: slider::HandleShape::Circle { radius: 8.0 },
                    background: Background::Color(palette.primary.strong.color),
                    border_width: 2.0,
                    border_color: Color::WHITE,
                },
            }
        }
    }

    /// 创建主标题文本样式
    pub fn title_text() -> fn(&Theme) -> text::Style {
        |theme: &Theme| {
            let palette = theme.extended_palette();
            text::Style {
                color: Some(palette.primary.strong.color),
            }
        }
    }

    /// 创建副标题文本样式
    pub fn subtitle_text() -> fn(&Theme) -> text::Style {
        |theme: &Theme| {
            let palette = theme.extended_palette();
            text::Style {
                color: Some(Color {
                    a: 0.8,
                    ..palette.background.base.text
                }),
            }
        }
    }

    /// 创建信息标签文本样式
    pub fn info_label_text() -> fn(&Theme) -> text::Style {
        |theme: &Theme| {
            let palette = theme.extended_palette();
            text::Style {
                color: Some(Color {
                    a: 0.7,
                    ..palette.background.base.text
                }),
            }
        }
    }

    /// 创建信息值文本样式
    pub fn info_value_text() -> fn(&Theme) -> text::Style {
        |theme: &Theme| {
            let palette = theme.extended_palette();
            text::Style {
                color: Some(palette.background.base.text),
            }
        }
    }

    /// 创建时间显示文本样式（当前时间）
    pub fn current_time_text() -> fn(&Theme) -> text::Style {
        |theme: &Theme| {
            let palette = theme.extended_palette();
            text::Style {
                color: Some(palette.primary.base.color),
            }
        }
    }

    /// 创建时间显示文本样式（总时间）
    pub fn total_time_text() -> fn(&Theme) -> text::Style {
        |theme: &Theme| {
            let palette = theme.extended_palette();
            text::Style {
                color: Some(Color {
                    a: 0.7,
                    ..palette.background.base.text
                }),
            }
        }
    }

    /// 创建歌词文本样式（当前行）
    pub fn current_lyrics_text() -> fn(&Theme) -> text::Style {
        |theme: &Theme| {
            let palette = theme.extended_palette();
            text::Style {
                color: Some(palette.primary.strong.color),
            }
        }
    }

    /// 创建歌词文本样式（其他行）
    pub fn lyrics_text() -> fn(&Theme) -> text::Style {
        |theme: &Theme| {
            let palette = theme.extended_palette();
            text::Style {
                color: Some(Color {
                    a: 0.6,
                    ..palette.background.base.text
                }),
            }
        }
    }

    /// 创建提示文本样式
    pub fn hint_text() -> fn(&Theme) -> text::Style {
        |theme: &Theme| {
            let palette = theme.extended_palette();
            text::Style {
                color: Some(Color {
                    a: 0.5,
                    ..palette.background.base.text
                }),
            }
        }
    }

    /// 创建强调文本样式
    pub fn emphasis_text() -> fn(&Theme) -> text::Style {
        |theme: &Theme| {
            let palette = theme.extended_palette();
            text::Style {
                color: Some(palette.primary.base.color),
            }
        }
    }

    /// 创建背景容器样式
    pub fn background_container() -> fn(&Theme) -> container::Style {
        |theme: &Theme| {
            let palette = theme.extended_palette();
            container::Style {
                background: Some(Background::Color(Color {
                    a: 0.5,
                    ..palette.background.weak.color
                })),
                border: Border {
                    radius: Radius::from(8.0),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
                shadow: Shadow::default(),
                text_color: Some(palette.background.base.text),
            }
        }
    }

    /// 创建透明容器样式
    pub fn transparent_container() -> fn(&Theme) -> container::Style {
        |_theme: &Theme| {
            container::Style {
                background: Some(Background::Color(Color::TRANSPARENT)),
                border: Border::default(),
                shadow: Shadow::default(),
                text_color: None,
            }
        }
    }
} 