use clap::Parser;
use iced::{Font, window};
use sys_locale::get_locale;

use summer_player::{
    PlayerApp,
    audio::{AudioFile, list_audio_devices},
    utils::format_duration,
    error::Result,
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
    let current_locale = get_locale().unwrap_or_else(|| String::from("en-US"));
    rust_i18n::set_locale(&current_locale);

    let args = Cli::parse();
    
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
    let (app, initial_task) = PlayerApp::new(initial_file);
    
    iced::application("Summer Audio Player", PlayerApp::update, PlayerApp::view)
        .subscription(PlayerApp::subscription)
        .theme(PlayerApp::theme)
        .window(window::Settings {
            icon: Some(icon),
            ..window::Settings::default()
        })
        .default_font(Font::with_name("Noto Sans CJK SC"))
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