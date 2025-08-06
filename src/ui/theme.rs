//! 主题样式模块
//! 
//! 包含应用程序所有UI组件的统一样式设置，支持Light和Dark模式。

use iced::{
    widget::{button, text, slider, container},
    Border, Shadow, Background, Color,
    theme::{Theme, Palette},
    border::Radius,
};

/// 自定义主题枚举
#[derive(Debug, Clone, PartialEq, Default)]
pub enum AppThemeVariant {
    #[default]
    Light,
    Dark,
}

impl AppThemeVariant {
    /// 切换主题
    pub fn toggle(&self) -> Self {
        match self {
            AppThemeVariant::Light => AppThemeVariant::Dark,
            AppThemeVariant::Dark => AppThemeVariant::Light,
        }
    }

    /// 获取主题名称
    pub fn name(&self) -> &'static str {
        match self {
            AppThemeVariant::Light => "Light",
            AppThemeVariant::Dark => "Dark",
        }
    }

    /// 转换为iced主题
    pub fn to_iced_theme(&self) -> Theme {
        match self {
            AppThemeVariant::Light => Theme::Light,
            AppThemeVariant::Dark => Theme::custom(
                "Dark".to_string(),
                Palette {
                    background: Color::from_rgb(0.09, 0.09, 0.11),           // #171719 - 深色背景
                    text: Color::from_rgb(0.95, 0.95, 0.97),                 // #F2F2F7 - 浅色文本
                    primary: Color::from_rgb(0.0, 0.48, 1.0),                // #007AFF - 蓝色主色
                    success: Color::from_rgb(0.20, 0.78, 0.35),              // #34C759 - 绿色
                    danger: Color::from_rgb(1.0, 0.23, 0.19),                // #FF3B30 - 红色
                }
            ),
        }
    }
}

/// 自定义颜色方案
pub struct AppColors;

impl AppColors {
    /// 检测当前主题是否为Dark模式
    fn is_dark_theme(theme: &Theme) -> bool {
        // 通过背景色判断是否为dark模式
        let bg = theme.extended_palette().background.base.color;
        bg.r + bg.g + bg.b < 1.5 // RGB三个值的总和小于1.5表示深色主题
    }

    /// 获取主色调
    pub fn primary(theme: &Theme) -> Color {
        if Self::is_dark_theme(theme) {
            Color::from_rgb(0.04, 0.52, 1.0)      // #0A84FF
        } else {
            Color::from_rgb(0.0, 0.48, 1.0)       // #007AFF
        }
    }

    /// 获取主色调渐变起始色
    pub fn primary_gradient_start(theme: &Theme) -> Color {
        if Self::is_dark_theme(theme) {
            Color::from_rgb(0.04, 0.52, 1.0)      // #0A84FF
        } else {
            Color::from_rgb(0.0, 0.48, 1.0)       // #007AFF
        }
    }

    /// 获取主色调渐变结束色
    pub fn primary_gradient_end(theme: &Theme) -> Color {
        if Self::is_dark_theme(theme) {
            Color::from_rgb(0.20, 0.34, 0.96)     // #3456F5
        } else {
            Color::from_rgb(0.35, 0.34, 0.84)     // #5856D6
        }
    }

    /// 获取次要色调
    pub fn secondary(theme: &Theme) -> Color {
        if Self::is_dark_theme(theme) {
            Color::from_rgb(0.37, 0.36, 0.90)     // #5E5CE6
        } else {
            Color::from_rgb(0.35, 0.34, 0.84)     // #5856D6
        }
    }

    /// 获取成功色
    pub fn success(theme: &Theme) -> Color {
        if Self::is_dark_theme(theme) {
            Color::from_rgb(0.20, 0.78, 0.35)     // #34C759
        } else {
            Color::from_rgb(0.13, 0.70, 0.29)     // #22B348
        }
    }

    /// 获取背景色
    pub fn background(theme: &Theme) -> Color {
        if Self::is_dark_theme(theme) {
            Color::from_rgb(0.06, 0.06, 0.08)     // #0F0F14 - 更深的背景
        } else {
            Color::from_rgb(0.97, 0.97, 0.99)     // #F7F7FC - 更柔和的背景
        }
    }

