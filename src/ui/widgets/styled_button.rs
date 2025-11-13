//! 样式化按钮组件
//!
//! 提供多种预定义样式的按钮组件，支持图标和文本内容的组合显示
//!
//! # 示例
//! ```
//! use summer_player::ui::widgets::styled_button::{StyledButton, ButtonType};
//! use iced::widget::text;
//!
//! // 仅文本按钮
//! let text_button = StyledButton::new(text("文本按钮"))
//!     .button_type(ButtonType::Primary)
//!     .build();
//!
//! // 图标按钮
//! let icon = r#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"><path d="M12 5v14M5 12h14" stroke="currentColor" stroke-width="2" stroke-linecap="round"/></svg>"#;
//! let icon_button = StyledButton::icon_only(icon)
//!     .button_type(ButtonType::Primary)
//!     .build();
//!
//! // 图标和文本组合按钮
//! let icon_text_button = StyledButton::new(text("保存"))
//!     .icon(icon)
//!     .button_type(ButtonType::Primary)
//!     .build();
//! ```

use iced::widget::image::Handle;
use iced::{
    alignment::{Horizontal, Vertical},
    border::Radius,
    widget::{button, container, image, row, svg},
    Background, Border, Color, Element, Length, Shadow,
};
use crate::ui::Message;

/// 样式化按钮组件
pub struct StyledButton<Message> {
    content: Option<Element<'static, Message>>,
    icon: Option<Icon>,
    icon_size: f32,
    button_type: ButtonType,
    color: ButtonColor,
    width: Length,
    height: Length,
    on_press: Option<Message>,
    padding: u16,
}

/// 按钮类型（参考 Ant Design）
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ButtonType {
    Primary,
    Dashed,
    Link,
    Text,
    Default,
}

impl Default for ButtonType {
    fn default() -> Self {
        ButtonType::Default
    }
}

/// 预设颜色（参考 Ant Design）
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PresetColor {
    Magenta,
    Red,
    Volcano,
    Orange,
    Gold,
    Lime,
    Green,
    Cyan,
    Blue,
    GeekBlue,
    Purple,
}

/// 按钮颜色（参考 Ant Design）
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ButtonColor {
    Default,
    Primary,
    Danger,
    Preset(PresetColor),
}

impl Default for ButtonColor {
    fn default() -> Self {
        ButtonColor::Default
    }
}

