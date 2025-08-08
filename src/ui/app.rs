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
    alignment::Vertical,
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
use super::theme::{AppThemeVariant, AppTheme};

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
    /// 当前页面类型
    current_page: PageType,
    /// 当前视图类型（主页面内的视图切换）
    current_view: ViewType,
    /// 动画状态（使用 anim-rs）
    view_animation: ViewTransitionAnimation,
    /// 当前歌词
    current_lyrics: Option<Lyrics>,
    /// 当前窗口大小
    window_size: (f32, f32), // (width, height)
    /// 当前主题
    current_theme: AppThemeVariant,
    /// 当前语言
    current_language: String,
    /// 当前播放模式
    play_mode: PlayMode,
}

impl PlayerApp {
    /// 创建新的应用程序实例
    pub fn new(initial_file: Option<String>, current_language: String) -> (Self, Task<Message>) {
        let mut app = Self {
            window_size: (1000.0, 700.0), // 初始窗口大小
            current_language,
            ..Self::default()
        };
        
        // 如果有初始文件，加载它并开始播放
        if let Some(file_path) = initial_file {
            if !file_path.is_empty() {
                app.handle_initial_file_load(&file_path);
                // 自动开始播放
                if !app.file_path.is_empty() {
                    let file_path_clone = app.file_path.clone();
                    return (app, Task::perform(
                        start_audio_playback(file_path_clone),
                        |(sender, _handle)| Message::AudioSessionStarted(sender)
                    ));
                }
            }
        }
        
        (app, Task::none())
    }

    /// 获取应用程序标题
    pub fn title(&self) -> String {
        "音频播放器".to_string()
    }