    /// 获取渐变背景起始色
    pub fn background_gradient_start(theme: &Theme) -> Color {
        if Self::is_dark_theme(theme) {
            Color::from_rgb(0.06, 0.06, 0.08)     // #0F0F14
        } else {
            Color::from_rgb(0.98, 0.98, 1.0)      // #FAFAFF
        }
    }

    /// 获取渐变背景结束色
    pub fn background_gradient_end(theme: &Theme) -> Color {
        if Self::is_dark_theme(theme) {
            Color::from_rgb(0.09, 0.09, 0.11)     // #171719
        } else {
            Color::from_rgb(0.95, 0.96, 0.98)     // #F2F4FA
        }
    }

    /// 获取卡片背景色
    pub fn card_background(theme: &Theme) -> Color {
        if Self::is_dark_theme(theme) {
            Color::from_rgb(0.11, 0.11, 0.13)     // #1C1C21 - 更柔和的卡片背景
        } else {
            Color::from_rgba(1.0, 1.0, 1.0, 0.95) // 半透明白色
        }
    }

    /// 获取卡片背景色（带透明度）
    pub fn card_background_translucent(theme: &Theme) -> Color {
        if Self::is_dark_theme(theme) {
            Color::from_rgba(0.13, 0.13, 0.15, 0.8) // 半透明深色
        } else {
            Color::from_rgba(1.0, 1.0, 1.0, 0.7)    // 半透明白色
        }
    }

    /// 获取表面色（稍微突出的背景）
    pub fn surface(theme: &Theme) -> Color {
        if Self::is_dark_theme(theme) {
            Color::from_rgb(0.15, 0.15, 0.17)     // #262629
        } else {
            Color::from_rgb(0.93, 0.94, 0.96)     // #EDF0F4
        }
    }

    /// 获取边框色
    pub fn border(theme: &Theme) -> Color {
        if Self::is_dark_theme(theme) {
            Color::from_rgb(0.20, 0.20, 0.22)     // #333338
        } else {
            Color::from_rgb(0.88, 0.89, 0.91)     // #E1E3E7
        }
    }

    /// 获取分隔线颜色
    pub fn divider(theme: &Theme) -> Color {
        if Self::is_dark_theme(theme) {
            Color::from_rgba(0.44, 0.44, 0.46, 0.3) // 半透明分隔线
        } else {
            Color::from_rgba(0.68, 0.68, 0.70, 0.3)
        }
    }

    /// 获取主要文本色
    pub fn text_primary(theme: &Theme) -> Color {
        if Self::is_dark_theme(theme) {
            Color::from_rgb(0.95, 0.95, 0.97)     // #F2F2F7
        } else {
            Color::from_rgb(0.0, 0.0, 0.0)        // #000000
        }
    }

    /// 获取次要文本色
    pub fn text_secondary(theme: &Theme) -> Color {
        if Self::is_dark_theme(theme) {
            Color::from_rgb(0.56, 0.56, 0.58)     // #8E8E93
        } else {
            Color::from_rgb(0.44, 0.44, 0.46)     // #6D6D75
        }
    }

    /// 获取提示文本色
    pub fn text_hint(theme: &Theme) -> Color {
        if Self::is_dark_theme(theme) {
            Color::from_rgb(0.44, 0.44, 0.46)     // #6D6D75
        } else {
            Color::from_rgb(0.68, 0.68, 0.70)     // #AEAEB2
        }
    }

    /// 获取阴影色
    pub fn shadow(theme: &Theme) -> Color {
        if Self::is_dark_theme(theme) {
            Color::from_rgba(0.0, 0.0, 0.0, 0.3)
        } else {
            Color::from_rgba(0.0, 0.0, 0.0, 0.1)
        }
    }

    /// 获取强阴影色
    pub fn shadow_strong(theme: &Theme) -> Color {
        if Self::is_dark_theme(theme) {
            Color::from_rgba(0.0, 0.0, 0.0, 0.5)
        } else {
            Color::from_rgba(0.0, 0.0, 0.0, 0.2)
        }
    }

    /// 获取警告色
    pub fn warning(theme: &Theme) -> Color {
        if Self::is_dark_theme(theme) {
            Color::from_rgb(1.0, 0.62, 0.04)      // #FF9F0A
        } else {
            Color::from_rgb(1.0, 0.58, 0.0)       // #FF9500
        }
    }
}

