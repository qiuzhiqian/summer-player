//! UI组件模块
//!
//! 包含可重用的UI组件和通用样式。

use crate::ui::widgets::{
    styled_button::{ButtonColor, ButtonType},
    styled_container::ContainerStyle,
};
use iced::advanced::text::Shaping;
use iced::{
    alignment::{Horizontal, Vertical},
    border::Radius,
    widget::{column, container, row, scrollable, slider, svg, text, tooltip, Space},
    Background, Border, Color, Element, Length, Shadow,
};

use crate::audio::{AudioInfo, PlaybackState};
use crate::playlist::Playlist;
use crate::utils::{extract_filename, format_duration};

use super::theme::{AppColors, AppTheme, AppThemeVariant};
use super::widgets::{
    icon_button, CreatePlaylistCard, PlaylistCard, StyledButton, StyledContainer, StyledText,
};
use super::Message;
use rust_i18n::t;

// use dirs;

// ============================================================================
// 常量和配置
// ============================================================================

pub mod constants {
    use iced::Color;

    // 尺寸常量
    pub const BUTTON_SIZE_SMALL: f32 = 40.0;
    pub const BUTTON_SIZE_MEDIUM: f32 = 48.0;
    pub const BUTTON_SIZE_LARGE: f32 = 60.0;

    pub const ICON_SIZE_SMALL: f32 = 22.0;
    pub const ICON_SIZE_MEDIUM: f32 = 24.0;
    pub const ICON_SIZE_LARGE: f32 = 30.0;
    pub const ICON_SIZE_XLARGE: f32 = 35.0;

    // 间距常量
    pub const SPACING_SMALL: u32 = 6;
    pub const SPACING_MEDIUM: u32 = 12;
    pub const SPACING_LARGE: u32 = 20;

    pub const PADDING_SMALL: u16 = 8;
    pub const PADDING_MEDIUM: u16 = 16;
    pub const PADDING_LARGE: u16 = 24;

    // 文本大小
    pub const TEXT_SMALL: u32 = 10;
    pub const TEXT_NORMAL: u32 = 12;
    pub const TEXT_MEDIUM: u32 = 14;
    pub const TEXT_LARGE: u32 = 16;
    pub const TEXT_TITLE: u32 = 20;

    // 截断长度
    pub const TEXT_TRUNCATE_DEFAULT: usize = 30;
    pub const TEXT_TRUNCATE_LONG: usize = 40;

    // 颜色
    pub const ICON_COLOR: Color = Color {
        r: 0.4,
        g: 0.4,
        b: 0.4,
        a: 0.9,
    };
    pub const ICON_COLOR_SUBTLE: Color = Color {
        r: 0.4,
        g: 0.4,
        b: 0.4,
        a: 0.8,
    };
}

// ============================================================================
// SVG 图标
// ============================================================================

