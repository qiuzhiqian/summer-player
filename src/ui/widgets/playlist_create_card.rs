//! 创建播放列表卡片控件

use iced::{
    widget::{button, column, row, text_input, Space},
    Element, Length, Background, Border, Shadow, Color,
    alignment::{Horizontal, Vertical},
    border::Radius,
};
use iced::advanced::text::Shaping;
use crate::ui::Message;
use crate::ui::components::{constants, icons, svg_icon};
use crate::ui::widgets::{StyledContainer, StyledText, StyledButton, styled_container::ContainerStyle};
use rust_i18n::t;

/// 创建播放列表卡片控件
pub struct CreatePlaylistCard;

impl CreatePlaylistCard {
    pub fn display_card() -> Element<'static, Message> {
        // 与普通卡片一致的样式（透明背景，悬停高亮）
        let content = StyledContainer::new(
            column![
                StyledContainer::new(svg_icon(icons::CD_ICON, 64.0, constants::ICON_COLOR))
                    .style(ContainerStyle::Decorative)
                    .align_x(Horizontal::Center)
                    .build(),
                StyledText::new("+")
                    .size(54)
                    .build(),
                StyledText::new(t!("Create Playlist"))
                    .size(constants::TEXT_MEDIUM)
                    .build(),
            ]
            .spacing(constants::SPACING_MEDIUM)
            .align_x(Horizontal::Center)
        )
        .style(ContainerStyle::Card)
        .width(Length::Fixed(170.0))
        .height(Length::Fixed(230.0))
        .align_x(Horizontal::Center)
        .align_y(Vertical::Center)
        .padding([constants::PADDING_MEDIUM, constants::PADDING_SMALL])
        .build();

        button(content)
            .on_press(Message::StartCreatePlaylist)
            .style(|theme: &iced::Theme, status| {
                let palette = theme.extended_palette();
                match status {
                    iced::widget::button::Status::Active => iced::widget::button::Style {
                        background: Some(Background::Color(Color::TRANSPARENT)),
                        text_color: palette.background.base.text,
                        border: Border { radius: Radius::from(8.0), width: 0.0, color: Color::TRANSPARENT },
                        shadow: Shadow::default(),
                        snap: false,
                    },
                    iced::widget::button::Status::Hovered => iced::widget::button::Style {
                        background: Some(Background::Color(Color { a: 0.15, ..palette.primary.base.color })),
                        text_color: palette.primary.strong.color,
                        border: Border { radius: Radius::from(8.0), width: 0.0, color: Color::TRANSPARENT },
                        shadow: Shadow { color: Color::from_rgba(0.0, 0.0, 0.0, 0.2), offset: iced::Vector::new(0.0, 4.0), blur_radius: 10.0 },
                        snap: false,
                    },
                    iced::widget::button::Status::Pressed => iced::widget::button::Style {
                        background: Some(Background::Color(Color { a: 0.25, ..palette.primary.base.color })),
                        text_color: palette.primary.strong.color,
                        border: Border { radius: Radius::from(8.0), width: 0.0, color: Color::TRANSPARENT },
                        shadow: Shadow { color: Color::from_rgba(0.0, 0.0, 0.0, 0.25), offset: iced::Vector::new(0.0, 2.0), blur_radius: 6.0 },
                        snap: false,
                    },
                    iced::widget::button::Status::Disabled => iced::widget::button::Style {
                        background: Some(Background::Color(Color { a: 0.05, ..palette.background.strong.color })),
                        text_color: Color { a: 0.5, ..palette.background.base.text },
                        border: Border { radius: Radius::from(8.0), width: 0.0, color: Color::TRANSPARENT },
                        shadow: Shadow::default(),
                        snap: false,
                    },
                }
            })
            .into()
    }

    pub fn input_card(current_name: &str) -> Element<'static, Message> {
        let input = text_input::<Message, iced::Theme, iced::Renderer>(t!("Playlist Name").as_ref(), current_name)
            .on_input(Message::CreatePlaylistNameChanged)
            .size(constants::TEXT_NORMAL)
            .padding(8)
            .width(Length::Fill);

        StyledContainer::new(
            column![
                StyledContainer::new(svg_icon(icons::CD_ICON, 56.0, constants::ICON_COLOR))
                    .style(ContainerStyle::Decorative)
                    .align_x(Horizontal::Center)
                    .build(),
                StyledText::new(t!("New Playlist")).size(constants::TEXT_MEDIUM).build(),
                {
                    let elem: Element<Message> = input.into();
                    elem
                },
                {
                    let confirm_btn = StyledButton::new(
                        StyledText::new("✔").shaping(Shaping::Advanced).size(constants::TEXT_MEDIUM).build()
                    )
                    .on_press(Message::ConfirmCreatePlaylist)
                    .style(super::styled_button::ButtonStyle::File)
                    .build();

                    let cancel_btn = StyledButton::new(
                        StyledText::new("✖").shaping(Shaping::Advanced).size(constants::TEXT_MEDIUM).build()
                    )
                    .on_press(Message::CancelCreatePlaylist)
                    .style(super::styled_button::ButtonStyle::File)
                    .build();

                    let actions = row![
                        confirm_btn,
                        Space::with_width(Length::Fixed(8.0)),
                        cancel_btn,
                    ]
                    .spacing(constants::SPACING_SMALL)
                    .align_y(Vertical::Center);

                    let elem: Element<Message> = actions.into();
                    elem
                },
            ]
            .spacing(constants::SPACING_SMALL)
            .align_x(Horizontal::Center)
        )
        .style(ContainerStyle::Card)
        .width(Length::Fixed(170.0))
        .height(Length::Fixed(240.0))
        .align_x(Horizontal::Center)
        .align_y(Vertical::Center)
        .padding([constants::PADDING_MEDIUM, constants::PADDING_SMALL])
        .build()
        .into()
    }
}


