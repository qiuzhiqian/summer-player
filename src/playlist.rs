//! 播放列表模块
//! 
//! 处理播放列表的解析、管理和操作。

use std::{
    fs,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    collections::HashMap,
};

use crate::error::{PlayerError, Result};
use crate::utils::{extract_filename, normalize_path};
use crate::audio::AudioFile;
use crate::ui::components::PlayMode;

pub struct PlaylistExtraInfo {
    /// 文件路径
    pub path: String,
    /// 文件名
    pub name: Option<String>,
    /// 文件大小
    pub duration: Option<f64>,
}

impl PlaylistExtraInfo {
    /// 创建新的播放列表项
    /// 
    /// # 参数
    /// * `path` - 文件路径
    /// 
    /// # 返回
    /// PlaylistItem实例
    pub fn new(path: String) -> Self {
        Self {
            path,
            name: None,
            duration: None,
        }
    }
    
    /// 设置时长
    /// 
    /// # 参数
    /// * `duration` - 时长（秒）
    /// 
    /// # 返回
    /// 更新后的PlaylistItem实例
    pub fn with_duration(mut self, duration: Option<f64>) -> Self {
        self.duration = duration;
        self
    }
    
    /// 设置显示名称
    /// 
    /// # 参数
    /// * `name` - 显示名称
    /// 
    /// # 返回
    /// 更新后的PlaylistItem实例
    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }
}

pub struct PlaylistItem {
    extra_info: PlaylistExtraInfo,
    audio_file: AudioFile,
}

impl PlaylistItem {
    /// 获取显示名称，优先使用PlaylistExtraInfo中的name，如果没有则使用AudioFile中的元数据或文件名
    pub fn display_name(&self) -> String {
        if let Some(name) = &self.extra_info.name {
            name.clone()
        } else if let Some(title) = &self.audio_file.info.metadata.title {
            title.clone()
        } else {
            extract_filename(&self.audio_file.file_path)
        }
    }
    
    /// 获取时长，优先使用PlaylistExtraInfo中的duration，如果没有则使用AudioFile中的时长
    pub fn duration(&self) -> Option<f64> {
        if let Some(duration) = self.extra_info.duration {
            Some(duration)
        } else {
            self.audio_file.info.duration
        }
    }
    
    /// 获取音频文件引用
    pub fn audio_file(&self) -> &AudioFile {
        &self.audio_file
    }
}

/// AudioFile缓存管理器
/// 
/// 管理播放列表中每个音频文件的AudioFile实例，避免重复解析
pub struct AudioFileCache {
    /// 文件路径 -> PlaylistItem 的映射
    cache: HashMap<String, PlaylistItem>,
}

impl AudioFileCache {
    /// 创建新的缓存管理器
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }
    
    /// 获取或加载AudioFile
    /// 
    /// # 参数
    /// * `file_path` - 音频文件路径
    /// 
    /// # 返回
    /// AudioFile的引用，如果是第一次访问则会加载文件
    pub fn get(&mut self, file_path: &str) -> Result<&PlaylistItem> {
        if !self.cache.contains_key(file_path) {
            return Err(PlayerError::FileNotFound(file_path.to_string()));
        }
        Ok(self.cache.get(file_path).unwrap())
    }
    
    /// 获取或加载AudioFile，如果不存在则加载
    /// 
    /// # 参数
    /// * `file_path` - 音频文件路径
    /// * `extra_info` - 播放列表额外信息
    /// 
    /// # 返回
    /// PlaylistItem的引用
    pub fn get_or_load(&mut self, file_path: &str, extra_info: PlaylistExtraInfo) -> Result<&PlaylistItem> {
        if !self.cache.contains_key(file_path) {
            let audio_file = AudioFile::open(file_path)?;
            let playlist_item = PlaylistItem {
                extra_info,
                audio_file,
            };
            self.cache.insert(file_path.to_string(), playlist_item);
        }
        Ok(self.cache.get(file_path).unwrap())
    }

    /// 只读获取已缓存的AudioFile（不会触发加载）
    pub fn get_ref(&self, file_path: &str) -> Option<&PlaylistItem> {
        self.cache.get(file_path)
    }
    
    /// 检查缓存中是否存在指定文件
    pub fn contains(&self, file_path: &str) -> bool {
        self.cache.contains_key(file_path)
    }
    
    /// 清空缓存
    pub fn clear(&mut self) {
        self.cache.clear();
    }
    
    /// 获取缓存的文件数量
    pub fn len(&self) -> usize {
        self.cache.len()
    }
}