pub mod icons {
    pub const FILE_FOLDER: &str = r#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"><path d="M3 6.2c0-1.12 0-1.68.218-2.108a2 2 0 0 1 .874-.874C4.52 3 5.08 3 6.2 3h1.2c.56 0 .84 0 1.054.109a1 1 0 0 1 .437.437C9 3.76 9 4.04 9 4.6V5h8.8c1.12 0 1.68 0 2.108.218a2 2 0 0 1 .874.874C21 6.52 21 7.08 21 8.2v9.6c0 1.12 0 1.68-.218 2.108a2 2 0 0 1-.874.874C19.48 21 18.92 21 17.8 21H6.2c-1.12 0-1.68 0-2.108-.218a2 2 0 0 1-.874-.874C3 19.48 3 18.92 3 17.8V6.2Z" stroke="currentColor" stroke-width="1.5"/></svg>"#;
    pub const LIST_LOOP: &str = r#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"><path d="M17 8.5V6a2 2 0 0 0-2-2H4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/><path d="m20 8.5-3-2.5v5l3-2.5Z" fill="currentColor"/><path d="M7 15.5V18a2 2 0 0 0 2 2h11" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/><path d="m4 15.5 3 2.5v-5l-3 2.5Z" fill="currentColor"/></svg>"#;
    pub const SINGLE_LOOP: &str = r#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"><path d="M17 8.5V6a2 2 0 0 0-2-2H4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/><path d="m20 8.5-3-2.5v5l3-2.5Z" fill="currentColor"/><path d="M7 15.5V18a2 2 0 0 0 2 2h11" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/><path d="m4 15.5 3 2.5v-5l-3 2.5Z" fill="currentColor"/><circle cx="12" cy="12" r="2" stroke="currentColor" stroke-width="1.5"/><text x="12" y="12" text-anchor="middle" dominant-baseline="central" font-size="6" font-weight="bold" fill="currentColor">1</text></svg>"#;
    pub const RANDOM_PLAY: &str = r#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"><path d="m3 17 6-4-6-4v8Z" fill="currentColor"/><path d="M14 6h5v5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/><path d="M19 6 9 16" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/><path d="M14 18h5v-5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/><path d="M19 18 9 8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/></svg>"#;
    pub const MUSIC_NOTE: &str = r#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"><circle cx="7" cy="17" r="3" stroke="currentColor" stroke-width="1.5"/><circle cx="17" cy="15" r="3" stroke="currentColor" stroke-width="1.5"/><path d="M10 17V5l10-2v12" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/><path d="M10 9l10-2" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/></svg>"#;
    pub const LIST_VIEW: &str = r#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"><path d="M3 6h18M3 12h18M3 18h18" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/></svg>"#;
    pub const CD_ICON: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE svg PUBLIC "-//W3C//DTD SVG 1.1//EN" "http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd">
<!-- Uploaded to: SVG Repo, www.svgrepo.com, Generator: SVG Repo Mixer Tools -->
<svg height="800px" width="800px" version="1.1" id="_x32_" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink"
	 viewBox="0 0 512 512"  xml:space="preserve">
<style type="text/css">
	.st0{fill:#000000;}
</style>
<g>
	<path class="st0" d="M256,0C114.616,0,0,114.616,0,256s114.616,256,256,256s256-114.616,256-256S397.384,0,256,0z M256,362.058
		C197.425,362.05,149.95,314.575,149.942,256C149.95,197.425,197.425,149.95,256,149.942C314.575,149.95,362.05,197.425,362.058,256
		C362.05,314.575,314.575,362.05,256,362.058z"/>
	<path class="st0" d="M256,179.2c-21.25,0.008-40.358,8.575-54.309,22.491C187.775,215.642,179.208,234.75,179.2,256
		c0.008,21.25,8.575,40.358,22.491,54.309c13.95,13.916,33.059,22.483,54.309,22.491c21.25-0.008,40.358-8.575,54.309-22.491
		c13.916-13.95,22.483-33.059,22.491-54.309c-0.008-21.25-8.575-40.358-22.491-54.309C296.358,187.775,277.25,179.208,256,179.2z
		 M256,297.633c-22.991,0-41.633-18.642-41.633-41.633s18.642-41.633,41.633-41.633c22.991,0,41.633,18.642,41.633,41.633
		S278.991,297.633,256,297.633z"/>
</g>
</svg>"#;
    pub const PREVIOUS: &str = r#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"><path d="M6 12l10-7v14L6 12Z" fill="currentColor"/><rect x="18" y="5" width="2" height="14" rx="1" fill="currentColor"/></svg>"#;
    pub const NEXT: &str = r#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"><rect x="4" y="5" width="2" height="14" rx="1" fill="currentColor"/><path d="M18 12L8 5v14l10-7Z" fill="currentColor"/></svg>"#;
    pub const PLAY: &str = r#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"><path d="M8 5v14l11-7L8 5Z" fill="currentColor"/></svg>"#;
    pub const PAUSE: &str = r#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"><rect x="6" y="4" width="4" height="16" rx="2" fill="currentColor"/><rect x="14" y="4" width="4" height="16" rx="2" fill="currentColor"/></svg>"#;
    pub const HOME: &str = r#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"><path d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m0 0V11a1 1 0 011-1h2a1 1 0 011 1v10m0 0h3a1 1 0 001-1V10" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/></svg>"#;
    pub const SETTINGS: &str = r#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"><path d="M12 15a3 3 0 100-6 3 3 0 000 6z" stroke="currentColor" stroke-width="1.5"/><path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-2 2 2 2 0 01-2-2v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83 0 2 2 0 010-2.83l.06-.06a1.65 1.65 0 00.33-1.82 1.65 1.65 0 00-1.51-1H3a2 2 0 01-2-2 2 2 0 012-2h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 010-2.83 2 2 0 012.83 0l.06.06a1.65 1.65 0 001.82.33H9a1.65 1.65 0 001-1.51V3a2 2 0 012-2 2 2 0 012 2v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 0 2 2 0 010 2.83l-.06.06a1.65 1.65 0 00-.33 1.82V9a1.65 1.65 0 001.51 1H21a2 2 0 012 2 2 2 0 01-2 2h-.09a1.65 1.65 0 00-1.51 1z" stroke="currentColor" stroke-width="1.5"/></svg>"#;
    pub const CONFIRM: &str = r#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"><circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="1.5"/><path d="M8 12.5l2.5 2.5L16 9" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/></svg>"#;
    pub const CANCEL: &str = r#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"><circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="1.5"/><path d="M9 9l6 6M15 9l-6 6" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/></svg>"#;
}

// ============================================================================
// 核心工具函数
// ============================================================================

/// 创建SVG图标
/// 创建SVG图标
pub fn svg_icon(content: &str, size: f32, color: Color) -> Element<'static, Message> {
    svg(svg::Handle::from_memory(content.as_bytes().to_vec()))
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .style(move |theme: &iced::Theme, _status: svg::Status| {
            // 在深色模式下使用更亮的颜色
            let is_dark_theme = {
                let bg = theme.extended_palette().background.base.color;
                bg.r + bg.g + bg.b < 1.5
            };

            svg::Style {
                color: Some(if is_dark_theme && color.a <= 0.9 {
                    // 只对默认图标颜色进行调整，保持自定义颜色不变
                    Color {
                        r: 0.8,
                        g: 0.8,
                        b: 0.8,
                        a: 1.0,
                    }
                } else {
                    color
                }),
            }
        })
        .into()
}

/// 创建简单文本（不带截断）

/// 创建带截断和tooltip的文本
fn truncated_text(
    full_text: String,
    max_len: usize,
    size: u32,
    color: Color,
) -> Element<'static, Message> {
    let display_text = if full_text.chars().count() > max_len {
        format!("{}...", full_text.chars().take(max_len).collect::<String>())
    } else {
        full_text.clone()
    };

    let text_elem = text(display_text)
        .size(size)
        .style(move |_: &iced::Theme| iced::widget::text::Style { color: Some(color) })
        .shaping(Shaping::Advanced);

    if full_text.chars().count() > max_len {
        tooltip(
            text_elem,
            text(full_text).size(constants::TEXT_NORMAL),
            tooltip::Position::Top,
        )
        .style(tooltip_style())
        .padding(constants::PADDING_SMALL as u32)
        .into()
    } else {
        text_elem.into()
    }
}

/// 通用tooltip样式
fn tooltip_style() -> impl Fn(&iced::Theme) -> container::Style {
    |theme: &iced::Theme| {
        let palette = theme.extended_palette();
        container::Style {
            background: Some(Background::Color(palette.background.strong.color)),
            text_color: Some(palette.background.strong.text),
            border: Border {
                radius: Radius::from(6.0),
                width: 1.0,
                color: palette.background.weak.color,
            },
            shadow: Shadow {
                color: AppColors::shadow(theme),
                offset: iced::Vector::new(0.0, 2.0),
                blur_radius: 8.0,
            },
            snap: false,
        }
    }
}

/// 透明文本样式
fn alpha_text_style(alpha: f32) -> impl Fn(&iced::Theme) -> iced::widget::text::Style {
    move |theme: &iced::Theme| {
        let base = AppColors::text_secondary(theme);
        iced::widget::text::Style {
            color: Some(Color {
                r: base.r,
                g: base.g,
                b: base.b,
                a: alpha,
            }),
        }
    }
}

/// 主色文本样式
fn primary_text_style() -> impl Fn(&iced::Theme) -> iced::widget::text::Style {
    |theme: &iced::Theme| {
        let palette = theme.extended_palette();
        iced::widget::text::Style {
            color: Some(palette.primary.strong.color),
        }
    }
}

// ============================================================================
// 通用组件
// ============================================================================

// ============================================================================
// 枚举定义
// ============================================================================

