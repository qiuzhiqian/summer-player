//! 样式化按钮组件
//! 
//! 提供多种预定义样式的按钮组件

use iced::{
    widget::button,
    Element,
    Length,
    Border,
    Shadow,
    Background,
    Color,
    border::Radius,
};

use crate::ui::Message;

/// 样式化按钮组件
pub struct StyledButton {
    content: Element<'static, Message>,
    style: ButtonStyle,
    width: Length,
    height: Length,
    on_press: Option<Message>,
    padding: u16,
}

/// 按钮样式类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ButtonStyle {
    /// 主按钮（播放按钮）
    Primary,
    /// 控制按钮（上一首、下一首等）
    Control,
    /// 文件按钮（打开文件等）
    File,
    /// 播放列表项按钮
    PlaylistItem { is_playing: bool, is_current: bool },
    /// 主题切换按钮
    ThemeToggle,
    /// 视图切换按钮
    ViewToggle,
}

impl StyledButton {
    /// 创建新的样式化按钮
    pub fn new(content: impl Into<Element<'static, Message>>) -> Self {
        Self {
            content: content.into(),
            style: ButtonStyle::Primary,
            width: Length::Shrink,
            height: Length::Shrink,
            on_press: None,
            padding: 0,
        }
    }

    /// 设置按钮样式
    pub fn style(mut self, style: ButtonStyle) -> Self {
        self.style = style;
        self
    }

    /// 设置按钮宽度
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// 设置按钮高度
    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    /// 设置按钮内边距
    pub fn padding(mut self, padding: u16) -> Self {
        self.padding = padding;
        self
    }

    /// 设置按钮点击事件
    pub fn on_press(mut self, msg: Message) -> Self {
        self.on_press = Some(msg);
        self
    }

    /// 构建按钮元素
    pub fn build(self) -> Element<'static, Message> {
        let style_fn = self.get_style_fn();
        let button_widget = button(self.content)
            .style(style_fn)
            .width(self.width)
            .height(self.height)
            .padding(self.padding);

        if let Some(msg) = self.on_press {
            button_widget.on_press(msg)
        } else {
            button_widget
        }.into()
    }

    /// 获取对应样式的函数
    fn get_style_fn(&self) -> Box<dyn Fn(&iced::Theme, iced::widget::button::Status) -> iced::widget::button::Style> {
        match self.style {
            ButtonStyle::Primary => Box::new(primary_button_style),
            ButtonStyle::Control => Box::new(control_button_style),
            ButtonStyle::File => Box::new(file_button_style),
            ButtonStyle::PlaylistItem { is_playing, is_current } => Box::new(move |theme, status| {
                playlist_item_button_style(theme, status, is_playing, is_current)
            }),
            ButtonStyle::ThemeToggle => Box::new(theme_toggle_button_style),
            ButtonStyle::ViewToggle => Box::new(view_toggle_button_style),
        }
    }
}

/// 主按钮样式（播放按钮）
fn primary_button_style(theme: &iced::Theme, status: iced::widget::button::Status) -> iced::widget::button::Style {
    let palette = theme.extended_palette();
    let primary = palette.primary.base.color;
    
    // 进一步淡化主色调，降低饱和度和亮度
    let muted_primary = Color {
        r: primary.r * 0.75,
        g: primary.g * 0.75,
        b: primary.b * 0.8,
        a: 0.8, // 进一步降低不透明度
    };
    
    match status {
        iced::widget::button::Status::Active => iced::widget::button::Style {
            background: Some(Background::Color(muted_primary)),
            text_color: Color::WHITE,
            border: Border {
                radius: Radius::from(20.0),
                width: 1.0,
                color: Color {
                    a: 0.3,
                    ..Color::WHITE
                },
            },
            shadow: Shadow {
                color: Color::from_rgba(muted_primary.r, muted_primary.g, muted_primary.b, 0.25),
                offset: iced::Vector::new(0.0, 4.0),
                blur_radius: 12.0,
            },
            snap: false,
        },
        iced::widget::button::Status::Hovered => iced::widget::button::Style {
            background: Some(Background::Color(Color {
                r: (muted_primary.r * 1.1).min(1.0),
                g: (muted_primary.g * 1.1).min(1.0),
                b: (muted_primary.b * 1.1).min(1.0),
                a: 0.9,
            })),
            text_color: Color::WHITE,
            border: Border {
                radius: Radius::from(20.0),
                width: 1.0,
                color: Color {
                    a: 0.5,
                    ..Color::WHITE
                },
            },
            shadow: Shadow {
                color: Color::from_rgba(muted_primary.r, muted_primary.g, muted_primary.b, 0.35),
                offset: iced::Vector::new(0.0, 6.0),
                blur_radius: 16.0,
            },
            snap: false,
        },
        iced::widget::button::Status::Pressed => iced::widget::button::Style {
            background: Some(Background::Color(Color {
                r: muted_primary.r * 0.8,
                g: muted_primary.g * 0.8,
                b: muted_primary.b * 0.85,
                a: 0.9,
            })),
            text_color: Color::WHITE,
            border: Border {
                radius: Radius::from(20.0),
                width: 1.0,
                color: Color {
                    a: 0.4,
                    ..Color::WHITE
                },
            },
            shadow: Shadow {
                color: Color::from_rgba(muted_primary.r, muted_primary.g, muted_primary.b, 0.15),
                offset: iced::Vector::new(0.0, 2.0),
                blur_radius: 6.0,
            },
            snap: false,
        },
        iced::widget::button::Status::Disabled => iced::widget::button::Style {
            background: Some(Background::Color(palette.background.weak.color)),
            text_color: palette.background.weak.text,
            border: Border {
                radius: Radius::from(20.0),
                width: 1.0,
                color: palette.background.strong.color,
            },
            shadow: Shadow::default(),
            snap: false,
        },
    }
}