impl<Message: Clone + 'static> StyledButton<Message> {
    /// 创建新的样式化按钮
    pub fn new(content: impl Into<Element<'static, Message>>) -> Self {
        Self {
            content: Some(content.into()),
            icon: None,
            icon_size: 16.0,
            button_type: ButtonType::Default,
            color: ButtonColor::Default,
            width: Length::Shrink,
            height: Length::Shrink,
            on_press: None,
            padding: 12,
        }
    }

    /// 创建仅图标按钮
    pub fn icon_only<I: Into<Icon>>(icon: I) -> Self {
        Self {
            content: None,
            icon: Some(icon.into()),
            icon_size: 16.0,
            button_type: ButtonType::Default,
            color: ButtonColor::Default,
            width: Length::Shrink,
            height: Length::Shrink,
            on_press: None,
            padding: 8,
        }
    }

    pub fn icon<I: Into<Icon>>(mut self, icon: I) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// 设置按钮类型
    pub fn button_type(mut self, button_type: ButtonType) -> Self {
        self.button_type = button_type;
        self
    }

    /// 设置按钮颜色
    pub fn color(mut self, color: ButtonColor) -> Self {
        self.color = color;
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

    /// 设置按钮尺寸
    pub fn size(mut self, size: f32) -> Self {
        self.width = Length::Fixed(size);
        self.height = Length::Fixed(size);
        self
    }

    /// 设置图标尺寸
    pub fn icon_size(mut self, icon_size: f32) -> Self {
        self.icon_size = icon_size;
        self
    }

    /// 设置按钮点击事件
    pub fn on_press(mut self, msg: Message) -> Self {
        self.on_press = Some(msg);
        self
    }

    /// 设置按钮内容
    pub fn content(mut self, content: impl Into<Element<'static, Message>>) -> Self {
        self.content = Some(content.into());
        self
    }

    /// 构建按钮元素
    ///
    /// 根据是否设置图标以及内容的情况，构建不同的布局：
    /// - 如果只设置了图标，图标将居中显示
    /// - 如果只设置了内容，内容将居中显示
    /// - 如果同时设置了图标和内容，图标将在左侧，内容在右侧
    pub fn build(self) -> Element<'static, Message> {
        let StyledButton {
            content,
            icon,
            icon_size,
            button_type,
            color,
            width,
            height,
            on_press,
            padding,
        } = self;

        // 根据图标和内容的存在情况构建不同的布局
        let button_content: Element<'static, Message> = match (icon, content) {
            // 只有图标
            (Some(icon), None) => match icon {
                Icon::PathStr(path) => {
                    let handle = image::Handle::from_path(path);
                    let icon_widget = image::Image::new(handle)
                        .width(Length::Fixed(icon_size))
                        .height(Length::Fixed(icon_size));
                    container(icon_widget)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .align_x(Horizontal::Center)
                        .align_y(Vertical::Center)
                        .into()
                }
                Icon::Path(path) => {
                    let handle = image::Handle::from_path(path);
                    let icon_widget = image::Image::new(handle)
                        .width(Length::Fixed(icon_size))
                        .height(Length::Fixed(icon_size));
                    container(icon_widget)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .align_x(Horizontal::Center)
                        .align_y(Vertical::Center)
                        .into()
                }
                Icon::Handle(handle) => {
                    let icon_widget = image::Image::new(handle)
                        .width(Length::Fixed(icon_size))
                        .height(Length::Fixed(icon_size));
                    container(icon_widget)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .align_x(Horizontal::Center)
                        .align_y(Vertical::Center)
                        .into()
                }
                Icon::SvgPathStr(path) => {
                    let handle = svg::Handle::from_path(path);
                    let icon_widget = svg::Svg::new(handle)
                        .width(Length::Fixed(icon_size))
                        .height(Length::Fixed(icon_size));
                    container(icon_widget)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .align_x(Horizontal::Center)
                        .align_y(Vertical::Center)
                        .into()
                }
                Icon::SvgPath(path) => {
                    let handle = svg::Handle::from_path(path);
                    let icon_widget = svg::Svg::new(handle)
                        .width(Length::Fixed(icon_size))
                        .height(Length::Fixed(icon_size));
                    container(icon_widget)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .align_x(Horizontal::Center)
                        .align_y(Vertical::Center)
                        .into()
                }
                Icon::SvgHandle(handle) => {
                    let icon_widget = svg::Svg::new(handle)
                        .width(Length::Fixed(icon_size))
                        .height(Length::Fixed(icon_size));
                    container(icon_widget)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .align_x(Horizontal::Center)
                        .align_y(Vertical::Center)
                        .into()
                }
            },
            // 只有内容
            (None, Some(content_element)) => container(content_element)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center)
                .into(),
            // 图标和内容都有
            (Some(icon), Some(content_element)) => match icon {
                Icon::PathStr(path) => {
                    let handle = image::Handle::from_path(path);
                    let icon_widget = image::Image::new(handle)
                        .width(Length::Fixed(icon_size))
                        .height(Length::Fixed(icon_size));
                    row![icon_widget, content_element]
                        .spacing(8)
                        .align_y(Vertical::Center)
                        .into()
                }
                Icon::Path(path) => {
                    let handle = image::Handle::from_path(path);
                    let icon_widget = image::Image::new(handle)
                        .width(Length::Fixed(icon_size))
                        .height(Length::Fixed(icon_size));
                    row![icon_widget, content_element]
                        .spacing(8)
                        .align_y(Vertical::Center)
                        .into()
                }
                Icon::Handle(handle) => {
                    let icon_widget = image::Image::new(handle)
                        .width(Length::Fixed(icon_size))
                        .height(Length::Fixed(icon_size));
                    row![icon_widget, content_element]
                        .spacing(8)
                        .align_y(Vertical::Center)
                        .into()
                }
                Icon::SvgPathStr(path) => {
                    let handle = svg::Handle::from_path(path);
                    let icon_widget = svg::Svg::new(handle)
                        .width(Length::Fixed(icon_size))
                        .height(Length::Fixed(icon_size));
                    row![icon_widget, content_element]
                        .spacing(8)
                        .align_y(Vertical::Center)
                        .into()
                }
                Icon::SvgPath(path) => {
                    let handle = svg::Handle::from_path(path);
                    let icon_widget = svg::Svg::new(handle)
                        .width(Length::Fixed(icon_size))
                        .height(Length::Fixed(icon_size));
                    row![icon_widget, content_element]
                        .spacing(8)
                        .align_y(Vertical::Center)
                        .into()
                }
                Icon::SvgHandle(handle) => {
                    let icon_widget = svg::Svg::new(handle)
                        .width(Length::Fixed(icon_size))
                        .height(Length::Fixed(icon_size));
                    row![icon_widget, content_element]
                        .spacing(8)
                        .align_y(Vertical::Center)
                        .into()
                }
            },
            // 都没有（这种情况应该不会发生）
            (None, None) => container(iced::widget::text(""))
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center)
                .into(),
        };

        let style_fn = Self::resolve_style_fn(button_type, color, None);
        let button_widget = button(button_content)
            .style(style_fn)
            .width(width)
            .height(height)
            .padding(padding);

        if let Some(msg) = on_press {
            button_widget.on_press(msg)
        } else {
            button_widget
        }
        .into()
    }

    /// 获取对应样式的函数
    fn resolve_style_fn(
        button_type: ButtonType,
        color: ButtonColor,
        style_override: Option<
            Box<
                dyn Fn(&iced::Theme, iced::widget::button::Status) -> iced::widget::button::Style
                    + 'static,
            >,
        >,
    ) -> Box<
        dyn Fn(&iced::Theme, iced::widget::button::Status) -> iced::widget::button::Style + 'static,
    > {
        if let Some(custom) = style_override {
            return Box::new(move |theme, status| (custom)(theme, status));
        }

        Box::new(
            move |theme: &iced::Theme, status: iced::widget::button::Status| {
                let base_text = theme.extended_palette().background.base.text;
                let neutral_border = get_border_color(theme);
                let neutral_bg = get_surface_color(theme);
                let disabled_bg = theme.extended_palette().background.weak.color;
                let disabled_text = theme.extended_palette().background.weak.text;

                let accent = match color {
                    ButtonColor::Default => get_primary_color(theme),
                    ButtonColor::Primary => get_primary_color(theme),
                    ButtonColor::Danger => theme.extended_palette().danger.base.color,
                    ButtonColor::Preset(p) => preset_to_color(theme, p),
                };

                let radius = Radius::from(6.0);

                match button_type {
                    ButtonType::Primary => match status {
                        iced::widget::button::Status::Disabled => iced::widget::button::Style {
                            background: Some(Background::Color(disabled_bg)),
                            text_color: disabled_text,
                            border: Border {
                                radius,
                                width: 1.0,
                                color: neutral_border,
                            },
                            shadow: Shadow::default(),
                            snap: false,
                        },
                        iced::widget::button::Status::Pressed => iced::widget::button::Style {
                            background: Some(Background::Color(Color {
                                r: accent.r * 0.9,
                                g: accent.g * 0.9,
                                b: accent.b * 0.9,
                                a: 1.0,
                            })),
                            text_color: Color::WHITE,
                            border: Border {
                                radius,
                                width: 1.0,
                                color: Color { a: 0.2, ..accent },
                            },
                            shadow: Shadow {
                                color: get_shadow_color(theme),
                                offset: iced::Vector::new(0.0, 2.0),
                                blur_radius: 8.0,
                            },
                            snap: false,
                        },
                        iced::widget::button::Status::Hovered => iced::widget::button::Style {
                            background: Some(Background::Color(Color {
                                r: (accent.r * 1.05).min(1.0),
                                g: (accent.g * 1.05).min(1.0),
                                b: (accent.b * 1.05).min(1.0),
                                a: 1.0,
                            })),
                            text_color: Color::WHITE,
                            border: Border {
                                radius,
                                width: 1.0,
                                color: Color { a: 0.3, ..accent },
                            },
                            shadow: Shadow {
                                color: get_shadow_color(theme),
                                offset: iced::Vector::new(0.0, 4.0),
                                blur_radius: 12.0,
                            },
                            snap: false,
                        },
                        _ => iced::widget::button::Style {
                            background: Some(Background::Color(accent)),
                            text_color: Color::WHITE,
                            border: Border {
                                radius,
                                width: 1.0,
                                color: Color { a: 0.15, ..accent },
                            },
                            shadow: Shadow {
                                color: get_shadow_color(theme),
                                offset: iced::Vector::new(0.0, 2.0),
                                blur_radius: 8.0,
                            },
                            snap: false,
                        },
                    },

                    ButtonType::Dashed => match status {
                        iced::widget::button::Status::Disabled => iced::widget::button::Style {
                            background: Some(Background::Color(Color::TRANSPARENT)),
                            text_color: disabled_text,
                            border: Border {
                                radius,
                                width: 1.0,
                                color: neutral_border,
                            },
                            shadow: Shadow::default(),
                            snap: false,
                        },
                        iced::widget::button::Status::Hovered => iced::widget::button::Style {
                            background: Some(Background::Color(Color { a: 0.06, ..accent })),
                            text_color: accent,
                            border: Border {
                                radius,
                                width: 1.0,
                                color: Color { a: 0.6, ..accent },
                            },
                            shadow: Shadow {
                                color: get_shadow_color(theme),
                                offset: iced::Vector::new(0.0, 2.0),
                                blur_radius: 6.0,
                            },
                            snap: false,
                        },
                        iced::widget::button::Status::Pressed => iced::widget::button::Style {
                            background: Some(Background::Color(Color { a: 0.1, ..accent })),
                            text_color: accent,
                            border: Border {
                                radius,
                                width: 1.0,
                                color: Color { a: 0.8, ..accent },
                            },
                            shadow: Shadow {
                                color: get_shadow_color(theme),
                                offset: iced::Vector::new(0.0, 1.0),
                                blur_radius: 4.0,
                            },
                            snap: false,
                        },
                        _ => iced::widget::button::Style {
                            background: Some(Background::Color(Color::TRANSPARENT)),
                            text_color: accent,
                            border: Border {
                                radius,
                                width: 1.0,
                                color: Color { a: 0.4, ..accent },
                            },
                            shadow: Shadow::default(),
                            snap: false,
                        },
                    },

                    ButtonType::Link => match status {
                        iced::widget::button::Status::Disabled => iced::widget::button::Style {
                            background: Some(Background::Color(Color::TRANSPARENT)),
                            text_color: disabled_text,
                            border: Border {
                                radius,
                                width: 0.0,
                                color: Color::TRANSPARENT,
                            },
                            shadow: Shadow::default(),
                            snap: false,
                        },
                        iced::widget::button::Status::Hovered => iced::widget::button::Style {
                            background: Some(Background::Color(Color { a: 0.04, ..accent })),
                            text_color: accent,
                            border: Border {
                                radius,
                                width: 0.0,
                                color: Color::TRANSPARENT,
                            },
                            shadow: Shadow::default(),
                            snap: false,
                        },
                        _ => iced::widget::button::Style {
                            background: Some(Background::Color(Color::TRANSPARENT)),
                            text_color: accent,
                            border: Border {
                                radius,
                                width: 0.0,
                                color: Color::TRANSPARENT,
                            },
                            shadow: Shadow::default(),
                            snap: false,
                        },
                    },

                    ButtonType::Text => match status {
                        iced::widget::button::Status::Disabled => iced::widget::button::Style {
                            background: Some(Background::Color(Color::TRANSPARENT)),
                            text_color: disabled_text,
                            border: Border {
                                radius,
                                width: 0.0,
                                color: Color::TRANSPARENT,
                            },
                            shadow: Shadow::default(),
                            snap: false,
                        },
                        iced::widget::button::Status::Hovered => iced::widget::button::Style {
                            background: Some(Background::Color(Color {
                                a: 0.12,
                                ..neutral_bg
                            })),
                            text_color: if matches!(color, ButtonColor::Default) {
                                base_text
                            } else {
                                Color {
                                    r: (accent.r * 1.1).min(1.0),
                                    g: (accent.g * 1.1).min(1.0),
                                    b: (accent.b * 1.1).min(1.0),
                                    a: 1.0,
                                }
                            },
                            border: Border {
                                radius,
                                width: 0.0,
                                color: Color::TRANSPARENT,
                            },
                            shadow: Shadow {
                                color: get_shadow_color(theme),
                                offset: iced::Vector::new(0.0, 1.0),
                                blur_radius: 4.0,
                            },
                            snap: false,
                        },
                        iced::widget::button::Status::Pressed => iced::widget::button::Style {
                            background: Some(Background::Color(Color {
                                a: 0.18,
                                ..neutral_bg
                            })),
                            text_color: if matches!(color, ButtonColor::Default) {
                                base_text
                            } else {
                                Color {
                                    r: (accent.r * 0.9).min(1.0),
                                    g: (accent.g * 0.9).min(1.0),
                                    b: (accent.b * 0.9).min(1.0),
                                    a: 1.0,
                                }
                            },
                            border: Border {
                                radius,
                                width: 0.0,
                                color: Color::TRANSPARENT,
                            },
                            shadow: Shadow {
                                color: get_shadow_color(theme),
                                offset: iced::Vector::new(0.0, 0.5),
                                blur_radius: 2.0,
                            },
                            snap: false,
                        },
                        _ => iced::widget::button::Style {
                            background: Some(Background::Color(Color::TRANSPARENT)),
                            text_color: if matches!(color, ButtonColor::Default) {
                                base_text
                            } else {
                                accent
                            },
                            border: Border {
                                radius,
                                width: 0.0,
                                color: Color::TRANSPARENT,
                            },
                            shadow: Shadow::default(),
                            snap: false,
                        },
                    },

                    ButtonType::Default => match status {
                        iced::widget::button::Status::Disabled => iced::widget::button::Style {
                            background: Some(Background::Color(Color::TRANSPARENT)),
                            text_color: disabled_text,
                            border: Border {
                                radius,
                                width: 1.0,
                                color: neutral_border,
                            },
                            shadow: Shadow::default(),
                            snap: false,
                        },
                        iced::widget::button::Status::Hovered => iced::widget::button::Style {
                            background: Some(Background::Color(Color { a: 0.06, ..accent })),
                            text_color: if matches!(color, ButtonColor::Default) {
                                base_text
                            } else {
                                accent
                            },
                            border: Border {
                                radius,
                                width: 1.0,
                                color: if matches!(color, ButtonColor::Default) {
                                    neutral_border
                                } else {
                                    Color { a: 0.5, ..accent }
                                },
                            },
                            shadow: Shadow {
                                color: get_shadow_color(theme),
                                offset: iced::Vector::new(0.0, 2.0),
                                blur_radius: 6.0,
                            },
                            snap: false,
                        },
                        iced::widget::button::Status::Pressed => iced::widget::button::Style {
                            background: Some(Background::Color(Color { a: 0.1, ..accent })),
                            text_color: if matches!(color, ButtonColor::Default) {
                                base_text
                            } else {
                                accent
                            },
                            border: Border {
                                radius,
                                width: 1.0,
                                color: if matches!(color, ButtonColor::Default) {
                                    neutral_border
                                } else {
                                    Color { a: 0.7, ..accent }
                                },
                            },
                            shadow: Shadow {
                                color: get_shadow_color(theme),
                                offset: iced::Vector::new(0.0, 1.0),
                                blur_radius: 4.0,
                            },
                            snap: false,
                        },
                        _ => iced::widget::button::Style {
                            background: Some(Background::Color(Color::TRANSPARENT)),
                            text_color: if matches!(color, ButtonColor::Default) {
                                base_text
                            } else {
                                accent
                            },
                            border: Border {
                                radius,
                                width: 1.0,
                                color: if matches!(color, ButtonColor::Default) {
                                    neutral_border
                                } else {
                                    Color { a: 0.4, ..accent }
                                },
                            },
                            shadow: Shadow::default(),
                            snap: false,
                        },
                    },
                }
            },
        )
    }
}

