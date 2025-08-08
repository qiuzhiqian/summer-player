//! UI组件模块
//! 
//! 包含可重用的UI组件和通用样式。

use iced::{
    widget::{button, column, row, text, slider, scrollable, Space, container, tooltip, svg, image},
    Element, Length, Border, Shadow, Background, Color,
    alignment::{Horizontal, Vertical},
    border::Radius,
};
use iced::advanced::text::Shaping;

use crate::audio::{AudioInfo, PlaybackState};
use crate::playlist::Playlist;
use crate::utils::format_duration;

use super::Message;
use super::theme::{AppTheme, AppThemeVariant};
use rust_i18n::t;

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
    pub const SPACING_SMALL: u16 = 6;
    pub const SPACING_MEDIUM: u16 = 12;
    pub const SPACING_LARGE: u16 = 20;
    
    pub const PADDING_SMALL: u16 = 8;
    pub const PADDING_MEDIUM: u16 = 16;
    pub const PADDING_LARGE: u16 = 24;
    
    // 文本大小
    pub const TEXT_SMALL: u16 = 10;
    pub const TEXT_NORMAL: u16 = 12;
    pub const TEXT_MEDIUM: u16 = 14;
    pub const TEXT_LARGE: u16 = 16;
    pub const TEXT_TITLE: u16 = 20;
    
    // 截断长度
    pub const TEXT_TRUNCATE_DEFAULT: usize = 30;
    pub const TEXT_TRUNCATE_LONG: usize = 40;
    
    // 颜色
    pub const ICON_COLOR: Color = Color { r: 0.4, g: 0.4, b: 0.4, a: 0.9 };
    pub const ICON_COLOR_SUBTLE: Color = Color { r: 0.4, g: 0.4, b: 0.4, a: 0.8 };
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
    pub const PREVIOUS: &str = r#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"><path d="M6 12l10-7v14L6 12Z" fill="currentColor"/><rect x="18" y="5" width="2" height="14" rx="1" fill="currentColor"/></svg>"#;
    pub const NEXT: &str = r#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"><rect x="4" y="5" width="2" height="14" rx="1" fill="currentColor"/><path d="M18 12L8 5v14l10-7Z" fill="currentColor"/></svg>"#;
    pub const PLAY: &str = r#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"><path d="M8 5v14l11-7L8 5Z" fill="currentColor"/></svg>"#;
    pub const PAUSE: &str = r#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"><rect x="6" y="4" width="4" height="16" rx="2" fill="currentColor"/><rect x="14" y="4" width="4" height="16" rx="2" fill="currentColor"/></svg>"#;
    pub const HOME: &str = r#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"><path d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m0 0V11a1 1 0 011-1h2a1 1 0 011 1v10m0 0h3a1 1 0 001-1V10" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/></svg>"#;
    pub const SETTINGS: &str = r#"<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"><path d="M12 15a3 3 0 100-6 3 3 0 000 6z" stroke="currentColor" stroke-width="1.5"/><path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-2 2 2 2 0 01-2-2v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83 0 2 2 0 010-2.83l.06-.06a1.65 1.65 0 00.33-1.82 1.65 1.65 0 00-1.51-1H3a2 2 0 01-2-2 2 2 0 012-2h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 010-2.83 2 2 0 012.83 0l.06.06a1.65 1.65 0 001.82.33H9a1.65 1.65 0 001-1.51V3a2 2 0 012-2 2 2 0 012 2v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 0 2 2 0 010 2.83l-.06.06a1.65 1.65 0 00-.33 1.82V9a1.65 1.65 0 001.51 1H21a2 2 0 012 2 2 2 0 01-2 2h-.09a1.65 1.65 0 00-1.51 1z" stroke="currentColor" stroke-width="1.5"/></svg>"#;
}

// ============================================================================
// 核心工具函数
// ============================================================================

/// 创建SVG图标
fn svg_icon(content: &str, size: f32, color: Color) -> Element<'static, Message> {
    svg(svg::Handle::from_memory(content.as_bytes().to_vec()))
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .style(move |_, _| svg::Style { color: Some(color) })
        .into()
}