/// 应用程序主题样式集合
pub struct AppTheme;

impl AppTheme {
    /// 创建现代化卡片容器样式
    pub fn card_container() -> fn(&Theme) -> container::Style {
        |theme: &Theme| {
            container::Style {
                background: Some(Background::Color(AppColors::card_background(theme))),
                border: Border {
                    radius: Radius::from(16.0), // 增加圆角
                    width: 1.0,
                    color: AppColors::border(theme),
                },
                shadow: Shadow {
                    color: AppColors::shadow(theme),
                    offset: iced::Vector::new(0.0, 4.0), // 增加阴影深度
                    blur_radius: 16.0, // 增加模糊半径
                },
                text_color: Some(AppColors::text_primary(theme)),
            }
        }
    }

    /// 创建毛玻璃效果卡片容器样式
    pub fn glass_card_container() -> fn(&Theme) -> container::Style {
        |theme: &Theme| {
            container::Style {
                background: Some(Background::Color(AppColors::card_background_translucent(theme))),
                border: Border {
                    radius: Radius::from(20.0),
                    width: 1.0,
                    color: AppColors::divider(theme),
                },
                shadow: Shadow {
                    color: AppColors::shadow_strong(theme),
                    offset: iced::Vector::new(0.0, 8.0),
                    blur_radius: 24.0,
                },
                text_color: Some(AppColors::text_primary(theme)),
            }
        }
    }

    /// 创建主要功能区域容器样式
    pub fn main_section_container() -> fn(&Theme) -> container::Style {
        |theme: &Theme| {
            container::Style {
                background: Some(Background::Color(AppColors::card_background(theme))),
                border: Border {
                    radius: Radius::from(20.0),
                    width: 1.0,
                    color: AppColors::border(theme),
                },
                shadow: Shadow {
                    color: AppColors::shadow(theme),
                    offset: iced::Vector::new(0.0, 6.0),
                    blur_radius: 20.0,
                },
                text_color: Some(AppColors::text_primary(theme)),
            }
        }
    }

    /// 创建背景容器样式
    pub fn background_container() -> fn(&Theme) -> container::Style {
        |theme: &Theme| {
            container::Style {
                background: Some(Background::Color(AppColors::background_gradient_start(theme))),
                border: Border {
                    radius: Radius::from(0.0),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
                shadow: Shadow::default(),
                text_color: Some(AppColors::text_primary(theme)),
            }
        }
    }

    /// 创建信息卡片容器样式
    pub fn info_card_container() -> fn(&Theme) -> container::Style {
        |theme: &Theme| {
            let primary = AppColors::primary(theme);
            container::Style {
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
                    color: AppColors::shadow(theme),
                    offset: iced::Vector::new(0.0, 1.0),
                    blur_radius: 3.0,
                },
                text_color: Some(AppColors::text_primary(theme)),
            }
        }
    }

    /// 创建圆形播放按钮样式
    pub fn play_button() -> fn(&Theme, button::Status) -> button::Style {
        |theme: &Theme, status| {
            let primary = AppColors::primary(theme);
            let _gradient_start = AppColors::primary_gradient_start(theme);
            let _gradient_end = AppColors::primary_gradient_end(theme);
            
            match status {
                button::Status::Active => button::Style {
                    background: Some(Background::Color(primary)), // 实际使用渐变时可以改进
                    text_color: Color::WHITE,
                    border: Border {
                        radius: Radius::from(26.0),
                        width: 3.0, // 增加边框宽度
                        color: Color {
                            a: 0.2,
                            ..Color::WHITE
                        },
                    },
                    shadow: Shadow {
                        color: Color::from_rgba(primary.r, primary.g, primary.b, 0.4), // 使用主色调阴影
                        offset: iced::Vector::new(0.0, 6.0),
                        blur_radius: 16.0,
                    },
                },
                button::Status::Hovered => button::Style {
                    background: Some(Background::Color(Color {
                        r: (primary.r * 1.15).min(1.0),
                        g: (primary.g * 1.15).min(1.0),
                        b: (primary.b * 1.15).min(1.0),
                        a: 1.0,
                    })),
                    text_color: Color::WHITE,
                    border: Border {
                        radius: Radius::from(26.0),
                        width: 3.0,
                        color: Color {
                            a: 0.4,
                            ..Color::WHITE
                        },
                    },
                    shadow: Shadow {
                        color: Color::from_rgba(primary.r, primary.g, primary.b, 0.6),
                        offset: iced::Vector::new(0.0, 8.0),
                        blur_radius: 20.0,
                    },
                },
                button::Status::Pressed => button::Style {
                    background: Some(Background::Color(Color {
                        r: primary.r * 0.85,
                        g: primary.g * 0.85,
                        b: primary.b * 0.85,
                        a: 1.0,
                    })),
                    text_color: Color::WHITE,
                    border: Border {
                        radius: Radius::from(26.0),
                        width: 3.0,
                        color: Color {
                            a: 0.6,
                            ..Color::WHITE
                        },
                    },
                    shadow: Shadow {
                        color: Color::from_rgba(primary.r, primary.g, primary.b, 0.3),
                        offset: iced::Vector::new(0.0, 3.0),
                        blur_radius: 8.0,
                    },
                },
                button::Status::Disabled => button::Style {
                    background: Some(Background::Color(AppColors::surface(theme))),
                    text_color: AppColors::text_hint(theme),
                    border: Border {
                        radius: Radius::from(26.0),
                        width: 1.0,
                        color: AppColors::border(theme),
                    },
                    shadow: Shadow::default(),
                },
            }
        }
    }

