//! 样式化容器组件
//! 
//! 提供多种预定义样式的容器组件

use iced::{
    widget::container, 
    Element, 
    Length, 
    Border, 
    Shadow, 
    Background, 
    Color,
    border::Radius,
};
use iced::widget::container::Style;

use crate::ui::Message;

/// 样式化容器组件
pub struct StyledContainer {
    content: Element<'static, Message>,
    style: ContainerStyle,
    width: Length,
    height: Length,
    padding: u16,
    center_x: bool,
    center_y: bool,
}

/// 容器样式类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ContainerStyle {
    /// 主要内容区域容器
    MainSection,
    /// 卡片容器
    Card,
    /// 背景容器
    Background,
    /// 信息卡片容器
    InfoCard,
    /// 玻璃态卡片容器
    GlassCard,
    /// 透明容器
    Transparent,
}

impl StyledContainer {
    /// 创建新的样式化容器
    pub fn new(content: impl Into<Element<'static, Message>>) -> Self {
        Self {
            content: content.into(),
            style: ContainerStyle::MainSection,
            width: Length::Shrink,
            height: Length::Shrink,
            padding: 0,
            center_x: false,
            center_y: false,
        }
    }

    /// 设置容器样式
    pub fn style(mut self, style: ContainerStyle) -> Self {
        self.style = style;
        self
    }

    /// 设置容器宽度
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// 设置容器高度
    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    /// 设置容器内边距
    pub fn padding(mut self, padding: u16) -> Self {
        self.padding = padding;
        self
    }

    /// 水平居中内容
    pub fn center_x(mut self) -> Self {
        self.center_x = true;
        self
    }

    /// 垂直居中内容
    pub fn center_y(mut self) -> Self {
        self.center_y = true;
        self
    }

    /// 构建容器元素
    pub fn build(self) -> Element<'static, Message> {
        let style_fn = self.get_style_fn();
        let mut container = container(self.content)
            .style(style_fn)
            .width(self.width)
            .height(self.height)
            .padding(self.padding);

        if self.center_x {
            container = container.center_x(Length::Fill);
        }

        if self.center_y {
            container = container.center_y(Length::Fill);
        }

        container.into()
    }

    /// 获取对应样式的函数
    fn get_style_fn(&self) -> Box<dyn Fn(&iced::Theme) -> Style> {
        match self.style {
            ContainerStyle::MainSection => Box::new(main_section_style),
            ContainerStyle::Card => Box::new(card_style),
            ContainerStyle::Background => Box::new(background_style),
            ContainerStyle::InfoCard => Box::new(info_card_style),
            ContainerStyle::GlassCard => Box::new(glass_card_style),
            ContainerStyle::Transparent => Box::new(transparent_style),
        }
    }
}

/// 主要内容区域容器样式
fn main_section_style(theme: &iced::Theme) -> Style {
    Style {
        background: Some(Background::Color(crate::ui::theme::AppColors::card_background(theme))),
        border: Border {
            radius: Radius::from(20.0), // 与AppTheme::main_section_container保持一致
            width: 1.0,
            color: crate::ui::theme::AppColors::border(theme),
        },
        shadow: Shadow {
            color: crate::ui::theme::AppColors::shadow(theme),
            offset: iced::Vector::new(0.0, 6.0), // 与AppTheme::main_section_container保持一致
            blur_radius: 20.0, // 与AppTheme::main_section_container保持一致
        },
        text_color: Some(crate::ui::theme::AppColors::text_primary(theme)),
    }
}

/// 卡片容器样式
fn card_style(theme: &iced::Theme) -> Style {
    Style {
        background: Some(Background::Color(crate::ui::theme::AppColors::card_background(theme))),
        border: Border {
            radius: Radius::from(16.0), // 与AppTheme::card_container保持一致
            width: 1.0,
            color: crate::ui::theme::AppColors::border(theme),
        },
        shadow: Shadow {
            color: crate::ui::theme::AppColors::shadow(theme),
            offset: iced::Vector::new(0.0, 4.0),
            blur_radius: 16.0,
        },
        text_color: Some(crate::ui::theme::AppColors::text_primary(theme)),
    }
}

/// 背景容器样式
fn background_style(theme: &iced::Theme) -> Style {
    Style {
        background: Some(Background::Color(crate::ui::theme::AppColors::background_gradient_start(theme))),
        border: Border {
            radius: Radius::from(0.0),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        shadow: Shadow::default(),
        text_color: Some(crate::ui::theme::AppColors::text_primary(theme)),
    }
}

/// 信息卡片容器样式
fn info_card_style(theme: &iced::Theme) -> Style {
    let primary = crate::ui::theme::AppColors::primary(theme);
    Style {
        background: Some(Background::Color(Color {
            a: 0.05,
            ..primary
        })),
        border: Border {
            radius: Radius::from(8.0),
            width: 1.0,
            color: Color {
                a: 0.2,
                ..primary
            },
        },
        shadow: Shadow {
            color: crate::ui::theme::AppColors::shadow(theme),
            offset: iced::Vector::new(0.0, 1.0),
            blur_radius: 3.0,
        },
        text_color: Some(crate::ui::theme::AppColors::text_primary(theme)),
    }
}

/// 玻璃态卡片容器样式
fn glass_card_style(theme: &iced::Theme) -> Style {
    Style {
        background: Some(Background::Color(crate::ui::theme::AppColors::card_background_translucent(theme))),
        border: Border {
            radius: Radius::from(20.0), // 与AppTheme::glass_card_container保持一致
            width: 1.0,
            color: crate::ui::theme::AppColors::divider(theme),
        },
        shadow: Shadow {
            color: crate::ui::theme::AppColors::shadow_strong(theme),
            offset: iced::Vector::new(0.0, 8.0),
            blur_radius: 24.0,
        },
        text_color: Some(crate::ui::theme::AppColors::text_primary(theme)),
    }
}

/// 透明容器样式
fn transparent_style(_theme: &iced::Theme) -> Style {
    Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        border: Border::default(),
        shadow: Shadow::default(),
        text_color: None,
    }
}