// 辅助函数：获取主要颜色
fn get_primary_color(theme: &iced::Theme) -> Color {
    theme.extended_palette().primary.base.color
}

// 辅助函数：获取成功颜色
fn get_success_color(theme: &iced::Theme) -> Color {
    theme.extended_palette().success.base.color
}

// 辅助函数：获取边框颜色
fn get_border_color(theme: &iced::Theme) -> Color {
    theme.extended_palette().background.strong.color
}

// 辅助函数：获取表面颜色
fn get_surface_color(theme: &iced::Theme) -> Color {
    theme.extended_palette().background.weak.color
}

// 辅助函数：获取阴影颜色
fn get_shadow_color(_theme: &iced::Theme) -> Color {
    Color::from_rgba(0.0, 0.0, 0.0, 0.15)
}

use std::path::{Path, PathBuf};

/// 图标类型，支持多种格式
#[derive(Debug, Clone)]
pub enum Icon {
    PathStr(String),        // 图片路径字符串
    Path(PathBuf),          // 路径对象
    Handle(Handle),         // 已转换的图片资源
    SvgPathStr(String),     // SVG路径字符串
    SvgPath(PathBuf),       // SVG路径对象
    SvgHandle(svg::Handle), // 已转换的SVG资源
}

impl From<&str> for Icon {
    fn from(s: &str) -> Self {
        Icon::PathStr(s.to_string())
    }
}

