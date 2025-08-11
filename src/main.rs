use clap::Parser;
use iced::{Font, window};
use sys_locale::get_locale;

use summer_player::{
    PlayerApp,
    audio::{AudioFile, list_audio_devices},
    utils::format_duration,
    error::Result,
    config::{fonts, AppConfig},
};



// 包含图标数据
const ICON_BYTES: &[u8] = include_bytes!("../icon.png");

/// CLI参数
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long, help = "List available audio output devices")]
    list_devices: bool,

    #[arg(short, long, help = "Select output device by index")]
    device: Option<usize>,

    #[arg(short, long, help = "Show audio file information and duration without playing")]
    info: bool,

    #[arg(help = "Path to audio file")]
    file: Option<String>,
}

fn main() {
    let args = Cli::parse();
    
    // 加载配置文件，检测是否从实际文件加载
    let (mut config, config_exists) = AppConfig::load_with_source();
    
    // 设置语言环境，优先级：配置文件 > 系统检测
    let locale = if config_exists {
        // 配置文件存在，使用配置文件中的语言（不管是什么语言）
        println!("Using language from config: {}", config.ui.language);
        config.ui.language.clone()
    } else {
        // 配置文件不存在，自动检测系统语言
        let detected_locale = get_locale().unwrap_or_else(|| String::from("en-US"));
        let mapped_locale = match detected_locale.as_str() {
            s if s.starts_with("zh") => "zh-CN",
            _ => "en",
        };
        
        println!("No config file, detected locale: {}, using: {}", detected_locale, mapped_locale);
        
        // 更新配置中的语言设置
        config.ui.language = mapped_locale.to_string();
        // 保存包含检测到的语言的配置文件
        config.save_safe();
        mapped_locale.to_string()
    };
    
    rust_i18n::set_locale(&locale);
    
    if args.list_devices {
        list_audio_devices();
        return;
    }
    
    let file_path = args.file.unwrap_or_default();
    
    if args.info {
        if let Err(e) = get_audio_info(&file_path) {
            eprintln!("Error: {}", e);
        }
        return;
    }
    
    // 创建窗口图标
    let icon = window::icon::from_file_data(ICON_BYTES, None)
        .expect("Failed to load icon");
    
    // 传递命令行文件参数给应用程序
    let initial_file = if !file_path.is_empty() {
        Some(file_path)
    } else {
        None
    };
    
    // 从配置文件创建窗口设置（在移动config之前）
    let window_settings = window::Settings {
        size: iced::Size::new(config.window.width, config.window.height),
        position: if let Some((x, y)) = config.window.position {
            window::Position::Specific(iced::Point::new(x as f32, y as f32))
        } else {
            window::Position::Default
        },
        resizable: true,
        decorations: true,
        transparent: false,
        level: window::Level::Normal,
        icon: Some(icon),
        ..window::Settings::default()
    };
    
    // 更新配置中的语言设置
    let mut final_config = config;
    final_config.ui.language = locale.clone();
    
    let (app, initial_task) = PlayerApp::new_with_config(initial_file, final_config);
    
    iced::application("Summer Audio Player", PlayerApp::update, PlayerApp::view)
        .subscription(PlayerApp::subscription)
        .theme(PlayerApp::theme)
        .window(window_settings)
        .default_font(Font::with_name(fonts::get_chinese_font()))
        .run_with(|| (app, initial_task))
        .unwrap();
}

fn get_audio_info(file_path: &str) -> Result<()> {
    let info = AudioFile::get_info(file_path)?;
    
    if let Some(duration) = info.duration {
        println!("Duration: {}", format_duration(duration));
    } else {
        println!("Duration: Unknown");
    }
    
    println!("Audio info: {} channels, {} Hz", info.channels, info.sample_rate);
    
    if let Some(bits_per_sample) = info.bits_per_sample {
        println!("Bits per sample: {}", bits_per_sample);
    }
    
    Ok(())
}