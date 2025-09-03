//! 图标按钮组件
//! 
//! 提供带有图标和提示文本的按钮组件

use iced::{
    widget::{container, tooltip, svg, text},
    Element,
    Length,
    Color,
    alignment::{Horizontal, Vertical},
};

use crate::ui::{Message, widgets::{styled_button::StyledButton}};
use crate::ui::theme::AppColors;

/// 图标按钮组件
pub struct IconButton {
    icon: String,
    tooltip_text: String,
    on_press: Option<Message>,
    size: f32,
    icon_size: f32,
    button_type: crate::ui::widgets::styled_button::ButtonType,
    color: crate::ui::widgets::styled_button::ButtonColor,
}

impl IconButton {
    /// 创建新的图标按钮
    pub fn new(icon: impl Into<String>, tooltip_text: impl Into<String>) -> Self {
        Self {
            icon: icon.into(),
            tooltip_text: tooltip_text.into(),
            on_press: None,
            size: 40.0,
            icon_size: 24.0,
            button_type: crate::ui::widgets::styled_button::ButtonType::Default,
            color: crate::ui::widgets::styled_button::ButtonColor::Default,
        }
    }

    /// 设置按钮点击事件
    pub fn on_press(mut self, msg: Message) -> Self {
        self.on_press = Some(msg);
        self
    }

    /// 设置按钮尺寸
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// 设置图标尺寸
    pub fn icon_size(mut self, icon_size: f32) -> Self {
        self.icon_size = icon_size;
        self
    }

    /// 设置按钮样式
    pub fn style(mut self, button_type: crate::ui::widgets::styled_button::ButtonType, color: crate::ui::widgets::styled_button::ButtonColor) -> Self {
        self.button_type = button_type;
        self.color = color;
        self
    }

    /// 构建图标按钮元素
    pub fn build(self) -> Element<'static, Message> {
        let icon_handle = svg::Handle::from_memory(self.icon.as_bytes().to_vec());
        let icon_svg = svg(icon_handle)
            .width(Length::Fixed(self.icon_size))
            .height(Length::Fixed(self.icon_size))
            .style(|theme: &iced::Theme, _status: svg::Status| {
                // 在深色模式下使用更亮的颜色
                let is_dark_theme = {
                    let bg = theme.extended_palette().background.base.color;
                    bg.r + bg.g + bg.b < 1.5
                };
                
                svg::Style { 
                    color: Some(if is_dark_theme {
                        Color { r: 0.8, g: 0.8, b: 0.8, a: 1.0 }  // 更亮的图标颜色
                    } else {
                        Color { r: 0.4, g: 0.4, b: 0.4, a: 0.9 }
                    })
                }
            });

        let icon_container = container(icon_svg)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center);

        let btn = StyledButton::new(icon_container)
            .button_type(self.button_type)
            .color(self.color)
            .width(Length::Fixed(self.size))
            .height(Length::Fixed(self.size))
            .padding(0);

        let btn = if let Some(msg) = self.on_press {
            btn.on_press(msg)
        } else {
            btn
        };

        let button_element = btn.build();

        tooltip(
            button_element,
            text(self.tooltip_text).size(12),
            tooltip::Position::Top,
        )
        .style(|theme: &iced::Theme| {
            let palette = theme.extended_palette();
            iced::widget::container::Style {
                background: Some(iced::Background::Color(palette.background.strong.color)),
                text_color: Some(palette.background.strong.text),
                border: iced::Border {
                    radius: iced::border::Radius::from(6.0),
                    width: 1.0,
                    color: palette.background.weak.color,
                },
                shadow: iced::Shadow {
                    color: AppColors::shadow(theme),
                    offset: iced::Vector::new(0.0, 2.0),
                    blur_radius: 8.0,
                },
                snap: false,
            }
        })
        .padding(8)
        .into()
    }
}