/// 播放列表
pub struct Playlist {
    /// 播放列表文件路径
    file_paths: Vec<String>,
    /// 当前播放索引
    current_index: Option<usize>,
    /// 播放列表名称
    name: Option<String>,
    /// 每个文件路径的额外信息（如名称、时长等）
    extra_infos: HashMap<String, PlaylistExtraInfo>,
    /// 播放列表文件路径（临时播放列表为None）
    file_path: Option<String>,
}

impl Playlist {
    /// 创建新的空播放列表
    pub fn new() -> Self {
        Self {
            file_paths: Vec::new(),
            current_index: None,
            name: None,
            extra_infos: HashMap::new(),
            file_path: None,
        }
    }
    
    /// 创建带名称的播放列表
    /// 
    /// # 参数
    /// * `name` - 播放列表名称
    pub fn with_name(name: String) -> Self {
        Self {
            file_paths: Vec::new(),
            current_index: None,
            name: Some(name),
            extra_infos: HashMap::new(),
            file_path: None,
        }
    }
    
    /// 添加文件路径到播放列表
    /// 
    /// # 参数
    /// * `file_path` - 音频文件路径
    pub fn add_file(&mut self, file_path: String) {
        self.file_paths.push(file_path);
    }
    
    /// 批量添加文件路径到播放列表
    /// 
    /// # 参数
    /// * `file_paths` - 音频文件路径向量
    pub fn add_files(&mut self, file_paths: Vec<String>) {
        self.file_paths.extend(file_paths);
    }
    
    /// 获取当前播放文件路径
    /// 
    /// # 返回
    /// 当前播放文件路径的引用，如果没有则返回None
    pub fn current_file_path(&self) -> Option<&String> {
        self.current_index.and_then(|i| self.file_paths.get(i))
    }
    
    /// 切换到下一首
    /// 
    /// # 返回
    /// 下一首文件路径的引用，如果没有则返回None
    pub fn next_file(&mut self) -> Option<&String> {
        if self.file_paths.is_empty() {
            return None;
        }
        
        let next_index = match self.current_index {
            Some(current) => {
                if current + 1 < self.file_paths.len() {
                    Some(current + 1)
                } else {
                    None // 播放列表结束
                }
            }
            None => Some(0), // 开始播放
        };
        
        self.current_index = next_index;
        self.current_file_path()
    }
    
    /// 切换到上一首
    /// 
    /// # 返回
    /// 上一首文件路径的引用，如果没有则返回None
    pub fn previous_file(&mut self) -> Option<&String> {
        if self.file_paths.is_empty() {
            return None;
        }
        
        let prev_index = match self.current_index {
            Some(current) => {
                if current > 0 {
                    Some(current - 1)
                } else {
                    None // 已经是第一首
                }
            }
            None => Some(0), // 开始播放
        };
        
        self.current_index = prev_index;
        self.current_file_path()
    }
    
    /// 根据播放模式切换到下一首
    /// 
    /// # 参数
    /// * `play_mode` - 播放模式
    /// 
    /// # 返回
    /// 下一首文件路径的引用和是否应该重新开始播放
    pub fn next_file_with_mode(&mut self, play_mode: &PlayMode) -> (Option<&String>, bool) {
        if self.file_paths.is_empty() {
            return (None, false);
        }
        
        match play_mode {
            PlayMode::SingleLoop => {
                // 单曲循环：保持当前歌曲
                (self.current_file_path(), true)
            }
            PlayMode::ListLoop => {
                // 列表循环：到末尾后回到开头
                let next_index = match self.current_index {
                    Some(current) => {
                        if current + 1 < self.file_paths.len() {
                            Some(current + 1)
                        } else {
                            Some(0) // 回到开头
                        }
                    }
                    None => Some(0), // 开始播放
                };
                
                self.current_index = next_index;
                (self.current_file_path(), false)
            }
            PlayMode::Random => {
                // 随机播放：随机选择一首歌曲
                if self.file_paths.len() == 1 {
                    // 只有一首歌，重复播放
                    (self.current_file_path(), true)
                } else {
                    use rand::Rng;
                    let mut rng = rand::thread_rng();
                    let mut next_index = rng.gen_range(0..self.file_paths.len());
                    
                    // 确保不会连续播放同一首歌（如果可能的话）
                    if let Some(current) = self.current_index {
                        while next_index == current && self.file_paths.len() > 1 {
                            next_index = rng.gen_range(0..self.file_paths.len());
                        }
                    }
                    
                    self.current_index = Some(next_index);
                    (self.current_file_path(), false)
                }
            }
        }
    }
    