    /// 创建控制按钮样式（停止、上一首、下一首等）
    pub fn control_button() -> fn(&Theme, button::Status) -> button::Style {
        |theme: &Theme, status| {
            let secondary = AppColors::secondary(theme);
            match status {
                button::Status::Active => button::Style {
                    background: Some(Background::Color(Color {
                        a: 0.85,
                        ..secondary
                    })),
                    text_color: Color::WHITE,
                    border: Border {
                        radius: Radius::from(20.0),
                        width: 1.0,
                        color: Color {
                            a: 0.3,
                            ..Color::WHITE
                        },
                    },
                    shadow: Shadow {
                        color: Color::from_rgba(secondary.r, secondary.g, secondary.b, 0.3),
                        offset: iced::Vector::new(0.0, 4.0),
                        blur_radius: 12.0,
                    },
                },
                button::Status::Hovered => button::Style {
                    background: Some(Background::Color(secondary)),
                    text_color: Color::WHITE,
                    border: Border {
                        radius: Radius::from(20.0),
                        width: 1.0,
                        color: Color {
                            a: 0.5,
                            ..Color::WHITE
                        },
                    },
                    shadow: Shadow {
                        color: Color::from_rgba(secondary.r, secondary.g, secondary.b, 0.4),
                        offset: iced::Vector::new(0.0, 6.0),
                        blur_radius: 16.0,
                    },
                },
                button::Status::Pressed => button::Style {
                    background: Some(Background::Color(Color {
                        r: secondary.r * 0.85,
                        g: secondary.g * 0.85,
                        b: secondary.b * 0.85,
                        a: 1.0,
                    })),
                    text_color: Color::WHITE,
                    border: Border {
                        radius: Radius::from(20.0),
                        width: 1.0,
                        color: Color {
                            a: 0.4,
                            ..Color::WHITE
                        },
                    },
                    shadow: Shadow {
                        color: Color::from_rgba(secondary.r, secondary.g, secondary.b, 0.2),
                        offset: iced::Vector::new(0.0, 2.0),
                        blur_radius: 6.0,
                    },
                },
                button::Status::Disabled => button::Style {
                    background: Some(Background::Color(AppColors::surface(theme))),
                    text_color: AppColors::text_hint(theme),
                    border: Border {
                        radius: Radius::from(20.0),
                        width: 1.0,
                        color: AppColors::border(theme),
                    },
                    shadow: Shadow::default(),
                },
            }
        }
    }