#[derive(Debug, Clone, PartialEq, Default)]
pub enum PageType {
    #[default]
    Home,
    Settings,
    Id3Tag,
    Lyrics,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum PlayMode {
    #[default]
    ListLoop,
    SingleLoop,
    Random,
}

impl PlayMode {
    pub fn icon(&self) -> &'static str {
        match self {
            PlayMode::ListLoop => icons::LIST_LOOP,
            PlayMode::SingleLoop => icons::SINGLE_LOOP,
            PlayMode::Random => icons::RANDOM_PLAY,
        }
    }

    pub fn name(&self) -> String {
        match self {
            PlayMode::ListLoop => t!("List Loop").to_string(),
            PlayMode::SingleLoop => t!("Single Loop").to_string(),
            PlayMode::Random => t!("Random Play").to_string(),
        }
    }

    pub fn description(&self) -> String {
        match self {
            PlayMode::ListLoop => t!("Play all songs in order and repeat").to_string(),
            PlayMode::SingleLoop => t!("Repeat current song").to_string(),
            PlayMode::Random => t!("Play songs in random order").to_string(),
        }
    }

    pub fn next(&self) -> Self {
        match self {
            PlayMode::ListLoop => PlayMode::SingleLoop,
            PlayMode::SingleLoop => PlayMode::Random,
            PlayMode::Random => PlayMode::ListLoop,
        }
    }
}

// ============================================================================
// 主要UI组件
// ============================================================================

/// 导航侧边栏
pub fn navigation_sidebar(current_page: &PageType) -> Element<'static, Message> {
    let nav_button = |icon: &'static str, _label: String, page: PageType, is_active: bool| {
        let (btn_type, _btn_color) = if is_active {
            (
                super::widgets::styled_button::ButtonType::Primary,
                super::widgets::styled_button::ButtonColor::Primary,
            )
        } else {
            (
                super::widgets::styled_button::ButtonType::Default,
                super::widgets::styled_button::ButtonColor::Default,
            )
        };
        icon_button(
            icon,
            Message::PageChanged(page),
            constants::BUTTON_SIZE_MEDIUM,
            btn_type,
        )
    };

    StyledContainer::new(
        column![
            nav_button(
                icons::HOME,
                t!("Home").to_string(),
                PageType::Home,
                *current_page == PageType::Home
            ),
            nav_button(
                icons::SETTINGS,
                t!("Settings").to_string(),
                PageType::Settings,
                *current_page == PageType::Settings
            ),
            Space::new().height(Length::Fill),
            StyledContainer::new(
                column![text("🎵")
                    .size(constants::TEXT_TITLE)
                    .shaping(Shaping::Advanced),]
                .align_x(Horizontal::Center)
                .spacing(4)
            )
            .width(Length::Fill)
            .align_x(Horizontal::Center)
            .padding(constants::PADDING_SMALL)
            .build(),
        ]
        .width(Length::Shrink)
        .height(Length::Fill)
        .spacing(constants::SPACING_MEDIUM)
        .padding(constants::PADDING_SMALL),
    )
    .style(super::widgets::styled_container::ContainerStyle::MainSection)
    .width(Length::Fixed(70.0))
    .height(Length::Fill)
    .build()
}

/// 设置页面
pub fn settings_page(
    current_theme: &AppThemeVariant,
    current_language: &str,
) -> Element<'static, Message> {
    let theme_setting = row![
        StyledText::new(match current_theme {
            AppThemeVariant::Light => "Light",
            AppThemeVariant::Dark => "Dark",
        })
        .size(constants::TEXT_MEDIUM)
        .build(),
        Space::new().width(Length::Fill),
        StyledButton::new(
            StyledText::new(t!("Toggle"))
                .size(constants::TEXT_NORMAL)
                .build()
        )
        .on_press(Message::ToggleTheme)
        .button_type(super::widgets::styled_button::ButtonType::Default)
        .color(super::widgets::styled_button::ButtonColor::Primary)
        .padding(constants::PADDING_SMALL)
        .build()
    ]
    .align_y(Vertical::Center);

    let language_setting = row![
        StyledText::new(match current_language {
            "zh-CN" => t!("Chinese").to_string(),
            _ => "English".to_string(),
        })
        .size(constants::TEXT_MEDIUM)
        .build(),
        Space::new().width(Length::Fill),
        StyledButton::new(
            StyledText::new(t!("Change"))
                .size(constants::TEXT_NORMAL)
                .build()
        )
        .button_type(super::widgets::styled_button::ButtonType::Default)
        .color(super::widgets::styled_button::ButtonColor::Primary)
        .padding(constants::PADDING_SMALL)
        .build()
    ]
    .align_y(Vertical::Center);

    StyledContainer::new(
        column![
            //StyledContainer::new(
            StyledText::new(t!("Settings"))
                .size(constants::TEXT_TITLE + 4)
                .style(super::widgets::styled_text::TextStyle::Emphasis)
                .build(),
            //)
            //.padding(constants::PADDING_MEDIUM)
            //.build(),
            column![
                StyledText::new(t!("Appearance"))
                    .size(constants::TEXT_LARGE)
                    .style(super::widgets::styled_text::TextStyle::Secondary)
                    .build(),
                StyledContainer::new(
                    row![
                        StyledText::new(t!("Theme"))
                            .size(constants::TEXT_MEDIUM)
                            .width(Length::Fixed(150.0))
                            .build(),
                        theme_setting
                    ]
                    .align_y(Vertical::Center)
                    .spacing(constants::SPACING_MEDIUM)
                    .padding(constants::PADDING_SMALL)
                )
                .style(super::widgets::styled_container::ContainerStyle::Card)
                .padding(constants::PADDING_MEDIUM)
                .width(Length::Fill)
                .build()
            ]
            .spacing(constants::SPACING_SMALL),
            column![
                StyledText::new(t!("Language"))
                    .size(constants::TEXT_LARGE)
                    .style(super::widgets::styled_text::TextStyle::Secondary)
                    .build(),
                StyledContainer::new(
                    row![
                        StyledText::new(t!("Interface Language"))
                            .size(constants::TEXT_MEDIUM)
                            .width(Length::Fixed(150.0))
                            .build(),
                        language_setting
                    ]
                    .align_y(Vertical::Center)
                    .spacing(constants::SPACING_MEDIUM)
                    .padding(constants::PADDING_SMALL)
                )
                .style(super::widgets::styled_container::ContainerStyle::Card)
                .padding(constants::PADDING_MEDIUM)
                .width(Length::Fill)
                .build()
            ]
            .spacing(constants::SPACING_SMALL),
            column![
                StyledText::new("Advanced Settings")
                    .size(constants::TEXT_LARGE)
                    .style(super::widgets::styled_text::TextStyle::Secondary)
                    .build(),
                StyledContainer::new(
                    row![
                        StyledText::new("Reset Settings")
                            .size(constants::TEXT_MEDIUM)
                            .width(Length::Fixed(150.0))
                            .build(),
                        StyledButton::new(
                            StyledText::new("Reset to Default")
                                .size(constants::TEXT_NORMAL)
                                .build()
                        )
                        .on_press(Message::ResetConfig)
                        .button_type(super::widgets::styled_button::ButtonType::Default)
                        .color(super::widgets::styled_button::ButtonColor::Primary)
                        .padding(constants::PADDING_SMALL)
                        .build()
                    ]
                    .align_y(Vertical::Center)
                    .spacing(constants::SPACING_MEDIUM),
                )
                .style(super::widgets::styled_container::ContainerStyle::Card)
                .padding(constants::PADDING_MEDIUM)
                .width(Length::Fill)
                .build()
            ]
            .spacing(constants::SPACING_SMALL),
            Space::new().height(Length::Fill),
            StyledContainer::new(
                column![
                    StyledText::new(format!(
                        "{} v{}",
                        t!("Summer Player"),
                        env!("CARGO_PKG_VERSION")
                    ))
                    .size(constants::TEXT_NORMAL)
                    .style(super::widgets::styled_text::TextStyle::Secondary)
                    .build(),
                    StyledText::new(format!("© 2025 {}", "xiamengliang@gmail.com"))
                        .size(constants::TEXT_SMALL)
                        .style(super::widgets::styled_text::TextStyle::Hint)
                        .build(),
                ]
                .align_x(Horizontal::Center)
                .spacing(2)
            )
            .center_x()
            .width(Length::Fill)
            .padding(constants::PADDING_MEDIUM)
            .build()
        ]
        .spacing(constants::SPACING_MEDIUM)
        .padding(constants::PADDING_LARGE),
    )
    .style(super::widgets::styled_container::ContainerStyle::Background)
    .width(Length::Fill)
    .build()
}