    /// 根据播放模式切换到上一首
    /// 
    /// # 参数
    /// * `play_mode` - 播放模式
    /// 
    /// # 返回
    /// 上一首文件路径的引用和是否应该重新开始播放
    pub fn previous_file_with_mode(&mut self, play_mode: &PlayMode) -> (Option<&String>, bool) {
        if self.file_paths.is_empty() {
            return (None, false);
        }
        
        match play_mode {
            PlayMode::SingleLoop => {
                // 单曲循环：保持当前歌曲
                (self.current_file_path(), true)
            }
            PlayMode::ListLoop => {
                // 列表循环：到开头后回到末尾
                let prev_index = match self.current_index {
                    Some(current) => {
                        if current > 0 {
                            Some(current - 1)
                        } else {
                            Some(self.file_paths.len() - 1) // 回到末尾
                        }
                    }
                    None => {
                        if !self.file_paths.is_empty() {
                            Some(self.file_paths.len() - 1) // 从末尾开始
                        } else {
                            None
                        }
                    }
                };
                
                self.current_index = prev_index;
                (self.current_file_path(), false)
            }
            PlayMode::Random => {
                // 随机播放：随机选择一首歌曲
                if self.file_paths.len() == 1 {
                    // 只有一首歌，重复播放
                    (self.current_file_path(), true)
                } else {
                    use rand::Rng;
                    let mut rng = rand::thread_rng();
                    let mut prev_index = rng.gen_range(0..self.file_paths.len());
                    
                    // 确保不会连续播放同一首歌（如果可能的话）
                    if let Some(current) = self.current_index {
                        while prev_index == current && self.file_paths.len() > 1 {
                            prev_index = rng.gen_range(0..self.file_paths.len());
                        }
                    }
                    
                    self.current_index = Some(prev_index);
                    (self.current_file_path(), false)
                }
            }
        }
    }
    
    /// 设置当前播放索引
    /// 
    /// # 参数
    /// * `index` - 播放索引
    /// 
    /// # 返回
    /// 指定索引的文件路径引用，如果索引无效则返回None
    pub fn set_current_index(&mut self, index: usize) -> Option<&String> {
        if index < self.file_paths.len() {
            self.current_index = Some(index);
            self.current_file_path()
        } else {
            None
        }
    }
    
    /// 获取当前播放索引
    /// 
    /// # 返回
    /// 当前播放索引，如果没有则返回None
    pub fn current_index(&self) -> Option<usize> {
        self.current_index
    }
    
    /// 检查播放列表是否为空
    /// 
    /// # 返回
    /// 如果播放列表为空则返回true
    pub fn is_empty(&self) -> bool {
        self.file_paths.is_empty()
    }
    
    /// 获取播放列表长度
    /// 
    /// # 返回
    /// 播放列表中的文件数量
    pub fn len(&self) -> usize {
        self.file_paths.len()
    }
    
    /// 获取所有文件路径的引用
    /// 
    /// # 返回
    /// 文件路径的向量引用
    pub fn file_paths(&self) -> &[String] {
        &self.file_paths
    }
    
    /// 获取播放列表名称
    /// 
    /// # 返回
    /// 播放列表名称的引用，如果没有则返回None
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
    
    /// 获取播放列表文件路径
    /// 
    /// # 返回
    /// 播放列表文件路径的引用，如果是临时播放列表则返回None
    pub fn file_path(&self) -> Option<&str> {
        self.file_path.as_deref()
    }
    
    /// 检查是否为临时播放列表
    /// 
    /// # 返回
    /// 如果是临时播放列表返回true
    pub fn is_temporary(&self) -> bool {
        self.file_path.is_none()
    }
    
