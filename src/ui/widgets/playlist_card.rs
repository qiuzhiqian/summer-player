//! 播放列表卡片控件
//! 
//! 用于显示播放列表信息的可重用卡片组件

use iced::{
    widget::{button, column, row, text, text_input, Space},
    Element, Length, Border, Shadow, Background, Color,
    alignment::{Horizontal, Vertical},
    border::Radius,
};
use iced::advanced::text::Shaping;
use crate::ui::Message;
use crate::ui::components::{constants, icons, svg_icon};
use crate::ui::widgets::{StyledContainer, styled_container::ContainerStyle};
use rust_i18n::t;
use crate::ui::theme::AppColors;

/// 播放列表卡片配置
#[derive(Clone, Debug)]
pub struct PlaylistCardConfig {
    /// 播放列表路径
    pub path: String,
    /// 播放列表名称
    pub name: String,
    /// 歌曲数量
    pub song_count: usize,
    /// 是否被选中
    pub is_selected: bool,
    /// 卡片宽度
    pub width: f32,
    /// 卡片高度
    pub height: f32,
    /// 是否显示更多菜单
    pub show_menu: bool,
    /// 是否处于重命名输入模式
    pub renaming: bool,
    /// 重命名输入中的名称
    pub renaming_name: String,
}

impl Default for PlaylistCardConfig {
    fn default() -> Self {
        Self {
            path: String::new(),
            name: String::new(),
            song_count: 0,
            is_selected: false,
            width: 170.0,
            height: 240.0,
            show_menu: false,
            renaming: false,
            renaming_name: String::new(),
        }
    }
}

/// 播放列表卡片控件
pub struct PlaylistCard {
    config: PlaylistCardConfig,
}

impl PlaylistCard {
    /// 创建新的播放列表卡片
    pub fn new(config: PlaylistCardConfig) -> Self {
        Self { config }
    }

    /// 使用构建器模式创建卡片
    pub fn builder() -> PlaylistCardBuilder {
        PlaylistCardBuilder::new()
    }

    /// 构建卡片元素
    pub fn build(self) -> Element<'static, Message> {
        let config = self.config;
        let is_selected = config.is_selected;
        
        // 处理播放列表名称显示
        let name_without_extension = std::path::Path::new(&config.name)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(&config.name)
            .to_string();
        
        let display_name = if name_without_extension.chars().count() > 18 {
            format!("{}...", name_without_extension.chars().take(15).collect::<String>())
        } else {
            name_without_extension
        };

        // 播放列表名称文本
        let name_text = text(display_name)
            .size(constants::TEXT_LARGE)
            .align_x(Horizontal::Center)
            .style(move |theme: &iced::Theme| {
                let palette = theme.extended_palette();
                iced::widget::text::Style {
                    color: Some(if is_selected {
                        palette.primary.strong.color
                    } else {
                        palette.background.base.text
                    }),
                }
            });

        // 歌曲数量文本
        let song_count_text = text(format!(
            "{} {}", 
            config.song_count, 
            if config.song_count == 1 { t!("song") } else { t!("songs") }
        ))
        .size(constants::TEXT_MEDIUM)
        .align_x(Horizontal::Center)
        .style(move |theme: &iced::Theme| {
            let palette = theme.extended_palette();
            iced::widget::text::Style {
                color: Some(if is_selected {
                    Color { a: 0.8, ..palette.primary.strong.color }
                } else {
                    Color { a: 0.7, ..palette.background.base.text }
                }),
            }
        });