    /// 创建文件打开按钮样式
    pub fn file_button() -> fn(&Theme, button::Status) -> button::Style {
        |theme: &Theme, status| {
            let primary = AppColors::primary(theme);
            match status {
                button::Status::Active => button::Style {
                    background: Some(Background::Color(Color {
                        a: 0.1,
                        ..primary
                    })),
                    text_color: primary,
                    border: Border {
                        radius: Radius::from(16.0), // 增加圆角
                        width: 1.5,
                        color: Color {
                            a: 0.35,
                            ..primary
                        },
                    },
                    shadow: Shadow {
                        color: Color::from_rgba(primary.r, primary.g, primary.b, 0.15),
                        offset: iced::Vector::new(0.0, 4.0),
                        blur_radius: 12.0,
                    },
                },
                button::Status::Hovered => button::Style {
                    background: Some(Background::Color(Color {
                        a: 0.15,
                        ..primary
                    })),
                    text_color: primary,
                    border: Border {
                        radius: Radius::from(16.0),
                        width: 1.5,
                        color: Color {
                            a: 0.55,
                            ..primary
                        },
                    },
                    shadow: Shadow {
                        color: Color::from_rgba(primary.r, primary.g, primary.b, 0.25),
                        offset: iced::Vector::new(0.0, 6.0),
                        blur_radius: 16.0,
                    },
                },
                button::Status::Pressed => button::Style {
                    background: Some(Background::Color(Color {
                        a: 0.2,
                        ..primary
                    })),
                    text_color: primary,
                    border: Border {
                        radius: Radius::from(16.0),
                        width: 1.5,
                        color: Color {
                            a: 0.7,
                            ..primary
                        },
                    },
                    shadow: Shadow {
                        color: Color::from_rgba(primary.r, primary.g, primary.b, 0.15),
                        offset: iced::Vector::new(0.0, 2.0),
                        blur_radius: 8.0,
                    },
                },
                button::Status::Disabled => button::Style {
                    background: Some(Background::Color(AppColors::surface(theme))),
                    text_color: AppColors::text_hint(theme),
                    border: Border {
                        radius: Radius::from(16.0),
                        width: 1.0,
                        color: AppColors::border(theme),
                    },
                    shadow: Shadow::default(),
                },
            }
        }
    }

    /// 创建视图切换按钮样式
    pub fn view_toggle_button() -> fn(&Theme, button::Status) -> button::Style {
        |theme: &Theme, status| {
            let primary = AppColors::primary(theme);
            match status {
                button::Status::Active => button::Style {
                    background: Some(Background::Color(Color {
                        a: 0.05,
                        ..primary
                    })),
                    text_color: AppColors::text_primary(theme),
                    border: Border {
                        radius: Radius::from(12.0),
                        width: 1.0,
                        color: Color {
                            a: 0.2,
                            ..primary
                        },
                    },
                    shadow: Shadow {
                        color: AppColors::shadow(theme),
                        offset: iced::Vector::new(0.0, 1.0),
                        blur_radius: 3.0,
                    },
                },
                button::Status::Hovered => button::Style {
                    background: Some(Background::Color(Color {
                        a: 0.1,
                        ..primary
                    })),
                    text_color: AppColors::text_primary(theme),
                    border: Border {
                        radius: Radius::from(12.0),
                        width: 1.0,
                        color: Color {
                            a: 0.3,
                            ..primary
                        },
                    },
                    shadow: Shadow {
                        color: AppColors::shadow_strong(theme),
                        offset: iced::Vector::new(0.0, 3.0),
                        blur_radius: 6.0,
                    },
                },
                button::Status::Pressed => button::Style {
                    background: Some(Background::Color(Color {
                        a: 0.15,
                        ..primary
                    })),
                    text_color: AppColors::text_primary(theme),
                    border: Border {
                        radius: Radius::from(12.0),
                        width: 1.0,
                        color: Color {
                            a: 0.4,
                            ..primary
                        },
                    },
                    shadow: Shadow {
                        color: AppColors::shadow(theme),
                        offset: iced::Vector::new(0.0, 1.0),
                        blur_radius: 2.0,
                    },
                },
                button::Status::Disabled => button::Style {
                    background: Some(Background::Color(AppColors::surface(theme))),
                    text_color: AppColors::text_hint(theme),
                    border: Border {
                        radius: Radius::from(12.0),
                        width: 1.0,
                        color: AppColors::border(theme),
                    },
                    shadow: Shadow::default(),
                },
            }
        }
    }