/// 创建简单文本（不带截断）

/// 创建带截断和tooltip的文本
fn truncated_text(
    full_text: String, 
    max_len: usize, 
    size: u16,
    color: Color
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
        tooltip(text_elem, text(full_text).size(constants::TEXT_NORMAL), tooltip::Position::Top)
            .style(tooltip_style()).padding(constants::PADDING_SMALL).into()
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
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.2),
                offset: iced::Vector::new(0.0, 2.0),
                blur_radius: 8.0,
            },
        }
    }
}

/// 透明文本样式
fn alpha_text_style(alpha: f32) -> impl Fn(&iced::Theme) -> iced::widget::text::Style {
    move |theme: &iced::Theme| {
        let palette = theme.extended_palette();
        iced::widget::text::Style {
            color: Some(Color { a: alpha, ..palette.background.base.text }),
        }
    }
}

/// 主色文本样式
fn primary_text_style() -> impl Fn(&iced::Theme) -> iced::widget::text::Style {
    |theme: &iced::Theme| {
        let palette = theme.extended_palette();
        iced::widget::text::Style {
            color: Some(palette.primary.base.color),
        }
    }
}

// ============================================================================
// 通用组件
// ============================================================================

/// 创建带tooltip的按钮
fn icon_button(
    icon: &'static str, 
    tooltip_text: String, 
    message: Message, 
    size: f32, 
    icon_size: f32,
    style_fn: fn() -> fn(&iced::Theme, iced::widget::button::Status) -> iced::widget::button::Style
) -> Element<'static, Message> {
    tooltip(
        button(
            container(svg_icon(icon, icon_size, constants::ICON_COLOR))
                .width(Length::Fill).height(Length::Fill)
                .align_x(Horizontal::Center).align_y(Vertical::Center)
        )
        .style(style_fn())
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .on_press(message),
        text(tooltip_text).size(constants::TEXT_NORMAL),
        tooltip::Position::Top
    )
    .style(tooltip_style())
    .padding(constants::PADDING_SMALL)
    .into()
}

// ============================================================================
// 枚举定义
// ============================================================================

#[derive(Debug, Clone, PartialEq, Default)]
pub enum PageType { #[default] Home, Settings }

#[derive(Debug, Clone, PartialEq, Default)]
pub enum ViewType { #[default] Playlist, Lyrics }

#[derive(Debug, Clone, PartialEq, Default)]
pub enum PlayMode { #[default] ListLoop, SingleLoop, Random }

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
    let nav_button = |icon: &'static str, label: String, page: PageType, is_active: bool| {
        let style_fn = if is_active { AppTheme::control_button } else { AppTheme::file_button };
        icon_button(icon, label, Message::PageChanged(page), constants::BUTTON_SIZE_SMALL, constants::ICON_SIZE_SMALL, style_fn)
    };

    column![
        nav_button(icons::HOME, t!("Home").to_string(), PageType::Home, *current_page == PageType::Home),
        nav_button(icons::SETTINGS, t!("Settings").to_string(), PageType::Settings, *current_page == PageType::Settings),
        Space::with_height(Length::Fill),
        container(
            column![
                text("🎵").size(constants::TEXT_TITLE).shaping(Shaping::Advanced),
                text("Summer").size(constants::TEXT_SMALL).style(alpha_text_style(0.7)),
            ].align_x(Horizontal::Center).spacing(4)
        ).width(Length::Fill).align_x(Horizontal::Center).padding(constants::PADDING_SMALL),
    ]
    .width(Length::Shrink).height(Length::Fill)
    .spacing(constants::SPACING_MEDIUM).padding(constants::PADDING_MEDIUM)
    .into()
}

/// 设置页面
pub fn settings_page(current_theme: &AppThemeVariant, current_language: &str) -> Element<'static, Message> {
    let theme_setting = row![
        text(match current_theme {
            AppThemeVariant::Light => "Light",
            AppThemeVariant::Dark => "Dark",
        }).size(constants::TEXT_MEDIUM),
        Space::with_width(Length::Fill),
        button(text(t!("Toggle")))
            .on_press(Message::ToggleTheme)
            .style(AppTheme::file_button())
            .padding(constants::PADDING_SMALL)
    ].align_y(Vertical::Center);

