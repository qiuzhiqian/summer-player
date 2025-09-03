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
use crate::ui::theme::AppColors;

/// 样式化按钮组件
pub struct StyledButton {
    content: Element<'static, Message>,
    button_type: ButtonType,
    color: ButtonColor,
    style_override: Option<Box<dyn Fn(&iced::Theme, iced::widget::button::Status) -> iced::widget::button::Style + 'static>>,
    width: Length,
    height: Length,
    on_press: Option<Message>,
    padding: u16,
}

/// 按钮类型（参考 Ant Design）
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ButtonType { Primary, Dashed, Link, Text, Default }

impl Default for ButtonType { fn default() -> Self { ButtonType::Default } }

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
pub enum ButtonColor { Default, Primary, Danger, Preset(PresetColor) }

impl Default for ButtonColor { fn default() -> Self { ButtonColor::Default } }

impl StyledButton {
    /// 创建新的样式化按钮
    pub fn new(content: impl Into<Element<'static, Message>>) -> Self {
        Self {
            content: content.into(),
            button_type: ButtonType::Default,
            color: ButtonColor::Default,
            style_override: None,
            width: Length::Shrink,
            height: Length::Shrink,
            on_press: None,
            padding: 0,
        }
    }

    /// 设置按钮类型
    pub fn button_type(mut self, button_type: ButtonType) -> Self { self.button_type = button_type; self }

    /// 设置按钮颜色
    pub fn color(mut self, color: ButtonColor) -> Self { self.color = color; self }

    /// 提供自定义样式（将覆盖类型+颜色的默认渲染）
    pub fn style_override(
        mut self,
        f: impl 'static + Fn(&iced::Theme, iced::widget::button::Status) -> iced::widget::button::Style,
    ) -> Self {
        self.style_override = Some(Box::new(f));
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
        let StyledButton { content, button_type, color, style_override, width, height, on_press, padding } = self;
        let style_fn = Self::resolve_style_fn(button_type, color, style_override);
        let button_widget = button(content)
            .style(style_fn)
            .width(width)
            .height(height)
            .padding(padding);

        if let Some(msg) = on_press {
            button_widget.on_press(msg)
        } else {
            button_widget
        }.into()
    }

    /// 获取对应样式的函数
    fn resolve_style_fn(
        button_type: ButtonType,
        color: ButtonColor,
        style_override: Option<Box<dyn Fn(&iced::Theme, iced::widget::button::Status) -> iced::widget::button::Style + 'static>>,
    ) -> Box<dyn Fn(&iced::Theme, iced::widget::button::Status) -> iced::widget::button::Style + 'static> {
        if let Some(custom) = style_override {
            return Box::new(move |theme, status| (custom)(theme, status));
        }