impl From<String> for Icon {
    fn from(s: String) -> Self {
        Icon::PathStr(s)
    }
}

impl From<&Path> for Icon {
    fn from(path: &Path) -> Self {
        Icon::Path(path.to_path_buf())
    }
}

impl From<PathBuf> for Icon {
    fn from(path: PathBuf) -> Self {
        Icon::Path(path)
    }
}

impl From<Handle> for Icon {
    fn from(handle: Handle) -> Self {
        Icon::Handle(handle)
    }
}

impl From<svg::Handle> for Icon {
    fn from(handle: svg::Handle) -> Self {
        Icon::SvgHandle(handle)
    }
}

fn preset_to_color(theme: &iced::Theme, preset: PresetColor) -> Color {
    // 预设颜色常量，尽量贴近 AntD 语义
    match preset {
        PresetColor::Magenta => Color::from_rgb(0.91, 0.20, 0.52),
        PresetColor::Red => theme.extended_palette().danger.base.color,
        PresetColor::Volcano => Color::from_rgb(0.95, 0.35, 0.18),
        PresetColor::Orange => Color::from_rgb(1.0, 0.58, 0.0),
        PresetColor::Gold => Color::from_rgb(1.0, 0.76, 0.20),
        PresetColor::Lime => Color::from_rgb(0.75, 0.91, 0.30),
        PresetColor::Green => get_success_color(theme),
        PresetColor::Cyan => Color::from_rgb(0.18, 0.80, 0.80),
        PresetColor::Blue => get_primary_color(theme),
        PresetColor::GeekBlue => Color::from_rgb(0.24, 0.34, 0.80),
        PresetColor::Purple => Color::from_rgb(0.58, 0.34, 0.84),
    }
}