/// 控制按钮组
pub fn control_buttons_view(is_playing: bool) -> Element<'static, Message> {
    let (play_icon, _play_tooltip) = if is_playing {
        (icons::PAUSE, t!("Pause").to_string())
    } else {
        (icons::PLAY, t!("Play").to_string())
    };

    row![
        icon_button(
            icons::PREVIOUS,
            Message::PreviousTrack,
            constants::BUTTON_SIZE_SMALL,
            ButtonType::Default
        ),
        icon_button(
            play_icon,
            Message::PlayPause,
            constants::BUTTON_SIZE_MEDIUM,
            super::widgets::styled_button::ButtonType::Primary
        ),
        icon_button(
            icons::NEXT,
            Message::NextTrack,
            constants::BUTTON_SIZE_SMALL,
            ButtonType::Default
        ),
    ]
    .spacing(constants::SPACING_SMALL)
    .align_y(Vertical::Center)
    .into()
}

/// 紧凑按钮组
pub fn compact_play_mode_button(current_mode: PlayMode) -> Element<'static, Message> {
    icon_button(
        current_mode.icon(),
        Message::TogglePlayMode,
        constants::BUTTON_SIZE_SMALL,
        ButtonType::Default,
    )
}

pub fn compact_file_button() -> Element<'static, Message> {
    icon_button(
        icons::FILE_FOLDER,
        Message::OpenFile,
        constants::BUTTON_SIZE_SMALL,
        ButtonType::Default,
    )
}

/// 细进度条视图（用于底部栏）
pub fn thin_progress_view(playback_state: &PlaybackState) -> Element<'static, Message> {
    let progress = if playback_state.total_duration > 0.0 {
        (playback_state.current_time / playback_state.total_duration) as f32
    } else {
        0.0
    };

    column![
        // 只显示进度条，不显示时间文本
        slider(0.0..=1.0, progress, Message::ProgressChanged)
            .step(0.001)
            .style(AppTheme::progress_slider())
            .height(4.0), // 设置更小的高度使进度条更细
    ]
    .width(Length::Fill)
    .into()
}

/// 简单时间显示（用于底部栏右侧）
pub fn simple_time_view(playback_state: &PlaybackState) -> Element<'static, Message> {
    let current_time_str = format_duration(playback_state.current_time);
    let total_time_str = format_duration(playback_state.total_duration);

    // 使用固定宽度的容器来防止播放控制按钮位置变动
    StyledContainer::new(
        row![
            text(current_time_str)
                .size(constants::TEXT_NORMAL)
                .style(AppTheme::current_time_text()),
            text(" / ")
                .size(constants::TEXT_NORMAL)
                .style(|theme: &iced::Theme| {
                    iced::widget::text::Style {
                        color: Some(AppColors::text_primary(theme)),
                    }
                }),
            text(total_time_str)
                .size(constants::TEXT_NORMAL)
                .style(AppTheme::total_time_text()),
        ]
        .spacing(4)
        .align_y(Vertical::Center),
    )
    .style(super::widgets::styled_container::ContainerStyle::Transparent)
    .width(Length::Fixed(110.0)) // 固定宽度防止位置变动 (例如: "00:00 / 99:59")
    .align_x(Horizontal::Center)
    .align_y(Vertical::Center)
    .build()
    .into()
}

