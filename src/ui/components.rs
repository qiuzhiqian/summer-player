//! UI组件模块
//! 
//! 包含可重用的UI组件。

use iced::{
    widget::{button, column, row, text, slider, scrollable, Space, container, image, tooltip, svg},
    Element, Length, Border, Shadow, Background, Color,
    alignment::{Horizontal, Vertical, Alignment},
    border::Radius,
};
use iced::advanced::text::Shaping;

use crate::audio::{AudioInfo, PlaybackState};
use crate::playlist::Playlist;
use crate::utils::format_duration;

use super::Message;
use super::theme::{AppTheme, AppThemeVariant};
use rust_i18n::t;

/// 获取当前语言的显示名称
fn get_current_language_display(current_language: &str) -> String {
    // 返回本地化的语言显示名称
    match current_language {
        "zh-CN" => t!("Chinese").to_string(),
        "en" => "English".to_string(),
        _ => "English".to_string(), // 默认显示英语
    }
}

/// SVG 图标常量
mod svg_icons {
    pub const FILE_FOLDER: &str = r#"
    <svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
        <path d="M3 6.2c0-1.12 0-1.68.218-2.108a2 2 0 0 1 .874-.874C4.52 3 5.08 3 6.2 3h1.2c.56 0 .84 0 1.054.109a1 1 0 0 1 .437.437C9 3.76 9 4.04 9 4.6V5h8.8c1.12 0 1.68 0 2.108.218a2 2 0 0 1 .874.874C21 6.52 21 7.08 21 8.2v9.6c0 1.12 0 1.68-.218 2.108a2 2 0 0 1-.874.874C19.48 21 18.92 21 17.8 21H6.2c-1.12 0-1.68 0-2.108-.218a2 2 0 0 1-.874-.874C3 19.48 3 18.92 3 17.8V6.2Z" stroke="currentColor" stroke-width="1.5"/>
    </svg>"#;

    pub const LIST_LOOP: &str = r#"
    <svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
        <path d="M17 8.5V6a2 2 0 0 0-2-2H4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
        <path d="m20 8.5-3-2.5v5l3-2.5Z" fill="currentColor"/>
        <path d="M7 15.5V18a2 2 0 0 0 2 2h11" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
        <path d="m4 15.5 3 2.5v-5l-3 2.5Z" fill="currentColor"/>
    </svg>"#;

    pub const SINGLE_LOOP: &str = r#"
    <svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
        <path d="M17 8.5V6a2 2 0 0 0-2-2H4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
        <path d="m20 8.5-3-2.5v5l3-2.5Z" fill="currentColor"/>
        <path d="M7 15.5V18a2 2 0 0 0 2 2h11" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
        <path d="m4 15.5 3 2.5v-5l-3 2.5Z" fill="currentColor"/>
        <circle cx="12" cy="12" r="2" stroke="currentColor" stroke-width="1.5"/>
        <text x="12" y="12" text-anchor="middle" dominant-baseline="central" font-size="6" font-weight="bold" fill="currentColor">1</text>
    </svg>"#;

    pub const RANDOM_PLAY: &str = r#"
    <svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
        <path d="m3 17 6-4-6-4v8Z" fill="currentColor"/>
        <path d="M14 6h5v5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
        <path d="M19 6 9 16" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
        <path d="M14 18h5v-5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
        <path d="M19 18 9 8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
    </svg>"#;

    pub const MUSIC_NOTE: &str = r#"
    <svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
        <circle cx="7" cy="17" r="3" stroke="currentColor" stroke-width="1.5"/>
        <circle cx="17" cy="15" r="3" stroke="currentColor" stroke-width="1.5"/>
        <path d="M10 17V5l10-2v12" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
        <path d="M10 9l10-2" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
    </svg>"#;

    pub const LIST_VIEW: &str = r#"
    <svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
        <path d="M3 6h18M3 12h18M3 18h18" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
    </svg>"#;
 }

/// 创建SVG图标组件
fn create_svg_icon(svg_content: String, size: f32, color: Color) -> Element<'static, Message> {
    svg(svg::Handle::from_memory(svg_content.as_bytes().to_vec()))
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .style(move |_theme: &iced::Theme, _status| svg::Style {
            color: Some(color),
        })
        .into()
}

/// 创建带tooltip的文本组件
/// 
/// # 参数
/// * `full_text` - 完整文本内容
/// * `max_length` - 最大显示长度
/// * `text_size` - 文本大小
/// * `text_style` - 文本样式函数
/// 
/// # 返回
/// 带tooltip的文本元素
fn create_text_with_tooltip<'a, F>(
    full_text: String,
    max_length: usize,
    text_size: u16,
    text_style: F,
) -> Element<'a, Message> 
where
    F: Fn(&iced::Theme) -> iced::widget::text::Style + 'a + Copy,
{
    let display_text = if full_text.chars().count() > max_length {
        format!("{}...", full_text.chars().take(max_length).collect::<String>())
    } else {
        full_text.clone()
    };
    
    let text_element = text(display_text)
        .size(text_size)
        .style(text_style)
        .shaping(Shaping::Advanced);
    
    if full_text.chars().count() > max_length {
        tooltip(
            text_element,
            text(full_text).size(12),
            tooltip::Position::Top
        )
        .style(|theme: &iced::Theme| {
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
        })
        .padding(8)
        .into()
    } else {
        text_element.into()
    }
}

/// 页面类型枚举 - 用于主导航
#[derive(Debug, Clone, PartialEq, Default)]
pub enum PageType {
    /// 主页面（播放器功能）
    #[default]
    Home,
    /// 设置页面
    Settings,
}

/// 视图类型枚举 - 用于主页面内部视图切换
#[derive(Debug, Clone, PartialEq, Default)]
pub enum ViewType {
    /// 播放列表视图
    #[default]
    Playlist,
    /// 歌词显示视图
    Lyrics,
}

/// 播放模式枚举
#[derive(Debug, Clone, PartialEq, Default)]
pub enum PlayMode {
    /// 列表循环（默认）
    #[default]
    ListLoop,
    /// 单曲循环
    SingleLoop,
    /// 随机播放
    Random,
}

