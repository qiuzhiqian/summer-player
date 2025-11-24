//! 模态窗口组件模块
//!
//! 定义应用程序中使用的所有模态窗口组件。

use iced::{
    widget::{column, container, text, text_input, button, stack, mouse_area, center, row, opaque},
    Element, Length, Color, Background,
};

use super::Message;
use crate::ui::theme::{AppColors, AppTheme};

/// 模态窗口组件函数
pub fn modal<'a, Message>(
    base: impl Into<Element<'a, Message>>,
    content: impl Into<Element<'a, Message>>,
    on_blur: Message,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
     stack![
        base.into(),
        opaque(
            mouse_area(center(opaque(content)).style(|_theme| {
                container::Style {
                    background: Some(
                        Color {
                            a: 0.8,
                            ..Color::BLACK
                        }
                        .into(),
                    ),
                    ..container::Style::default()
                }
            }))
            .on_press(on_blur)
        )
    ]
    .into()
}

// 播放列表重命名模态窗口
/*pub fn playlist_rename_modal(
    playlist_path: &'static str,
    new_name: &'static str,
) -> Element<'static, Message> {
    container(
        column![
            text("重命名播放列表").size(24),
            column![
                column![
                    text("播放列表路径").size(12),
                    text(playlist_path).size(10),
                ]
                .spacing(5),
                column![
                    text("新名称").size(12),
                    text_input("输入新名称", new_name)
                        .on_input(|v| Message::ModalInputChanged {
                            field_type: "playlist_rename".to_string(),
                            field: "new_name".to_string(),
                            value: v
                        }),
                ]
                .spacing(5),
                row![
                    button(text("取消")).on_press(Message::HideModal),
                    button(text("确认")).on_press(Message::PlaylistCardRenameModal(playlist_path.to_string(), new_name.to_string())),
                ]
                .spacing(10)
            ]
            .spacing(20),
        ]
    )
    .width(Length::Fixed(350.0)) // 设置固定宽度350px
    .padding(10)
    .into()
}*/