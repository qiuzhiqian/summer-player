//! 样式化文本组件
//! 
//! 提供多种预定义样式的文本组件

use iced::{
    widget::text,
    Element,
    Color,
    Length,
};
use iced::advanced::text::Shaping;

use crate::ui::Message;
use crate::ui::theme::AppColors;

/// 样式化文本组件
pub struct StyledText {
    content: String,
    size: u16,
    style: TextStyle,
    width: Length,
    align: Option<iced::alignment::Horizontal>,
    shaping: Shaping,
}

/// 文本样式类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextStyle {
    /// 默认文本
    Default,
    /// 主要文本
    Primary,
    /// 次要文本
    Secondary,
    /// 提示文本
    Hint,
    /// 强调文本
    Emphasis,
    /// 当前歌词文本
    CurrentLyrics,
    /// 歌词文本
    Lyrics,
    /// 带透明度的文本
    WithAlpha(f32),
}

impl StyledText {
    /// 创建新的样式化文本
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            size: 16,
            style: TextStyle::Default,
            width: Length::Shrink,
            align: None,
            shaping: Shaping::Basic,
        }
    }

    /// 设置文本大小
    pub fn size(mut self, size: u16) -> Self {
        self.size = size;
        self
    }

    /// 设置文本样式
    pub fn style(mut self, style: TextStyle) -> Self {
        self.style = style;
        self
    }

    /// 设置文本宽度
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// 设置文本对齐方式
    pub fn align(mut self, align: iced::alignment::Horizontal) -> Self {
        self.align = Some(align);
        self
    }

    /// 设置文本形状
    pub fn shaping(mut self, shaping: Shaping) -> Self {
        self.shaping = shaping;
        self
    }

    /// 构建文本元素
    pub fn build(self) -> Element<'static, Message> {
        let style_fn = self.get_style_fn();
        let mut text_widget = text(self.content)
            .size(self.size)
            .style(style_fn)
            .width(self.width)
            .shaping(self.shaping);

        if let Some(align) = self.align {
            text_widget = text_widget.align_x(align);
        }

        text_widget.into()
    }

    /// 获取对应样式的函数
    fn get_style_fn(&self) -> Box<dyn Fn(&iced::Theme) -> iced::widget::text::Style> {
        match self.style {
            TextStyle::Default => Box::new(default_text_style),
            TextStyle::Primary => Box::new(primary_text_style),
            TextStyle::Secondary => Box::new(secondary_text_style),
            TextStyle::Hint => Box::new(hint_text_style),
            TextStyle::Emphasis => Box::new(emphasis_text_style),
            TextStyle::CurrentLyrics => Box::new(current_lyrics_text_style),
            TextStyle::Lyrics => Box::new(lyrics_text_style),
            TextStyle::WithAlpha(alpha) => Box::new(move |theme| alpha_text_style(theme, alpha)),
        }
    }
}

/// 默认文本样式
fn default_text_style(theme: &iced::Theme) -> iced::widget::text::Style {
    let palette = theme.extended_palette();
    iced::widget::text::Style {
        color: Some(palette.background.base.text),
    }
}

/// 主要文本样式
fn primary_text_style(theme: &iced::Theme) -> iced::widget::text::Style {
    let palette = theme.extended_palette();
    iced::widget::text::Style {
        color: Some(palette.primary.base.color),
    }
}

/// 次要文本样式
fn secondary_text_style(theme: &iced::Theme) -> iced::widget::text::Style {
    iced::widget::text::Style {
        color: Some(AppColors::text_secondary(theme)),
    }
}

/// 提示文本样式
fn hint_text_style(theme: &iced::Theme) -> iced::widget::text::Style {
    let palette = theme.extended_palette();
    iced::widget::text::Style {
        color: Some(Color {
            a: 0.7,
            ..palette.background.base.text
        }),
    }
}

/// 强调文本样式
fn emphasis_text_style(theme: &iced::Theme) -> iced::widget::text::Style {
    let palette = theme.extended_palette();
    iced::widget::text::Style {
        color: Some(palette.primary.strong.color),
    }
}

/// 当前歌词文本样式
fn current_lyrics_text_style(theme: &iced::Theme) -> iced::widget::text::Style {
    let palette = theme.extended_palette();
    iced::widget::text::Style {
        color: Some(palette.primary.strong.color),
    }
}

/// 歌词文本样式
fn lyrics_text_style(theme: &iced::Theme) -> iced::widget::text::Style {
    let palette = theme.extended_palette();
    iced::widget::text::Style {
        color: Some(Color {
            a: 0.7,
            ..palette.background.base.text
        }),
    }
}

/// 带透明度的文本样式
fn alpha_text_style(theme: &iced::Theme, alpha: f32) -> iced::widget::text::Style {
    let palette = theme.extended_palette();
    iced::widget::text::Style {
        color: Some(Color {
            a: alpha,
            ..palette.background.base.text
        }),
    }
}