impl PlayMode {
    /// 获取播放模式的SVG图标
    pub fn svg_icon(&self) -> &'static str {
        match self {
            PlayMode::ListLoop => svg_icons::LIST_LOOP,
            PlayMode::SingleLoop => svg_icons::SINGLE_LOOP, 
            PlayMode::Random => svg_icons::RANDOM_PLAY,
        }
    }
    
    /// 获取播放模式的显示名称
    pub fn display_name(&self) -> String {
        match self {
            PlayMode::ListLoop => t!("List Loop").to_string(),
            PlayMode::SingleLoop => t!("Single Loop").to_string(),
            PlayMode::Random => t!("Random Play").to_string(),
        }
    }
    
    /// 获取下一个播放模式
    pub fn next(&self) -> Self {
        match self {
            PlayMode::ListLoop => PlayMode::SingleLoop,
            PlayMode::SingleLoop => PlayMode::Random,
            PlayMode::Random => PlayMode::ListLoop,
        }
    }
}

/// 创建导航栏组件
/// 
/// # 参数
/// * `current_page` - 当前选中的页面
/// 
/// # 返回
/// 导航栏UI元素
pub fn navigation_sidebar(current_page: &PageType) -> Element<'static, Message> {
    let nav_button = |icon: String, label: String, page: PageType, is_active: bool| {
        let style = if is_active {
            AppTheme::control_button()
        } else {
            AppTheme::file_button()
        };
        
        tooltip(
            button(
                text(icon).size(28).shaping(Shaping::Advanced) // 增大图标，移除文字
            )
            .style(style)
            .padding(16) // 调整内边距
            .width(Length::Fixed(60.0))
            .height(Length::Fixed(60.0))
            .on_press(Message::PageChanged(page)),
            text(label).size(12),
            tooltip::Position::Right
        )
        .style(|theme: &iced::Theme| {
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
        })
        .padding(8)
    };

    column![
        nav_button("🏠".to_string(), t!("Home").to_string(), PageType::Home, *current_page == PageType::Home),
        nav_button("⚙️".to_string(), t!("Settings").to_string(), PageType::Settings, *current_page == PageType::Settings),
        
        // 底部空间
        Space::with_height(Length::Fill),
        
        // 应用信息
        container(
            column![
                text("🎵").size(20).shaping(Shaping::Advanced),
                text("Summer").size(10).style(|theme: &iced::Theme| {
                    let palette = theme.extended_palette();
                    text::Style {
                        color: Some(Color {
                            a: 0.7,
                            ..palette.background.base.text
                        }),
                    }
                }),
            ]
            .align_x(Horizontal::Center)
            .spacing(4)
        )
        .width(Length::Fill)
        .align_x(Horizontal::Center)
        .padding(8),
    ]
    .width(Length::Shrink)
    .height(Length::Fill)
    .spacing(12) // 增加间距
    .padding(16) // 增加内边距
    .into()
}

/// 创建设置页面组件
/// 
/// # 参数
/// * `current_theme` - 当前主题
/// * `current_language` - 当前语言
/// 
/// # 返回
/// 设置页面UI元素
pub fn settings_page(current_theme: &AppThemeVariant, current_language: &str) -> Element<'static, Message> {
    column![
        // 页面标题
        container(
            text(t!("Settings")).size(24).style(AppTheme::emphasis_text())
        )
        .padding(16),
        
        // 主题设置
        setting_section(
            t!("Appearance").to_string(),
            column![
                setting_row(
                    t!("Theme").to_string(),
                    row![
                        text(match current_theme {
                            AppThemeVariant::Light => "Light",
                            AppThemeVariant::Dark => "Dark",
                        }).size(14),
                        Space::with_width(Length::Fill),
                        button(text(t!("Toggle")))
                            .on_press(Message::ToggleTheme)
                            .style(AppTheme::file_button())
                            .padding(8)
                    ]
                    .align_y(Vertical::Center)
                )
            ].into()
        ),
        
        // 音频设置
        setting_section(
            t!("Audio").to_string(),
            column![
                setting_row(
                    t!("Output Device").to_string(),
                    row![
                        text(t!("Default")).size(14),
                        Space::with_width(Length::Fill),
                        button(text(t!("Change")))
                            .style(AppTheme::file_button())
                            .padding(8)
                    ]
                    .align_y(Vertical::Center)
                ),
                setting_row(
                    t!("Volume").to_string(),
                    slider(0.0..=100.0, 75.0, |_| Message::Tick) // 临时消息，后续可以添加音量控制
                        .width(Length::Fixed(200.0))
                        .style(AppTheme::progress_slider())
                )
            ].into()
        ),
        
        // 语言设置
        setting_section(
            t!("Language").to_string(),
            column![
                setting_row(
                    t!("Interface Language").to_string(),
                    row![
                        text(get_current_language_display(current_language)).size(14),
                        Space::with_width(Length::Fill),
                        button(text(t!("Change")))
                            .style(AppTheme::file_button())
                            .padding(8)
                    ]
                    .align_y(Vertical::Center)
                )
            ].into()
        ),
        
        // 版本信息
        Space::with_height(Length::Fill),
        container(
            column![
                text(format!("{} v{}", t!("Summer Player"), env!("CARGO_PKG_VERSION"))).size(12).style(AppTheme::subtitle_text()),
                text(format!("© 2025 {}", t!("xml"))).size(10).style(AppTheme::hint_text()),
            ]
            .align_x(Horizontal::Center)
            .spacing(2)
        )
        .center_x(Length::Fill)
        .padding(16)
    ]
    .spacing(16)
    .padding(24)
    .into()
}

