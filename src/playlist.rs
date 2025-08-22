//! 播放列表模块
//! 
//! 处理播放列表的解析、管理和操作。

use std::{
    fs,
    io::{BufRead, BufReader},
    path::Path,
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
    /// AudioFile缓存
    audio_cache: AudioFileCache,
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
            audio_cache: AudioFileCache::new(),
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
            audio_cache: AudioFileCache::new(),
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
            audio_cache: AudioFileCache::new(),
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
    /// 持久播放列表实例
    pub fn create_from_playlist_file(file_path: String) -> Self {
        let name = extract_filename(&file_path);
        Self {
            file_paths: Vec::new(),
            current_index: None,
            name: Some(name),
            audio_cache: AudioFileCache::new(),
            file_path: Some(file_path),
        }
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
    
    /// 获取或加载指定文件路径的AudioFile
    /// 
    /// # 参数
    /// * `file_path` - 音频文件路径
    /// 
    /// # 返回
    /// AudioFile的引用
    pub fn get_audio_file(&mut self, file_path: &str) -> Result<&PlaylistItem> {
        self.audio_cache.get(file_path)
    }

    /// 获取当前播放文件的AudioFile
    /// 
    /// # 返回
    /// 当前播放文件的AudioFile引用
    pub fn get_current_audio_file(&mut self) -> Result<Option<&PlaylistItem>> {
        if let Some(current_index) = self.current_index {
            if let Some(file_path) = self.file_paths.get(current_index) {
                let path = file_path.clone(); // 克隆路径避免借用问题
                Ok(Some(self.get_audio_file(&path)?))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
    
    /// 获取指定索引文件的AudioFile
    /// 
    /// # 参数
    /// * `index` - 文件索引
    /// 
    /// # 返回
    /// AudioFile的引用
    pub fn get_audio_file_by_index(&mut self, index: usize) -> Result<Option<&PlaylistItem>> {
        if let Some(file_path) = self.file_paths.get(index) {
            let path = file_path.clone(); // 克隆路径避免借用问题
            Ok(Some(self.get_audio_file(&path)?))
        } else {
            Ok(None)
        }
    }
    
    /// 根据文件路径获取AudioFile（便于从外部访问缓存）
    /// 
    /// # 参数
    /// * `file_path` - 音频文件路径
    /// 
    /// # 返回
    /// AudioFile的引用
    pub fn get_audio_file_by_path(&mut self, file_path: &str) -> Result<Option<&PlaylistItem>> {
        // 首先尝试从缓存中获取
        if self.audio_cache.contains(file_path) {
            Ok(Some(self.get_audio_file(file_path)?))
        } else {
            // 如果缓存中没有，尝试加载
            match self.get_audio_file(file_path) {
                Ok(audio_file) => Ok(Some(audio_file)),
                Err(_) => Ok(None), // 加载失败时返回None而不是错误
            }
        }
    }
    
    /// 检查缓存中是否包含指定文件
    /// 
    /// # 参数
    /// * `file_path` - 音频文件路径
    /// 
    /// # 返回
    /// 如果缓存中包含指定文件返回true，否则返回false
    pub fn contains_audio_file(&self, file_path: &str) -> bool {
        self.audio_cache.contains(file_path)
    }
    
    /// 清空播放列表和缓存
    pub fn clear(&mut self) {
        self.file_paths.clear();
        self.current_index = None;
        self.audio_cache.clear();
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
}

impl PlaylistManager {
    /// 创建新的播放列表管理器
    pub fn new() -> Self {
        Self {
            playlists: HashMap::new(),
            current_playlist_path: None,
            temporary_playlist: None,
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
                parse_m3u_playlist(playlist_path)?
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
                                if let Ok(playlist) = parse_m3u_playlist(path_str) {
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
}

/// 解析M3U播放列表
/// 
/// # 参数
/// * `file_path` - M3U文件路径
/// 
/// # 返回
/// 成功时返回Playlist实例，失败时返回错误
pub fn parse_m3u_playlist(file_path: &str) -> Result<Playlist> {
    let file = fs::File::open(file_path)
        .map_err(|e| PlayerError::PlaylistError(format!("Failed to open playlist file: {}", e)))?;
    
    let reader = BufReader::new(file);
    let mut playlist = Playlist::create_from_playlist_file(
        file_path.to_string(),
    );
    let playlist_dir = Path::new(file_path).parent()
        .ok_or_else(|| PlayerError::PlaylistError("Invalid playlist path".to_string()))?;
    
    let mut current_track_info: Option<(i32, String)> = None; // (duration, title)
    
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
                        
                        if let Ok(duration) = duration_str.parse::<i32>() {
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
        
        // 如果有EXTINF信息，更新对应的AudioFile元数据
        if let Some((duration, title)) = current_track_info.take() {
            // 获取或加载AudioFile以更新其元数据
            if let Ok(audio_file) = playlist.get_audio_file(&file_path) {
                // 更新AudioFile的标题（如果EXTINF中的标题与文件名不同）
                if !title.is_empty() && title != extract_filename(&file_path) {
                    // 克隆AudioFile并更新元数据
                    let mut updated_audio_file = audio_file.clone();
                    if updated_audio_file.info.metadata.title.is_none() || 
                       updated_audio_file.info.metadata.title.as_ref().unwrap() != &title {
                        updated_audio_file.info.metadata.title = Some(title);
                    }
                    
                    // 更新缓存中的AudioFile
                    // 注意：由于Rust的借用检查，我们不能直接修改缓存中的值
                    // 这里我们只是记录需要更新的信息，实际更新将在播放时进行
                }
            }
        }
    }
    
    if playlist.is_empty() {
        return Err(PlayerError::PlaylistError("No valid files found in playlist".to_string()));
    }
    
    Ok(playlist)
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