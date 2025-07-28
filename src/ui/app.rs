//! 主应用程序模块
//! 
//! 包含PlayerApp的实现和主要的应用程序逻辑。

use std::time::Duration;
use iced::{
    widget::{column, row, container},
    window::Event as WindowEvent,
    time,
    Element,
    Length,
    Subscription,
    Task,
    event::{self, Event},
};
use tokio::sync::mpsc;

use crate::audio::{AudioInfo, PlaybackState, PlaybackCommand, start_audio_playback, AudioFile};
use crate::playlist::{Playlist, parse_m3u_playlist, create_single_file_playlist};
use crate::lyrics::{Lyrics, load_lyrics_for_audio};
use crate::utils::is_m3u_playlist;
use crate::config::ui::MAIN_PANEL_WIDTH;
use super::Message;
use super::components::*;
use super::animation::ViewTransitionAnimation;

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
    /// 当前视图类型
    current_view: ViewType,
    /// 动画状态（使用 anim-rs）
    view_animation: ViewTransitionAnimation,
    /// 当前歌词
    current_lyrics: Option<Lyrics>,
    /// 当前窗口大小
    window_size: (f32, f32), // (width, height)
}

impl PlayerApp {
    /// 创建新的应用程序实例
    pub fn new() -> Self {
        Self {
            window_size: (1000.0, 700.0), // 初始窗口大小
            ..Self::default()
        }
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
            Message::OpenFile => self.handle_open_file(),
            Message::FileSelected(file_path) => self.handle_file_selected(file_path),
            Message::PlaylistItemSelected(index) => self.handle_playlist_item_selected(index),
            Message::NextTrack => self.handle_next_track(),
            Message::PreviousTrack => self.handle_previous_track(),
            Message::Tick => self.handle_tick(),
            Message::PlaybackStateUpdate(state) => self.handle_playback_state_update(state),
            Message::AudioSessionStarted(sender) => self.handle_audio_session_started(sender),
            Message::EventOccurred(event) => self.handle_event_occurred(event),
            Message::ToggleView => self.handle_toggle_view(),
            Message::AnimationTick => self.handle_animation_tick(),
            Message::WindowResized(width, height) => self.handle_window_resized(width, height),
        }
    }

    /// 创建应用程序视图
    pub fn view(&self) -> Element<Message> {
        let left_panel = column![
            title_view(),
            file_info_view(self.audio_info.as_ref(), &self.file_path),
            file_controls_view(),
            control_buttons_view(),
            status_view(self.is_playing),
            spacer(),
        ].spacing(10)
         .width(Length::Fixed(MAIN_PANEL_WIDTH))
         .height(Length::Fill);

        // 右侧面板根据当前视图类型显示不同内容
        let right_panel_content = if self.view_animation.is_active() {
            // 动画期间同时显示两个视图，通过宽度比例实现滑动
            self.create_sliding_animation_view()
        } else {
            // 正常状态显示对应内容
            match self.current_view {
                ViewType::Playlist => playlist_view(&self.playlist, self.playlist_loaded, self.is_playing),
                ViewType::Lyrics => lyrics_view(&self.file_path, self.is_playing, self.playback_state.current_time, &self.current_lyrics, self.window_size.1),
            }
        };

        let right_panel = column![
            view_toggle_button(&self.current_view),
            right_panel_content,
        ].spacing(10).width(Length::Fill);

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
        
        let mut subscriptions = vec![
            time::every(Duration::from_millis(PROGRESS_UPDATE_INTERVAL)).map(|_| Message::Tick),
            event::listen().map(|event| {
                match event {
                    Event::Window(WindowEvent::Resized(size)) => {
                        Message::WindowResized(size.width, size.height)
                    },
                    _ => Message::EventOccurred(event),
                }
            }),
        ];
        
        // 如果正在动画中，添加动画定时器
        if self.view_animation.is_active() {
            subscriptions.push(
                time::every(Duration::from_millis(16)).map(|_| Message::AnimationTick) // ~60 FPS
            );
        }
        
        Subscription::batch(subscriptions)
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
            
            // 发送命令到播放器
            if let Err(e) = sender.send(command.clone()) {
                eprintln!("Failed to send playback command: {}", e);
                return Task::none();
            }
            
            // 立即更新UI状态以提供即时反馈
            match command {
                PlaybackCommand::Pause => {
                    self.is_playing = false;
                    self.playback_state.is_playing = false;
                    self.playback_state.is_paused = true;
                }
                PlaybackCommand::Resume => {
                    self.is_playing = true;
                    self.playback_state.is_playing = true;
                    self.playback_state.is_paused = false;
                }
                _ => {}
            }
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
        // 只有在真正播放状态下（is_playing = true 且 is_paused = false）才更新时间
        if self.is_playing && !self.playback_state.is_paused {
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

    fn handle_toggle_view(&mut self) -> Task<Message> {
        // 如果已经在动画中，忽略新的切换请求
        if self.view_animation.is_active() {
            return Task::none();
        }
        
        // 确定目标视图
        let target_view = match self.current_view {
            ViewType::Playlist => ViewType::Lyrics,
            ViewType::Lyrics => ViewType::Playlist,
        };
        
        // 启动动画
        self.view_animation.start_transition(target_view);
        
        Task::none()
    }

    fn handle_animation_tick(&mut self) -> Task<Message> {
        // 在更新动画之前先获取目标视图，因为动画完成时会清空target_view
        let target_view = self.view_animation.target_view().cloned();
        
        if self.view_animation.update() {
            // 动画完成时切换视图
            if let Some(target_view) = target_view {
                self.current_view = target_view;
            }
        }
        Task::none()
    }

    fn handle_window_resized(&mut self, width: f32, height: f32) -> Task<Message> {
        self.window_size = (width, height);
        Task::none()
    }

    // 辅助方法

    fn load_audio_file(&mut self, file_path: &str) {
        self.file_path = file_path.to_string();
        
        // 加载音频信息
        if let Ok(audio_file) = AudioFile::open(file_path) {
            self.audio_info = Some(audio_file.info.clone());
            self.playback_state.total_duration = audio_file.info.duration.unwrap_or(0.0);
        }
        
        // 加载歌词
        match load_lyrics_for_audio(file_path) {
            Ok(lyrics) => {
                self.current_lyrics = lyrics;
                if self.current_lyrics.is_some() {
                    println!("歌词加载成功: {}", file_path);
                }
            }
            Err(e) => {
                eprintln!("加载歌词失败: {}", e);
                self.current_lyrics = None;
            }
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
        if let Some(handle) = self.audio_handle.take() {
            handle.abort();
        }
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

    fn create_sliding_animation_view(&self) -> Element<Message> {
        // 获取动画进度（已经通过 anim-rs 进行了缓动处理）
        let progress = self.view_animation.progress();
        
        // 获取当前视图和目标视图
        let current_view_content = match self.current_view {
            ViewType::Playlist => playlist_view(&self.playlist, self.playlist_loaded, self.is_playing),
            ViewType::Lyrics => lyrics_view(&self.file_path, self.is_playing, self.playback_state.current_time, &self.current_lyrics, self.window_size.1),
        };
        
        let target_view_content = if let Some(target) = self.view_animation.target_view() {
            match target {
                ViewType::Playlist => playlist_view(&self.playlist, self.playlist_loaded, self.is_playing),
                ViewType::Lyrics => lyrics_view(&self.file_path, self.is_playing, self.playback_state.current_time, &self.current_lyrics, self.window_size.1),
            }
        } else {
            // 如果没有目标视图，生成与当前视图相同的内容
            match self.current_view {
                ViewType::Playlist => playlist_view(&self.playlist, self.playlist_loaded, self.is_playing),
                ViewType::Lyrics => lyrics_view(&self.file_path, self.is_playing, self.playback_state.current_time, &self.current_lyrics, self.window_size.1),
            }
        };
        
        // 判断滑动方向：Playlist -> Lyrics (向左滑动), Lyrics -> Playlist (向右滑动)
        let is_slide_left = matches!(
            (&self.current_view, self.view_animation.target_view()),
            (ViewType::Playlist, Some(ViewType::Lyrics))
        );
        
        // 为了防止闪烁，在动画接近结束时提前给目标视图更多空间
        let adjusted_progress = if progress > 0.9 {
            // 最后10%时加速完成，使切换更干脆
            0.9 + (progress - 0.9) * 10.0
        } else {
            progress
        }.clamp(0.0, 1.0);
        
        // 计算宽度比例，确保平滑过渡
        let min_width = 0.02; // 最小2%宽度
        let current_width = (1.0 - adjusted_progress).max(min_width);
        let target_width = adjusted_progress.max(min_width);
        
        // 转换为整数比例，确保总和不超过100
        let total_width = current_width + target_width;
        let current_portion = ((current_width / total_width * 98.0) + 1.0) as u16;
        let target_portion = ((target_width / total_width * 98.0) + 1.0) as u16;
        
        // 创建滑动效果
        if is_slide_left {
            // 向左滑动：当前视图在左，目标视图在右
            row![
                container(current_view_content)
                    .width(Length::FillPortion(current_portion))
                    .height(Length::Fill),
                container(target_view_content)
                    .width(Length::FillPortion(target_portion))
                    .height(Length::Fill),
            ]
            .spacing(0)
            .into()
        } else {
            // 向右滑动：目标视图在左，当前视图在右
            row![
                container(target_view_content)
                    .width(Length::FillPortion(target_portion))
                    .height(Length::Fill),
                container(current_view_content)
                    .width(Length::FillPortion(current_portion))
                    .height(Length::Fill),
            ]
            .spacing(0)
            .into()
        }
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

 