    let language_setting = row![
        text(match current_language {
            "zh-CN" => t!("Chinese").to_string(),
            _ => "English".to_string(),
        }).size(constants::TEXT_MEDIUM),
        Space::with_width(Length::Fill),
        button(text(t!("Change")))
            .style(AppTheme::file_button())
            .padding(constants::PADDING_SMALL)
    ].align_y(Vertical::Center);

    column![
        container(text(t!("Settings")).size(constants::TEXT_TITLE + 4).style(AppTheme::emphasis_text()))
            .padding(constants::PADDING_MEDIUM),
        
        column![
            text(t!("Appearance")).size(constants::TEXT_LARGE).style(AppTheme::subtitle_text()),
            container(
                row![
                    text(t!("Theme")).size(constants::TEXT_MEDIUM).width(Length::Fixed(150.0)),
                    theme_setting
                ].align_y(Vertical::Center).spacing(constants::SPACING_MEDIUM).padding(constants::PADDING_SMALL)
            ).style(AppTheme::card_container()).padding(constants::PADDING_MEDIUM).width(Length::Fill)
        ].spacing(constants::SPACING_SMALL),
        
        column![
            text(t!("Language")).size(constants::TEXT_LARGE).style(AppTheme::subtitle_text()),
            container(
                row![
                    text(t!("Interface Language")).size(constants::TEXT_MEDIUM).width(Length::Fixed(150.0)),
                    language_setting
                ].align_y(Vertical::Center).spacing(constants::SPACING_MEDIUM).padding(constants::PADDING_SMALL)
            ).style(AppTheme::card_container()).padding(constants::PADDING_MEDIUM).width(Length::Fill)
        ].spacing(constants::SPACING_SMALL),
        
        Space::with_height(Length::Fill),
        container(
            column![
                text(format!("{} v{}", t!("Summer Player"), env!("CARGO_PKG_VERSION")))
                    .size(constants::TEXT_NORMAL).style(AppTheme::subtitle_text()),
                text(format!("© 2025 {}", t!("xml")))
                    .size(constants::TEXT_SMALL).style(AppTheme::hint_text()),
            ].align_x(Horizontal::Center).spacing(2)
        ).center_x(Length::Fill).padding(constants::PADDING_MEDIUM)
    ]
    .spacing(constants::SPACING_MEDIUM).padding(constants::PADDING_LARGE)
    .into()
}