    /// 创建主题切换按钮样式
    pub fn theme_toggle_button() -> fn(&Theme, button::Status) -> button::Style {
        |theme: &Theme, status| {
            let warning = AppColors::warning(theme);
            match status {
                button::Status::Active => button::Style {
                    background: Some(Background::Color(Color {
                        a: 0.1,
                        ..warning
                    })),
                    text_color: warning,
                    border: Border {
                        radius: Radius::from(20.0),
                        width: 1.0,
                        color: Color {
                            a: 0.3,
                            ..warning
                        },
                    },
                    shadow: Shadow {
                        color: AppColors::shadow(theme),
                        offset: iced::Vector::new(0.0, 2.0),
                        blur_radius: 4.0,
                    },
                },
                button::Status::Hovered => button::Style {
                    background: Some(Background::Color(Color {
                        a: 0.15,
                        ..warning
                    })),
                    text_color: warning,
                    border: Border {
                        radius: Radius::from(20.0),
                        width: 1.0,
                        color: warning,
                    },
                    shadow: Shadow {
                        color: AppColors::shadow_strong(theme),
                        offset: iced::Vector::new(0.0, 4.0),
                        blur_radius: 8.0,
                    },
                },
                button::Status::Pressed => button::Style {
                    background: Some(Background::Color(Color {
                        a: 0.2,
                        ..warning
                    })),
                    text_color: warning,
                    border: Border {
                        radius: Radius::from(20.0),
                        width: 1.0,
                        color: warning,
                    },
                    shadow: Shadow {
                        color: AppColors::shadow(theme),
                        offset: iced::Vector::new(0.0, 1.0),
                        blur_radius: 2.0,
                    },
                },
                button::Status::Disabled => button::Style {
                    background: Some(Background::Color(AppColors::surface(theme))),
                    text_color: AppColors::text_hint(theme),
                    border: Border {
                        radius: Radius::from(20.0),
                        width: 1.0,
                        color: AppColors::border(theme),
                    },
                    shadow: Shadow::default(),
                },
            }
        }
    }

    /// 创建播放列表项按钮样式
    pub fn playlist_item_button(is_playing_current: bool, is_current: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
        move |theme: &Theme, status| {
            let primary = AppColors::primary(theme);
            let secondary = AppColors::secondary(theme);
            
            if is_playing_current {
                // 正在播放的当前项目
                match status {
                    button::Status::Active => button::Style {
                        background: Some(Background::Color(Color {
                            a: 0.15,
                            ..primary
                        })),
                        text_color: primary,
                        border: Border {
                            radius: Radius::from(8.0),
                            width: 1.0,
                            color: Color {
                                a: 0.3,
                                ..primary
                            },
                        },
                        shadow: Shadow {
                            color: AppColors::shadow(theme),
                            offset: iced::Vector::new(0.0, 1.0),
                            blur_radius: 3.0,
                        },
                    },
                    button::Status::Hovered => button::Style {
                        background: Some(Background::Color(Color {
                            a: 0.2,
                            ..primary
                        })),
                        text_color: primary,
                        border: Border {
                            radius: Radius::from(8.0),
                            width: 1.0,
                            color: primary,
                        },
                        shadow: Shadow {
                            color: AppColors::shadow_strong(theme),
                            offset: iced::Vector::new(0.0, 2.0),
                            blur_radius: 4.0,
                        },
                    },
                    _ => button::Style::default(),
                }
            } else if is_current {
                // 当前选中的项目
                match status {
                    button::Status::Active => button::Style {
                        background: Some(Background::Color(Color {
                            a: 0.1,
                            ..secondary
                        })),
                        text_color: secondary,
                        border: Border {
                            radius: Radius::from(8.0),
                            width: 1.0,
                            color: Color {
                                a: 0.2,
                                ..secondary
                            },
                        },
                        shadow: Shadow {
                            color: AppColors::shadow(theme),
                            offset: iced::Vector::new(0.0, 1.0),
                            blur_radius: 2.0,
                        },
                    },
                    button::Status::Hovered => button::Style {
                        background: Some(Background::Color(Color {
                            a: 0.15,
                            ..secondary
                        })),
                        text_color: secondary,
                        border: Border {
                            radius: Radius::from(8.0),
                            width: 1.0,
                            color: secondary,
                        },
                        shadow: Shadow {
                            color: AppColors::shadow_strong(theme),
                            offset: iced::Vector::new(0.0, 2.0),
                            blur_radius: 4.0,
                        },
                    },
                    _ => button::Style::default(),
                }
            } else {
                // 普通项目
                match status {
                    button::Status::Hovered => button::Style {
                        background: Some(Background::Color(AppColors::surface(theme))),
                        text_color: AppColors::text_primary(theme),
                        border: Border {
                            radius: Radius::from(8.0),
                            width: 1.0,
                            color: AppColors::border(theme),
                        },
                        shadow: Shadow {
                            color: AppColors::shadow(theme),
                            offset: iced::Vector::new(0.0, 1.0),
                            blur_radius: 2.0,
                        },
                    },
                    _ => button::Style {
                        background: Some(Background::Color(Color::TRANSPARENT)),
                        text_color: AppColors::text_primary(theme),
                        border: Border {
                            radius: Radius::from(8.0),
                            width: 0.0,
                            color: Color::TRANSPARENT,
                        },
                        shadow: Shadow::default(),
                    },
                }
            }
        }
    }