    /// 创建临时播放列表（用于多个音频文件）
    /// 
    /// # 参数
    /// * `file_paths` - 音频文件路径向量
    /// 
    /// # 返回
    /// 临时播放列表实例
    pub fn create_from_audio_files(file_paths: Vec<String>) -> Self {
        let mut playlist = Self {
            file_paths: Vec::new(),
            current_index: None,
            name: None,
            extra_infos: HashMap::new(),
            file_path: None, // 临时播放列表没有文件路径
        };
        
        // 为每个文件路径添加到播放列表
        for file_path in file_paths {
            playlist.add_file(file_path);
        }
        
        // 如果有文件，设置第一个为当前播放项
        if !playlist.is_empty() {
            playlist.set_current_index(0);
        }
        
        playlist
    }
    
    /// 创建持久播放列表（从文件加载）
    /// 
    /// # 参数
    /// * `file_path` - 播放列表文件路径
    /// * `name` - 播放列表名称
    /// 
    /// # 返回
    /// 成功时返回Playlist实例，失败时返回错误
    pub fn create_from_playlist_file(file_path: String) -> Result<Self> {
        let name = extract_filename(&file_path);
        // 打开并读取播放列表文件
        let file = fs::File::open(&file_path)
            .map_err(|e| PlayerError::PlaylistError(format!("Failed to open playlist file: {}", e)))?;

        let reader = BufReader::new(file);
        let mut playlist = Self {
            file_paths: Vec::new(),
            current_index: None,
            name: Some(name),
            extra_infos: HashMap::new(),
            file_path: Some(file_path.clone()),
        };

        let playlist_dir = Path::new(&file_path).parent()
            .ok_or_else(|| PlayerError::PlaylistError("Invalid playlist path".to_string()))?;

        let mut current_track_info: Option<(f64, String)> = None; // (duration, title)

        for (line_num, line) in reader.lines().enumerate() {
            let line = line.map_err(|e| PlayerError::PlaylistError(format!("Failed to read line {}: {}", line_num + 1, e)))?;
            let line = line.trim();

            // 跳过空行
            if line.is_empty() {
                continue;
            }

            // 处理M3U指令
            if line.starts_with('#') {
                if line.starts_with("#EXTINF:") {
                    // 解析 #EXTINF: 指令
                    if let Some(info_part) = line.strip_prefix("#EXTINF:") {
                        if let Some(comma_pos) = info_part.find(',') {
                            let duration_str = &info_part[..comma_pos];
                            let title = &info_part[comma_pos + 1..];

                            if let Ok(duration) = duration_str.parse::<f64>() {
                                current_track_info = Some((duration, title.to_string()));
                            }
                        }
                    }
                }
                // 跳过其他注释行
                continue;
            }

            // 处理文件路径
            let file_path = normalize_path(line, Some(playlist_dir));

            // 检查文件是否存在
            if !Path::new(&file_path).exists() {
                eprintln!("Warning: File not found: {}", file_path);
                continue;
            }

            // 添加文件到播放列表
            playlist.add_file(file_path.clone());

            // 如果有EXTINF信息，创建包含元数据的PlaylistExtraInfo
            let extra_info = if let Some((duration, title)) = current_track_info.take() {
                PlaylistExtraInfo::new(file_path.clone())
                    .with_duration(Some(duration))
                    .with_name(title)
            } else {
                PlaylistExtraInfo::new(file_path.clone())
            };

            // 记录额外信息（不在此处加载音频文件，避免重复加载）
            playlist.set_extra_info(extra_info);
        }

        Ok(playlist)
    }
    
    /// 移除指定索引的文件
    /// 
    /// # 参数
    /// * `index` - 要移除的文件索引
    /// 
    /// # 返回
    /// 如果成功移除则返回true
    pub fn remove_file(&mut self, index: usize) -> bool {
        if index < self.file_paths.len() {
            self.file_paths.remove(index);
            
            // 调整当前播放索引
            if let Some(current) = self.current_index {
                if current == index {
                    // 移除的是当前播放项
                    if self.file_paths.is_empty() {
                        self.current_index = None;
                    } else if current >= self.file_paths.len() {
                        self.current_index = Some(self.file_paths.len() - 1);
                    }
                    // 如果当前索引仍然有效，则保持不变
                } else if current > index {
                    // 当前播放项在被移除项之后，需要调整索引
                    self.current_index = Some(current - 1);
                }
            }
            
            true
        } else {
            false
        }
    }
    