/// 文件信息视图
pub fn file_info_view(audio_info: Option<&AudioInfo>, file_path: &str) -> Element<'static, Message> {
    let content: Element<'static, Message> = if let Some(info) = audio_info {
        let file_name = std::path::Path::new(file_path)
            .file_stem().and_then(|s| s.to_str())
            .unwrap_or("unknown file").to_string();
        
        let display_title = info.metadata.title.clone().unwrap_or(file_name);
        
        let mut main_col = column![
            container(
                {
                    let palette_color = Color { r: 0.0, g: 0.6, b: 1.0, a: 1.0 };
                    truncated_text(display_title, constants::TEXT_TRUNCATE_LONG, constants::TEXT_LARGE, palette_color)
                }
            ).width(Length::Fill),
        ].spacing(constants::SPACING_MEDIUM);

        // 添加封面图片
        if let Some(cover_art) = &info.metadata.cover_art {
            main_col = main_col.push(
                container(
                    image(image::Handle::from_bytes(cover_art.data.clone()))
                        .width(Length::Shrink)
                        .height(Length::Shrink)
                )
                .style(|theme: &iced::Theme| {
                    let palette = theme.extended_palette();
                    container::Style {
                        background: Some(Background::Color(palette.background.weak.color)),
                        border: Border {
                            radius: Radius::from(8.0),
                            width: 1.0,
                            color: palette.background.strong.color,
                        },
                        shadow: Shadow::default(),
                        text_color: None,
                    }
                })
                .padding(4)
                .width(Length::Shrink)
                .align_x(Horizontal::Center)
            );
        }

        // 音频信息
        main_col = main_col.push(
            column![
                text(t!("Audio Info")).size(constants::TEXT_MEDIUM).style(alpha_text_style(0.8)),
                row![text("🎵").size(constants::TEXT_MEDIUM).shaping(Shaping::Advanced), text(format!("频道: {}", info.channels)).size(constants::TEXT_NORMAL).style(alpha_text_style(0.8))].spacing(constants::SPACING_SMALL),
                row![text("📡").size(constants::TEXT_MEDIUM).shaping(Shaping::Advanced), text(format!("采样率: {} Hz", info.sample_rate)).size(constants::TEXT_NORMAL).style(alpha_text_style(0.8))].spacing(constants::SPACING_SMALL),
                row![text("⏱️").size(constants::TEXT_MEDIUM).shaping(Shaping::Advanced), text(format!("时长: {}", info.duration.map_or("未知".to_string(), |d| format_duration(d)))).size(constants::TEXT_NORMAL).style(alpha_text_style(0.8))].spacing(constants::SPACING_SMALL),
            ].spacing(constants::SPACING_SMALL)
        );

        // 元数据
        if info.metadata.title.is_some() || info.metadata.artist.is_some() || info.metadata.album.is_some() {
            let mut metadata_col = column![
                text(t!("Metadata")).size(constants::TEXT_MEDIUM).style(alpha_text_style(0.8)),
            ].spacing(constants::SPACING_SMALL);
            
            if let Some(ref title) = info.metadata.title {
                metadata_col = metadata_col.push(
                    row![text("🎤").size(constants::TEXT_MEDIUM).shaping(Shaping::Advanced), 
                        {
                            let text_color = Color { r: 0.4, g: 0.4, b: 0.4, a: 0.8 };
                            truncated_text(format!("标题: {}", title), 25, constants::TEXT_NORMAL, text_color)
                        }
                    ].spacing(constants::SPACING_SMALL)
                );
            }
            
            if let Some(ref artist) = info.metadata.artist {
                metadata_col = metadata_col.push(
                    row![text("🎨").size(constants::TEXT_MEDIUM).shaping(Shaping::Advanced), 
                        {
                            let text_color = Color { r: 0.4, g: 0.4, b: 0.4, a: 0.8 };
                            truncated_text(format!("艺术家: {}", artist), 25, constants::TEXT_NORMAL, text_color)
                        }
                    ].spacing(constants::SPACING_SMALL)
                );
            }
            
            main_col = main_col.push(metadata_col);
        }
        
        main_col.into()
    } else {
        column![
            text("🎼").size(32).align_x(Horizontal::Center).shaping(Shaping::Advanced),
            text(t!("File not selected")).size(constants::TEXT_MEDIUM).align_x(Horizontal::Center).style(alpha_text_style(0.7)),
        ].spacing(constants::SPACING_SMALL).align_x(Horizontal::Center).into()
    };

    container(content)
        .style(AppTheme::main_section_container())
        .padding(constants::PADDING_LARGE)
        .width(Length::Fill)
        .into()
}

/// 控制按钮组
pub fn control_buttons_view(is_playing: bool) -> Element<'static, Message> {
    let (play_icon, play_tooltip) = if is_playing {
        (icons::PAUSE, t!("Pause").to_string())
    } else {
        (icons::PLAY, t!("Play").to_string())
    };

    container(
        row![
            icon_button(icons::PREVIOUS, t!("Previous Track").to_string(), Message::PreviousTrack, constants::BUTTON_SIZE_MEDIUM, constants::ICON_SIZE_LARGE, AppTheme::control_button),
            icon_button(play_icon, play_tooltip, Message::PlayPause, constants::BUTTON_SIZE_LARGE, constants::ICON_SIZE_XLARGE, AppTheme::play_button),
            icon_button(icons::NEXT, t!("Next Track").to_string(), Message::NextTrack, constants::BUTTON_SIZE_MEDIUM, constants::ICON_SIZE_LARGE, AppTheme::control_button),
        ].spacing(constants::SPACING_MEDIUM).align_y(Vertical::Center)
    )
    .style(AppTheme::main_section_container())
    .padding(constants::PADDING_MEDIUM)
    .width(Length::Fill)
    .align_x(Horizontal::Center)
    .into()
}