/// 播放列表视图
pub fn playlist_view(
    playlist: &Playlist,
    playlist_loaded: bool,
    is_playing: bool,
    playlist_manager: &crate::playlist::PlaylistManager,
    menu_song_index: Option<usize>,
) -> Element<'static, Message> {
    if !playlist_loaded {
        return StyledContainer::new(
            column![
                StyledText::new("📂")
                    .size(48)
                    .align(Horizontal::Center)
                    .shaping(Shaping::Advanced)
                    .build(),
                StyledText::new(t!("No playlist started"))
                    .size(constants::TEXT_LARGE)
                    .align(Horizontal::Center)
                    .style(super::widgets::styled_text::TextStyle::Secondary)
                    .build(),
                StyledText::new(t!(
                    r#"Please click the playlist card you want to open"#.to_string()
                ))
                .size(constants::TEXT_NORMAL)
                .align(Horizontal::Center)
                .style(super::widgets::styled_text::TextStyle::Hint)
                .build(),
            ]
            .spacing(constants::SPACING_MEDIUM)
            .align_x(Horizontal::Center),
        )
        .style(super::widgets::styled_container::ContainerStyle::Card)
        .padding(32)
        .width(Length::Fill)
        .height(Length::Fill)
        .build();
    }

    // 预先获取所有显示信息以避免借用冲突
    let file_paths: Vec<String> = {
        let paths_ref = playlist.file_paths();
        paths_ref.to_vec()
    };
    let display_items: Vec<_> = file_paths
        .iter()
        .enumerate()
        .map(|(index, file_path)| {
            // 使用播放列表中存储的额外信息（避免每个播放列表独立缓存AudioFile）
            let extra = playlist.extra_info_for(file_path);
            let display_name = extra
                .and_then(|e| e.name.clone())
                .unwrap_or_else(|| extract_filename(file_path));
            let duration = extra.and_then(|e| e.duration).or_else(|| {
                // 当播放列表中的额外信息没有时长时，回退到读取缓存中的AudioFile时长（不触发加载）
                playlist_manager.get_cached_audio_duration(file_path)
            });
            (index, display_name, duration, file_path.clone())
        })
        .collect();

    let items: Vec<Element<Message>> = display_items
        .into_iter()
        .map(|(index, display_name, duration, _file_path)| {
            let is_current = playlist.current_index() == Some(index);
            let is_playing_current = is_current && is_playing;

            let icon = if is_current {
                if is_playing_current {
                    "🎵"
                } else {
                    "⏸"
                }
            } else {
                "🎼"
            };

            let text_color = if is_current {
                Color {
                    r: 0.0,
                    g: 0.6,
                    b: 1.0,
                    a: 1.0,
                }
            } else {
                Color {
                    r: 0.4,
                    g: 0.4,
                    b: 0.4,
                    a: 1.0,
                }
            };

            let content = StyledContainer::new(
                row![
                    StyledText::new(icon)
                        .size(constants::TEXT_MEDIUM)
                        .shaping(Shaping::Advanced)
                        .build(),
                    // 歌曲名使用剩余空间
                    StyledContainer::new(truncated_text(
                        display_name,
                        constants::TEXT_TRUNCATE_DEFAULT,
                        constants::TEXT_MEDIUM,
                        text_color
                    ))
                    .style(super::widgets::styled_container::ContainerStyle::Transparent)
                    .width(Length::Fill)
                    .build(),
                    // 时间区域固定宽度并右对齐
                    StyledContainer::new(
                        StyledText::new(
                            duration.map_or("--:--".to_string(), |d| format_duration(d))
                        )
                        .size(constants::TEXT_NORMAL)
                        .align(Horizontal::Right)
                        .style(super::widgets::styled_text::TextStyle::WithAlpha(0.7))
                        .build()
                    )
                    .style(ContainerStyle::Transparent)
                    .width(Length::Fixed(60.0))
                    .align_x(Horizontal::Right)
                    .build(),
                    {
                        let details_btn = StyledButton::new(
                            text(t!("Song Details")).size(constants::TEXT_MEDIUM),
                        )
                        .button_type(ButtonType::Text)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .on_press(Message::SongItemActionDetails(index))
                        .build();

                        let edit_btn =
                            StyledButton::new(text(t!("Edit Tags")).size(constants::TEXT_MEDIUM))
                                .button_type(ButtonType::Text)
                                .width(Length::Fill)
                                .on_press(Message::SongItemActionEditTags(index))
                                .build();

                        let remove_btn =
                            StyledButton::new(text(t!("Remove")).size(constants::TEXT_MEDIUM))
                                .button_type(ButtonType::Text)
                                .width(Length::Fill)
                                .on_press(Message::SongItemActionRemove(index))
                                .build();

                        let options = StyledContainer::new(
                            column![details_btn, edit_btn, remove_btn]
                                .spacing(2)
                                .padding([8, 0]),
                        )
                        .style(ContainerStyle::Background)
                        .width(Length::Fixed(110.0))
                        .height(Length::Fill)
                        .build();
                        // 右侧的更多操作按钮（切换菜单/图标）- 美化版本
                        let drop_down: Element<Message> = iced_aw::DropDown::new(
                            StyledButton::new(
                                text("⋮")
                                    .shaping(Shaping::Advanced)
                                    .size(constants::TEXT_NORMAL),
                            )
                            .button_type(ButtonType::Text)
                            .color(ButtonColor::Default)
                            .height(Length::Fill)
                            .width(Length::Fixed(32.0))
                            .on_press(Message::ExpandSongItemMenu(index)) // 添加点击事件处理
                            .build(),
                            options,
                            menu_song_index == Some(index),
                        )
                        .width(Length::Fill)
                        .on_dismiss(Message::DismissSongItemMenu(index))
                        .alignment(iced_aw::drop_down::Alignment::BottomStart)
                        .into();
                        

                        StyledContainer::new(drop_down)
                            .style(ContainerStyle::Background)
                            .height(Length::Fill)
                            .align_x(Horizontal::Right)
                            .build()
                    }
                ]
                .spacing(constants::SPACING_MEDIUM)
                .padding(iced::Padding::ZERO)
                .height(Length::Fill)
                .align_y(Vertical::Center),
            )
            .style(ContainerStyle::Transparent)
            .padding(iced::Padding::ZERO)
            .width(Length::Fill)
            .height(Length::Fill)
            .build();

            StyledButton::new(content)
                .on_press(Message::PlaylistItemSelected(index))
                .width(Length::Fill)
                .height(Length::Fixed(56.0))
                .button_type(if is_playing_current {
                    super::widgets::styled_button::ButtonType::Default
                } else if is_current {
                    super::widgets::styled_button::ButtonType::Dashed
                } else {
                    super::widgets::styled_button::ButtonType::Text
                })
                .color(if is_playing_current {
                    super::widgets::styled_button::ButtonColor::Primary
                } else if is_current {
                    super::widgets::styled_button::ButtonColor::Primary
                } else {
                    super::widgets::styled_button::ButtonColor::Default
                })
                .build()
        })
        .collect();

    StyledContainer::new(
        column![
            //StyledContainer::new(
            row![
                StyledText::new("📋")
                    .size(constants::TEXT_TITLE)
                    .shaping(Shaping::Advanced)
                    .build(),
                StyledText::new(t!(
                    "messages.CurrentPlaylist",
                    count = format!("{}", playlist.len())
                ))
                .size(constants::TEXT_TITLE - 2)
                .style(super::widgets::styled_text::TextStyle::Primary)
                .build(),
            ]
            .spacing(constants::SPACING_MEDIUM)
            .align_y(Vertical::Center),
            //).padding(constants::PADDING_SMALL).build(),
            scrollable(
                column(items)
                    .spacing(constants::SPACING_SMALL)
                    .padding([constants::PADDING_SMALL, constants::PADDING_SMALL])
            )
            .height(Length::Fill)
            .width(Length::Fill),
        ]
        .spacing(constants::SPACING_LARGE),
    )
    .style(ContainerStyle::Transparent)
    .padding(constants::PADDING_SMALL)
    .width(Length::Fill)
    .height(Length::Fill)
    .build()
}

