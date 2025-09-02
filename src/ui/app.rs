//! 主应用程序模块
//! 
//! 包含PlayerApp的实现和主要的应用程序逻辑。

use std::time::Duration;
use std::collections::HashSet;
use iced::{
    widget::{column, row, container},
    window::Event as WindowEvent,
    time,
    Element,
    Length,
    Subscription,
    Task,
    event::{self, Event},
    alignment::{Horizontal, Vertical},
};
use iced::advanced::text::Shaping;
use tokio::sync::mpsc;

use crate::audio::{AudioInfo, PlaybackState, PlaybackCommand, start_audio_playback, AudioSource};
use crate::audio::file::estimate_duration_by_parsing;
use crate::playlist::{Playlist, PlaylistManager, PlaylistExtraInfo};
use crate::lyrics::Lyrics;
use crate::utils::is_m3u_playlist;
use crate::config::AppConfig;
use super::Message;
use super::components::*;
use super::theme::{AppThemeVariant};
use super::widgets::StyledContainer;
use super::widgets::StyledText;
use super::widgets::styled_text::TextStyle;

const RIGHT_PANEL_WIDTH: f32 = 720.0;
const LEFT_INFO_WIDTH: f32 = 260.0;

/// 主应用程序结构
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
    /// 播放列表管理器
    playlist_manager: PlaylistManager,
    /// 播放列表是否已加载
    playlist_loaded: bool,
    /// 当前页面类型
    current_page: PageType,
    /// 当前视图类型（主页面内的视图切换）
    current_view: ViewType,
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
    /// 应用程序配置
    app_config: AppConfig,
    /// 是否正在创建播放列表
    creating_playlist: bool,
    /// 创建播放列表的名称
    creating_playlist_name: String,
}

impl Default for PlayerApp {
    fn default() -> Self {
        Self {
            playback_state: PlaybackState::default(),
            audio_info: None,
            file_path: String::new(),
            is_playing: false,
            command_sender: None,
            audio_handle: None,
            playlist_manager: PlaylistManager::new(),
            playlist_loaded: false,
            current_page: PageType::default(),
            current_view: ViewType::default(),
            current_lyrics: None,
            window_size: (1000.0, 700.0),
            current_theme: AppThemeVariant::default(),
            current_language: "en".to_string(),
            play_mode: PlayMode::default(),
            app_config: AppConfig::default(),
            creating_playlist: false,
            creating_playlist_name: String::new(),
        }
    }
}

impl PlayerApp {
    /// 创建新的应用程序实例
    pub fn new(initial_file: Option<String>, current_language: String) -> (Self, Task<Message>) {
        // 加载配置
        let mut config = AppConfig::load();
        
        // 如果传入了语言参数，则覆盖配置中的语言设置
        if !current_language.is_empty() {
            config.ui.language = current_language;
        }

        Self::new_with_config(initial_file, config)
    }