/// 紧凑按钮组
pub fn compact_play_mode_button(current_mode: PlayMode) -> Element<'static, Message> {
    icon_button(current_mode.icon(), current_mode.name(), Message::TogglePlayMode, constants::BUTTON_SIZE_MEDIUM, constants::ICON_SIZE_MEDIUM, AppTheme::file_button)
}

pub fn compact_file_button() -> Element<'static, Message> {
    icon_button(icons::FILE_FOLDER, t!("Open File").to_string(), Message::OpenFile, constants::BUTTON_SIZE_MEDIUM, constants::ICON_SIZE_MEDIUM, AppTheme::file_button)
}

pub fn compact_view_toggle_button(current_view: ViewType) -> Element<'static, Message> {
    let (icon, tooltip) = match current_view {
        ViewType::Playlist => (icons::MUSIC_NOTE, t!("Switch to Lyrics").to_string()),
        ViewType::Lyrics => (icons::LIST_VIEW, t!("Switch to Playlist").to_string()),
    };
    icon_button(icon, tooltip, Message::ToggleView, constants::BUTTON_SIZE_MEDIUM, constants::ICON_SIZE_MEDIUM, AppTheme::file_button)
}

/// 进度条视图
pub fn progress_view(playback_state: &PlaybackState) -> Element<'static, Message> {
    let progress = if playback_state.total_duration > 0.0 {
        (playback_state.current_time / playback_state.total_duration) as f32
    } else { 0.0 };
    
    container(
        column![
            row![
                text(format_duration(playback_state.current_time)).size(constants::TEXT_MEDIUM).style(AppTheme::current_time_text()),
                Space::new(Length::Fill, Length::Shrink),
                text(format_duration(playback_state.total_duration)).size(constants::TEXT_MEDIUM).style(AppTheme::total_time_text()),
            ].padding(4),
            container(
                slider(0.0..=1.0, progress, Message::ProgressChanged)
                    .step(0.001).style(AppTheme::progress_slider())
            ).padding(4),
        ].spacing(constants::SPACING_MEDIUM)
    )
    .style(AppTheme::glass_card_container())
    .padding(constants::PADDING_LARGE - 4)
    .width(Length::Fill)
    .into()
}

