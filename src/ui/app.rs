//! 主应用程序模块
//! 
//! 包含PlayerApp的实现和主要的应用程序逻辑。

use std::time::Duration;
use iced::{
    widget::{column, row, container},
    executor, Font,
    window::Event as WindowEvent,
    time,
    Settings,
    Theme,
    Element,
    Length,
    Subscription,
    Task,
    event::{self, Event},
    window,
};
use tokio::sync::mpsc;

use crate::error::Result;
use crate::audio::{AudioInfo, PlaybackState, PlaybackCommand, start_audio_playback, AudioFile};
use crate::playlist::{Playlist, PlaylistItem, parse_m3u_playlist, create_single_file_playlist};
use crate::utils::{is_m3u_playlist, format_duration};
use crate::config::{fonts::CHINESE_FONT, ui::MAIN_PANEL_WIDTH};
use super::Message;
use super::components::*;

/// 主应用程序结构
#[derive(Default)]
pub struct PlayerApp {
    /// 播放状态
    playback_state: PlaybackState,
    /// 音频信息
    audio_info: Option<AudioInfo>,
    /// 当前文件路径
    file_path: String,
    /// 是否正在播放
    is_playing: bool,
    /// 命令发送器
    command_sender: Option<mpsc::UnboundedSender<PlaybackCommand>>,
    /// 音频处理任务句柄
    audio_handle: Option<tokio::task::JoinHandle<()>>,
    /// 播放列表
    playlist: Playlist,
    /// 播放列表是否已加载
    playlist_loaded: bool,
}

impl PlayerApp {
    /// 创建新的应用程序实例
    pub fn new() -> Self {
        Self::default()
    }

    /// 获取应用程序标题
    pub fn title(&self) -> String {
        "音频播放器".to_string()
    }