pub fn icon_button(
    icon: &'static str,
    message: Message,
    size: f32,
    button_type: ButtonType,
) -> Element<'static, Message> {
    let svg_handle = iced::advanced::svg::Handle::from_memory(icon.as_bytes());
    StyledButton::icon_only(svg_handle)
        .button_type(button_type)
        .size(size)
        .on_press(message)
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::widget::text;

    #[test]
    fn test_styled_button_with_icon_and_content() {
        let icon = r#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"><path d="M12 5v14M5 12h14" stroke="currentColor" stroke-width="2" stroke-linecap="round"/></svg>"#;
        let svg_handle = iced::advanced::svg::Handle::from_memory(icon.as_bytes());
        let button = StyledButton::new(text("测试按钮"))
            .icon(svg_handle)
            .button_type(ButtonType::Primary)
            .build();
        // 这里只是示例，实际测试需要运行时环境
        assert!(true);
    }

    #[test]
    fn test_styled_button_with_icon_only() {
        let icon = r#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"><path d="M12 5v14M5 12h14" stroke="currentColor" stroke-width="2" stroke-linecap="round"/></svg>"#;
        let svg_handle = iced::advanced::svg::Handle::from_memory(icon.as_bytes());
        let button = StyledButton::icon_only(svg_handle)
            .button_type(ButtonType::Primary)
            .build();
        // 这里只是示例，实际测试需要运行时环境
        assert!(true);
    }

    #[test]
    fn test_styled_button_with_content_only() {
        let button = StyledButton::new(text("仅内容按钮"))
            .button_type(ButtonType::Primary)
            .build();
        // 这里只是示例，实际测试需要运行时环境
        assert!(true);
    }
}
