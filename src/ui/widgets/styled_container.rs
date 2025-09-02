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
    alignment::{Horizontal, Vertical},
    Padding,
};
use iced::widget::container::Style;

use crate::ui::Message;

/// 样式化容器组件
pub struct StyledContainer<'a> {
    content: Element<'a, Message>,
    style: ContainerStyle,
    width: Length,
    height: Length,
    padding: Padding,
    align_x: Option<Horizontal>,
    align_y: Option<Vertical>,
}

/// 容器样式类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ContainerStyle {
    /// 主要内容区域容器 - 用于主要内容区域，带阴影和圆角
    MainSection,
    /// 卡片容器 - 用于一般卡片，中等阴影和圆角
    Card,
    /// 背景容器 - 用于页面背景，无边框无阴影
    Background,
    /// 强调容器 - 用于需要突出显示的内容（合并InfoCard, CurrentLyric, Hint）
    Emphasis,
    /// 装饰容器 - 用于装饰性元素（合并AlbumCover, PlaylistIcon, PlaylistCard, TimeDisplay, SongInfo）
    Decorative,
    /// 透明容器 - 完全透明，无背景、无边框、无阴影，仅用于布局控制
    Transparent,
}

impl<'a> StyledContainer<'a> {
    /// 创建新的样式化容器
    pub fn new(content: impl Into<Element<'a, Message>>) -> Self {
        Self {
            content: content.into(),
            style: ContainerStyle::MainSection,
            width: Length::Shrink,
            height: Length::Shrink,
            padding: Padding::ZERO,
            align_x: None,
            align_y: None,
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
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// 水平居中内容
    pub fn center_x(mut self) -> Self {
        self.align_x = Some(Horizontal::Center);
        self
    }

    /// 垂直居中内容
    pub fn center_y(mut self) -> Self {
        self.align_y = Some(Vertical::Center);
        self
    }

    /// 设置水平对齐
    pub fn align_x(mut self, align: Horizontal) -> Self {
        self.align_x = Some(align);
        self
    }

    /// 设置垂直对齐
    pub fn align_y(mut self, align: Vertical) -> Self {
        self.align_y = Some(align);
        self
    }

    /// 构建容器元素
    pub fn build(self) -> Element<'a, Message> {
        let style_fn = self.get_style_fn();
        let mut container = container(self.content)
            .style(style_fn)
            .width(self.width)
            .height(self.height)
            .padding(self.padding);

        if let Some(align) = self.align_x {
            container = container.align_x(align);
        }

        if let Some(align) = self.align_y {
            container = container.align_y(align);
        }

        container.into()
    }

    /// 获取对应样式的函数
    fn get_style_fn(&self) -> Box<dyn Fn(&iced::Theme) -> Style> {
        match self.style {
            ContainerStyle::MainSection => Box::new(main_section_style),
            ContainerStyle::Card => Box::new(card_style),
            ContainerStyle::Background => Box::new(background_style),
            ContainerStyle::Emphasis => Box::new(emphasis_style),
            ContainerStyle::Decorative => Box::new(decorative_style),
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
        snap: false,
    }
}

/// 卡片容器样式
fn card_style(theme: &iced::Theme) -> Style {
    Style {
        background: Some(Background::Color(crate::ui::theme::AppColors::card_background(theme))),
        border: Border {
            radius: Radius::from(6.0), // 与AppTheme::card_container保持一致
            width: 1.0,
            color: crate::ui::theme::AppColors::border(theme),
        },
        shadow: Shadow {
            color: crate::ui::theme::AppColors::shadow(theme),
            offset: iced::Vector::new(0.0, 4.0),
            blur_radius: 16.0,
        },
        text_color: Some(crate::ui::theme::AppColors::text_primary(theme)),
        snap: false,
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
        snap: false,
    }
}

/// 强调容器样式 - 用于需要突出显示的内容（合并InfoCard, CurrentLyric, Hint）
fn emphasis_style(theme: &iced::Theme) -> Style {
    let palette = theme.extended_palette();
    Style {
        background: Some(Background::Color(Color { a: 0.08, ..palette.primary.base.color })),
        border: Border {
            radius: Radius::from(8.0),
            width: 1.0,
            color: Color { a: 0.2, ..palette.primary.base.color },
        },
        shadow: Shadow {
            color: crate::ui::theme::AppColors::shadow(theme),
            offset: iced::Vector::new(0.0, 2.0),
            blur_radius: 6.0,
        },
        text_color: Some(crate::ui::theme::AppColors::text_primary(theme)),
        snap: false,
    }
}

/// 装饰容器样式 - 用于装饰性元素（合并AlbumCover, PlaylistIcon, PlaylistCard, TimeDisplay, SongInfo）
fn decorative_style(theme: &iced::Theme) -> Style {
    let palette = theme.extended_palette();
    Style {
        background: Some(Background::Color(Color { a: 0.05, ..palette.background.strong.color })),
        border: Border {
            radius: Radius::from(6.0),
            width: 1.0,
            color: Color { a: 0.15, ..palette.background.strong.color },
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.1),
            offset: iced::Vector::new(0.0, 2.0),
            blur_radius: 4.0,
        },
        text_color: None,
        snap: false,
    }
}

/// 透明容器样式 - 完全透明，无背景、无边框、无阴影，仅用于布局控制
fn transparent_style(_theme: &iced::Theme) -> Style {
    Style {
        background: None,
        border: Border {
            radius: Radius::from(0.0),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        shadow: Shadow::default(),
        text_color: None,
        snap: false,
    }
}