/// 创建设置区块
fn setting_section(title: String, content: Element<'static, Message>) -> Element<'static, Message> {
    column![
        text(title).size(16).style(AppTheme::subtitle_text()),
        container(content)
            .style(AppTheme::card_container())
            .padding(16)
            .width(Length::Fill)
    ]
    .spacing(8)
    .into()
}

/// 创建设置行
fn setting_row(label: String, control: impl Into<Element<'static, Message>>) -> Element<'static, Message> {
    row![
        text(label).size(14).width(Length::Fixed(150.0)),
        control.into()
    ]
    .align_y(Vertical::Center)
    .spacing(16)
    .padding(8)
    .into()
}

/// 创建文件信息显示组件
/// 
/// # 参数
/// * `audio_info` - 音频信息
/// * `file_path` - 文件路径
/// 
/// # 返回
/// 文件信息UI元素
pub fn file_info_view(audio_info: Option<&AudioInfo>, file_path: &str) -> Element<'static, Message> {
    let content = if let Some(info) = audio_info {
        let file_name = std::path::Path::new(file_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown file")
            .to_string();
        
        // 创建音频信息列
        let mut audio_info_column = column![
            info_row("🎵", t!("Channel Count").as_ref(), &format!("{}", info.channels)),
            info_row("📡", t!("Sample Rate").as_ref(), &format!("{} Hz", info.sample_rate)),
            info_row("⏱️", t!("Duration").as_ref(), &if let Some(duration) = info.duration {
                format_duration(duration)
            } else {
                "未知".to_string()
            }),
        ].spacing(8);
        
        // 如果有比特深度信息，添加它
        if let Some(bits) = info.bits_per_sample {
            audio_info_column = audio_info_column.push(
                info_row("🎚️", t!("Bit Depth").as_ref(), &format!("{} {}", bits, "位"))
            );
        }
        
        // 创建元数据信息列
        let mut metadata_column = column![].spacing(8);
        
        // 添加元数据信息
        if let Some(title) = &info.metadata.title {
            metadata_column = metadata_column.push(
                info_row_with_tooltip("🎤", t!("Title").as_ref(), &title.clone(), 25)
            );
        }
        
        if let Some(artist) = &info.metadata.artist {
            metadata_column = metadata_column.push(
                info_row_with_tooltip("🎨", t!("Artist").as_ref(), &artist.clone(), 25)
            );
        }
        
        if let Some(album) = &info.metadata.album {
            metadata_column = metadata_column.push(
                info_row_with_tooltip("💿", t!("Album").as_ref(), &album.clone(), 25)
            );
        }
        
        if let Some(year) = &info.metadata.year {
            metadata_column = metadata_column.push(
                info_row("📅", t!("Year").as_ref(), &year.clone())
            );
        }
        
        if let Some(genre) = &info.metadata.genre {
            metadata_column = metadata_column.push(
                info_row("🎭", t!("Genre").as_ref(), &genre.clone())
            );
        }
        
        if let Some(track_number) = &info.metadata.track_number {
            metadata_column = metadata_column.push(
                info_row("🔢", t!("Track Number").as_ref(), &track_number.clone())
            );
        }
        
        if let Some(composer) = &info.metadata.composer {
            metadata_column = metadata_column.push(
                info_row("✍️", t!("Composer").as_ref(), &composer.clone())
            );
        }
        
        // 如果没有元数据，显示文件名
        let display_title = info.metadata.title.clone()
            .unwrap_or(file_name);
        
        {
            let mut main_column = column![
                // 显示标题（优先使用元数据中的标题）
                container(
                    create_text_with_tooltip(
                        display_title,
                        30, // 最大显示30个字符
                        16,
                        |theme: &iced::Theme| {
                            let palette = theme.extended_palette();
                            iced::widget::text::Style {
                                color: Some(palette.primary.base.color),
                            }
                        }
                    )
                )
                .width(Length::Fill),
            ].spacing(12);
            
            // 如果有封面图片，显示封面
            if let Some(cover_art) = &info.metadata.cover_art {
                let cover_image = image::Handle::from_bytes(cover_art.data.clone());
                main_column = main_column.push(
                    container(
                        image(cover_image)
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
                            ..Default::default()
                        }
                    })
                    .padding(4)
                    .width(Length::Shrink)
                    .align_x(Horizontal::Center)
                );
            }
            
            main_column = main_column.push(
                // 音频信息部分
                text(t!("Audio Info"))
                    .size(14)
                    .style(|theme: &iced::Theme| {
                        let palette = theme.extended_palette();
                        text::Style {
                            color: Some(Color {
                                a: 0.8,
                                ..palette.background.base.text
                            }),
                        }
                    })
            ).push(audio_info_column);
            
            // 如果有元数据信息，添加元数据部分
            if info.metadata.title.is_some() || info.metadata.artist.is_some() || 
               info.metadata.album.is_some() || info.metadata.year.is_some() ||
               info.metadata.genre.is_some() || info.metadata.track_number.is_some() ||
               info.metadata.composer.is_some() {
                main_column = main_column.push(
                    column![
                        text(t!("Metadata"))
                            .size(14)
                            .style(|theme: &iced::Theme| {
                                let palette = theme.extended_palette();
                                text::Style {
                                    color: Some(Color {
                                        a: 0.8,
                                        ..palette.background.base.text
                                    }),
                                }
                            }),
                        metadata_column,
                    ].spacing(8)
                );
            }
            
            main_column
        }
    } else {
        column![
            text("🎼")
                .size(32)
                .align_x(Horizontal::Center)
                .shaping(Shaping::Advanced),
            text(t!("File not selected"))
                .size(14)
                .align_x(Horizontal::Center)
                .style(|theme: &iced::Theme| {
                    let palette = theme.extended_palette();
                    text::Style {
                        color: Some(Color {
                            a: 0.7,
                            ..palette.background.base.text
                        }),
                    }
                }),
        ].spacing(8).align_x(Horizontal::Center)
    };

    container(content)
        .style(AppTheme::main_section_container())
        .padding(20) // 增加内边距
        .width(Length::Fill)
        .into()
}