    /// 设置或更新指定文件的额外信息
    pub fn set_extra_info(&mut self, extra_info: PlaylistExtraInfo) {
        self.extra_infos.insert(extra_info.path.clone(), extra_info);
    }

    /// 获取指定文件的额外信息
    pub fn extra_info_for(&self, file_path: &str) -> Option<&PlaylistExtraInfo> {
        self.extra_infos.get(file_path)
    }
    
    /// 清空播放列表和缓存
    pub fn clear(&mut self) {
        self.file_paths.clear();
        self.current_index = None;
        self.extra_infos.clear();
    }
}

/// 播放列表管理器
/// 
/// 管理多个播放列表的缓存，避免重复加载播放列表文件
pub struct PlaylistManager {
    /// 播放列表文件路径 -> Playlist 的映射（持久播放列表）
    playlists: HashMap<String, Playlist>,
    /// 当前活跃的播放列表路径
    current_playlist_path: Option<String>,
    /// 临时播放列表（单个音频文件）
    temporary_playlist: Option<Playlist>,
    /// 全局共享的AudioFile缓存（所有播放列表共享）
    audio_cache: HashMap<String, AudioFile>,
}

impl PlaylistManager {
    /// 创建新的播放列表管理器
    pub fn new() -> Self {
        Self {
            playlists: HashMap::new(),
            current_playlist_path: None,
            temporary_playlist: None,
            audio_cache: HashMap::new(),
        }
    }
    
    /// 获取或加载播放列表
    /// 
    /// # 参数
    /// * `playlist_path` - 播放列表文件路径
    /// 
    /// # 返回
    /// 播放列表的可变引用
    pub fn get_or_load_playlist(&mut self, playlist_path: &str) -> Result<&mut Playlist> {
        if !self.playlists.contains_key(playlist_path) {
            // 首次加载播放列表
            let playlist = if playlist_path.ends_with(".m3u") || playlist_path.ends_with(".m3u8") {
                Playlist::create_from_playlist_file(playlist_path.to_string())?
            } else {
                // 单个音频文件创建播放列表
                Playlist::create_from_audio_files(vec![playlist_path.to_string()])
            };
            
            self.playlists.insert(playlist_path.to_string(), playlist);
        }
        
        Ok(self.playlists.get_mut(playlist_path).unwrap())
    }
    
    /// 设置当前活跃的播放列表
    /// 
    /// # 参数
    /// * `playlist_path` - 播放列表文件路径或音频文件路径
    pub fn set_current_playlist(&mut self, playlist_path: &str) -> Result<()> {
        if playlist_path.is_empty() {
            self.current_playlist_path = None;
            self.temporary_playlist = None;
            return Ok(());
        }
        if !self.playlists.contains_key(playlist_path) {
            // 首次加载播放列表
            return Err(PlayerError::FileNotFound(playlist_path.to_string()));
        }
        self.current_playlist_path = Some(playlist_path.to_string());
        self.temporary_playlist = None; // 清除临时播放列表
        Ok(())
    }
    
    /// 设置当前活跃的播放列表（从多个音频文件）
    /// 
    /// # 参数
    /// * `file_paths` - 音频文件路径列表
    pub fn set_current_playlist_from_files(&mut self, file_paths: Vec<String>) -> Result<()> {
        // 创建临时播放列表
        let temp_playlist = Playlist::create_from_audio_files(file_paths);
        self.temporary_playlist = Some(temp_playlist);
        self.current_playlist_path = None; // 清除持久播放列表路径
        Ok(())
    }
    
    /// 获取当前活跃的播放列表
    /// 
    /// # 返回
    /// 当前播放列表的可变引用
    pub fn current_playlist(&mut self) -> Option<&mut Playlist> {
        // 优先返回临时播放列表
        if let Some(ref mut temp_playlist) = self.temporary_playlist {
            Some(temp_playlist)
        } else if let Some(path) = &self.current_playlist_path.clone() {
            self.playlists.get_mut(path)
        } else {
            None
        }
    }
    