/// 歌词视图
pub fn lyrics_view(
    file_path: &str,
    is_playing: bool,
    current_time: f64,
    lyrics: Option<crate::lyrics::Lyrics>,
    window_height: f32,
) -> Element<'static, Message> {
    if file_path.is_empty() {
        return StyledContainer::new(
            column![
                text("🎵")
                    .size(48)
                    .align_x(Horizontal::Center)
                    .shaping(Shaping::Advanced),
                text(t!("Lyrics Display"))
                    .size(constants::TEXT_TITLE)
                    .align_x(Horizontal::Center)
                    .style(primary_text_style()),
                text(t!("Please select an audio file"))
                    .size(constants::TEXT_MEDIUM)
                    .align_x(Horizontal::Center)
                    .style(alpha_text_style(0.7)),
            ]
            .spacing(constants::SPACING_MEDIUM)
            .align_x(Horizontal::Center),
        )
        .style(ContainerStyle::Transparent)
        .padding(constants::PADDING_SMALL)
        .width(Length::Fill)
        .height(Length::Fill)
        .build()
        .into();
    }

    let mut elements = Vec::<Element<Message>>::new();

    if let Some(lyrics_data) = lyrics {
        // 标题
        let title = lyrics_data.metadata.title.clone().unwrap_or_else(|| {
            std::path::Path::new(file_path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("未知歌曲")
                .to_string()
        });

        elements.push(
            StyledContainer::new({
                let title_color = Color {
                    r: 0.0,
                    g: 0.6,
                    b: 1.0,
                    a: 1.0,
                };
                truncated_text(
                    title,
                    constants::TEXT_TRUNCATE_LONG,
                    constants::TEXT_TITLE,
                    title_color,
                )
            })
            .width(Length::Fill)
            .align_x(Horizontal::Center)
            .build()
            .into(),
        );

        if let Some(ref artist) = lyrics_data.metadata.artist {
            elements.push(
                StyledContainer::new({
                    let artist_color = Color {
                        r: 0.4,
                        g: 0.4,
                        b: 0.4,
                        a: 0.8,
                    };
                    truncated_text(
                        format!("🎤 {}", artist),
                        35,
                        constants::TEXT_MEDIUM,
                        artist_color,
                    )
                })
                .width(Length::Fill)
                .align_x(Horizontal::Center)
                .build()
                .into(),
            );
        }

        elements.push(text("").into());

        if lyrics_data.has_lyrics() {
            let current_line = lyrics_data.get_current_line_index(current_time);
            let display_lines = calculate_display_lines(lyrics_data.lines.len(), window_height);

            // 简化的歌词显示 - 只显示当前和周围几行
            let start = current_line.unwrap_or(0).saturating_sub(display_lines / 2);
            let end = (start + display_lines).min(lyrics_data.lines.len());

            for (i, line) in lyrics_data.lines[start..end].iter().enumerate() {
                let line_index = start + i;
                let is_current = Some(line_index) == current_line;

                let text_elem = if is_current && is_playing {
                    StyledContainer::new(
                        text(format!(
                            "▶ {}",
                            if line.text.trim().is_empty() {
                                "♪".to_string()
                            } else {
                                line.text.clone()
                            }
                        ))
                        .size(constants::TEXT_TITLE - 2)
                        .align_x(Horizontal::Center)
                        .shaping(Shaping::Advanced)
                        .style(|theme: &iced::Theme| {
                            iced::widget::text::Style {
                                color: Some(AppColors::primary(theme)),
                            }
                        }),
                    )
                    .style(ContainerStyle::Emphasis)
                    .padding(constants::PADDING_SMALL)
                    .width(Length::Fill)
                    .build()
                    .into()
                } else {
                    text(if line.text.trim().is_empty() {
                        "♪".to_string()
                    } else {
                        line.text.clone()
                    })
                    .size(constants::TEXT_MEDIUM)
                    .align_x(Horizontal::Center)
                    .shaping(Shaping::Advanced)
                    .style(alpha_text_style(
                        if current_line.map_or(false, |c| line_index <= c) {
                            0.4
                        } else {
                            0.7
                        },
                    ))
                    .into()
                };

                elements.push(text_elem);
            }
        } else {
            elements.push(
                text("⚠️ 歌词文件已加载，但没有找到歌词内容")
                    .align_x(Horizontal::Center)
                    .shaping(Shaping::Advanced)
                    .style(alpha_text_style(0.7))
                    .into(),
            );
        }
    } else {
        if is_playing {
            elements.extend([
                text("♪ 正在播放中... ♪")
                    .size(constants::TEXT_TITLE - 2)
                    .align_x(Horizontal::Center)
                    .shaping(Shaping::Advanced)
                    .style(primary_text_style())
                    .into(),
                text("").into(),
                text("🎵 暂无歌词文件")
                    .align_x(Horizontal::Center)
                    .shaping(Shaping::Advanced)
                    .style(alpha_text_style(0.7))
                    .into(),
                text("").into(),
                text(format!("⏱️ {}", format_duration(current_time)))
                    .size(constants::TEXT_NORMAL)
                    .align_x(Horizontal::Center)
                    .shaping(Shaping::Advanced)
                    .style(alpha_text_style(0.6))
                    .into(),
            ]);
        } else {
            elements.extend([
                text("♪ 歌词显示 ♪")
                    .size(constants::TEXT_TITLE - 2)
                    .align_x(Horizontal::Center)
                    .shaping(Shaping::Advanced)
                    .style(primary_text_style())
                    .into(),
                text("").into(),
                text("⏸️ 暂停播放中")
                    .align_x(Horizontal::Center)
                    .shaping(Shaping::Advanced)
                    .style(alpha_text_style(0.7))
                    .into(),
            ]);
        }

        // 使用提示
        elements.push(text("").into());
        elements.push(
            StyledContainer::new(
                column![
                    text("💡 使用提示")
                        .size(constants::TEXT_MEDIUM)
                        .shaping(Shaping::Advanced)
                        .style(primary_text_style()),
                    text("📁 将 .lrc 歌词文件放在音频文件同目录下")
                        .size(11)
                        .shaping(Shaping::Advanced),
                    text("📝 歌词文件名需与音频文件名相同")
                        .size(11)
                        .shaping(Shaping::Advanced),
                    text("🕐 支持时间同步的LRC格式歌词")
                        .size(11)
                        .shaping(Shaping::Advanced),
                ]
                .spacing(constants::SPACING_SMALL),
            )
            .style(ContainerStyle::Emphasis)
            .padding(constants::PADDING_MEDIUM)
            .build()
            .into(),
        );
    }

    StyledContainer::new(
        column(elements)
            .spacing(constants::SPACING_LARGE)
            .width(Length::Fill)
            .align_x(Horizontal::Center),
    )
    .style(super::widgets::styled_container::ContainerStyle::Transparent)
    .padding(constants::PADDING_SMALL)
    .width(Length::Fill)
    .height(Length::Fill)
    .build()
    .into()
}