/// 创建信息行
fn info_row(icon: &'static str, label: &str, value: &str) -> Element<'static, Message> {
    row![
        text(icon).size(14).shaping(Shaping::Advanced),
        text(format!("{}: {}", label, value))
            .shaping(Shaping::Advanced)
            .size(12)
            .style(|theme: &iced::Theme| {
                let palette = theme.extended_palette();
                text::Style {
                    color: Some(Color {
                        a: 0.8,
                        ..palette.background.base.text
                    }),
                }
            }),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .into()
}

/// 创建带tooltip的信息行
fn info_row_with_tooltip(icon: &'static str, label: &str, value: &str, max_length: usize) -> Element<'static, Message> {
    let value_element = create_text_with_tooltip(
        value.to_string(),
        max_length,
        12,
        |theme: &iced::Theme| {
            let palette = theme.extended_palette();
            iced::widget::text::Style {
                color: Some(Color {
                    a: 0.8,
                    ..palette.background.base.text
                }),
            }
        }
    );
    
    row![
        text(icon).size(14).shaping(Shaping::Advanced),
        text(format!("{}: ", label))
            .shaping(Shaping::Advanced)
            .size(12)
            .style(|theme: &iced::Theme| {
                let palette = theme.extended_palette();
                text::Style {
                    color: Some(Color {
                        a: 0.8,
                        ..palette.background.base.text
                    }),
                }
            }),
        value_element,
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .into()
}

/// 创建播放控制按钮组
/// 
/// # 参数
/// * `is_playing` - 是否正在播放
/// 
/// # 返回
/// 控制按钮UI元素
pub fn control_buttons_view(is_playing: bool) -> Element<'static, Message> {
    container(
        row![
            // 上一首
            tooltip(
                button(
                    container(text("⏮").size(16).shaping(Shaping::Advanced)) // 增大图标
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .align_x(Horizontal::Center)
                        .align_y(Vertical::Center)
                )
                .style(AppTheme::control_button())
                .width(Length::Fixed(48.0)) // 增大按钮
                .height(Length::Fixed(48.0))
                .on_press(Message::PreviousTrack),
                text(t!("Previous Track")).size(12),
                tooltip::Position::Top
            )
            .style(|theme: &iced::Theme| {
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
            })
            .padding(8),
            
            // 播放/暂停 - 主要按钮，更大更突出
            tooltip(
                button(
                    container(text(if is_playing { "⏸️" } else { "▶️" }).size(20).shaping(Shaping::Advanced)) // 增大图标
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .align_x(Horizontal::Center)
                        .align_y(Vertical::Center)
                )
                .style(AppTheme::play_button())
                .width(Length::Fixed(60.0)) // 增大主按钮
                .height(Length::Fixed(60.0))
                .on_press(Message::PlayPause),
                text(if is_playing { t!("Pause") } else { t!("Play") }).size(12),
                tooltip::Position::Top
            )
            .style(|theme: &iced::Theme| {
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
            })
            .padding(8),
            
            // 下一首
            tooltip(
                button(
                    container(text("⏭").size(16).shaping(Shaping::Advanced)) // 增大图标
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .align_x(Horizontal::Center)
                        .align_y(Vertical::Center)
                )
                .style(AppTheme::control_button())
                .width(Length::Fixed(48.0)) // 增大按钮
                .height(Length::Fixed(48.0))
                .on_press(Message::NextTrack),
                text(t!("Next Track")).size(12),
                tooltip::Position::Top
            )
            .style(|theme: &iced::Theme| {
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
            })
            .padding(8),
        ]
        .spacing(16) // 增加间距
        .align_y(Vertical::Center)
    )
    .style(AppTheme::main_section_container()) // 使用更好的容器样式
    .padding(16) // 增加内边距
    .width(Length::Fill)
    .align_x(Horizontal::Center)
    .into()
}

/// 创建紧凑的播放模式切换按钮（用于底部工具栏）
/// 
/// # 参数
/// * `current_mode` - 当前播放模式
/// 
/// # 返回
/// 紧凑播放模式切换按钮UI元素
pub fn compact_play_mode_button(current_mode: PlayMode) -> Element<'static, Message> {
    let svg_content = current_mode.svg_icon();
    let text_content = current_mode.display_name();
    let subtitle = match current_mode {
        PlayMode::ListLoop => t!("Play all songs in order and repeat").to_string(),
        PlayMode::SingleLoop => t!("Repeat current song").to_string(),
        PlayMode::Random => t!("Play songs in random order").to_string(),
    };
    
    // 统一使用主题色
    let icon_color = Color {
        r: 0.4,
        g: 0.4,
        b: 0.4,
        a: 0.8,
    };
    
    tooltip(
        button(
            container(
                create_svg_icon(svg_content.to_string(), 24.0, icon_color)
            )
            .style(|theme: &iced::Theme| {
                let palette = theme.extended_palette();
                container::Style {
                    background: Some(Background::Color(Color {
                        a: 0.05,
                        ..palette.background.weak.color
                    })),
                    border: Border {
                        radius: Radius::from(8.0),
                        width: 0.0,
                        color: Color::TRANSPARENT,
                    },
                    shadow: Shadow::default(),
                    text_color: None,
                }
            })
            .padding(2)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
        )
        .style(AppTheme::file_button())
        .width(Length::Fixed(48.0))
        .height(Length::Fixed(48.0))
        .on_press(Message::TogglePlayMode),
        column![
            text(text_content).size(12),
            text(subtitle).size(10)
                .style(|theme: &iced::Theme| {
                    let palette = theme.extended_palette();
                    text::Style {
                        color: Some(Color {
                            a: 0.7,
                            ..palette.background.base.text
                        }),
                    }
                })
        ].spacing(2),
        tooltip::Position::Top
    )
    .style(|theme: &iced::Theme| {
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
    })
    .padding(8)
    .into()
}

/// 创建紧凑的打开文件按钮（用于底部工具栏）
/// 
/// # 返回
/// 紧凑打开文件按钮UI元素
pub fn compact_file_button() -> Element<'static, Message> {
    tooltip(
        button(
            container({
                let icon_color = Color {
                    r: 0.4,
                    g: 0.4,
                    b: 0.4,
                    a: 0.8,
                };
                create_svg_icon(svg_icons::FILE_FOLDER.to_string(), 24.0, icon_color)
            })
            .style(|theme: &iced::Theme| {
                let palette = theme.extended_palette();
                container::Style {
                    background: Some(Background::Color(Color {
                        a: 0.05,
                        ..palette.background.weak.color
                    })),
                    border: Border {
                        radius: Radius::from(8.0),
                        width: 0.0,
                        color: Color::TRANSPARENT,
                    },
                    shadow: Shadow::default(),
                    text_color: None,
                }
            })
            .padding(2)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
        )
        .style(AppTheme::file_button())
        .width(Length::Fixed(48.0))
        .height(Length::Fixed(48.0))
        .on_press(Message::OpenFile),
        text(t!("Open File")).size(12),
        tooltip::Position::Top
    )
    .style(|theme: &iced::Theme| {
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
    })
    .padding(8)
    .into()
}

/// 创建紧凑的视图切换按钮（用于底部工具栏）
/// 
/// # 参数
/// * `current_view` - 当前视图类型
/// 
/// # 返回
/// 紧凑视图切换按钮UI元素
pub fn compact_view_toggle_button(current_view: ViewType) -> Element<'static, Message> {
    let (svg_content, text_content, subtitle) = match current_view {
        ViewType::Playlist => (svg_icons::MUSIC_NOTE, t!("Switch to Lyrics").to_string(), t!("View Lyrics Synchronization").to_string()),
        ViewType::Lyrics => (svg_icons::LIST_VIEW, t!("Switch to Playlist").to_string(), t!("Browse Music Library").to_string()),
    };
    
    // 统一使用主题色
    let icon_color = Color {
        r: 0.4,
        g: 0.4,
        b: 0.4,
        a: 0.8,
    };
    
    tooltip(
        button(
            container(
                create_svg_icon(svg_content.to_string(), 24.0, icon_color)
            )
            .style(|theme: &iced::Theme| {
                let palette = theme.extended_palette();
                container::Style {
                    background: Some(Background::Color(Color {
                        a: 0.05,
                        ..palette.background.weak.color
                    })),
                    border: Border {
                        radius: Radius::from(8.0),
                        width: 0.0,
                        color: Color::TRANSPARENT,
                    },
                    shadow: Shadow::default(),
                    text_color: None,
                }
            })
            .padding(2)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
        )
        .style(AppTheme::file_button())
        .width(Length::Fixed(48.0))
        .height(Length::Fixed(48.0))
        .on_press(Message::ToggleView),
        column![
            text(text_content).size(12),
            text(subtitle).size(10)
                .style(|theme: &iced::Theme| {
                    let palette = theme.extended_palette();
                    text::Style {
                        color: Some(Color {
                            a: 0.7,
                            ..palette.background.base.text
                        }),
                    }
                })
        ].spacing(2),
        tooltip::Position::Top
    )
    .style(|theme: &iced::Theme| {
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
    })
    .padding(8)
    .into()
}