        // 根据状态创建顶部区域：图标、菜单或重命名输入
        let top_area: Element<Message> = if config.renaming {
            let input: Element<Message> = text_input::<Message, iced::Theme, iced::Renderer>(t!("Playlist Name").as_ref(), &config.renaming_name)
                .on_input(Message::PlaylistCardRenameNameChanged)
                .size(constants::TEXT_NORMAL)
                .padding(8)
                .width(Length::Fill)
                .into();

            let actions: Element<Message> = row![
                button(text("✔").shaping(Shaping::Advanced).size(constants::TEXT_MEDIUM))
                    .on_press(Message::PlaylistCardRenameConfirm)
                    .style(|theme: &iced::Theme, status: iced::widget::button::Status| {
                        let palette = theme.extended_palette();
                        match status {
                            iced::widget::button::Status::Hovered => iced::widget::button::Style { background: Some(Background::Color(Color { a: 0.15, ..palette.primary.base.color })), text_color: palette.primary.base.text, border: Border { radius: Radius::from(8.0), width: 0.0, color: Color::TRANSPARENT }, shadow: Shadow::default(), snap: false },
                            _ => iced::widget::button::Style { background: Some(Background::Color(Color { a: 0.1, ..palette.primary.base.color })), text_color: palette.primary.base.text, border: Border { radius: Radius::from(8.0), width: 0.0, color: Color::TRANSPARENT }, shadow: Shadow::default(), snap: false },
                        }
                    }),
                Space::with_width(Length::Fixed(8.0)),
                button(text("✖").shaping(Shaping::Advanced).size(constants::TEXT_MEDIUM))
                    .on_press(Message::PlaylistCardRenameCancel)
                    .style(|theme: &iced::Theme, status: iced::widget::button::Status| {
                        let palette = theme.extended_palette();
                        match status {
                            iced::widget::button::Status::Hovered => iced::widget::button::Style { background: Some(Background::Color(Color { a: 0.15, ..palette.background.weak.color })), text_color: palette.background.base.text, border: Border { radius: Radius::from(8.0), width: 0.0, color: Color::TRANSPARENT }, shadow: Shadow::default(), snap: false },
                            _ => iced::widget::button::Style { background: Some(Background::Color(Color { a: 0.1, ..palette.background.weak.color })), text_color: palette.background.base.text, border: Border { radius: Radius::from(8.0), width: 0.0, color: Color::TRANSPARENT }, shadow: Shadow::default(), snap: false },
                        }
                    }),
            ]
            .spacing(constants::SPACING_SMALL)
            .align_y(Vertical::Center)
            .into();

            StyledContainer::new(
                column![input, actions]
                    .spacing(constants::SPACING_SMALL)
                    .width(Length::Fill)
                    .align_x(Horizontal::Center)
            )
            .style(ContainerStyle::Decorative)
            .width(Length::Fixed(160.0))
            .height(Length::Fixed(160.0))
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .build()
        } else if config.show_menu {
            // 三个按钮：重命名、添加音乐、删除
            let rename_btn = button(text(t!("Rename")).size(constants::TEXT_MEDIUM))
                .on_press(Message::PlaylistCardActionRenameStart(config.path.clone()))
                .style(|theme: &iced::Theme, status: iced::widget::button::Status| {
                    let palette = theme.extended_palette();
                    match status {
                        iced::widget::button::Status::Hovered => iced::widget::button::Style { background: Some(Background::Color(Color { a: 0.12, ..palette.primary.base.color })), text_color: palette.primary.strong.color, border: Border { radius: Radius::from(8.0), width: 0.0, color: Color::TRANSPARENT }, shadow: Shadow::default(), snap: false },
                        _ => iced::widget::button::Style { background: Some(Background::Color(Color::TRANSPARENT)), text_color: palette.primary.strong.color, border: Border { radius: Radius::from(8.0), width: 0.0, color: Color::TRANSPARENT }, shadow: Shadow::default(), snap: false },
                    }
                });
            let add_btn = button(text(t!("Add Music")).size(constants::TEXT_MEDIUM))
                .on_press(Message::PlaylistCardActionAddMusic(config.path.clone()))
                .style(|theme: &iced::Theme, status: iced::widget::button::Status| {
                    let palette = theme.extended_palette();
                    match status {
                        iced::widget::button::Status::Hovered => iced::widget::button::Style { background: Some(Background::Color(Color { a: 0.12, ..palette.primary.base.color })), text_color: palette.primary.strong.color, border: Border { radius: Radius::from(8.0), width: 0.0, color: Color::TRANSPARENT }, shadow: Shadow::default(), snap: false },
                        _ => iced::widget::button::Style { background: Some(Background::Color(Color::TRANSPARENT)), text_color: palette.primary.strong.color, border: Border { radius: Radius::from(8.0), width: 0.0, color: Color::TRANSPARENT }, shadow: Shadow::default(), snap: false },
                    }
                });
            let delete_btn = button(text(t!("Delete")).size(constants::TEXT_MEDIUM))
                .on_press(Message::PlaylistCardActionDelete(config.path.clone()))
                .style(|theme: &iced::Theme, status: iced::widget::button::Status| {
                    let palette = theme.extended_palette();
                    match status {
                        iced::widget::button::Status::Hovered => iced::widget::button::Style { background: Some(Background::Color(Color { a: 0.12, ..palette.background.strong.color })), text_color: palette.background.base.text, border: Border { radius: Radius::from(8.0), width: 0.0, color: Color::TRANSPARENT }, shadow: Shadow::default(), snap: false },
                        _ => iced::widget::button::Style { background: Some(Background::Color(Color::TRANSPARENT)), text_color: palette.background.base.text, border: Border { radius: Radius::from(8.0), width: 0.0, color: Color::TRANSPARENT }, shadow: Shadow::default(), snap: false },
                    }
                });
            StyledContainer::new(
                column![rename_btn, add_btn, delete_btn]
                    .spacing(constants::SPACING_SMALL)
                    .width(Length::Fill)
                    .align_x(Horizontal::Center)
            )
            .style(ContainerStyle::Decorative)
            .width(Length::Fixed(160.0))
            .height(Length::Fixed(160.0))
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .build()
        } else {
                StyledContainer::new(
                svg_icon(icons::CD_ICON, 90.0, if is_selected { Color { a: 0.9, ..constants::ICON_COLOR } } else { constants::ICON_COLOR })
                )
                .style(ContainerStyle::Decorative)
                .width(Length::Fixed(160.0))
                .height(Length::Fixed(160.0))
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center)
            .build()
        };
                
        // 创建卡片内容
        let card_content = StyledContainer::new(
            column![
                top_area,
                // 播放列表信息
                column![
                    // 使用一个行布局来实现左右对齐
                    row![
                        // 名称占用剩余空间，左对齐
                        StyledContainer::new(name_text)
                            .style(ContainerStyle::Transparent)
                            .width(Length::Fill)
                            .align_x(Horizontal::Left)
                            .build(),
                        // 歌曲数量固定宽度，右对齐
                        StyledContainer::new(song_count_text)
                            .style(ContainerStyle::Transparent)
                            .width(Length::Shrink)
                            .align_x(Horizontal::Right)
                            .build(),
                        // 右侧的更多操作按钮（切换菜单/图标）
                        {
                            let playlist_path_for_more = config.path.clone();
                            let more_btn = button(text("⋮").shaping(Shaping::Advanced).size(constants::TEXT_LARGE))
                                .padding(constants::PADDING_SMALL)
                                .width(Length::Fill)
                                .on_press(Message::PlaylistCardMoreClicked(playlist_path_for_more))
                                .style(move |theme: &iced::Theme, status: iced::widget::button::Status| {
                                    let palette = theme.extended_palette();
                                    match status {
                                        iced::widget::button::Status::Hovered => iced::widget::button::Style { background: Some(Background::Color(Color { a: 0.12, ..palette.primary.base.color })), text_color: palette.primary.strong.color, border: Border { radius: Radius::from(6.0), width: 0.0, color: Color::TRANSPARENT }, shadow: Shadow::default(), snap: false },
                                        iced::widget::button::Status::Pressed => iced::widget::button::Style { background: Some(Background::Color(Color { a: 0.2, ..palette.primary.base.color })), text_color: palette.primary.strong.color, border: Border { radius: Radius::from(6.0), width: 0.0, color: Color::TRANSPARENT }, shadow: Shadow::default(), snap: false },
                                        _ => iced::widget::button::Style { background: Some(Background::Color(Color::TRANSPARENT)), text_color: palette.primary.strong.color, border: Border { radius: Radius::from(6.0), width: 0.0, color: Color::TRANSPARENT }, shadow: Shadow::default(), snap: false },
                                    }
                                });
                            StyledContainer::new(more_btn)
                                .style(ContainerStyle::Transparent)
                                .width(Length::Fixed(32.0))
                                .align_x(Horizontal::Right)
                                .build()
                        }
                    ]
                    .spacing(6)
                    .width(Length::Fill)
                    .align_y(Vertical::Center)
                ]
                .spacing(constants::SPACING_MEDIUM)
                .width(Length::Fill)
                .align_x(Horizontal::Center)
            ]
            .spacing(constants::SPACING_MEDIUM)
            .align_x(Horizontal::Center)
            .height(Length::Fill)
        )
        .style(ContainerStyle::Card)
        .width(Length::Fixed(config.width))
        .height(Length::Fixed(config.height))
        .align_x(Horizontal::Center)
        .align_y(Vertical::Center)
        .padding([constants::PADDING_MEDIUM, constants::PADDING_SMALL])
        .build();

        // 创建可点击的按钮
        let playlist_path = config.path.clone();
        button(card_content)
            .on_press(Message::PlaylistCardToggled(playlist_path))
            .style(move |theme: &iced::Theme, status| {
                Self::card_button_style(theme, status, is_selected)
            })
            .into()
    }

    /// 卡片按钮样式
    fn card_button_style(
        theme: &iced::Theme,
        status: iced::widget::button::Status,
        is_selected: bool,
    ) -> iced::widget::button::Style {
        let palette = theme.extended_palette();
        
        match status {
            iced::widget::button::Status::Active => iced::widget::button::Style {
                background: if is_selected {
                    Some(Background::Color(Color { a: 0.3, ..palette.primary.base.color }))
                } else {
                    Some(Background::Color(Color::TRANSPARENT))
                },
                text_color: if is_selected {
                    palette.primary.strong.color
                } else {
                    palette.background.base.text
                },
                border: Border {
                    radius: Radius::from(8.0),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
                shadow: if is_selected {
                    Shadow {
                        color: AppColors::shadow(theme),
                        offset: iced::Vector::new(0.0, 3.0),
                        blur_radius: 8.0,
                    }
                } else {
                    Shadow::default()
                },
                snap: false,
            },
            iced::widget::button::Status::Hovered => iced::widget::button::Style {
                background: if is_selected {
                    Some(Background::Color(Color { a: 0.4, ..palette.primary.base.color }))
                } else {
                    Some(Background::Color(Color { a: 0.15, ..palette.primary.base.color }))
                },
                text_color: palette.primary.strong.color,
                border: Border {
                    radius: Radius::from(8.0),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
                shadow: Shadow {
                    color: AppColors::shadow(theme),
                    offset: iced::Vector::new(0.0, 4.0),
                    blur_radius: 10.0,
                },
                snap: false,
            },
            iced::widget::button::Status::Pressed => iced::widget::button::Style {
                background: if is_selected {
                    Some(Background::Color(Color { a: 0.5, ..palette.primary.base.color }))
                } else {
                    Some(Background::Color(Color { a: 0.25, ..palette.primary.base.color }))
                },
                text_color: palette.primary.strong.color,
                border: Border {
                    radius: Radius::from(8.0),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
                shadow: Shadow {
                    color: AppColors::shadow(theme),
                    offset: iced::Vector::new(0.0, 2.0),
                    blur_radius: 6.0,
                },
                snap: false,
            },
            iced::widget::button::Status::Disabled => iced::widget::button::Style {
                background: Some(Background::Color(Color { a: 0.05, ..palette.background.strong.color })),
                text_color: Color { a: 0.5, ..palette.background.base.text },
                border: Border {
                    radius: Radius::from(8.0),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
                shadow: Shadow::default(),
                snap: false,
            },
        }
    }
}

/// 播放列表卡片构建器
pub struct PlaylistCardBuilder {
    config: PlaylistCardConfig,
}

impl PlaylistCardBuilder {
    /// 创建新的构建器
    pub fn new() -> Self {
        Self {
            config: PlaylistCardConfig::default(),
        }
    }

    /// 设置播放列表路径
    pub fn path<S: Into<String>>(mut self, path: S) -> Self {
        self.config.path = path.into();
        self
    }

    /// 设置播放列表名称
    pub fn name<S: Into<String>>(mut self, name: S) -> Self {
        self.config.name = name.into();
        self
    }

    /// 设置歌曲数量
    pub fn song_count(mut self, count: usize) -> Self {
        self.config.song_count = count;
        self
    }

    /// 设置是否选中
    pub fn selected(mut self, selected: bool) -> Self {
        self.config.is_selected = selected;
        self
    }

    /// 设置卡片宽度
    pub fn width(mut self, width: f32) -> Self {
        self.config.width = width;
        self
    }

    /// 设置卡片高度
    pub fn height(mut self, height: f32) -> Self {
        self.config.height = height;
        self
    }

    /// 设置是否显示更多菜单
    pub fn show_menu(mut self, show: bool) -> Self {
        self.config.show_menu = show;
        self
    }

    /// 设置是否处于重命名模式
    pub fn renaming(mut self, renaming: bool) -> Self {
        self.config.renaming = renaming;
        self
    }

    /// 设置重命名输入中的名称
    pub fn renaming_name<S: Into<String>>(mut self, name: S) -> Self {
        self.config.renaming_name = name.into();
        self
    }

    /// 构建卡片
    pub fn build(self) -> Element<'static, Message> {
        PlaylistCard::new(self.config).build()
    }
}

impl Default for PlaylistCardBuilder {
    fn default() -> Self {
        Self::new()
    }
}