// ============================================================================
// 工具函数
// ============================================================================

/// 计算歌词显示行数
///
/// 根据窗口高度和布局结构动态计算合适的歌词显示行数
fn calculate_display_lines(total_lyrics: usize, window_height: f32) -> usize {
    // 分析当前布局的固定高度占用：
    // 1. 整体外边距：8px (上) + 8px (下) = 16px
    // 2. 主内容区域内边距：16px (上) + 16px (下) = 32px
    // 3. 主内容和底部区域间距：16px
    // 4. 底部区域：
    //    - 控制按钮高度：~54px (BUTTON_SIZE_MEDIUM + 6.0)
    //    - 进度条区域高度：~40px (文本 + slider + spacing + padding)
    //    - 功能按钮高度：~48px (BUTTON_SIZE_MEDIUM)
    //    - 底部区域padding：8px (上) + 8px (下) = 16px
    //    - 底部区域总高度：max(54, 40, 48) + 16 = 70px
    // 5. 歌词视图内部padding：constants::PADDING_LARGE + 4 = 28px (上下各28px = 56px)
    // 6. 歌词标题和艺术家信息占用：~80px (标题 + 艺术家 + spacing)

    let fixed_ui_height = 16.0  // 整体外边距
        + 32.0  // 主内容区域内边距
        + 16.0  // 主内容和底部间距
        + 70.0  // 底部区域
        + 56.0  // 歌词视图内部padding
        + 80.0; // 歌词标题区域

    let available_height = window_height - fixed_ui_height;

    // 歌词行高度：标准文本大小 + 行间距
    // TEXT_TITLE-2 (18px) + SPACING_LARGE (20px) = 38px 用于当前行
    // TEXT_MEDIUM (14px) + SPACING_LARGE (20px) = 34px 用于其他行
    // 平均行高取 35px
    let line_height = 35.0;

    // 计算基础显示行数
    let calculated_lines = (available_height / line_height) as usize;

    // 应用智能调整策略
    let adjusted_lines = if total_lyrics == 0 {
        // 无歌词时，返回默认值
        7
    } else if total_lyrics <= 5 {
        // 歌词很少时，稍微多显示一些行以保持视觉平衡
        9.min(calculated_lines.max(7))
    } else if total_lyrics <= 15 {
        // 中等长度歌词，显示合理的行数
        calculated_lines.max(9).min(15)
    } else {
        // 长歌词，允许更多行显示
        calculated_lines.max(11).min(25)
    };

    // 确保最小行数为5，最大行数为25
    let final_lines = adjusted_lines.max(5).min(25);

    // 确保为奇数以保持当前行居中
    if final_lines % 2 == 0 {
        final_lines + 1
    } else {
        final_lines
    }
}

pub fn spacer() -> Element<'static, Message> {
    Space::new().into()
}

/// 紧凑的专辑封面视图（用于底部栏下层）
pub fn compact_album_cover_view(audio_info: Option<&AudioInfo>) -> Element<'static, Message> {
    let content = if let Some(info) = audio_info {
        if let Some(cover_art) = &info.metadata.cover_art {
            // 显示可点击的专辑封面
            let handle = iced::widget::image::Handle::from_bytes(cover_art.data.clone());
            
            StyledButton::icon_only(handle)
                .button_type(ButtonType::Default)
                .size(56.0)
                .on_press(Message::ToggleHomeLyrics)
                .build()
        } else {
            // 没有封面时显示可点击的音乐图标按钮
            icon_button(icons::MUSIC_NOTE, Message::ToggleHomeLyrics, 56.0,ButtonType::Default)
        }
    } else {
        // 没有音频信息时显示可点击的音乐图标按钮
        icon_button(icons::MUSIC_NOTE, Message::ToggleHomeLyrics,56.0,ButtonType::Default)
    };

    StyledContainer::new(content)
        .style(super::widgets::styled_container::ContainerStyle::Decorative)
        .width(Length::Fixed(56.0))
        .height(Length::Fixed(56.0))
        .align_x(Horizontal::Center)
        .align_y(Vertical::Center)
        .padding(3)
        .build()
        .into()
}

