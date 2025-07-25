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
use crate::utils::is_m3u_playlist;
use crate::config::ui::MAIN_PANEL_WIDTH;
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
    /// 当前视图类型
    current_view: ViewType,
    /// 动画状态
    animation_state: AnimationState,
}

/// 动画状态
#[derive(Debug, Clone)]
pub struct AnimationState {
    /// 是否正在动画中
    pub is_animating: bool,
    /// 动画进度 (0.0 到 1.0)
    pub progress: f32,
    /// 目标视图
    pub target_view: Option<ViewType>,
    /// 动画持续时间 (毫秒)
    pub duration_ms: u64,
    /// 动画开始时间戳
    pub start_time: Option<std::time::Instant>,
}

impl Default for AnimationState {
    fn default() -> Self {
        Self {
            is_animating: false,
            progress: 0.0,
            target_view: None,
            duration_ms: 500, // 500ms 动画，让滑动效果更明显
            start_time: None,
        }
    }
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
         .width(Length::Fixed(MAIN_PANEL_WIDTH));

        // 右侧面板根据当前视图类型显示不同内容
        let right_panel_content = if self.animation_state.is_animating {
            // 动画期间同时显示两个视图，通过宽度比例实现滑动
            let progress = self.animation_state.progress;
            
            // 获取当前视图和目标视图
            let current_view_content = match self.current_view {
                ViewType::Playlist => playlist_view(&self.playlist, self.playlist_loaded, self.is_playing),
                ViewType::Lyrics => lyrics_view(&self.file_path, self.is_playing, self.playback_state.current_time),
            };
            
            let target_view_content = if let Some(target) = &self.animation_state.target_view {
                match target {
                    ViewType::Playlist => playlist_view(&self.playlist, self.playlist_loaded, self.is_playing),
                    ViewType::Lyrics => lyrics_view(&self.file_path, self.is_playing, self.playback_state.current_time),
                }
            } else {
                // 如果没有目标视图，生成与当前视图相同的内容
                match self.current_view {
                    ViewType::Playlist => playlist_view(&self.playlist, self.playlist_loaded, self.is_playing),
                    ViewType::Lyrics => lyrics_view(&self.file_path, self.is_playing, self.playback_state.current_time),
                }
            };
            
            // 判断滑动方向：Playlist -> Lyrics (向左滑动), Lyrics -> Playlist (向右滑动)
            let is_slide_left = matches!(
                (&self.current_view, &self.animation_state.target_view),
                (ViewType::Playlist, Some(ViewType::Lyrics))
            );
            
            // 计算滑动比例，使用缓动函数让动画更平滑
            let eased_progress = ease_in_out_cubic(progress);
            let current_width = (1.0 - eased_progress).max(0.01); // 确保不为0
            let target_width = eased_progress.max(0.01); // 确保不为0
            
            // 转换为整数比例 (1-99)
            let current_portion = ((current_width * 98.0) + 1.0) as u16;
            let target_portion = ((target_width * 98.0) + 1.0) as u16;
            
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
        } else {
            // 正常状态显示对应内容
            match self.current_view {
                ViewType::Playlist => playlist_view(&self.playlist, self.playlist_loaded, self.is_playing),
                ViewType::Lyrics => lyrics_view(&self.file_path, self.is_playing, self.playback_state.current_time),
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
            event::listen().map(Message::EventOccurred),
        ];
        
        // 如果正在动画中，添加动画定时器
        if self.animation_state.is_animating {
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

    fn handle_toggle_view(&mut self) -> Task<Message> {
        // 如果已经在动画中，忽略新的切换请求
        if self.animation_state.is_animating {
            return Task::none();
        }
        
        // 确定目标视图
        let target_view = match self.current_view {
            ViewType::Playlist => ViewType::Lyrics,
            ViewType::Lyrics => ViewType::Playlist,
        };
        
        // 启动动画
        self.animation_state.is_animating = true;
        self.animation_state.progress = 0.0;
        self.animation_state.target_view = Some(target_view);
        self.animation_state.start_time = Some(std::time::Instant::now());
        
        Task::none()
    }

    fn handle_animation_tick(&mut self) -> Task<Message> {
        if self.animation_state.is_animating {
            let elapsed = self.animation_state.start_time.unwrap().elapsed().as_millis();
            let progress = (elapsed as f32 / self.animation_state.duration_ms as f32).clamp(0.0, 1.0);
            
            // 在动画中点切换视图
            if progress >= 0.5 && self.animation_state.target_view.is_some() {
                self.current_view = self.animation_state.target_view.take().unwrap();
            }
            
            if progress >= 1.0 {
                self.animation_state.is_animating = false;
                self.animation_state.progress = 1.0;
                self.animation_state.target_view = None;
                self.animation_state.start_time = None;
                return Task::none();
            }
            
            self.animation_state.progress = progress;
            Task::none()
        } else {
            Task::none()
        }
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

/// 缓动函数：ease-in-out cubic
/// 让动画开始和结束时更平滑
fn ease_in_out_cubic(t: f32) -> f32 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    }
} 