    /// 使用指定配置创建新的应用程序实例
    pub fn new_with_config(_initial_file: Option<String>, config: AppConfig) -> (Self, Task<Message>) {

        let mut app = Self {
            window_size: (config.window.width, config.window.height),
            current_language: config.ui.language.clone(),
            current_theme: config.ui.theme.clone().into(),
            current_page: config.ui.current_page.clone().into(),
            current_view: config.ui.current_view.clone().into(),
            play_mode: config.player.play_mode.clone().into(),
            app_config: config,
            ..Self::default()
        };
        
        // 自动加载配置目录下的播放列表文件
        let loaded_count = app.playlist_manager.load_config_playlists();
        if loaded_count > 0 {
            println!("自动加载了 {} 个播放列表文件", loaded_count);
        }
        
        // 如果配置中有最后播放的文件且没有传入初始文件，使用配置中的文件
        //let file_to_load = initial_file.or_else(|| app.app_config.player.last_file_path.clone());
        
        // 如果有文件需要加载，加载它并开始播放
        /*if let Some(file_path) = file_to_load {
            if !file_path.is_empty() {
                app.handle_initial_file_load(&file_path);
                // 自动开始播放（如果配置中启用了记住播放位置）
                if !app.file_path.is_empty() {
                    let file_path_clone = app.file_path.clone();
                    return (app, Task::perform(
                        start_audio_playback(AudioSource::FilePath(file_path_clone), None),
                        |(sender, _handle)| Message::AudioSessionStarted(sender)
                    ));
                }
            }
        }*/
        
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
            Message::MultipleAudioFilesSelected(file_paths) => self.handle_multiple_audio_files_selected(file_paths),
            Message::PlaylistItemSelected(index) => self.handle_playlist_item_selected(index),
            Message::PlaylistCardToggled(playlist_path) => self.handle_playlist_card_toggled(playlist_path),
            Message::PlaylistCardMoreClicked(_playlist_path) => {
                // 预留：后续在这里弹出编辑菜单
                Task::none()
            }
            Message::StartCreatePlaylist => { self.creating_playlist = true; Task::none() },
            Message::CreatePlaylistNameChanged(name) => { self.creating_playlist_name = name; Task::none() },
            Message::ConfirmCreatePlaylist => self.handle_confirm_create_playlist(),
            Message::CancelCreatePlaylist => { self.creating_playlist = false; self.creating_playlist_name.clear(); Task::none() },
            Message::NextTrack => self.handle_next_track(),
            Message::PreviousTrack => self.handle_previous_track(),
            Message::Tick => self.handle_tick(),
            Message::PlaybackStateUpdate(state) => self.handle_playback_state_update(state),
            Message::AudioSessionStarted(sender) => self.handle_audio_session_started(sender),
            Message::EventOccurred(event) => self.handle_event_occurred(event),
            Message::ToggleView => self.handle_toggle_view(),
            Message::WindowResized(width, height) => self.handle_window_resized(width, height),
            Message::ProgressChanged(progress) => self.handle_progress_changed(progress),
            Message::ToggleTheme => self.handle_toggle_theme(),
            Message::PageChanged(page) => self.handle_page_changed(page),
            Message::TogglePlayMode => self.handle_toggle_play_mode(),
            Message::ConfigUpdate => self.handle_config_update(),
            Message::LanguageChanged(lang) => self.handle_language_changed(lang),
            Message::ResetConfig => self.handle_reset_config(),
            Message::AudioFileLoaded(file_path, success) => self.handle_audio_file_loaded(file_path, success),
            Message::AudioDurationEstimated(file_path, duration) => self.handle_audio_duration_estimated(file_path, duration),
        }
    }

    /// 获取当前主题
    pub fn theme(&self) -> iced::Theme {
        self.current_theme.to_iced_theme()
    }

    /// 创建应用程序视图
    pub fn view(&self) -> Element<Message> {
        // 顶部主区域：根据当前页面切换，但底部栏保持不变
        let nav = navigation_sidebar(&self.current_page);
        let top_row: Element<Message> = match self.current_page {
            PageType::Home => {
                // 左侧面板：播放列表文件网格视图（自适应宽度和高度）
                let left_panel = column![
                    playlist_files_grid_view(&self.playlist_manager, self.creating_playlist, &self.creating_playlist_name),
                ].spacing(16)
                 .width(Length::Fill)
                 .height(Length::Fill);

                // 右侧面板：主内容区域
                let right_panel = if self.playlist_manager.current_playlist_path().is_some() && self.playlist_loaded {
                    self.create_main_player_view()
                } else {
                    self.create_welcome_view()
                };

                row![
                    nav,
                    StyledContainer::new(left_panel)
                        .style(super::widgets::styled_container::ContainerStyle::Card)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .padding(constants::PADDING_LARGE)
                        .build(),
                    StyledContainer::new(right_panel)
                        .style(super::widgets::styled_container::ContainerStyle::Card)
                        .width(Length::Fixed(RIGHT_PANEL_WIDTH))
                        .height(Length::Fill)
                        .padding(constants::PADDING_LARGE)
                        .build(),
                ]
                .spacing(constants::SPACING_LARGE)
                .height(Length::Fill)
                .into()
            }
            PageType::Settings => {
                let settings = StyledContainer::new(
                    settings_page(&self.current_theme, &self.current_language)
                )
                .style(super::widgets::styled_container::ContainerStyle::Card)
                .padding(constants::PADDING_MEDIUM)
                .width(Length::Fill)
                .height(Length::Fill)
                .build();

                row![
                    nav,
                    settings,
                ]
                .spacing(constants::SPACING_LARGE)
                .height(Length::Fill)
                .into()
            }
        };

        // 底部栏（统一）
        let left_info = StyledContainer::new(
            row![
                compact_album_cover_view(self.audio_info.as_ref()),
                compact_song_info_view(self.audio_info.as_ref(), &self.file_path),
            ]
            .spacing(constants::SPACING_SMALL)
            .align_y(Vertical::Center)
        )
        .style(super::widgets::styled_container::ContainerStyle::Transparent)
        .width(Length::Fixed(LEFT_INFO_WIDTH))
        .build();

        let right_controls = row![
            simple_time_view(&self.playback_state),
            compact_file_button(),
            compact_play_mode_button(self.play_mode.clone()),
            compact_view_toggle_button(self.current_view.clone()),
        ]
        .spacing(constants::SPACING_SMALL)
        .align_y(Vertical::Center);

        let bottom_bar = StyledContainer::new(
            row![
                left_info,
                container(control_buttons_view(self.is_playing))
                    .width(Length::Fill)
                    .align_x(Horizontal::Center),
                right_controls,
            ]
            .spacing(constants::SPACING_MEDIUM)
            .align_y(Vertical::Center)
        )
        .style(super::widgets::styled_container::ContainerStyle::Decorative)
        .padding([constants::PADDING_SMALL, constants::PADDING_MEDIUM])
        .width(Length::Fill)
        .height(Length::Fixed(72.0))
        .build();

        StyledContainer::new(
            column![
                container(top_row).width(Length::Fill).height(Length::Fill),
                container(thin_progress_view(&self.playback_state)).padding([0_u16, constants::PADDING_MEDIUM]).height(Length::Fixed(8.0)).width(Length::Fill),
                bottom_bar,
            ]
            .spacing(constants::SPACING_MEDIUM)
            .height(Length::Fill)
        )
        .style(super::widgets::styled_container::ContainerStyle::Transparent)
        .width(Length::Fill)
        .height(Length::Fill)
        .build()
        .into()
    }

    /// 创建应用程序订阅
    pub fn subscription(&self) -> Subscription<Message> {
        use crate::config::ui::PROGRESS_UPDATE_INTERVAL;
        
        let subscriptions = vec![
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
            return self.start_audio_playback_task(self.file_path.clone());
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
        // 使用音频文件多选对话框，这是更常用的场景
        Task::perform(open_audio_files_dialog(), Message::MultipleAudioFilesSelected)
    }



    /// 处理用户选择的文件（智能模式）
    /// 
    /// 当用户通过智能文件选择对话框选择一个文件后，此函数会被调用。
    /// 它会根据文件类型（普通音频文件或播放列表文件）进行相应处理，
    /// 并更新播放器状态以开始播放。
    /// 
    /// 对于播放列表文件，直接加载播放列表。
    /// 对于音频文件，如果是单选，创建临时播放列表；如果需要多选，应使用OpenAudioFiles消息。
    /// 
    /// # 参数
    /// * `file_path` - 用户选择的文件路径，如果用户取消选择则为None
    /// 
    /// # 返回
    /// 返回一个Task，用于执行后续的异步操作
    /*fn handle_file_selected(&mut self, file_path: Option<String>) -> Task<Message> {
        let Some(path) = file_path else {
            return Task::none();
        };

        // 记录是否之前正在播放
        let was_playing = self.is_playing;

        if is_m3u_playlist(&path) {
            // 使用播放列表管理器加载播放列表
            match self.playlist_manager.set_current_playlist(&path) {
                Ok(_) => {
                    self.playlist_loaded = true;
                    
                    // 启动后台AudioFile加载任务
                    let background_task = self.start_background_audio_loading();
                    
                    if let Some(playlist) = self.playlist_manager.current_playlist() {
                        if let Some(first_item) = playlist.set_current_index(0) {
                            let file_path = first_item.path.clone();
                            self.update_ui_for_track(&file_path);
                            
                            // 停止当前播放，然后如果之前正在播放则启动新的播放会话
                            self.stop_current_playback();
                            
                            if was_playing {
                                let playback_task = self.start_audio_playback_task(file_path);
                                return Task::batch([background_task, playback_task]);
                            } else {
                                return background_task;
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to load playlist: {}", e);
                }
            }
        } else {
            self.file_path = path.clone();
            // 使用播放列表管理器为单个文件创建播放列表
            match self.playlist_manager.set_current_playlist(&path) {
                Ok(_) => {
                    self.playlist_loaded = true;
                    // 清除选中状态，因为这是单个文件，不是播放列表文件
                    self.selected_playlist_path = None;
                    self.update_ui_for_track(&path);
                    
                    // 停止当前播放，然后如果之前正在播放则启动新的播放会话
                    self.stop_current_playback();
                    
                    if was_playing {
                        return self.start_audio_playback_task(path);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to create playlist: {}", e);
                }
            }
        }
        
        Task::none()
    }*/

    /// 处理用户选择的多个音频文件
    /// 
    /// 当用户通过文件选择对话框选择一个或多个音频文件后，此函数会被调用。
    /// 它会验证选择的文件类型，然后创建一个临时播放列表并开始播放第一个文件。
    /// 
    /// 验证规则：
    /// - 如果选择了播放列表文件，则只能选择一个文件，且必须是播放列表文件
    /// - 如果选择了多个文件，则所有文件都必须是音频文件，不能包含播放列表文件
    /// 
    /// # 参数
    /// * `file_paths` - 用户选择的文件路径列表
    /// 
    /// # 返回
    /// 返回一个Task，用于执行后续的异步操作
    fn handle_multiple_audio_files_selected(&mut self, file_paths: Vec<String>) -> Task<Message> {
        if file_paths.is_empty() {
            return Task::none();
        }

        // 验证文件选择的合法性
        let playlist_files: Vec<&String> = file_paths.iter().filter(|path| is_m3u_playlist(path)).collect();
        let audio_files: Vec<String> = file_paths.iter().filter(|path| !is_m3u_playlist(path)).cloned().collect();

        // 验证选择规则
        if !playlist_files.is_empty() && !audio_files.is_empty() {
            eprintln!("错误：不能同时选择音频文件和播放列表文件！");
            return Task::none();
        }

        if playlist_files.len() > 1 {
            eprintln!("错误：播放列表文件一次只能选择一个！");
            return Task::none();
        }

        // 如果选择的是播放列表文件，使用播放列表处理逻辑
        let mut new_playlist = if playlist_files.len() == 1 {
            let playlist_path = playlist_files[0].clone();
            //return self.handle_file_selected(Some(playlist_path));
            if self.playlist_manager.contains_playlist(&playlist_path) {
                // switch to playlist
                return Task::none();
            }
            match Playlist::create_from_playlist_file(playlist_path) {
                Ok(pl) => pl,
                Err(err) => {
                    eprintln!("加载播放列表失败: {:?}", err);
                    return Task::none();
                }
            }
        } else if audio_files.len() > 0 {
            // 创建临时播放列表
            Playlist::create_from_audio_files(audio_files.clone())
        } else {
            // 无效流程
            return Task::none();
        };

        //let playlist_path = new_playlist.
        let audio_file_path = new_playlist.set_current_index(0).unwrap().clone();
        self.playlist_manager.insert_and_set_current_playlist(new_playlist);
        // 选择/创建播放列表后，强制切换到播放列表视图
        self.current_view = ViewType::Playlist;
        self.app_config.ui.current_view = self.current_view.clone().into();
        self.app_config.save_safe();
        //self.playlist_manager.set_current_playlist(new_playlist.file_path())
        let background_task = self.start_background_audio_duration_loading();
        self.stop_current_playback();
        let playback_task = self.start_audio_playback_task(audio_file_path.clone());
        return Task::batch([background_task, playback_task]);
    }

    fn handle_playlist_item_selected(&mut self, index: usize) -> Task<Message> {
        if self.playlist_loaded {
            if let Some(playlist) = self.playlist_manager.current_playlist() {
                if let Some(item) = playlist.set_current_index(index) {
                    let file_path = item.clone();
                    self.update_ui_for_track(&file_path);
                    
                    // 停止当前播放，然后立即启动新歌曲的播放
                    self.stop_current_playback();
                    
                    // 启动新的音频播放会话
                    return self.start_audio_playback_task(file_path);
                }
            }
        }
        Task::none()
    }

    fn handle_next_track(&mut self) -> Task<Message> {
        if self.playlist_loaded {
            if let Some(playlist) = self.playlist_manager.current_playlist() {
                let (next_item, should_restart) = playlist.next_file_with_mode(&self.play_mode);
                if let Some(item) = next_item {
                    let file_path = item.clone();
                    
                    if should_restart {
                        // 单曲循环或随机播放到同一首歌 - 重新开始播放
                        self.stop_current_playback();
                        return self.start_audio_playback_task(file_path);
                    } else {
                        // 切换到不同的歌曲 - 使用缓存的AudioFile实例
                        self.update_ui_for_track(&file_path);
                        self.stop_current_playback();
                        return self.start_audio_playback_task(file_path);
                    }
                }
            }
        }
        Task::none()
    }

    fn handle_previous_track(&mut self) -> Task<Message> {
        if self.playlist_loaded {
            if let Some(playlist) = self.playlist_manager.current_playlist() {
                let (prev_item, should_restart) = playlist.previous_file_with_mode(&self.play_mode);
                if let Some(item) = prev_item {
                    let file_path = item.clone();
                    
                    if should_restart {
                        // 单曲循环或随机播放到同一首歌 - 重新开始播放
                        self.stop_current_playback();
                        return self.start_audio_playback_task(file_path);
                    } else {
                        // 切换到不同的歌曲 - 使用缓存的AudioFile实例
                        self.update_ui_for_track(&file_path);
                        self.stop_current_playback();
                        return self.start_audio_playback_task(file_path);
                    }
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
        // 直接切换视图并保存配置（去除动画逻辑）
        let target_view = match self.current_view {
            ViewType::Playlist => ViewType::Lyrics,
            ViewType::Lyrics => ViewType::Playlist,
        };

        self.current_view = target_view.clone();
        self.app_config.ui.current_view = target_view.into();
        self.app_config.save_safe();

        Task::none()
    }

    

    fn handle_window_resized(&mut self, width: f32, height: f32) -> Task<Message> {
        self.window_size = (width, height);
        // 更新配置
        self.app_config.window.width = width;
        self.app_config.window.height = height;
        self.app_config.save_safe();
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
        // 更新配置
        self.app_config.ui.theme = self.current_theme.clone().into();
        self.app_config.save_safe();
        Task::none()
    }

    fn handle_page_changed(&mut self, page: PageType) -> Task<Message> {
        self.current_page = page.clone();
        // 更新配置
        self.app_config.ui.current_page = page.into();
        self.app_config.save_safe();
        Task::none()
    }

    fn handle_toggle_play_mode(&mut self) -> Task<Message> {
        self.play_mode = self.play_mode.next();
        // 更新配置
        self.app_config.player.play_mode = self.play_mode.clone().into();
        self.app_config.save_safe();
        Task::none()
    }

    fn handle_config_update(&mut self) -> Task<Message> {
        // 强制保存当前配置
        self.update_config_from_state();
        self.app_config.save_safe();
        Task::none()
    }

    fn handle_language_changed(&mut self, language: String) -> Task<Message> {
        self.current_language = language.clone();
        self.app_config.ui.language = language;
        self.app_config.save_safe();
        
        // 可以在这里添加重新加载UI文本的逻辑
        Task::none()
    }

    fn handle_reset_config(&mut self) -> Task<Message> {
        // 重置配置为默认值
        self.app_config = AppConfig::default();
        
        // 更新应用状态以匹配默认配置
        self.current_theme = self.app_config.ui.theme.clone().into();
        self.current_language = self.app_config.ui.language.clone();
        self.current_page = self.app_config.ui.current_page.clone().into();
        self.current_view = self.app_config.ui.current_view.clone().into();
        self.play_mode = self.app_config.player.play_mode.clone().into();
        
        // 保存重置后的配置
        self.app_config.save_safe();
        
        Task::none()
    }

    fn handle_audio_file_loaded(&mut self, file_path: String, success: bool) -> Task<Message> {
        if success {
            println!("AudioFile loaded successfully: {}", file_path);
            // 确保加入全局缓存，并尝试更新当前播放列表中的额外信息（如时长）
            if let Ok(audio_file) = self.playlist_manager.get_or_load_audio_file(&file_path) {
                if let Some(playlist) = self.playlist_manager.current_playlist() {
                    let duration = audio_file.info.duration;
                    if let Some(existing) = playlist.extra_info_for(&file_path) {
                        let mut updated = PlaylistExtraInfo::new(existing.path.clone());
                        if let Some(name) = existing.name.clone() { updated = updated.with_name(name); }
                        let updated = updated.with_duration(duration);
                        playlist.set_extra_info(updated);
                    } else {
                        let info = PlaylistExtraInfo::new(file_path.clone()).with_duration(duration);
                        playlist.set_extra_info(info);
                    }
                }
            }
        } else {
            eprintln!("Failed to load AudioFile: {}", file_path);
        }
        Task::none()
    }

    fn handle_audio_duration_estimated(&mut self, file_path: String, duration: Option<f64>) -> Task<Message> {
        // 更新全局缓存中的 AudioFile 时长
        let updated = self.playlist_manager.update_audio_file_duration(&file_path, duration);

        // 更新当前播放列表的额外信息（用于列表显示）
        if let Some(playlist) = self.playlist_manager.current_playlist() {
            if let Some(existing) = playlist.extra_info_for(&file_path) {
                let mut updated_info = PlaylistExtraInfo::new(existing.path.clone());
                if let Some(name) = existing.name.clone() { updated_info = updated_info.with_name(name); }
                let updated_info = updated_info.with_duration(duration);
                playlist.set_extra_info(updated_info);
            } else {
                let info = PlaylistExtraInfo::new(file_path.clone()).with_duration(duration);
                playlist.set_extra_info(info);
            }
        }

        // 如果是当前正在显示/播放的文件，刷新UI中的时长
        if self.file_path == file_path {
            if let Some(mut info) = self.audio_info.clone() {
                info.duration = duration;
                self.playback_state.total_duration = duration.unwrap_or(0.0);
                self.audio_info = Some(info);
            } else if updated {
                // 从缓存回读最新信息
                if let Ok(af) = self.playlist_manager.get_or_load_audio_file(&file_path) {
                    let info = af.info.clone();
                    self.playback_state.total_duration = info.duration.unwrap_or(0.0);
                    self.audio_info = Some(info);
                }
            }
        }

        Task::none()
    }

    fn handle_confirm_create_playlist(&mut self) -> Task<Message> {
        let name = self.creating_playlist_name.trim().to_string();
        if name.is_empty() {
            // 空名则忽略
            self.creating_playlist = false;
            self.creating_playlist_name.clear();
            return Task::none();
        }
        match self.playlist_manager.create_empty_playlist(&name) {
            Ok(path) => {
                // 重置创建状态
                self.creating_playlist = false;
                self.creating_playlist_name.clear();
                // 立即选中该播放列表以便显示
                self.handle_playlist_card_toggled(path)
            }
            Err(e) => {
                eprintln!("创建播放列表失败: {}", e);
                Task::none()
            }
        }
    }

    fn handle_playlist_card_toggled(&mut self, playlist_path: String) -> Task<Message> {
        // 直接通过 PlaylistManager 管理当前激活的播放列表
        // 总是加载选中的播放列表到右侧显示（但不开始播放）
        match self.playlist_manager.set_current_playlist(&playlist_path) {
            Ok(_) => {
                self.stop_current_playback();
                self.playlist_loaded = true;
                // 切换播放列表时默认显示播放列表视图
                self.current_view = ViewType::Playlist;
                self.app_config.ui.current_view = self.current_view.clone().into();
                self.app_config.save_safe();
                // 先将播放列表中的音频加载到全局缓存中，再启动后台时长估算
                self.playlist_manager.preload_current_playlist_audio_to_cache();
                self.start_background_audio_duration_loading()
            }
            Err(e) => {
                eprintln!("加载播放列表失败: {}", e);
                Task::none()
            }
        }
    }

    // 辅助方法

    /// 从当前应用状态更新配置
    fn update_config_from_state(&mut self) {
        self.app_config.window.width = self.window_size.0;
        self.app_config.window.height = self.window_size.1;
        self.app_config.ui.theme = self.current_theme.clone().into();
        self.app_config.ui.language = self.current_language.clone();
        self.app_config.ui.current_page = self.current_page.clone().into();
        self.app_config.ui.current_view = self.current_view.clone().into();
        self.app_config.player.play_mode = self.play_mode.clone().into();
        
        if !self.file_path.is_empty() {
            self.app_config.player.last_file_path = Some(self.file_path.clone());
        }
        
        // 记住播放位置
        if self.app_config.player.remember_position {
            self.app_config.player.last_position = self.playback_state.current_time;
        }
    }

    /// 启动音频播放，优先使用全局缓存的AudioFile实例
    fn start_audio_playback_task(&mut self, file_path: String) -> Task<Message> {
        if let Ok(audio_file) = self.playlist_manager.get_or_load_audio_file(&file_path) {
            return Task::perform(
                start_audio_playback(AudioSource::AudioFile(audio_file), None),
                |(sender, _handle)| Message::AudioSessionStarted(sender)
            );
        }
        // 回退到路径方式
        Task::perform(
            start_audio_playback(AudioSource::FilePath(file_path), None),
            |(sender, _handle)| Message::AudioSessionStarted(sender)
        )
    }

    /// 启动后台来刷新AudioInfo中的duration信息
    fn start_background_audio_duration_loading(&mut self) -> Task<Message> {
        if let Some(playlist) = self.playlist_manager.current_playlist() {
            // 克隆路径，避免借用冲突
            let paths: Vec<String> = playlist.file_paths().to_vec();

            // 先预计算播放列表中缺少时长的条目集合，然后释放对playlist的后续使用
            let missing_in_playlist: HashSet<String> = {
                let mut set = HashSet::new();
                for p in playlist.file_paths() {
                    let missing = playlist.extra_info_for(p).map_or(true, |ei| ei.duration.is_none());
                    if missing {
                        set.insert(p.clone());
                    }
                }
                set
            };

            // 选择需要异步估算时长的文件：
            // 条件：已经在全局缓存中，且其时长为None或0.0；
            // 或者不在缓存中，但播放列表extra_infos中缺少时长。
            let mut candidates: Vec<String> = Vec::new();
            for file_path in paths.into_iter() {
                if self.playlist_manager.contains_audio_file(&file_path) {
                    if let Ok(af) = self.playlist_manager.get_or_load_audio_file(&file_path) {
                        if af.info.duration.unwrap_or(0.0) <= 0.0 {
                            candidates.push(file_path);
                        }
                    } else {
                        candidates.push(file_path);
                    }
                } else {
                    if missing_in_playlist.contains(&file_path) {
                        candidates.push(file_path);
                    }
                }
            }

            if candidates.is_empty() {
                return Task::none();
            }

            println!("启动后台时长估算任务，需要估算 {} 个文件", candidates.len());

            // 为每个候选文件并发启动耗时的估算任务
            let tasks: Vec<Task<Message>> = candidates
                .into_iter()
                .map(|file_path| {
                    Task::perform(async move {
                        let dur = estimate_duration_by_parsing(&file_path);
                        (file_path, dur)
                    }, |(fp, dur)| Message::AudioDurationEstimated(fp, dur))
                })
                .collect();

            Task::batch(tasks)
        } else {
            Task::none()
        }
    }

    

    /// 仅更新UI信息，使用全局缓存，避免重复打开AudioFile
    fn update_ui_for_track(&mut self, file_path: &str) {
        self.file_path = file_path.to_string();
        
        // 重置播放状态
        self.playback_state.current_time = 0.0;
        self.playback_state.current_samples = 0;
        
        // 从全局缓存获取AudioFile信息
        if let Ok(audio_file) = self.playlist_manager.get_or_load_audio_file(file_path) {
            let info = &audio_file.info;
            self.audio_info = Some(info.clone());
            self.playback_state.total_duration = info.duration.unwrap_or(0.0);
            self.playback_state.sample_rate = info.sample_rate;
            // 使用AudioFile的内置歌词加载方法
            match audio_file.load_lyrics() {
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
        } else {
            eprintln!("无法从全局缓存中获取音频文件: {}", file_path);
            self.current_lyrics = None;
        }
        
        // 保存最后播放的文件到配置
        self.app_config.player.last_file_path = Some(file_path.to_string());
        self.app_config.save_safe();
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
            if let Some(playlist) = self.playlist_manager.current_playlist() {
                let (next_item, should_restart) = playlist.next_file_with_mode(&self.play_mode);
                if let Some(item) = next_item {
                    let file_path = item.clone();
                    
                    if should_restart {
                        // 单曲循环 - 重新开始播放当前歌曲
                        return self.start_audio_playback_task(file_path);
                    } else {
                        // 切换到下一首歌曲 - 使用缓存的AudioFile实例
                        self.update_ui_for_track(&file_path);
                        return self.start_audio_playback_task(file_path);
                    }
                }
            }
        }
        
        Task::none()
    }

    /// 创建主页面内容
    #[allow(dead_code)]
    fn create_home_page(&self) -> Element<Message> {
        // 左侧面板：播放列表文件网格视图（自适应宽度和高度）
        let left_panel = column![
            playlist_files_grid_view(&self.playlist_manager, self.creating_playlist, &self.creating_playlist_name),
        ].spacing(16)
         .width(Length::Fill)
         .height(Length::Fill); // 确保填满可用高度

        // 右侧面板：主内容区域
        let right_panel = if self.playlist_manager.current_playlist_path().is_some() && self.playlist_loaded {
            // 播放列表已加载，显示主播放界面
            self.create_main_player_view()
        } else {
            // 播放列表未加载，显示欢迎界面
            self.create_welcome_view()
        };

        // 左侧导航栏
        let nav = navigation_sidebar(&self.current_page);

        // 顶部：导航 + 左右面板（右侧固定宽度，左侧自适应）
        let top_row = row![
            // 导航侧边栏
            nav,
            // 左侧面板（播放列表网格）
            StyledContainer::new(left_panel)
                .style(super::widgets::styled_container::ContainerStyle::Card)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(constants::PADDING_LARGE)
                .build(),
            // 右侧面板（主内容）
            StyledContainer::new(right_panel)
                .style(super::widgets::styled_container::ContainerStyle::Transparent)
                .width(Length::Fixed(RIGHT_PANEL_WIDTH))
                .height(Length::Fill)
                .padding(constants::PADDING_LARGE)
                .build(),
        ]
        .spacing(constants::SPACING_LARGE);

        // 左侧固定宽度：封面 + 歌曲信息
        let left_info = StyledContainer::new(
            row![
                compact_album_cover_view(self.audio_info.as_ref()),
                compact_song_info_view(self.audio_info.as_ref(), &self.file_path),
            ]
            .spacing(constants::SPACING_SMALL)
            .align_y(Vertical::Center)
        )
        .style(super::widgets::styled_container::ContainerStyle::Transparent)
        .width(Length::Fixed(LEFT_INFO_WIDTH))
        .build();

        // 右侧功能按钮组（时间 + 文件打开 + 模式切换 + 歌词切换）
        let right_controls = row![
            simple_time_view(&self.playback_state),
            compact_file_button(),
            compact_play_mode_button(self.play_mode.clone()),
            compact_view_toggle_button(self.current_view.clone()),
        ]
        .spacing(constants::SPACING_SMALL)
        .align_y(Vertical::Center);

        // 全局底部栏（跨越全宽）：左固定，中间居中控制，右侧功能组
        let bottom_bar = StyledContainer::new(
            row![
                left_info,
                container(control_buttons_view(self.is_playing))
                    .width(Length::Fill)
                    .align_x(Horizontal::Center),
                right_controls,
            ]
            .spacing(constants::SPACING_MEDIUM)
            .align_y(Vertical::Center)
        )
        .style(super::widgets::styled_container::ContainerStyle::Decorative)
        .padding([constants::PADDING_SMALL, constants::PADDING_MEDIUM])
        .width(Length::Fill)
        .height(Length::Fixed(72.0))
        .build();

        // 顶部行 + 进度条 + 底部栏
        StyledContainer::new(
            column![
                container(top_row).width(Length::Fill).height(Length::Fill),
                container(thin_progress_view(&self.playback_state)).padding([0_u16, constants::PADDING_MEDIUM]).height(Length::Fixed(8.0)).width(Length::Fill),
                bottom_bar,
            ]
            .spacing(constants::SPACING_MEDIUM)
            .height(Length::Fill)
        )
        .style(super::widgets::styled_container::ContainerStyle::Background)
        .padding(constants::PADDING_MEDIUM)
        .width(Length::Fill)
        .height(Length::Fill)
        .build()
        .into()
    }

    fn create_sliding_animation_view(&self) -> Element<Message> {
        let playlist_content = if let Some(playlist) = self.playlist_manager.current_playlist_ref() {
            playlist_view(playlist, self.playlist_loaded, self.is_playing, &self.playlist_manager)
        } else {
            let empty_playlist = Playlist::new();
            playlist_view(&empty_playlist, false, self.is_playing, &self.playlist_manager)
        };
        let lyrics_content = lyrics_view(&self.file_path, self.is_playing, self.playback_state.current_time, self.current_lyrics.clone(), self.window_size.1);

        match self.current_view {
            ViewType::Playlist => playlist_content,
            ViewType::Lyrics => lyrics_content,
        }
    }

    fn create_main_player_view(&self) -> Element<Message> {
        // 主内容（不包含底部栏与进度条，由首页统一布局承载）
        let main_content = self.create_sliding_animation_view();

        StyledContainer::new(container(main_content).height(Length::Fill).width(Length::Fill))
            .style(super::widgets::styled_container::ContainerStyle::Transparent)
            .padding(constants::PADDING_SMALL)
            .width(Length::Fill)
            .height(Length::Fill)
            .build()
            .into()
    }

    fn create_welcome_view(&self) -> Element<Message> {
        let welcome_main = StyledContainer::new(
            column![
                StyledText::new("🎵").size(32).style(TextStyle::Primary).shaping(Shaping::Advanced).build(),
                StyledText::new("Welcome to Summer Player").size(constants::TEXT_TITLE).style(TextStyle::Primary).build(),
                StyledText::new("Click the playlist card to load playlist").size(constants::TEXT_MEDIUM).style(TextStyle::Hint).build(),
            ]
            .spacing(constants::SPACING_MEDIUM)
            .align_x(Horizontal::Center)
        )
        .style(super::widgets::styled_container::ContainerStyle::Transparent)
        .padding(constants::PADDING_SMALL)
        .width(Length::Fill)
        .height(Length::Fill)
        .build();

        // 欢迎内容（不包含底部栏与进度条，由首页统一布局承载）
        StyledContainer::new(container(welcome_main).height(Length::Fill).width(Length::Fill))
            .style(super::widgets::styled_container::ContainerStyle::Transparent)
            .padding(constants::PADDING_SMALL)
            .width(Length::Fill)
            .height(Length::Fill)
            .build()
            .into()
    }
}




/// 打开文件对话框（支持音频文件多选和播放列表单选）
async fn open_audio_files_dialog() -> Vec<String> {
    let files = rfd::AsyncFileDialog::new()
        .add_filter("Audio Files", &["mp3", "flac", "wav", "ogg", "aac", "m4a", "m4s"])
        .add_filter("Playlist Files", &["m3u", "m3u8"])
        .add_filter("All Files", &["*"])
        .pick_files()
        .await;
    
    files.map(|files_vec| 
        files_vec.into_iter()
            .map(|f| f.path().to_string_lossy().to_string())
            .collect()
    ).unwrap_or_default()
}

 