/// 紧凑的歌曲信息视图（用于底部栏下层）
pub fn compact_song_info_view(
    audio_info: Option<&AudioInfo>,
    file_path: &str,
) -> Element<'static, Message> {
    if let Some(info) = audio_info {
        let file_name = std::path::Path::new(file_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown Track")
            .to_string();

        let display_title = info.metadata.title.clone().unwrap_or(file_name);
        let display_artist = info
            .metadata
            .artist
            .clone()
            .unwrap_or("Unknown Artist".to_string());

        StyledContainer::new(
            column![
                // 第一行：歌曲名（使用主题强调色，高对比度）
                text(if display_title.chars().count() > 25 {
                    format!("{}...", display_title.chars().take(22).collect::<String>())
                } else {
                    display_title
                })
                .size(constants::TEXT_MEDIUM)
                .style(|theme: &iced::Theme| {
                    let palette = theme.extended_palette();
                    iced::widget::text::Style {
                        color: Some(palette.primary.strong.color), // 使用主题强调色，高对比度
                    }
                }),
                // 第二行：艺术家（使用主题文本色）
                text(if display_artist.chars().count() > 25 {
                    format!("{}...", display_artist.chars().take(22).collect::<String>())
                } else {
                    display_artist
                })
                .size(constants::TEXT_SMALL)
                .style(|theme: &iced::Theme| {
                    let palette = theme.extended_palette();
                    iced::widget::text::Style {
                        color: Some(Color {
                            a: 0.9,
                            ..palette.background.base.text
                        }), // 主题文本色，稍微透明
                    }
                }),
            ]
            .spacing(3),
        )
        .style(super::widgets::styled_container::ContainerStyle::Transparent)
        .width(Length::Fixed(180.0))
        .align_y(Vertical::Center)
        .padding([4, 8])
        .build()
        .into()
    } else {
        column![
            text("No Track")
                .size(constants::TEXT_MEDIUM)
                .style(alpha_text_style(0.8)),
            text("Select a file to play")
                .size(constants::TEXT_SMALL)
                .style(alpha_text_style(0.6))
        ]
        .spacing(2)
        .into()
    }
}

/// 播放列表文件展示控件（网格布局）
/// 显示配置目录下的所有m3u播放列表文件
pub fn playlist_files_grid_view(
    playlist_manager: &crate::playlist::PlaylistManager,
    creating: bool,
    creating_name: &str,
    menu_playlist_path: Option<&str>,
    renaming_playlist_path: Option<&str>,
    renaming_playlist_name: &str,
) -> Element<'static, Message> {
    // 从PlaylistManager获取播放列表文件信息
    let playlist_infos = get_playlist_files_info_from_manager(playlist_manager);

    // 创建网格布局，每行显示3个卡片（包括创建卡片）
    let mut grid_rows = Vec::<Element<Message>>::new();
    let mut current_row = Vec::<Element<Message>>::new();

    let current_path = playlist_manager.current_playlist_path();
    for playlist_info in playlist_infos.iter() {
        // 检查当前卡片是否被选中（由 PlaylistManager 提供）
        let is_selected = current_path == Some(playlist_info.path.as_str());

        // 使用新的PlaylistCard控件
        let playlist_card = PlaylistCard::builder()
            .path(playlist_info.path.clone())
            .name(playlist_info.name.clone())
            .song_count(playlist_info.song_count)
            .selected(is_selected)
            .show_menu(menu_playlist_path == Some(playlist_info.path.as_str()))
            .renaming(renaming_playlist_path == Some(playlist_info.path.as_str()))
            .renaming_name(renaming_playlist_name)
            .width(170.0)
            .height(240.0)
            .build();

        current_row.push(playlist_card);

        // 满3个则落一行
        if current_row.len() == 3 {
            // 填充不足3个的行
            while current_row.len() < 3 {
                current_row.push(Space::new().into());
            }

            let grid_row = row(current_row.drain(..).collect::<Vec<_>>())
                .spacing(constants::SPACING_LARGE)
                .align_y(Vertical::Center)
                .into();

            grid_rows.push(grid_row);
        }
    }

    // 追加“创建播放列表”卡片，计入布局
    current_row.push(if !creating {
        CreatePlaylistCard::display_card()
    } else {
        CreatePlaylistCard::input_card(creating_name)
    });

    while current_row.len() < 3 {
        current_row.push(Space::new().into());
    }

    let grid_row = row(current_row.drain(..).collect::<Vec<_>>())
        .spacing(constants::SPACING_LARGE)
        .align_y(Vertical::Center)
        .into();

    grid_rows.push(grid_row);

    StyledContainer::new(
        column![
            // 标题
            //StyledContainer::new(
            row![
                text("📋")
                    .size(constants::TEXT_TITLE)
                    .shaping(Shaping::Advanced),
                text(t!("Playlists"))
                    .size(constants::TEXT_TITLE - 2)
                    .style(primary_text_style()),
            ]
            .spacing(constants::SPACING_MEDIUM)
            .align_y(Vertical::Center),
            //).padding(constants::PADDING_SMALL).build(),

            // 网格布局的播放列表（自适应高度，滚动条）
            scrollable(
                column(grid_rows)
                    .spacing(constants::SPACING_MEDIUM)
                    .padding([constants::PADDING_SMALL, constants::PADDING_SMALL])
            )
            .height(Length::Fill)
            .width(Length::Fill), // 高度填满可用空间，超出时自动滚动
        ]
        .spacing(constants::SPACING_LARGE)
        .height(Length::Fill), // 确保列也填满高度
    )
    .style(super::widgets::styled_container::ContainerStyle::Transparent)
    .padding(constants::PADDING_SMALL)
    .width(Length::Fill)
    .height(Length::Fill) // 容器填满可用空间
    .build()
    .into()
}

/// 播放列表文件信息
#[derive(Clone)]
struct PlaylistFileInfo {
    path: String,
    name: String,
    song_count: usize,
}

/// 从PlaylistManager获取播放列表文件信息（只包含持久播放列表，不包含临时播放列表）
fn get_playlist_files_info_from_manager(
    playlist_manager: &crate::playlist::PlaylistManager,
) -> Vec<PlaylistFileInfo> {
    let mut playlist_infos = Vec::new();

    // 遍历PlaylistManager中的持久播放列表
    for (playlist_path, playlist) in playlist_manager.get_persistent_playlists_with_paths() {
        // 只包含持久播放列表（不包含临时播放列表）
        if !playlist.is_temporary() {
            let name = playlist.name().unwrap_or("Unknown Playlist").to_string();

            playlist_infos.push(PlaylistFileInfo {
                path: playlist_path.to_string(),
                name,
                song_count: playlist.len(),
            });
        }
    }

    // 按文件名排序
    playlist_infos.sort_by(|a, b| a.name.cmp(&b.name));
    playlist_infos
}