    /// 创建进度滑块样式
    pub fn progress_slider() -> fn(&Theme, slider::Status) -> slider::Style {
        |theme: &Theme, _status| {
            let primary = AppColors::primary(theme);
            slider::Style {
                rail: slider::Rail {
                    backgrounds: (
                        Background::Color(primary),
                        Background::Color(Color {
                            a: 0.3,
                            ..AppColors::border(theme)
                        }),
                    ),
                    width: 6.0,
                    border: Border {
                        radius: Radius::from(3.0),
                        width: 0.0,
                        color: Color::TRANSPARENT,
                    },
                },
                handle: slider::Handle {
                    shape: slider::HandleShape::Circle { radius: 8.0 },
                    background: Background::Color(primary),
                    border_width: 2.0,
                    border_color: AppColors::card_background(theme),
                },
            }
        }
    }

    /// 创建主标题文本样式
    pub fn title_text() -> fn(&Theme) -> text::Style {
        |theme: &Theme| {
            text::Style {
                color: Some(AppColors::primary(theme)),
            }
        }
    }

    /// 创建副标题文本样式
    pub fn subtitle_text() -> fn(&Theme) -> text::Style {
        |theme: &Theme| {
            text::Style {
                color: Some(AppColors::text_secondary(theme)),
            }
        }
    }

    /// 创建信息标签文本样式
    pub fn info_label_text() -> fn(&Theme) -> text::Style {
        |theme: &Theme| {
            text::Style {
                color: Some(AppColors::text_secondary(theme)),
            }
        }
    }

    /// 创建信息值文本样式
    pub fn info_value_text() -> fn(&Theme) -> text::Style {
        |theme: &Theme| {
            text::Style {
                color: Some(AppColors::text_primary(theme)),
            }
        }
    }

    /// 创建时间显示文本样式（当前时间）
    pub fn current_time_text() -> fn(&Theme) -> text::Style {
        |theme: &Theme| {
            text::Style {
                color: Some(AppColors::primary(theme)),
            }
        }
    }

    /// 创建时间显示文本样式（总时间）
    pub fn total_time_text() -> fn(&Theme) -> text::Style {
        |theme: &Theme| {
            text::Style {
                color: Some(AppColors::text_secondary(theme)),
            }
        }
    }

    /// 创建歌词文本样式（当前行）
    pub fn current_lyrics_text() -> fn(&Theme) -> text::Style {
        |theme: &Theme| {
            text::Style {
                color: Some(AppColors::primary(theme)),
            }
        }
    }

    /// 创建歌词文本样式（其他行）
    pub fn lyrics_text() -> fn(&Theme) -> text::Style {
        |theme: &Theme| {
            text::Style {
                color: Some(AppColors::text_secondary(theme)),
            }
        }
    }

    /// 创建提示文本样式
    pub fn hint_text() -> fn(&Theme) -> text::Style {
        |theme: &Theme| {
            text::Style {
                color: Some(AppColors::text_hint(theme)),
            }
        }
    }

    /// 创建强调文本样式
    pub fn emphasis_text() -> fn(&Theme) -> text::Style {
        |theme: &Theme| {
            text::Style {
                color: Some(AppColors::primary(theme)),
            }
        }
    }



    /// 创建透明容器样式
    pub fn transparent_container() -> fn(&Theme) -> container::Style {
        |_theme: &Theme| {
            container::Style {
                background: Some(Background::Color(Color::TRANSPARENT)),
                border: Border::default(),
                shadow: Shadow::default(),
                text_color: None,
            }
        }
    }
} 