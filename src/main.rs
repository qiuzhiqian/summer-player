use clap::Parser;
use iced::Font;

use player::{
    PlayerApp, 
    audio::{AudioFile, list_audio_devices},
    utils::format_duration,
    error::Result,
};

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
    
    let app = PlayerApp::new();
    
    iced::application("player", PlayerApp::update, PlayerApp::view)
        .subscription(PlayerApp::subscription)
        /*.font(include_bytes!("../fonts/NotoColorEmoji.ttf"))*/
        .default_font(Font::with_name("Noto Sans CJK SC"))
        .run_with(|| (app, iced::Task::none()))
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