    /// 处理应用程序消息
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::PlayPause => self.handle_play_pause(),
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
            Message::ProgressChanged(progress) => self.handle_progress_changed(progress),
            Message::ToggleTheme => self.handle_toggle_theme(),
            Message::PageChanged(page) => self.handle_page_changed(page),
            Message::TogglePlayMode => self.handle_toggle_play_mode(),
        }
    }

    /// 获取当前主题
    pub fn theme(&self) -> iced::Theme {
        self.current_theme.to_iced_theme()
    }

    /// 创建应用程序视图
    pub fn view(&self) -> Element<Message> {
        // 左侧导航栏
        let navigation = navigation_sidebar(&self.current_page);
        
        // 主内容区域根据当前页面显示不同内容
        let main_content = match self.current_page {
            PageType::Home => self.create_home_page(),
            PageType::Settings => settings_page(&self.current_theme, &self.current_language),
        };

        // 整体布局：导航栏 + 主内容
        row![
            container(navigation)
                .style(AppTheme::main_section_container())
                .width(Length::Shrink)
                .height(Length::Fill),
            container(main_content)
                .style(AppTheme::background_container())
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(16) // 增加内边距
        ]
        .spacing(12) // 增加间距
        .padding(8) // 整体外边距
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



    fn handle_open_file(&mut self) -> Task<Message> {
        Task::perform(open_file_dialog(), Message::FileSelected)
    }

    fn handle_file_selected(&mut self, file_path: Option<String>) -> Task<Message> {
        let Some(path) = file_path else {
            return Task::none();
        };

        // 记录是否之前正在播放
        let was_playing = self.is_playing;

        if is_m3u_playlist(&path) {
            match parse_m3u_playlist(&path) {
                Ok(playlist) => {
                    self.playlist = playlist;
                    self.playlist_loaded = true;
                    if let Some(first_item) = self.playlist.set_current_index(0) {
                        let file_path = first_item.path.clone();
                        self.load_audio_file(&file_path);
                        
                        // 停止当前播放，然后如果之前正在播放则启动新的播放会话
                        self.stop_current_playback();
                        
                        if was_playing {
                            return Task::perform(
                                start_audio_playback(file_path),
                                |(sender, _handle)| Message::AudioSessionStarted(sender)
                            );
                        }
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
                    
                    // 停止当前播放，然后如果之前正在播放则启动新的播放会话
                    self.stop_current_playback();
                    
                    if was_playing {
                        return Task::perform(
                            start_audio_playback(path),
                            |(sender, _handle)| Message::AudioSessionStarted(sender)
                        );
                    }
                }
                Err(e) => {
                    eprintln!("Failed to create playlist: {}", e);
                }
            }
        }
        
        Task::none()
    }

    fn handle_initial_file_load(&mut self, file_path: &str) {
        if is_m3u_playlist(file_path) {
            match parse_m3u_playlist(file_path) {
                Ok(playlist) => {
                    self.playlist = playlist;
                    self.playlist_loaded = true;
                    if let Some(first_item) = self.playlist.set_current_index(0) {
                        let file_path = first_item.path.clone();
                        self.load_audio_file(&file_path);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to parse initial playlist: {}", e);
                }
            }
        } else {
            let path = file_path.to_string();
            match create_single_file_playlist(&path) {
                Ok(playlist) => {
                    self.playlist = playlist;
                    self.playlist_loaded = true;
                    self.load_audio_file(&path);
                }
                Err(e) => {
                    eprintln!("Failed to create initial playlist: {}", e);
                }
            }
        }
    }

    fn handle_playlist_item_selected(&mut self, index: usize) -> Task<Message> {
        if self.playlist_loaded {
            if let Some(item) = self.playlist.set_current_index(index) {
                let file_path = item.path.clone();
                self.load_audio_file(&file_path);
                
                // 停止当前播放，然后立即启动新歌曲的播放
                self.stop_current_playback();
                
                // 启动新的音频播放会话
                return Task::perform(
                    start_audio_playback(file_path),
                    |(sender, _handle)| Message::AudioSessionStarted(sender)
                );
            }
        }
        Task::none()
    }

    fn handle_next_track(&mut self) -> Task<Message> {
        if self.playlist_loaded {
            let (next_item, should_restart) = self.playlist.next_item_with_mode(&self.play_mode);
            if let Some(item) = next_item {
                let file_path = item.path.clone();
                
                if should_restart {
                    // 单曲循环或随机播放到同一首歌 - 重新开始播放
                    self.stop_current_playback();
                    return Task::perform(
                        start_audio_playback(file_path),
                        |(sender, _handle)| Message::AudioSessionStarted(sender)
                    );
                } else {
                    // 切换到不同的歌曲
                    self.load_audio_file(&file_path);
                    self.stop_current_playback();
                    return Task::perform(
                        start_audio_playback(file_path),
                        |(sender, _handle)| Message::AudioSessionStarted(sender)
                    );
                }
            }
        }
        Task::none()
    }

    fn handle_previous_track(&mut self) -> Task<Message> {
        if self.playlist_loaded {
            let (prev_item, should_restart) = self.playlist.previous_item_with_mode(&self.play_mode);
            if let Some(item) = prev_item {
                let file_path = item.path.clone();
                
                if should_restart {
                    // 单曲循环或随机播放到同一首歌 - 重新开始播放
                    self.stop_current_playback();
                    return Task::perform(
                        start_audio_playback(file_path),
                        |(sender, _handle)| Message::AudioSessionStarted(sender)
                    );
                } else {
                    // 切换到不同的歌曲
                    self.load_audio_file(&file_path);
                    self.stop_current_playback();
                    return Task::perform(
                        start_audio_playback(file_path),
                        |(sender, _handle)| Message::AudioSessionStarted(sender)
                    );
                }
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

    fn handle_progress_changed(&mut self, progress: f32) -> Task<Message> {
        // 如果没有加载文件或总时长为0，忽略进度变化
        if self.file_path.is_empty() || self.playback_state.total_duration <= 0.0 {
            return Task::none();
        }
        
        // 计算新的播放时间
        let new_time = progress as f64 * self.playback_state.total_duration;
        
        // 更新当前时间状态，提供即时UI反馈
        self.playback_state.current_time = new_time;
        
        // 如果有播放会话，发送跳转命令
        if let Some(sender) = &self.command_sender {
            if let Err(e) = sender.send(PlaybackCommand::Seek(new_time)) {
                eprintln!("Failed to send seek command: {}", e);
            } else {
                println!("UI: Seek command sent for {:.2}s ({:.1}%)", new_time, progress * 100.0);
            }
        }
        
        Task::none()
    }

    fn handle_toggle_theme(&mut self) -> Task<Message> {
        self.current_theme = self.current_theme.toggle();
        Task::none()
    }

    fn handle_page_changed(&mut self, page: PageType) -> Task<Message> {
        self.current_page = page;
        Task::none()
    }

    fn handle_toggle_play_mode(&mut self) -> Task<Message> {
        self.play_mode = self.play_mode.next();
        Task::none()
    }

    // 辅助方法

    fn load_audio_file(&mut self, file_path: &str) {
        self.file_path = file_path.to_string();
        
        // 重置播放状态
        self.playback_state.current_time = 0.0;
        self.playback_state.current_samples = 0;
        
        // 加载音频信息
        if let Ok(audio_file) = AudioFile::open(file_path) {
            self.audio_info = Some(audio_file.info.clone());
            self.playback_state.total_duration = audio_file.info.duration.unwrap_or(0.0);
            self.playback_state.sample_rate = audio_file.info.sample_rate;
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
            let (next_item, should_restart) = self.playlist.next_item_with_mode(&self.play_mode);
            if let Some(item) = next_item {
                let file_path = item.path.clone();
                
                if should_restart {
                    // 单曲循环 - 重新开始播放当前歌曲
                    return Task::perform(
                        start_audio_playback(file_path),
                        |(sender, _handle)| Message::AudioSessionStarted(sender)
                    );
                } else {
                    // 切换到下一首歌曲
                    self.load_audio_file(&file_path);
                    return Task::perform(
                        start_audio_playback(file_path),
                        |(sender, _handle)| Message::AudioSessionStarted(sender)
                    );
                }
            }
        }
        
        Task::none()
    }

    /// 创建主页面内容
    fn create_home_page(&self) -> Element<Message> {
        let left_panel = column![
            file_info_view(self.audio_info.as_ref(), &self.file_path),
            spacer(),
        ].spacing(16) // 增加间距
         .width(Length::Fixed(MAIN_PANEL_WIDTH + 20.0)) // 稍微增加宽度
         .height(Length::Fill);

        // 右侧面板根据当前视图类型显示不同内容
        let right_panel_content = if self.view_animation.is_active() {
            // 动画期间同时显示两个视图，通过宽度比例实现滑动
            self.create_sliding_animation_view()
        } else {
            // 正常状态显示对应内容
            match self.current_view {
                ViewType::Playlist => playlist_view(&self.playlist, self.playlist_loaded, self.is_playing),
                ViewType::Lyrics => lyrics_view(&self.file_path, self.is_playing, self.playback_state.current_time, self.current_lyrics.clone(), self.window_size.1),
            }
        };

        let right_panel = column![
            right_panel_content,
        ].spacing(16).width(Length::Fill); // 增加间距

        let main_content = row![left_panel, right_panel].spacing(20); // 增加左右面板间距
        
        // 底部区域：控制按钮 + 进度条 + 功能按钮
        let bottom_section = container(
            row![
                container(control_buttons_view(self.is_playing))
                    .width(Length::Fixed(190.0))
                    .height(Length::Shrink),
                column![progress_view(&self.playback_state)]
                    .width(Length::Fill),
                // 右侧功能按钮组
                row![
                    compact_file_button(),
                    compact_play_mode_button(self.play_mode.clone()),
                    compact_view_toggle_button(self.current_view.clone()),
                ].spacing(6).align_y(Vertical::Center)
            ].spacing(10).align_y(Vertical::Center)
        )
        .style(AppTheme::glass_card_container()) // 使用毛玻璃效果
        .padding(8);

        column![
            main_content, 
            bottom_section
        ]
        .spacing(16) // 减少主内容和底部的间距，使布局更紧凑
        .into()
    }

    fn create_sliding_animation_view(&self) -> Element<Message> {
        // 获取动画进度（已经通过 anim-rs 进行了缓动处理）
        let progress = self.view_animation.progress();
        
        // 获取播放列表和歌词视图内容
        let playlist_content = playlist_view(&self.playlist, self.playlist_loaded, self.is_playing);
        let lyrics_content = lyrics_view(&self.file_path, self.is_playing, self.playback_state.current_time, self.current_lyrics.clone(), self.window_size.1);
        
        // 判断滑动方向：Playlist -> Lyrics (歌词从下方向上滑动), Lyrics -> Playlist (歌词向下滑出)
        let is_switching_to_lyrics = matches!(
            (&self.current_view, self.view_animation.target_view()),
            (ViewType::Playlist, Some(ViewType::Lyrics))
        );
        
        // 使用线性进度，确保动画均匀变化
        let adjusted_progress = progress.clamp(0.0, 1.0);
        
        // 调试：打印动画进度（可以在需要时启用）
        // println!("Animation progress: {:.3} -> {:.3}", progress, adjusted_progress);
        
        if is_switching_to_lyrics {
            // 切换到歌词视图：歌词从下方向上滑入
            let slide_in_progress = adjusted_progress; // 滑入进度从0到1
            
            // 使用column布局实现滑入效果
            let visible_height_percent = slide_in_progress * 100.0;
            let hidden_height_percent = 100.0 - visible_height_percent;
            
            // 调试输出
            // println!("Slide IN: visible={:.1}%, hidden={:.1}%", visible_height_percent, hidden_height_percent);
            
            column![
                // 播放列表区域，高度逐渐减少
                container(playlist_content)
                    .height(Length::FillPortion((hidden_height_percent + 1.0) as u16))
                    .width(Length::Fill),
                // 歌词区域，从底部向上增长
                container(lyrics_content)
                    .height(Length::FillPortion((visible_height_percent + 1.0) as u16))
                    .width(Length::Fill),
            ]
            .spacing(0)
            .into()
        } else {
            // 切换到播放列表：歌词从上向下滑出视图区域
            let slide_out_progress = adjusted_progress; // 滑出进度从0到1
            
            // 关键改变：使用上方空白空间来"推动"歌词向下滑出
            let top_spacer_percent = slide_out_progress * 100.0; // 上方空白空间逐渐增加
            let lyrics_visible_percent = (1.0 - slide_out_progress) * 100.0; // 歌词可见高度逐渐减少
            
            // 调试输出
            // println!("Slide OUT: spacer={:.1}%, lyrics_visible={:.1}%", top_spacer_percent, lyrics_visible_percent);
            
            // 始终使用三层布局，确保动画连续性，避免突然跳转
            column![
                // 上方空白空间，逐渐增加，"推动"歌词向下
                container(iced::widget::Space::new(Length::Fill, Length::FillPortion((top_spacer_percent + 1.0) as u16)))
                    .width(Length::Fill),
                // 歌词内容，被推向下方，逐渐减少直到完全消失
                container(lyrics_content)
                    .height(Length::FillPortion((lyrics_visible_percent + 1.0) as u16))
                    .width(Length::Fill),
                // 播放列表在底部作为背景，始终存在
                container(playlist_content)
                    .height(Length::Fill)
                    .width(Length::Fill),
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

 