    /// 获取当前活跃的播放列表（不可变引用）
    /// 
    /// # 返回
    /// 当前播放列表的不可变引用
    pub fn current_playlist_ref(&self) -> Option<&Playlist> {
        // 优先返回临时播放列表
        if let Some(ref temp_playlist) = self.temporary_playlist {
            Some(temp_playlist)
        } else if let Some(path) = &self.current_playlist_path {
            self.playlists.get(path)
        } else {
            None
        }
    }
    
    /// 获取当前播放列表的路径
    pub fn current_playlist_path(&self) -> Option<&str> {
        self.current_playlist_path.as_deref()
    }
    
    /// 检查播放列表是否已缓存
    /// 
    /// # 参数
    /// * `playlist_path` - 播放列表文件路径
    /// 
    /// # 返回
    /// 如果已缓存返回true
    pub fn contains_playlist(&self, playlist_path: &str) -> bool {
        self.playlists.contains_key(playlist_path)
    }

    /// 插入一个播放列表缓存
    /// 
    /// # 参数
    /// * `playlist` - 播放列表
    pub fn insert_playlist(&mut self, playlist: Playlist) {
        match playlist.file_path {
            Some(ref path) => {
                self.playlists.insert(path.clone(), playlist);
            },
            None => {
                self.temporary_playlist = Some(playlist);
            }
        }
    }

    pub fn insert_and_set_current_playlist(&mut self, playlist: Playlist) {
        match playlist.file_path {
            Some(ref path) => {
                let path = path.clone();
                self.playlists.insert(path.clone(), playlist);
                self.temporary_playlist = None;
                self.current_playlist_path = Some(path.clone());
            },
            None => {
                self.temporary_playlist = Some(playlist);
                self.current_playlist_path = None;
            }
        }
    }
    
    /// 移除指定的播放列表缓存
    /// 
    /// # 参数
    /// * `playlist_path` - 播放列表文件路径
    pub fn remove_playlist(&mut self, playlist_path: &str) {
        self.playlists.remove(playlist_path);
        if self.current_playlist_path.as_deref() == Some(playlist_path) {
            self.current_playlist_path = None;
        }
    }
    
    /// 清空所有播放列表缓存
    pub fn clear_all(&mut self) {
        self.playlists.clear();
        self.current_playlist_path = None;
    }
    
    /// 获取缓存的播放列表数量
    pub fn cached_count(&self) -> usize {
        self.playlists.len()
    }
    
    /// 获取所有缓存的播放列表路径
    pub fn cached_playlist_paths(&self) -> Vec<&str> {
        self.playlists.keys().map(|s| s.as_str()).collect()
    }
    
    /// 获取所有持久播放列表（不包括临时播放列表）
    /// 
    /// # 返回
    /// 持久播放列表的引用向量
    pub fn get_persistent_playlists(&self) -> Vec<&Playlist> {
        self.playlists.values().collect()
    }
    
    /// 获取所有持久播放列表的路径和播放列表对
    /// 
    /// # 返回
    /// (路径, 播放列表)的向量
    pub fn get_persistent_playlists_with_paths(&self) -> Vec<(&str, &Playlist)> {
        self.playlists.iter().map(|(path, playlist)| (path.as_str(), playlist)).collect()
    }
    
    /// 检查当前播放列表是否为临时播放列表
    /// 
    /// # 返回
    /// 如果当前播放列表是临时播放列表返回true
    pub fn is_current_temporary(&self) -> bool {
        self.temporary_playlist.is_some()
    }

    /// 检查全局缓存中是否已存在指定音频文件
    pub fn contains_audio_file(&self, file_path: &str) -> bool {
        self.audio_cache.contains_key(file_path)
    }

    /// 读取全局缓存中已存在的音频文件时长（只读，不触发加载）
    pub fn get_cached_audio_duration(&self, file_path: &str) -> Option<f64> {
        self.audio_cache.get(file_path).and_then(|af| af.info.duration)
    }

    /// 获取或加载全局共享的AudioFile（返回克隆以便安全使用）
    pub fn get_or_load_audio_file(&mut self, file_path: &str) -> Result<AudioFile> {
        if !self.audio_cache.contains_key(file_path) {
            let audio_file = AudioFile::open(file_path)?;
            self.audio_cache.insert(file_path.to_string(), audio_file);
        }
        Ok(self.audio_cache.get(file_path).unwrap().clone())
    }