/// 创建播放进度显示组件
/// 
/// # 参数
/// * `playback_state` - 播放状态
/// 
/// # 返回
/// 进度显示UI元素
pub fn progress_view(playback_state: &PlaybackState) -> Element<'static, Message> {
    let progress_value = if playback_state.total_duration > 0.0 {
        (playback_state.current_time / playback_state.total_duration) as f32
    } else {
        0.0
    };
    
    let current_time_text = format_duration(playback_state.current_time);
    let total_time_text = format_duration(playback_state.total_duration);
    
    container(
        column![
            // 时间显示
            row![
                text(current_time_text)
                    .size(14) // 增大字体
                    .style(AppTheme::current_time_text()),
                Space::new(Length::Fill, Length::Shrink),
                text(total_time_text)
                    .size(14) // 增大字体
                    .style(AppTheme::total_time_text()),
            ]
            .padding(4), // 添加内边距
            
            // 进度滑块
            container(
                slider(0.0..=1.0, progress_value, Message::ProgressChanged)
                    .step(0.001)
                    .style(AppTheme::progress_slider())
            )
            .style(|theme: &iced::Theme| {
                let palette = theme.extended_palette();
                container::Style {
                    background: Some(Background::Color(Color {
                        a: 0.08, // 增加透明度
                        ..palette.primary.base.color
                    })),
                    border: Border {
                        radius: Radius::from(10.0), // 增加圆角
                        width: 0.0,
                        color: Color::TRANSPARENT,
                    },
                    shadow: Shadow {
                        color: Color::from_rgba(0.0, 0.0, 0.0, 0.08),
                        offset: iced::Vector::new(0.0, 2.0),
                        blur_radius: 4.0,
                    },
                    text_color: None,
                }
            })
            .padding(4), // 增加内边距
        ].spacing(12) // 增加间距
    )
    .style(AppTheme::glass_card_container()) // 使用毛玻璃效果
    .padding(20) // 增加内边距
    .width(Length::Fill)
    .into()
}