/// 控制按钮样式（上一首、下一首等）
fn control_button_style(theme: &iced::Theme, status: iced::widget::button::Status) -> iced::widget::button::Style {
    let palette = theme.extended_palette();
    let secondary = palette.secondary.base.color;
    
    match status {
        iced::widget::button::Status::Active => iced::widget::button::Style {
            background: Some(Background::Color(Color {
                a: 0.85,
                ..secondary
            })),
            text_color: Color::WHITE,
            border: Border {
                radius: Radius::from(20.0),
                width: 1.0,
                color: Color {
                    a: 0.3,
                    ..Color::WHITE
                },
            },
            shadow: Shadow {
                color: Color::from_rgba(secondary.r, secondary.g, secondary.b, 0.3),
                offset: iced::Vector::new(0.0, 4.0),
                blur_radius: 12.0,
            },
            snap: false,
        },
        iced::widget::button::Status::Hovered => iced::widget::button::Style {
            background: Some(Background::Color(secondary)),
            text_color: Color::WHITE,
            border: Border {
                radius: Radius::from(20.0),
                width: 1.0,
                color: Color {
                    a: 0.5,
                    ..Color::WHITE
                },
            },
            shadow: Shadow {
                color: Color::from_rgba(secondary.r, secondary.g, secondary.b, 0.4),
                offset: iced::Vector::new(0.0, 6.0),
                blur_radius: 16.0,
            },
            snap: false,
        },
        iced::widget::button::Status::Pressed => iced::widget::button::Style {
            background: Some(Background::Color(Color {
                r: secondary.r * 0.85,
                g: secondary.g * 0.85,
                b: secondary.b * 0.85,
                a: 1.0,
            })),
            text_color: Color::WHITE,
            border: Border {
                radius: Radius::from(20.0),
                width: 1.0,
                color: Color {
                    a: 0.4,
                    ..Color::WHITE
                },
            },
            shadow: Shadow {
                color: Color::from_rgba(secondary.r, secondary.g, secondary.b, 0.2),
                offset: iced::Vector::new(0.0, 2.0),
                blur_radius: 6.0,
            },
            snap: false,
        },
        iced::widget::button::Status::Disabled => iced::widget::button::Style {
            background: Some(Background::Color(palette.background.weak.color)),
            text_color: palette.background.weak.text,
            border: Border {
                radius: Radius::from(20.0),
                width: 1.0,
                color: palette.background.strong.color,
            },
            shadow: Shadow::default(),
            snap: false,
        },
    }
}