    /// 处理应用程序消息
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::PlayPause => self.handle_play_pause(),
            Message::Stop => self.handle_stop(),
            Message::VolumeChanged(volume) => self.handle_volume_change(volume),
            Message::OpenFile => self.handle_open_file(),
            Message::FileSelected(file_path) => self.handle_file_selected(file_path),
            Message::PlaylistItemSelected(index) => self.handle_playlist_item_selected(index),
            Message::NextTrack => self.handle_next_track(),
            Message::PreviousTrack => self.handle_previous_track(),
            Message::Tick => self.handle_tick(),
            Message::PlaybackStateUpdate(state) => self.handle_playback_state_update(state),
            Message::AudioSessionStarted(sender) => self.handle_audio_session_started(sender),
            Message::EventOccurred(event) => self.handle_event_occurred(event),
        }
    }

    /// 创建应用程序视图
    pub fn view(&self) -> Element<Message> {
        let left_panel = column![
            title_view(),
            file_info_view(self.audio_info.as_ref(), &self.file_path),
            file_controls_view(),
            control_buttons_view(),
            volume_control_view(self.playback_state.volume),
            status_view(self.is_playing),
            spacer(),
        ].spacing(10)
         .width(Length::Fixed(MAIN_PANEL_WIDTH));

        let right_panel = column![playlist_view(&self.playlist, self.playlist_loaded, self.is_playing)]
            .width(Length::Fill);

        let main_content = row![left_panel, right_panel].spacing(10);
        
        let progress = column![progress_view(&self.playback_state)]
            .width(Length::Fill);

        column![main_content, progress]
            .spacing(10)
            .padding(10)
            .into()
    }

    /// 创建应用程序订阅
    pub fn subscription(&self) -> Subscription<Message> {
        use crate::config::ui::PROGRESS_UPDATE_INTERVAL;
        
        Subscription::batch([
            time::every(Duration::from_millis(PROGRESS_UPDATE_INTERVAL)).map(|_| Message::Tick),
            event::listen().map(Message::EventOccurred)
        ])
    }

    // 私有方法：处理各种消息

    fn handle_play_pause(&mut self) -> Task<Message> {
        if self.file_path.is_empty() {
            return Task::none();
        }
        
        let should_start_new_session = self.command_sender.is_none() && !self.is_playing ||
            (self.playback_state.total_duration > 0.0 && 
             self.playback_state.current_time >= self.playback_state.total_duration);
        
        if should_start_new_session {
            self.cleanup_playback_state();
            return Task::perform(
                start_audio_playback(self.file_path.clone()),
                |(sender, _handle)| Message::AudioSessionStarted(sender)
            );
        }
        
        if let Some(sender) = &self.command_sender {
            let command = if self.is_playing {
                PlaybackCommand::Pause
            } else {
                PlaybackCommand::Resume
            };
            let _ = sender.send(command);
        }
        
        Task::none()
    }

    fn handle_stop(&mut self) -> Task<Message> {
        if let Some(sender) = &self.command_sender {
            let _ = sender.send(PlaybackCommand::Stop);
        }
        self.cleanup_playback_state();
        Task::none()
    }

    fn handle_volume_change(&mut self, volume: f32) -> Task<Message> {
        self.playback_state.volume = volume;
        if let Some(sender) = &self.command_sender {
            let _ = sender.send(PlaybackCommand::SetVolume(volume));
        }
        Task::none()
    }

    fn handle_open_file(&mut self) -> Task<Message> {
        Task::perform(open_file_dialog(), Message::FileSelected)
    }

    fn handle_file_selected(&mut self, file_path: Option<String>) -> Task<Message> {
        let Some(path) = file_path else {
            return Task::none();
        };

        if is_m3u_playlist(&path) {
            match parse_m3u_playlist(&path) {
                Ok(playlist) => {
                    self.playlist = playlist;
                    self.playlist_loaded = true;
                    if let Some(first_item) = self.playlist.set_current_index(0) {
                        let file_path = first_item.path.clone();
                        self.load_audio_file(&file_path);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to parse playlist: {}", e);
                }
            }
        } else {
            self.file_path = path.clone();
            match create_single_file_playlist(&path) {
                Ok(playlist) => {
                    self.playlist = playlist;
                    self.playlist_loaded = true;
                    self.load_audio_file(&path);
                }
                Err(e) => {
                    eprintln!("Failed to create playlist: {}", e);
                }
            }
        }
        
        Task::none()
    }

    fn handle_playlist_item_selected(&mut self, index: usize) -> Task<Message> {
        if self.playlist_loaded {
            if let Some(item) = self.playlist.set_current_index(index) {
                let file_path = item.path.clone();
                self.load_audio_file(&file_path);
                self.stop_current_playback();
            }
        }
        Task::none()
    }

    fn handle_next_track(&mut self) -> Task<Message> {
        if self.playlist_loaded {
            if let Some(next_item) = self.playlist.next_item() {
                let file_path = next_item.path.clone();
                self.load_audio_file(&file_path);
                self.stop_current_playback();
            }
        }
        Task::none()
    }

    fn handle_previous_track(&mut self) -> Task<Message> {
        if self.playlist_loaded {
            if let Some(prev_item) = self.playlist.previous_item() {
                let file_path = prev_item.path.clone();
                self.load_audio_file(&file_path);
                self.stop_current_playback();
            }
        }
        Task::none()
    }

    fn handle_tick(&mut self) -> Task<Message> {
        if self.is_playing {
            self.playback_state.current_time += 0.1;
            if self.playback_state.total_duration > 0.0 && 
               self.playback_state.current_time >= self.playback_state.total_duration {
                self.handle_track_finished()
            } else {
                Task::none()
            }
        } else {
            Task::none()
        }
    }

    fn handle_playback_state_update(&mut self, state: PlaybackState) -> Task<Message> {
        self.playback_state = state.clone();
        self.is_playing = state.is_playing && !state.is_paused;
        Task::none()
    }

    fn handle_audio_session_started(&mut self, sender: mpsc::UnboundedSender<PlaybackCommand>) -> Task<Message> {
        self.command_sender = Some(sender);
        self.is_playing = true;
        self.playback_state.is_playing = true;
        self.playback_state.is_paused = false;
        Task::none()
    }

    fn handle_event_occurred(&mut self, event: Event) -> Task<Message> {
        if let Event::Window(WindowEvent::Closed) = event {
            self.cleanup_on_exit();
        }
        Task::none()
    }

    // 辅助方法

    fn load_audio_file(&mut self, file_path: &str) {
        self.file_path = file_path.to_string();
        if let Ok(audio_file) = AudioFile::open(file_path) {
            self.audio_info = Some(audio_file.info.clone());
            self.playback_state.total_duration = audio_file.info.duration.unwrap_or(0.0);
        }
    }

    fn stop_current_playback(&mut self) {
        if let Some(sender) = &self.command_sender {
            let _ = sender.send(PlaybackCommand::Stop);
        }
        self.cleanup_playback_state();
    }

    fn cleanup_playback_state(&mut self) {
        self.is_playing = false;
        self.playback_state.is_playing = false;
        self.playback_state.is_paused = false;
        self.playback_state.current_time = 0.0;
        self.command_sender = None;
    }

    fn cleanup_on_exit(&mut self) {
        if let Some(sender) = &self.command_sender {
            let _ = sender.send(PlaybackCommand::Stop);
            std::thread::sleep(Duration::from_millis(100));
        }
        
        self.is_playing = false;
        self.playback_state.is_playing = false;
        self.playback_state.is_paused = false;
        self.command_sender = None;
        self.audio_handle = None;
    }

    fn handle_track_finished(&mut self) -> Task<Message> {
        self.playback_state.current_time = self.playback_state.total_duration;
        self.cleanup_playback_state();
        
        if self.playlist_loaded {
            if let Some(next_item) = self.playlist.next_item() {
                let file_path = next_item.path.clone();
                self.load_audio_file(&file_path);
                return Task::perform(
                    start_audio_playback(file_path),
                    |(sender, _handle)| Message::AudioSessionStarted(sender)
                );
            }
        }
        
        Task::none()
    }
}

/// 打开文件对话框
async fn open_file_dialog() -> Option<String> {
    let file = rfd::AsyncFileDialog::new()
        .add_filter("Audio Files", &["mp3", "flac", "wav", "ogg", "aac", "m4a", "m4s"])
        .add_filter("Playlist Files", &["m3u", "m3u8"])
        .add_filter("All Files", &["*"])
        .pick_file()
        .await;
    
    file.map(|f| f.path().to_string_lossy().to_string())
} 