/// 播放列表视图
pub fn playlist_view(playlist: &Playlist, playlist_loaded: bool, is_playing: bool) -> Element<'static, Message> {
    if !playlist_loaded {
        return container(
            column![
                text("📂").size(48).align_x(Horizontal::Center).shaping(Shaping::Advanced),
                text(t!("No playlist loaded")).size(constants::TEXT_LARGE).align_x(Horizontal::Center).style(AppTheme::subtitle_text()),
                text(t!(r#"Click "Open File" to start"#.to_string())).size(constants::TEXT_NORMAL).align_x(Horizontal::Center).style(AppTheme::hint_text()),
            ].spacing(constants::SPACING_MEDIUM).align_x(Horizontal::Center)
        ).style(AppTheme::card_container()).padding(32).width(Length::Fill).height(Length::Fill).into();
    }

    let items: Vec<Element<Message>> = playlist.items().iter().enumerate().map(|(index, item)| {
        let is_current = playlist.current_index() == Some(index);
        let is_playing_current = is_current && is_playing;
        
        let icon = if is_current {
            if is_playing_current { "🎵" } else { "⏸" }
        } else { "🎼" };
        
        let content = container(
            row![
                text(icon).size(constants::TEXT_MEDIUM).shaping(Shaping::Advanced),
                container(
                    {
                        let text_color = if is_current { 
                            Color { r: 0.0, g: 0.6, b: 1.0, a: 1.0 }
                        } else { 
                            Color { r: 0.4, g: 0.4, b: 0.4, a: 1.0 }
                        };
                        truncated_text(item.name.clone(), constants::TEXT_TRUNCATE_DEFAULT, constants::TEXT_MEDIUM, text_color)
                    }
                ).width(Length::FillPortion(4)),
                text(item.duration.map_or("--:--".to_string(), |d| format_duration(d)))
                    .width(Length::FillPortion(1))
                    .size(constants::TEXT_NORMAL)
                    .align_x(Horizontal::Right)
                    .style(alpha_text_style(0.7)),
            ].spacing(constants::SPACING_MEDIUM).align_y(Vertical::Center)
        ).padding([constants::PADDING_SMALL, constants::PADDING_MEDIUM]).width(Length::Fill);
        
        button(content)
            .on_press(Message::PlaylistItemSelected(index))
            .width(Length::Fill)
            .style(AppTheme::playlist_item_button(is_playing_current, is_current))
            .into()
    }).collect();
    
    container(
        column![
            container(
                row![
                    text("📋").size(constants::TEXT_TITLE).shaping(Shaping::Advanced),
                    text(t!("messages.Playlist", count = format!("{}", playlist.len())))
                        .size(constants::TEXT_TITLE - 2).style(primary_text_style()),
                ].spacing(constants::SPACING_MEDIUM).align_y(Vertical::Center)
            ).padding(constants::PADDING_SMALL),
            scrollable(
                column(items).spacing(constants::SPACING_SMALL).padding([constants::SPACING_MEDIUM, constants::SPACING_SMALL])
            ).height(Length::Fill).width(Length::Fill),
        ].spacing(constants::SPACING_LARGE)
    )
    .style(AppTheme::main_section_container())
    .padding(constants::SPACING_LARGE)
    .width(Length::Fill).height(Length::Fill)
    .into()
}

/// 歌词视图
pub fn lyrics_view(file_path: &str, is_playing: bool, current_time: f64, lyrics: Option<crate::lyrics::Lyrics>, window_height: f32) -> Element<'static, Message> {
    if file_path.is_empty() {
        return container(
            column![
                text("🎵").size(48).align_x(Horizontal::Center).shaping(Shaping::Advanced),
                text(t!("Lyrics Display")).size(constants::TEXT_TITLE).align_x(Horizontal::Center).style(primary_text_style()),
                text(t!("Please select an audio file")).size(constants::TEXT_MEDIUM).align_x(Horizontal::Center).style(alpha_text_style(0.7)),
            ].spacing(constants::SPACING_MEDIUM).align_x(Horizontal::Center)
        ).style(AppTheme::card_container()).padding(32).width(Length::Fill).height(Length::Fill).into();
    }
    
    let mut elements = Vec::<Element<Message>>::new();
    
    if let Some(lyrics_data) = lyrics {
        // 标题
        let title = lyrics_data.metadata.title.clone().unwrap_or_else(|| {
            std::path::Path::new(file_path).file_stem()
                .and_then(|s| s.to_str()).unwrap_or("未知歌曲").to_string()
        });
        
        elements.push(
            container(
                {
                    let title_color = Color { r: 0.0, g: 0.6, b: 1.0, a: 1.0 };
                    truncated_text(title, constants::TEXT_TRUNCATE_LONG, constants::TEXT_TITLE, title_color)
                }
            ).width(Length::Fill).align_x(Horizontal::Center).into()
        );
        
        if let Some(ref artist) = lyrics_data.metadata.artist {
            elements.push(
                container(
                    {
                        let artist_color = Color { r: 0.4, g: 0.4, b: 0.4, a: 0.8 };
                        truncated_text(format!("🎤 {}", artist), 35, constants::TEXT_MEDIUM, artist_color)
                    }
                ).width(Length::Fill).align_x(Horizontal::Center).into()
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
                    container(
                        text(format!("▶ {}", if line.text.trim().is_empty() { "♪".to_string() } else { line.text.clone() }))
                            .size(constants::TEXT_TITLE - 2).align_x(Horizontal::Center).shaping(Shaping::Advanced)
                            .style(|theme: &iced::Theme| {
                                let palette = theme.extended_palette();
                                iced::widget::text::Style { color: Some(palette.primary.strong.color) }
                            })
                    ).style(|theme: &iced::Theme| {
                        let palette = theme.extended_palette();
                        container::Style {
                            background: Some(Background::Color(Color { a: 0.1, ..palette.primary.base.color })),
                            border: Border { radius: Radius::from(8.0), width: 0.0, color: Color::TRANSPARENT },
                            shadow: Shadow::default(), text_color: None,
                        }
                    }).padding([constants::PADDING_SMALL, constants::PADDING_MEDIUM]).width(Length::Fill).into()
                } else {
                    text(if line.text.trim().is_empty() { "♪".to_string() } else { line.text.clone() })
                        .size(constants::TEXT_MEDIUM).align_x(Horizontal::Center).shaping(Shaping::Advanced)
                        .style(alpha_text_style(if current_line.map_or(false, |c| line_index <= c) { 0.4 } else { 0.7 }))
                        .into()
                };
                
                elements.push(text_elem);
            }
        } else {
            elements.push(
                text("⚠️ 歌词文件已加载，但没有找到歌词内容")
                    .align_x(Horizontal::Center).shaping(Shaping::Advanced)
                    .style(alpha_text_style(0.7)).into()
            );
        }
    } else {
        if is_playing {
            elements.extend([
                text("♪ 正在播放中... ♪").size(constants::TEXT_TITLE - 2).align_x(Horizontal::Center).shaping(Shaping::Advanced).style(primary_text_style()).into(),
                text("").into(),
                text("🎵 暂无歌词文件").align_x(Horizontal::Center).shaping(Shaping::Advanced).style(alpha_text_style(0.7)).into(),
                text("").into(),
                text(format!("⏱️ {}", format_duration(current_time))).size(constants::TEXT_NORMAL).align_x(Horizontal::Center).shaping(Shaping::Advanced).style(alpha_text_style(0.6)).into(),
            ]);
        } else {
            elements.extend([
                text("♪ 歌词显示 ♪").size(constants::TEXT_TITLE - 2).align_x(Horizontal::Center).shaping(Shaping::Advanced).style(primary_text_style()).into(),
                text("").into(),
                text("⏸️ 暂停播放中").align_x(Horizontal::Center).shaping(Shaping::Advanced).style(alpha_text_style(0.7)).into(),
            ]);
        }
        
        // 使用提示
        elements.push(text("").into());
        elements.push(
            container(
                column![
                    text("💡 使用提示").size(constants::TEXT_MEDIUM).shaping(Shaping::Advanced).style(primary_text_style()),
                    text("📁 将 .lrc 歌词文件放在音频文件同目录下").size(11).shaping(Shaping::Advanced),
                    text("📝 歌词文件名需与音频文件名相同").size(11).shaping(Shaping::Advanced),
                    text("🕐 支持时间同步的LRC格式歌词").size(11).shaping(Shaping::Advanced),
                ].spacing(constants::SPACING_SMALL)
            ).style(|theme: &iced::Theme| {
                let palette = theme.extended_palette();
                container::Style {
                    background: Some(Background::Color(Color { a: 0.05, ..palette.primary.base.color })),
                    border: Border { radius: Radius::from(8.0), width: 1.0, color: Color { a: 0.2, ..palette.primary.base.color } },
                    shadow: Shadow::default(),
                    text_color: Some(Color { a: 0.8, ..palette.background.base.text }),
                }
            }).padding(constants::PADDING_MEDIUM).into()
        );
    }
    
    container(
        column(elements).spacing(constants::SPACING_LARGE).width(Length::Fill).align_x(Horizontal::Center)
    )
    .style(AppTheme::main_section_container())
    .padding(constants::PADDING_LARGE + 4)
    .width(Length::Fill).height(Length::Fill)
    .into()
}

// ============================================================================
// 工具函数
// ============================================================================

fn calculate_display_lines(total_lyrics: usize, window_height: f32) -> usize {
    let available_height = window_height - 350.0; // 减去其他UI元素高度
    let line_height = 28.0;
    let calculated = (available_height / line_height) as usize;
    
    let lines = if total_lyrics <= 7 { 9 } else { calculated.min(total_lyrics + 4) };
    let final_lines = lines.max(5).min(21);
    if final_lines % 2 == 0 { final_lines + 1 } else { final_lines }
}

pub fn spacer() -> Element<'static, Message> {
    Space::new(Length::Fill, Length::Fill).into()
}

 