/// 创建播放列表视图组件
/// 
/// # 参数
/// * `playlist` - 播放列表
/// * `playlist_loaded` - 是否已加载播放列表
/// * `is_playing` - 是否正在播放
/// 
/// # 返回
/// 播放列表UI元素
pub fn playlist_view(
    playlist: &Playlist, 
    playlist_loaded: bool, 
    is_playing: bool
) -> Element<'static, Message> {
    if playlist_loaded {
        let playlist_items: Vec<Element<Message>> = playlist.items().iter().enumerate().map(|(index, item)| {
            let is_current = playlist.current_index() == Some(index);
            let is_playing_current = is_current && is_playing;
            
            let (icon, song_name) = if is_current {
                if is_playing_current {
                    ("🎵", item.name.clone())
                } else {
                    ("⏸", item.name.clone())
                }
            } else {
                ("🎼", item.name.clone())
            };
            
            let duration_text = item.duration.map_or("--:--".to_string(), |d| format_duration(d));
            
            let song_name_with_tooltip = container(
                create_text_with_tooltip(
                    song_name.clone(),
                    30, // 最大显示30个字符
                    14,
                    move |theme: &iced::Theme| {
                        let palette = theme.extended_palette();
                        iced::widget::text::Style {
                            color: Some(if is_current {
                                palette.primary.base.color
                            } else {
                                palette.background.base.text
                            }),
                        }
                    }
                )
            )
            .width(Length::FillPortion(4));
            
            let content = container(
                row![
                    text(icon).size(14).shaping(Shaping::Advanced),
                    song_name_with_tooltip,
                    text(duration_text)
                        .width(Length::FillPortion(1))
                        .size(12)
                        .align_x(Horizontal::Right)
                        .style(|theme: &iced::Theme| {
                            let palette = theme.extended_palette();
                            text::Style {
                                color: Some(Color {
                                    a: 0.7,
                                    ..palette.background.base.text
                                }),
                            }
                        }),
                ].spacing(12).align_y(Vertical::Center)
            )
            .padding([8, 12])
            .width(Length::Fill);
            
            let btn = button(content)
                .on_press(Message::PlaylistItemSelected(index))
                .width(Length::Fill)
                .style(AppTheme::playlist_item_button(is_playing_current, is_current));
            
            btn.into()
        }).collect();
        
        container(
            column![
                // 播放列表标题
                container(
                    row![
                        text("📋").size(20).shaping(Shaping::Advanced), // 增大图标
                        //text(format!("Playlist ({} songs)", playlist.len()))
                        text(t!("messages.Playlist", count = format!("{}", playlist.len())))
                            .size(18) // 增大标题字体
                            .style(|theme: &iced::Theme| {
                                let palette = theme.extended_palette();
                                text::Style {
                                    color: Some(palette.primary.base.color),
                                }
                            }),
                    ].spacing(12).align_y(Vertical::Center) // 增加间距
                )
                .padding(8), // 底部间距
                
                // 播放列表项目
                scrollable(
                    column(playlist_items).spacing(6).padding([12, 8]) // 增加间距和内边距
                ).height(Length::Fill).width(Length::Fill),
            ].spacing(20) // 增加间距
        )
        .style(AppTheme::main_section_container()) // 使用更好的容器样式
        .padding(20) // 增加内边距
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    } else {
        container(
            column![
                text("📂").size(48).align_x(Horizontal::Center).shaping(Shaping::Advanced),
                text(t!("No playlist loaded"))
                    .size(16)
                    .align_x(Horizontal::Center)
                    .style(AppTheme::subtitle_text()),
                text(t!(r#"Click "Open File" to start"#.to_string()))
                    .size(12)
                    .align_x(Horizontal::Center)
                    .style(AppTheme::hint_text()),
            ].spacing(12).align_x(Horizontal::Center)
        )
        .style(AppTheme::card_container())
        .padding(32)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}

/// 创建歌词显示组件
/// 
/// # 参数
/// * `file_path` - 当前文件路径
/// * `is_playing` - 是否正在播放
/// * `current_time` - 当前播放时间
/// * `lyrics` - 当前歌词数据
/// 
/// # 返回
/// 歌词显示UI元素
pub fn lyrics_view(file_path: &str, is_playing: bool, current_time: f64, lyrics: &Option<crate::lyrics::Lyrics>, window_height: f32) -> Element<'static, Message> {
    if file_path.is_empty() {
        return container(
            column![
                text("🎵").size(48).align_x(Horizontal::Center).shaping(Shaping::Advanced),
                text(t!("Lyrics Display"))
                    .size(20)
                    .align_x(Horizontal::Center)
                    .style(|theme: &iced::Theme| {
                        let palette = theme.extended_palette();
                        text::Style {
                            color: Some(palette.primary.base.color),
                        }
                    }),
                text(t!("Please select an audio file"))
                    .size(14)
                    .align_x(Horizontal::Center)
                    .style(|theme: &iced::Theme| {
                        let palette = theme.extended_palette();
                        text::Style {
                            color: Some(Color {
                                a: 0.7,
                                ..palette.background.base.text
                            }),
                        }
                    }),
            ].spacing(16).align_x(Horizontal::Center)
        )
        .style(AppTheme::card_container())
        .padding(32)
        .width(Length::Fill)
        .height(Length::Fill)
        .into();
    }
    
    // 创建歌词内容
    let mut lyrics_elements = Vec::<Element<Message>>::new();
    
    // 添加标题，包含歌曲信息
    if let Some(ref lyrics_data) = lyrics {
        let title = if let Some(ref title) = lyrics_data.metadata.title {
            title.clone()
        } else {
            // 从文件路径提取文件名
            std::path::Path::new(file_path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("未知歌曲")
                .to_string()
        };
        
        lyrics_elements.push(
            container(
                create_text_with_tooltip(
                    title,
                    40, // 最大显示40个字符
                    20,
                    |theme: &iced::Theme| {
                        let palette = theme.extended_palette();
                        iced::widget::text::Style {
                            color: Some(palette.primary.base.color),
                        }
                    }
                )
            )
            .width(Length::Fill)
            .align_x(Horizontal::Center)
            .into()
        );
        
        if let Some(ref artist) = lyrics_data.metadata.artist {
            lyrics_elements.push(
                container(
                    create_text_with_tooltip(
                        format!("🎤 {}", artist),
                        35, // 最大显示35个字符
                        14,
                        |theme: &iced::Theme| {
                            let palette = theme.extended_palette();
                            iced::widget::text::Style {
                                color: Some(Color {
                                    a: 0.8,
                                    ..palette.background.base.text
                                }),
                            }
                        }
                    )
                )
                .width(Length::Fill)
                .align_x(Horizontal::Center)
                .into()
            );
        }
        
        lyrics_elements.push(text("").into()); // 空行
    } else {
        lyrics_elements.push(
            text("🎵 歌词显示")
                .size(18)
                .align_x(Horizontal::Center)
                .shaping(Shaping::Advanced)
                .style(|theme: &iced::Theme| {
                    let palette = theme.extended_palette();
                    text::Style {
                        color: Some(palette.primary.base.color),
                    }
                })
                .into()
        );
    }
    
    // 显示歌词内容 - 动态行数显示，当前行居中
    if let Some(ref lyrics_data) = lyrics {
        if lyrics_data.has_lyrics() {
            // 动态计算显示行数 - 基于窗口高度和内容
            let total_lyrics_count = lyrics_data.lines.len();
            let display_lines = calculate_optimal_display_lines(total_lyrics_count, window_height);
            let center_line = display_lines / 2; // 动态中心位置
            
            // 获取当前歌词行索引
            let current_line_index = lyrics_data.get_current_line_index(current_time);
            
            // 计算显示范围 - 让当前行尽量居中
            let (start_index, visible_count) = if let Some(current_idx) = current_line_index {
                // 计算显示窗口的起始位置，让当前行居中
                let ideal_start = if current_idx >= center_line {
                    current_idx - center_line
                } else {
                    0
                };
                
                // 确保不超出歌词总数
                let available_lyrics = lyrics_data.lines.len();
                let actual_start = if ideal_start + display_lines > available_lyrics {
                    if available_lyrics > display_lines {
                        available_lyrics - display_lines
                    } else {
                        0
                    }
                } else {
                    ideal_start
                };
                
                let visible_count = (available_lyrics - actual_start).min(display_lines);
                (actual_start, visible_count)
            } else {
                // 如果没有当前行，显示前面的歌词
                let visible_count = lyrics_data.lines.len().min(display_lines);
                (0, visible_count)
            };
            
            // 如果歌词总数少于显示行数，添加前置空行来保持居中效果
            let total_lyrics = lyrics_data.lines.len();
            let (pre_empty_lines, post_empty_lines) = if total_lyrics < display_lines {
                let empty_lines = display_lines - total_lyrics;
                let pre_lines = empty_lines / 2;
                let post_lines = empty_lines - pre_lines;
                (pre_lines, post_lines)
            } else {
                (0, 0)
            };
            
            // 添加前置空行
            for _ in 0..pre_empty_lines {
                lyrics_elements.push(
                    text("")
                        .size(16)
                        .align_x(Horizontal::Center)
                        .into()
                );
            }
            
            // 创建实际歌词显示行
            for i in 0..visible_count {
                let lyrics_index = start_index + i;
                
                if lyrics_index < lyrics_data.lines.len() {
                    let line = &lyrics_data.lines[lyrics_index];
                    let is_current = current_line_index == Some(lyrics_index);
                    let is_upcoming = current_line_index.map_or(false, |current| lyrics_index == current + 1);
                    
                    // 创建歌词文本
                    let lyric_text = if line.text.trim().is_empty() {
                        "♪".to_string() // 空行显示音符
                    } else {
                        line.text.clone()
                    };
                    
                    // 根据状态设置样式
                    let text_element: Element<Message> = if is_current && is_playing {
                        // 当前播放行 - 高亮显示，居中对齐
                        container(
                            text(format!("▶ {}", lyric_text))
                                .size(18)
                                .align_x(Horizontal::Center)
                                .shaping(Shaping::Advanced)
                                .style(|theme: &iced::Theme| {
                                    let palette = theme.extended_palette();
                                    text::Style {
                                        color: Some(palette.primary.strong.color),
                                    }
                                })
                        )
                        .style(|theme: &iced::Theme| {
                            let palette = theme.extended_palette();
                            container::Style {
                                background: Some(Background::Color(Color {
                                    a: 0.1,
                                    ..palette.primary.base.color
                                })),
                                border: Border {
                                    radius: Radius::from(8.0),
                                    width: 0.0,
                                    color: Color::TRANSPARENT,
                                },
                                shadow: Shadow::default(),
                                text_color: None,
                            }
                        })
                        .padding([8, 16])
                        .width(Length::Fill)
                        .into()
                    } else if is_upcoming && is_playing {
                        // 下一行 - 稍微突出显示
                        text(lyric_text)
                            .size(16)
                            .align_x(Horizontal::Center)
                            .shaping(Shaping::Advanced)
                            .style(|theme: &iced::Theme| {
                                let palette = theme.extended_palette();
                                text::Style {
                                    color: Some(palette.secondary.base.color),
                                }
                            })
                            .into()
                    } else if current_line_index.map_or(false, |current| lyrics_index <= current) {
                        // 已播放的行 - 淡化显示
                        text(lyric_text)
                            .size(14)
                            .align_x(Horizontal::Center)
                            .shaping(Shaping::Advanced)
                            .style(|theme: &iced::Theme| {
                                let palette = theme.extended_palette();
                                text::Style {
                                    color: Some(Color {
                                        a: 0.4,
                                        ..palette.background.weak.text
                                    }),
                                }
                            })
                            .into()
                    } else {
                        // 未播放的行 - 正常显示但稍微淡一些
                        text(lyric_text)
                            .size(14)
                            .align_x(Horizontal::Center)
                            .shaping(Shaping::Advanced)
                            .style(|theme: &iced::Theme| {
                                let palette = theme.extended_palette();
                                text::Style {
                                    color: Some(Color {
                                        a: 0.7,
                                        ..palette.background.weak.text
                                    }),
                                }
                            })
                            .into()
                    };
                    
                    lyrics_elements.push(text_element.into());
                }
            }
            
            // 添加后置空行
            for _ in 0..post_empty_lines {
                lyrics_elements.push(
                    text("")
                        .size(16)
                        .align_x(Horizontal::Center)
                        .into()
                );
            }
            
            // 如果没有当前行且正在播放，在底部显示提示
            if current_line_index.is_none() && is_playing {
                lyrics_elements.push(text("").into());
                lyrics_elements.push(
                    text("♪ 音乐开始了... ♪")
                        .size(14)
                        .align_x(Horizontal::Center)
                        .shaping(Shaping::Advanced)
                        .style(|theme: &iced::Theme| {
                            let palette = theme.extended_palette();
                            text::Style {
                                color: Some(palette.primary.base.color),
                            }
                        })
                        .into()
                );
            }
            
        } else {
            // 歌词文件存在但没有歌词内容
            lyrics_elements.push(
                text("⚠️ 歌词文件已加载，但没有找到歌词内容")
                    .align_x(Horizontal::Center)
                    .shaping(Shaping::Advanced)
                    .style(|theme: &iced::Theme| {
                        let palette = theme.extended_palette();
                        text::Style {
                            color: Some(Color {
                                a: 0.7,
                                ..palette.background.base.text
                            }),
                        }
                    })
                    .into()
            );
        }
    } else {
        // 没有歌词文件
        if is_playing {
            lyrics_elements.push(
                text("♪ 正在播放中... ♪")
                    .size(18)
                    .align_x(Horizontal::Center)
                    .shaping(Shaping::Advanced)
                    .style(|theme: &iced::Theme| {
                        let palette = theme.extended_palette();
                        text::Style {
                            color: Some(palette.primary.base.color),
                        }
                    })
                    .into()
            );
            lyrics_elements.push(text("").into());
            lyrics_elements.push(
                text("🎵 暂无歌词文件")
                    .align_x(Horizontal::Center)
                    .shaping(Shaping::Advanced)
                    .style(|theme: &iced::Theme| {
                        let palette = theme.extended_palette();
                        text::Style {
                            color: Some(Color {
                                a: 0.7,
                                ..palette.background.base.text
                            }),
                        }
                    })
                    .into()
            );
            lyrics_elements.push(text("").into());
            lyrics_elements.push(
                text(format!("⏱️ {}", format_duration(current_time)))
                    .size(12)
                    .align_x(Horizontal::Center)
                    .shaping(Shaping::Advanced)
                    .style(|theme: &iced::Theme| {
                        let palette = theme.extended_palette();
                        text::Style {
                            color: Some(Color {
                                a: 0.6,
                                ..palette.background.base.text
                            }),
                        }
                    })
                    .into()
            );
        } else {
            lyrics_elements.push(
                text("♪ 歌词显示 ♪")
                    .size(18)
                    .align_x(Horizontal::Center)
                    .shaping(Shaping::Advanced)
                    .style(|theme: &iced::Theme| {
                        let palette = theme.extended_palette();
                        text::Style {
                            color: Some(palette.primary.base.color),
                        }
                    })
                    .into()
            );
            lyrics_elements.push(text("").into());
            lyrics_elements.push(
                text("⏸️ 暂停播放中")
                    .align_x(Horizontal::Center)
                    .shaping(Shaping::Advanced)
                    .style(|theme: &iced::Theme| {
                        let palette = theme.extended_palette();
                        text::Style {
                            color: Some(Color {
                                a: 0.7,
                                ..palette.background.base.text
                            }),
                        }
                    })
                    .into()
            );
        }
        
        lyrics_elements.push(text("").into());
        lyrics_elements.push(
            container(
                column![
                    text("💡 使用提示")
                        .size(14)
                        .shaping(Shaping::Advanced)
                        .style(|theme: &iced::Theme| {
                            let palette = theme.extended_palette();
                            text::Style {
                                color: Some(palette.primary.base.color),
                            }
                        }),
                    text("📁 将 .lrc 歌词文件放在音频文件同目录下").size(11).shaping(Shaping::Advanced),
                    text("📝 歌词文件名需与音频文件名相同").size(11).shaping(Shaping::Advanced),
                    text("🕐 支持时间同步的LRC格式歌词").size(11).shaping(Shaping::Advanced),
                ].spacing(6)
            )
            .style(|theme: &iced::Theme| {
                let palette = theme.extended_palette();
                container::Style {
                    background: Some(Background::Color(Color {
                        a: 0.05,
                        ..palette.primary.base.color
                    })),
                    border: Border {
                        radius: Radius::from(8.0),
                        width: 1.0,
                        color: Color {
                            a: 0.2,
                            ..palette.primary.base.color
                        },
                    },
                    shadow: Shadow::default(),
                    text_color: Some(Color {
                        a: 0.8,
                        ..palette.background.base.text
                    }),
                }
            })
            .padding(12)
            .into()
        );
    }
    
    // 创建高度自适应的歌词显示区域，不使用滚动条
    container(
        column(lyrics_elements)
            .spacing(20)  // 进一步增加行间距
            .width(Length::Fill)
            .align_x(Horizontal::Center)
    )
    .style(AppTheme::main_section_container()) // 使用更好的容器样式
    .padding(28)  // 增加内边距
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

/// 创建空白填充组件
/// 
/// # 返回
/// 空白填充UI元素
pub fn spacer() -> Element<'static, Message> {
    Space::new(Length::Fill, Length::Fill).into()
}

/// 计算最佳歌词显示行数
/// 
/// # 参数
/// * `total_lyrics_count` - 歌词总行数
/// * `window_height` - 当前窗口高度
/// 
/// # 返回
/// 最佳显示行数
fn calculate_optimal_display_lines(total_lyrics_count: usize, window_height: f32) -> usize {
    // 基于实际窗口高度和歌词总数的动态策略
    
    // 1. 根据窗口高度计算可用空间
    let title_and_metadata_height = 120.0;  // 标题和艺术家信息
    let toggle_button_height = 50.0;        // 切换按钮
    let progress_bar_height = 80.0;         // 进度条区域
    let padding_and_spacing = 100.0;        // 内边距和间距
    
    let available_height = window_height 
        - title_and_metadata_height 
        - toggle_button_height 
        - progress_bar_height 
        - padding_and_spacing;
    
    // 2. 根据可用高度计算行数
    let line_height = 28.0; // 每行预估高度（字体 + 行间距）
    let calculated_lines = (available_height / line_height) as usize;
    
    // 3. 基于歌词数量调整策略
    let content_based_lines = if total_lyrics_count <= 7 {
        9  // 歌词很少时，固定显示9行保持居中
    } else {
        // 根据歌词数量和计算出的行数取较小值
        calculated_lines.min(total_lyrics_count + 4) // 允许前后各2行的上下文
    };
    
    // 4. 确保在合理范围内，并优先保持奇数（有助于居中）
    let final_lines = content_based_lines.max(5).min(21);
    if final_lines % 2 == 0 {
        final_lines + 1
    } else {
        final_lines
    }
}

 