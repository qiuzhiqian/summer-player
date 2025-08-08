use clap::Parser;
use iced::{Font, window};
use sys_locale::get_locale;

use summer_player::{
    PlayerApp,
    audio::{AudioFile, list_audio_devices},
    utils::format_duration,
    error::Result,
    config::fonts,
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

    #[arg(short = 'L', long, help = "Set interface language (en, zh-CN)")]
    language: Option<String>,

    #[arg(help = "Path to audio file")]
    file: Option<String>,
}

fn main() {
    let args = Cli::parse();
    
    // 设置语言环境
    let locale = if let Some(lang) = &args.language {
        // 验证用户指定的语言是否支持
        let supported_languages = ["en", "zh-CN"];
        if supported_languages.contains(&lang.as_str()) {
            println!("Language set to: {}", lang);
            lang.clone()
        } else {
            eprintln!("Error: Unsupported language '{}'. Supported languages: {}", 
                     lang, supported_languages.join(", "));
            eprintln!("Falling back to system locale detection.");
            // 回退到系统语言检测
            let detected_locale = get_locale().unwrap_or_else(|| String::from("en-US"));
            let mapped_locale = match detected_locale.as_str() {
                s if s.starts_with("zh") => "zh-CN",
                _ => "en",
            };
            println!("Detected locale: {}, using: {}", detected_locale, mapped_locale);
            mapped_locale.to_string()
        }
    } else {
        // 自动检测系统语言
        let detected_locale = get_locale().unwrap_or_else(|| String::from("en-US"));
        let mapped_locale = match detected_locale.as_str() {
            s if s.starts_with("zh") => "zh-CN",
            _ => "en",
        };
        println!("Detected locale: {}, using: {}", detected_locale, mapped_locale);
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
    let (app, initial_task) = PlayerApp::new(initial_file, locale.clone());
    
    iced::application("Summer Audio Player", PlayerApp::update, PlayerApp::view)
        .subscription(PlayerApp::subscription)
        .theme(PlayerApp::theme)
        .window(window::Settings {
            icon: Some(icon),
            ..window::Settings::default()
        })
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