/// 文件按钮样式（打开文件等）
fn file_button_style(theme: &iced::Theme, status: iced::widget::button::Status) -> iced::widget::button::Style {
    let palette = theme.extended_palette();
    let primary = palette.primary.base.color;
    
    match status {
        iced::widget::button::Status::Active => iced::widget::button::Style {
            background: Some(Background::Color(Color {
                a: 0.1,
                ..primary
            })),
            text_color: primary,
            border: Border {
                radius: Radius::from(16.0),
                width: 1.5,
                color: Color {
                    a: 0.35,
                    ..primary
                },
            },
            shadow: Shadow {
                color: Color::from_rgba(primary.r, primary.g, primary.b, 0.15),
                offset: iced::Vector::new(0.0, 4.0),
                blur_radius: 12.0,
            },
            snap: false,
        },
        iced::widget::button::Status::Hovered => iced::widget::button::Style {
            background: Some(Background::Color(Color {
                a: 0.15,
                ..primary
            })),
            text_color: primary,
            border: Border {
                radius: Radius::from(16.0),
                width: 1.5,
                color: Color {
                    a: 0.55,
                    ..primary
                },
            },
            shadow: Shadow {
                color: Color::from_rgba(primary.r, primary.g, primary.b, 0.25),
                offset: iced::Vector::new(0.0, 6.0),
                blur_radius: 16.0,
            },
            snap: false,
        },
        iced::widget::button::Status::Pressed => iced::widget::button::Style {
            background: Some(Background::Color(Color {
                a: 0.2,
                ..primary
            })),
            text_color: primary,
            border: Border {
                radius: Radius::from(16.0),
                width: 1.5,
                color: Color {
                    a: 0.7,
                    ..primary
                },
            },
            shadow: Shadow {
                color: Color::from_rgba(primary.r, primary.g, primary.b, 0.15),
                offset: iced::Vector::new(0.0, 2.0),
                blur_radius: 8.0,
            },
            snap: false,
        },
        iced::widget::button::Status::Disabled => iced::widget::button::Style {
            background: Some(Background::Color(palette.background.weak.color)),
            text_color: palette.background.weak.text,
            border: Border {
                radius: Radius::from(16.0),
                width: 1.0,
                color: palette.background.strong.color,
            },
            shadow: Shadow::default(),
            snap: false,
        },
    }
}

/// 播放列表项按钮样式
fn playlist_item_button_style(
    theme: &iced::Theme, 
    status: iced::widget::button::Status, 
    is_playing: bool, 
    is_current: bool
) -> iced::widget::button::Style {
    let palette = theme.extended_palette();
    let primary = palette.primary.base.color;
    let secondary = palette.secondary.base.color;
    
    if is_playing {
        // 正在播放的当前项目
        match status {
            iced::widget::button::Status::Active => iced::widget::button::Style {
                background: Some(Background::Color(Color {
                    a: 0.25,
                    ..primary
                })),
                text_color: primary,
                border: Border {
                    radius: Radius::from(8.0),
                    width: 1.0,
                    color: Color {
                        a: 0.4,
                        ..primary
                    },
                },
                shadow: Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.1),
                    offset: iced::Vector::new(0.0, 1.0),
                    blur_radius: 3.0,
                },
                snap: false,
            },
            iced::widget::button::Status::Hovered => iced::widget::button::Style {
                background: Some(Background::Color(Color {
                    a: 0.35,
                    ..primary
                })),
                text_color: primary,
                border: Border {
                    radius: Radius::from(8.0),
                    width: 1.0,
                    color: primary,
                },
                shadow: Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.15),
                    offset: iced::Vector::new(0.0, 2.0),
                    blur_radius: 4.0,
                },
                snap: false,
            },
            _ => iced::widget::button::Style::default(),
        }
    } else if is_current {
        // 当前选中的项目
        match status {
            iced::widget::button::Status::Active => iced::widget::button::Style {
                background: Some(Background::Color(Color {
                    a: 0.1,
                    ..secondary
                })),
                text_color: secondary,
                border: Border {
                    radius: Radius::from(8.0),
                    width: 1.0,
                    color: Color {
                        a: 0.2,
                        ..secondary
                    },
                },
                shadow: Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.1),
                    offset: iced::Vector::new(0.0, 1.0),
                    blur_radius: 2.0,
                },
                snap: false,
            },
            iced::widget::button::Status::Hovered => iced::widget::button::Style {
                background: Some(Background::Color(Color {
                    a: 0.15,
                    ..secondary
                })),
                text_color: secondary,
                border: Border {
                    radius: Radius::from(8.0),
                    width: 1.0,
                    color: secondary,
                },
                shadow: Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.15),
                    offset: iced::Vector::new(0.0, 2.0),
                    blur_radius: 4.0,
                },
                snap: false,
            },
            _ => iced::widget::button::Style::default(),
        }
    } else {
        // 普通项目
        match status {
            iced::widget::button::Status::Hovered => iced::widget::button::Style {
                background: Some(Background::Color(palette.background.weak.color)),
                text_color: palette.background.base.text,
                border: Border {
                    radius: Radius::from(8.0),
                    width: 1.0,
                    color: palette.background.strong.color,
                },
                shadow: Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.1),
                    offset: iced::Vector::new(0.0, 1.0),
                    blur_radius: 2.0,
                },
                snap: false,
            },
            _ => iced::widget::button::Style {
                background: Some(Background::Color(Color::TRANSPARENT)),
                text_color: palette.background.base.text,
                border: Border {
                    radius: Radius::from(8.0),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
                shadow: Shadow::default(),
                snap: false,
            },
        }
    }
}