    /// 更新已缓存的AudioFile的时长，并返回是否更新成功
    pub fn update_audio_file_duration(&mut self, file_path: &str, duration: Option<f64>) -> bool {
        if let Some(audio_file) = self.audio_cache.get_mut(file_path) {
            audio_file.info.duration = duration;
            true
        } else {
            false
        }
    }
    
    /// 加载配置目录下的所有播放列表文件
    /// 
    /// # 返回
    /// 成功加载的播放列表数量
    pub fn load_config_playlists(&mut self) -> usize {
        use std::fs;
        
        // 获取配置目录
        let config_dir = match dirs::config_dir() {
            Some(dir) => dir.join("summer-player"),
            None => return 0,
        };
        
        // 如果配置目录不存在，返回0
        if !config_dir.exists() {
            return 0;
        }
        
        let mut loaded_count = 0;
        
        // 读取配置目录下的所有文件
        if let Ok(entries) = fs::read_dir(&config_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(extension) = path.extension() {
                    if extension == "m3u" || extension == "m3u8" {
                        if let Some(path_str) = path.to_str() {
                            // 检查是否已经加载过
                            if !self.playlists.contains_key(path_str) {
                                // 尝试加载播放列表
                                if let Ok(playlist) = Playlist::create_from_playlist_file(path_str.to_string()) {
                                    self.playlists.insert(path_str.to_string(), playlist);
                                    loaded_count += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
        
        loaded_count
    }

    /// 预加载当前播放列表中的音频到全局缓存
    pub fn preload_current_playlist_audio_to_cache(&mut self) {
        if let Some(playlist) = self.current_playlist_ref() {
            let paths: Vec<String> = playlist.file_paths().to_vec();
            for file_path in paths.into_iter() {
                if !self.contains_audio_file(&file_path) {
                    if let Err(e) = self.get_or_load_audio_file(&file_path) {
                        eprintln!("预加载音频到缓存失败: {} -> {}", file_path, e);
                    }
                }
            }
        }
    }

    /// 在配置目录中新建一个空的持久播放列表（.m3u）并缓存
    /// 返回创建后的播放列表文件完整路径
    pub fn create_empty_playlist(&mut self, name: &str) -> Result<String> {
        // 计算配置目录
        let config_dir = dirs::config_dir()
            .map(|d| d.join("summer-player"))
            .ok_or_else(|| PlayerError::PlaylistError("Config dir not available".to_string()))?;
        // 确保目录存在
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)
                .map_err(|e| PlayerError::IoError(e))?;
        }

        // 生成文件名，确保以 .m3u 结尾
        let mut base = name.trim();
        if base.is_empty() {
            base = "New Playlist";
        }
        let mut file_name = format!("{}.m3u", base);
        // 处理重名：追加 (n)
        let mut counter = 1usize;
        let path_for = |fname: &str| -> PathBuf { config_dir.join(fname) };
        let mut full_path = path_for(&file_name);
        while full_path.exists() {
            file_name = format!("{} ({}).m3u", base, counter);
            full_path = path_for(&file_name);
            counter += 1;
        }

        // 写入包含 M3U 头的空文件
        let content = "#EXTM3U\n";
        fs::write(&full_path, content)
            .map_err(|e| PlayerError::IoError(e))?;

        let full_path_str = full_path
            .to_str()
            .ok_or_else(|| PlayerError::PlaylistError("Invalid playlist path".to_string()))?
            .to_string();

        // 加载并插入到缓存
        let playlist = Playlist::create_from_playlist_file(full_path_str.clone())?;
        self.playlists.insert(full_path_str.clone(), playlist);

        Ok(full_path_str)
    }

    /// 通过重命名文件来重命名播放列表，并更新缓存键
    pub fn rename_playlist(&mut self, old_path: &str, new_name: &str) -> Result<String> {
        let old = PathBuf::from(old_path);
        if !old.exists() {
            return Err(PlayerError::PlaylistError("Playlist file not found".to_string()));
        }
        let parent = old.parent().ok_or_else(|| PlayerError::PlaylistError("Invalid playlist path".to_string()))?;
        let mut base = new_name.trim();
        if base.is_empty() { base = "New Playlist"; }
        let mut candidate = parent.join(format!("{}.m3u", base));
        let mut counter = 1usize;
        while candidate.exists() {
            candidate = parent.join(format!("{} ({}).m3u", base, counter));
            counter += 1;
        }
        fs::rename(&old, &candidate).map_err(|e| PlayerError::IoError(e))?;
        let new_path = candidate.to_string_lossy().to_string();
        let _ = self.playlists.remove(old_path);
        // 重新加载，确保名称等信息更新
        if let Ok(new_pl) = Playlist::create_from_playlist_file(new_path.clone()) {
            self.playlists.insert(new_path.clone(), new_pl);
        }
        if self.current_playlist_path.as_deref() == Some(old_path) {
            self.current_playlist_path = Some(new_path.clone());
        }
        Ok(new_path)
    }

    /// 删除播放列表文件并从缓存中移除
    pub fn delete_playlist(&mut self, playlist_path: &str) -> Result<()> {
        let path = PathBuf::from(playlist_path);
        if path.exists() {
            fs::remove_file(&path).map_err(|e| PlayerError::IoError(e))?;
        }
        self.remove_playlist(playlist_path);
        Ok(())
    }

    /// 追加文件到指定的m3u播放列表文件，并更新缓存
    pub fn append_files_to_playlist(&mut self, playlist_path: &str, files: &[String]) -> Result<()> {
        if files.is_empty() { return Ok(()); }
        // 只支持持久播放列表
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .create(false)
            .open(playlist_path)
            .map_err(|e| PlayerError::IoError(e))?;
        use std::io::Write;
        for f in files {
            // 写入相对路径或绝对路径，保持简单使用绝对路径
            writeln!(file, "{}", f).map_err(|e| PlayerError::IoError(e))?;
        }
        // 刷新缓存中的该播放列表
        if let Ok(pl) = Playlist::create_from_playlist_file(playlist_path.to_string()) {
            self.playlists.insert(playlist_path.to_string(), pl);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_playlist_creation() {
        let playlist = Playlist::new();
        assert!(playlist.is_empty());
        assert_eq!(playlist.len(), 0);
        assert!(playlist.current_file_path().is_none());
    }

    #[test]
    fn test_playlist_operations() {
        let mut playlist = Playlist::new();
        
        playlist.add_file("test1.mp3".to_string());
        playlist.add_file("test2.mp3".to_string());
        
        assert_eq!(playlist.len(), 2);
        assert!(!playlist.is_empty());
        
        // 测试导航
        assert!(playlist.next_file().is_some());
        assert_eq!(playlist.current_index(), Some(0));
        
        assert!(playlist.next_file().is_some());
        assert_eq!(playlist.current_index(), Some(1));
        
        assert!(playlist.next_file().is_none()); // 到达末尾
        
        assert!(playlist.previous_file().is_some());
        assert_eq!(playlist.current_index(), Some(0));
    }
    
    #[test]
    fn test_create_from_multiple_audio_files() {
        let file_paths = vec![
            "song1.mp3".to_string(),
            "song2.flac".to_string(),
            "song3.wav".to_string(),
        ];
        
        let playlist = Playlist::create_from_audio_files(file_paths);
        
        assert_eq!(playlist.len(), 3);
        assert!(!playlist.is_empty());
        assert_eq!(playlist.current_index(), Some(0));
        assert!(playlist.is_temporary());
        
        // 验证所有文件都被添加到播放列表中
        let paths = playlist.file_paths();
        assert_eq!(paths[0], "song1.mp3");
        assert_eq!(paths[1], "song2.flac");
        assert_eq!(paths[2], "song3.wav");
    }
    
    #[test]
    fn test_create_from_single_audio_file_convenience() {
        let playlist = Playlist::create_from_audio_files(vec!["single_song.mp3".to_string()]);
        
        assert_eq!(playlist.len(), 1);
        assert!(!playlist.is_empty());
        assert_eq!(playlist.current_index(), Some(0));
        assert!(playlist.is_temporary());
        
        let paths = playlist.file_paths();
        assert_eq!(paths[0], "single_song.mp3");
    }
    
    #[test]
    fn test_create_from_empty_audio_files() {
        let playlist = Playlist::create_from_audio_files(vec![]);
        
        assert_eq!(playlist.len(), 0);
        assert!(playlist.is_empty());
        assert_eq!(playlist.current_index(), None);
        assert!(playlist.is_temporary());
    }
}