        Box::new(move |theme: &iced::Theme, status: iced::widget::button::Status| {
            let base_text = theme.extended_palette().background.base.text;
            let neutral_border = AppColors::border(theme);
            let neutral_bg = AppColors::surface(theme);
            let disabled_bg = theme.extended_palette().background.weak.color;
            let disabled_text = theme.extended_palette().background.weak.text;

            let accent = match color {
                ButtonColor::Default => AppColors::primary(theme),
                ButtonColor::Primary => AppColors::primary(theme),
                ButtonColor::Danger => theme.extended_palette().danger.base.color,
                ButtonColor::Preset(p) => preset_to_color(theme, p),
            };

            let radius = Radius::from(12.0);

            match button_type {
                ButtonType::Primary => match status {
                    iced::widget::button::Status::Disabled => iced::widget::button::Style {
                        background: Some(Background::Color(disabled_bg)),
                        text_color: disabled_text,
                        border: Border { radius, width: 1.0, color: neutral_border },
                        shadow: Shadow::default(),
                        snap: false,
                    },
                    iced::widget::button::Status::Pressed => iced::widget::button::Style {
                        background: Some(Background::Color(Color { r: accent.r * 0.9, g: accent.g * 0.9, b: accent.b * 0.9, a: 1.0 })),
                        text_color: Color::WHITE,
                        border: Border { radius, width: 1.0, color: Color { a: 0.2, ..accent } },
                        shadow: Shadow { color: AppColors::shadow(theme), offset: iced::Vector::new(0.0, 2.0), blur_radius: 8.0 },
                        snap: false,
                    },
                    iced::widget::button::Status::Hovered => iced::widget::button::Style {
                        background: Some(Background::Color(Color { r: (accent.r * 1.05).min(1.0), g: (accent.g * 1.05).min(1.0), b: (accent.b * 1.05).min(1.0), a: 1.0 })),
                        text_color: Color::WHITE,
                        border: Border { radius, width: 1.0, color: Color { a: 0.3, ..accent } },
                        shadow: Shadow { color: AppColors::shadow(theme), offset: iced::Vector::new(0.0, 4.0), blur_radius: 12.0 },
                        snap: false,
                    },
                    _ => iced::widget::button::Style {
                        background: Some(Background::Color(accent)),
                        text_color: Color::WHITE,
                        border: Border { radius, width: 1.0, color: Color { a: 0.15, ..accent } },
                        shadow: Shadow { color: AppColors::shadow(theme), offset: iced::Vector::new(0.0, 2.0), blur_radius: 8.0 },
                        snap: false,
                    },
                },

                ButtonType::Dashed => match status {
                    iced::widget::button::Status::Disabled => iced::widget::button::Style {
                        background: Some(Background::Color(Color::TRANSPARENT)),
                        text_color: disabled_text,
                        border: Border { radius, width: 1.0, color: neutral_border },
                        shadow: Shadow::default(),
                        snap: false,
                    },
                    iced::widget::button::Status::Hovered => iced::widget::button::Style {
                        background: Some(Background::Color(Color { a: 0.06, ..accent })),
                        text_color: accent,
                        border: Border { radius, width: 1.0, color: Color { a: 0.6, ..accent } },
                        shadow: Shadow { color: AppColors::shadow(theme), offset: iced::Vector::new(0.0, 2.0), blur_radius: 6.0 },
                        snap: false,
                    },
                    iced::widget::button::Status::Pressed => iced::widget::button::Style {
                        background: Some(Background::Color(Color { a: 0.1, ..accent })),
                        text_color: accent,
                        border: Border { radius, width: 1.0, color: Color { a: 0.8, ..accent } },
                        shadow: Shadow { color: AppColors::shadow(theme), offset: iced::Vector::new(0.0, 1.0), blur_radius: 4.0 },
                        snap: false,
                    },
                    _ => iced::widget::button::Style {
                        background: Some(Background::Color(Color::TRANSPARENT)),
                        text_color: accent,
                        border: Border { radius, width: 1.0, color: Color { a: 0.4, ..accent } },
                        shadow: Shadow::default(),
                        snap: false,
                    },
                },

                ButtonType::Link => match status {
                    iced::widget::button::Status::Disabled => iced::widget::button::Style {
                        background: Some(Background::Color(Color::TRANSPARENT)),
                        text_color: disabled_text,
                        border: Border { radius, width: 0.0, color: Color::TRANSPARENT },
                        shadow: Shadow::default(),
                        snap: false,
                    },
                    iced::widget::button::Status::Hovered => iced::widget::button::Style {
                        background: Some(Background::Color(Color { a: 0.04, ..accent })),
                        text_color: accent,
                        border: Border { radius, width: 0.0, color: Color::TRANSPARENT },
                        shadow: Shadow::default(),
                        snap: false,
                    },
                    _ => iced::widget::button::Style {
                        background: Some(Background::Color(Color::TRANSPARENT)),
                        text_color: accent,
                        border: Border { radius, width: 0.0, color: Color::TRANSPARENT },
                        shadow: Shadow::default(),
                        snap: false,
                    },
                },

                ButtonType::Text => match status {
                    iced::widget::button::Status::Disabled => iced::widget::button::Style {
                        background: Some(Background::Color(Color::TRANSPARENT)),
                        text_color: disabled_text,
                        border: Border { radius, width: 0.0, color: Color::TRANSPARENT },
                        shadow: Shadow::default(),
                        snap: false,
                    },
                    iced::widget::button::Status::Hovered => iced::widget::button::Style {
                        background: Some(Background::Color(Color { a: 0.06, ..neutral_bg })),
                        text_color: if matches!(color, ButtonColor::Default) { base_text } else { accent },
                        border: Border { radius, width: 0.0, color: Color::TRANSPARENT },
                        shadow: Shadow::default(),
                        snap: false,
                    },
                    _ => iced::widget::button::Style {
                        background: Some(Background::Color(Color::TRANSPARENT)),
                        text_color: if matches!(color, ButtonColor::Default) { base_text } else { accent },
                        border: Border { radius, width: 0.0, color: Color::TRANSPARENT },
                        shadow: Shadow::default(),
                        snap: false,
                    },
                },

                ButtonType::Default => match status {
                    iced::widget::button::Status::Disabled => iced::widget::button::Style {
                        background: Some(Background::Color(Color::TRANSPARENT)),
                        text_color: disabled_text,
                        border: Border { radius, width: 1.0, color: neutral_border },
                        shadow: Shadow::default(),
                        snap: false,
                    },
                    iced::widget::button::Status::Hovered => iced::widget::button::Style {
                        background: Some(Background::Color(Color { a: 0.06, ..accent })),
                        text_color: if matches!(color, ButtonColor::Default) { base_text } else { accent },
                        border: Border { radius, width: 1.0, color: if matches!(color, ButtonColor::Default) { neutral_border } else { Color { a: 0.5, ..accent } } },
                        shadow: Shadow { color: AppColors::shadow(theme), offset: iced::Vector::new(0.0, 2.0), blur_radius: 6.0 },
                        snap: false,
                    },
                    iced::widget::button::Status::Pressed => iced::widget::button::Style {
                        background: Some(Background::Color(Color { a: 0.1, ..accent })),
                        text_color: if matches!(color, ButtonColor::Default) { base_text } else { accent },
                        border: Border { radius, width: 1.0, color: if matches!(color, ButtonColor::Default) { neutral_border } else { Color { a: 0.7, ..accent } } },
                        shadow: Shadow { color: AppColors::shadow(theme), offset: iced::Vector::new(0.0, 1.0), blur_radius: 4.0 },
                        snap: false,
                    },
                    _ => iced::widget::button::Style {
                        background: Some(Background::Color(Color::TRANSPARENT)),
                        text_color: if matches!(color, ButtonColor::Default) { base_text } else { accent },
                        border: Border { radius, width: 1.0, color: if matches!(color, ButtonColor::Default) { neutral_border } else { Color { a: 0.4, ..accent } } },
                        shadow: Shadow::default(),
                        snap: false,
                    },
                },
            }
        })
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
        PresetColor::Green => AppColors::success(theme),
        PresetColor::Cyan => Color::from_rgb(0.18, 0.80, 0.80),
        PresetColor::Blue => AppColors::primary(theme),
        PresetColor::GeekBlue => Color::from_rgb(0.24, 0.34, 0.80),
        PresetColor::Purple => Color::from_rgb(0.58, 0.34, 0.84),
    }
}