/// 主题切换按钮样式
fn theme_toggle_button_style(theme: &iced::Theme, status: iced::widget::button::Status) -> iced::widget::button::Style {
    let palette = theme.extended_palette();
    let warning = palette.primary.base.color; // 使用primary替代warning
    
    match status {
        iced::widget::button::Status::Active => iced::widget::button::Style {
            background: Some(Background::Color(Color {
                a: 0.1,
                ..warning
            })),
            text_color: warning,
            border: Border {
                radius: Radius::from(20.0),
                width: 1.0,
                color: Color {
                    a: 0.3,
                    ..warning
                },
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.1),
                offset: iced::Vector::new(0.0, 2.0),
                blur_radius: 4.0,
            },
            snap: false,
        },
        iced::widget::button::Status::Hovered => iced::widget::button::Style {
            background: Some(Background::Color(Color {
                a: 0.15,
                ..warning
            })),
            text_color: warning,
            border: Border {
                radius: Radius::from(20.0),
                width: 1.0,
                color: warning,
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.15),
                offset: iced::Vector::new(0.0, 4.0),
                blur_radius: 8.0,
            },
            snap: false,
        },
        iced::widget::button::Status::Pressed => iced::widget::button::Style {
            background: Some(Background::Color(Color {
                a: 0.2,
                ..warning
            })),
            text_color: warning,
            border: Border {
                radius: Radius::from(20.0),
                width: 1.0,
                color: warning,
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.1),
                offset: iced::Vector::new(0.0, 1.0),
                blur_radius: 2.0,
            },
            snap: false,
        },
        iced::widget::button::Status::Disabled => iced::widget::button::Style {
            background: Some(Background::Color(palette.background.weak.color)),
            text_color: palette.background.weak.text,
            border: Border {
                radius: Radius::from(20.0),
                width: 1.0,
                color: palette.background.strong.color,
            },
            shadow: Shadow::default(),
            snap: false,
        },
    }
}

/// 视图切换按钮样式
fn view_toggle_button_style(theme: &iced::Theme, status: iced::widget::button::Status) -> iced::widget::button::Style {
    let palette = theme.extended_palette();
    let primary = palette.primary.base.color;
    
    match status {
        iced::widget::button::Status::Active => iced::widget::button::Style {
            background: Some(Background::Color(Color {
                a: 0.05,
                ..primary
            })),
            text_color: palette.background.base.text,
            border: Border {
                radius: Radius::from(12.0),
                width: 1.0,
                color: Color {
                    a: 0.2,
                    ..primary
                },
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.1),
                offset: iced::Vector::new(0.0, 1.0),
                blur_radius: 3.0,
            },
            snap: false,
        },
        iced::widget::button::Status::Hovered => iced::widget::button::Style {
            background: Some(Background::Color(Color {
                a: 0.1,
                ..primary
            })),
            text_color: palette.background.base.text,
            border: Border {
                radius: Radius::from(12.0),
                width: 1.0,
                color: Color {
                    a: 0.3,
                    ..primary
                },
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.15),
                offset: iced::Vector::new(0.0, 3.0),
                blur_radius: 6.0,
            },
            snap: false,
        },
        iced::widget::button::Status::Pressed => iced::widget::button::Style {
            background: Some(Background::Color(Color {
                a: 0.15,
                ..primary
            })),
            text_color: palette.background.base.text,
            border: Border {
                radius: Radius::from(12.0),
                width: 1.0,
                color: Color {
                    a: 0.4,
                    ..primary
                },
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.1),
                offset: iced::Vector::new(0.0, 1.0),
                blur_radius: 2.0,
            },
            snap: false,
        },
        iced::widget::button::Status::Disabled => iced::widget::button::Style {
            background: Some(Background::Color(palette.background.weak.color)),
            text_color: palette.background.weak.text,
            border: Border {
                radius: Radius::from(12.0),
                width: 1.0,
                color: palette.background.strong.color,
            },
            shadow: Shadow::default(),
            snap: false,
        },
    }
}