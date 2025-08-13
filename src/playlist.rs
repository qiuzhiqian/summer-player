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

/// 播放列表项
#[derive(Debug, Clone)]
pub struct PlaylistItem {
    /// 文件路径
    pub path: String,
    /// 显示名称
    pub name: String,
    /// 时长（秒）
    pub duration: Option<f64>,
}

impl PlaylistItem {
    /// 创建新的播放列表项
    /// 
    /// # 参数
    /// * `path` - 文件路径
    /// 
    /// # 返回
    /// PlaylistItem实例
    pub fn new(path: String) -> Self {
        let name = extract_filename(&path);
        
        Self {
            path,
            name,
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
        self.name = name;
        self
    }
}

/// AudioFile缓存管理器
/// 
/// 管理播放列表中每个音频文件的AudioFile实例，避免重复解析
pub struct AudioFileCache {
    /// 文件路径 -> AudioFile 的映射
    cache: HashMap<String, AudioFile>,
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
    pub fn get_or_load(&mut self, file_path: &str) -> Result<&AudioFile> {
        if !self.cache.contains_key(file_path) {
            let audio_file = AudioFile::open(file_path)?;
            self.cache.insert(file_path.to_string(), audio_file);
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
    /// 播放列表项
    items: Vec<PlaylistItem>,
    /// 当前播放索引
    current_index: Option<usize>,
    /// 播放列表名称
    name: Option<String>,
    /// AudioFile缓存
    audio_cache: AudioFileCache,
}

impl Playlist {
    /// 创建新的空播放列表
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            current_index: None,
            name: None,
            audio_cache: AudioFileCache::new(),
        }
    }
    
    /// 创建带名称的播放列表
    /// 
    /// # 参数
    /// * `name` - 播放列表名称
    pub fn with_name(name: String) -> Self {
        Self {
            items: Vec::new(),
            current_index: None,
            name: Some(name),
            audio_cache: AudioFileCache::new(),
        }
    }
    
    /// 添加播放列表项
    /// 
    /// # 参数
    /// * `item` - 播放列表项
    pub fn add_item(&mut self, item: PlaylistItem) {
        self.items.push(item);
    }
    
    /// 批量添加播放列表项
    /// 
    /// # 参数
    /// * `items` - 播放列表项向量
    pub fn add_items(&mut self, items: Vec<PlaylistItem>) {
        self.items.extend(items);
    }
    
    /// 获取当前播放项
    /// 
    /// # 返回
    /// 当前播放项的引用，如果没有则返回None
    pub fn current_item(&self) -> Option<&PlaylistItem> {
        self.current_index.and_then(|i| self.items.get(i))
    }
    
    /// 切换到下一首
    /// 
    /// # 返回
    /// 下一首播放项的引用，如果没有则返回None
    pub fn next_item(&mut self) -> Option<&PlaylistItem> {
        if self.items.is_empty() {
            return None;
        }
        
        let next_index = match self.current_index {
            Some(current) => {
                if current + 1 < self.items.len() {
                    Some(current + 1)
                } else {
                    None // 播放列表结束
                }
            }
            None => Some(0), // 开始播放
        };
        
        self.current_index = next_index;
        self.current_item()
    }
    
    /// 切换到上一首
    /// 
    /// # 返回
    /// 上一首播放项的引用，如果没有则返回None
    pub fn previous_item(&mut self) -> Option<&PlaylistItem> {
        if self.items.is_empty() {
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
        self.current_item()
    }
    
    /// 根据播放模式切换到下一首
    /// 
    /// # 参数
    /// * `play_mode` - 播放模式
    /// 
    /// # 返回
    /// 下一首播放项的引用和是否应该重新开始播放
    pub fn next_item_with_mode(&mut self, play_mode: &PlayMode) -> (Option<&PlaylistItem>, bool) {
        if self.items.is_empty() {
            return (None, false);
        }
        
        match play_mode {
            PlayMode::SingleLoop => {
                // 单曲循环：保持当前歌曲
                (self.current_item(), true)
            }
            PlayMode::ListLoop => {
                // 列表循环：到末尾后回到开头
                let next_index = match self.current_index {
                    Some(current) => {
                        if current + 1 < self.items.len() {
                            Some(current + 1)
                        } else {
                            Some(0) // 回到开头
                        }
                    }
                    None => Some(0), // 开始播放
                };
                
                self.current_index = next_index;
                (self.current_item(), false)
            }
            PlayMode::Random => {
                // 随机播放：随机选择一首歌曲
                if self.items.len() == 1 {
                    // 只有一首歌，重复播放
                    (self.current_item(), true)
                } else {
                    use rand::Rng;
                    let mut rng = rand::thread_rng();
                    let mut next_index = rng.gen_range(0..self.items.len());
                    
                    // 确保不会连续播放同一首歌（如果可能的话）
                    if let Some(current) = self.current_index {
                        while next_index == current && self.items.len() > 1 {
                            next_index = rng.gen_range(0..self.items.len());
                        }
                    }
                    
                    self.current_index = Some(next_index);
                    (self.current_item(), false)
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
    /// 上一首播放项的引用和是否应该重新开始播放
    pub fn previous_item_with_mode(&mut self, play_mode: &PlayMode) -> (Option<&PlaylistItem>, bool) {
        if self.items.is_empty() {
            return (None, false);
        }
        
        match play_mode {
            PlayMode::SingleLoop => {
                // 单曲循环：保持当前歌曲
                (self.current_item(), true)
            }
            PlayMode::ListLoop => {
                // 列表循环：到开头后回到末尾
                let prev_index = match self.current_index {
                    Some(current) => {
                        if current > 0 {
                            Some(current - 1)
                        } else {
                            Some(self.items.len() - 1) // 回到末尾
                        }
                    }
                    None => Some(0), // 开始播放
                };
                
                self.current_index = prev_index;
                (self.current_item(), false)
            }
            PlayMode::Random => {
                // 随机播放：随机选择一首歌曲
                if self.items.len() == 1 {
                    // 只有一首歌，重复播放
                    (self.current_item(), true)
                } else {
                    use rand::Rng;
                    let mut rng = rand::thread_rng();
                    let mut prev_index = rng.gen_range(0..self.items.len());
                    
                    // 确保不会连续播放同一首歌（如果可能的话）
                    if let Some(current) = self.current_index {
                        while prev_index == current && self.items.len() > 1 {
                            prev_index = rng.gen_range(0..self.items.len());
                        }
                    }
                    
                    self.current_index = Some(prev_index);
                    (self.current_item(), false)
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
    /// 指定索引的播放项引用，如果索引无效则返回None
    pub fn set_current_index(&mut self, index: usize) -> Option<&PlaylistItem> {
        if index < self.items.len() {
            self.current_index = Some(index);
            self.current_item()
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
        self.items.is_empty()
    }
    
    /// 获取播放列表长度
    /// 
    /// # 返回
    /// 播放列表中的项目数量
    pub fn len(&self) -> usize {
        self.items.len()
    }
    
        /// 获取所有播放列表项的引用
    /// 
    /// # 返回
    /// 播放列表项的向量引用
    pub fn items(&self) -> &[PlaylistItem] {
        &self.items
    }

    /// 更新指定文件路径对应项目的时长信息
    /// 
    /// # 参数
    /// * `file_path` - 文件路径
    /// * `duration` - 新的时长（秒）
    /// 
    /// # 返回
    /// 如果找到并更新成功返回true，否则返回false
    pub fn update_item_duration(&mut self, file_path: &str, duration: Option<f64>) -> bool {
        for item in &mut self.items {
            if item.path == file_path {
                item.duration = duration;
                return true;
            }
        }
        false
    }

    /// 获取播放列表名称
    /// 
    /// # 返回
    /// 播放列表名称的引用，如果没有则返回None
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
    
    /// 移除指定索引的项目
    /// 
    /// # 参数
    /// * `index` - 要移除的项目索引
    /// 
    /// # 返回
    /// 如果成功移除则返回true
    pub fn remove_item(&mut self, index: usize) -> bool {
        if index < self.items.len() {
            self.items.remove(index);
            
            // 调整当前播放索引
            if let Some(current) = self.current_index {
                if current == index {
                    // 移除的是当前播放项
                    if self.items.is_empty() {
                        self.current_index = None;
                    } else if current >= self.items.len() {
                        self.current_index = Some(self.items.len() - 1);
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
    pub fn get_audio_file(&mut self, file_path: &str) -> Result<&AudioFile> {
        self.audio_cache.get_or_load(file_path)
    }
    
    /// 获取当前播放项的AudioFile
    /// 
    /// # 返回
    /// 当前播放项的AudioFile引用
    pub fn get_current_audio_file(&mut self) -> Result<Option<&AudioFile>> {
        if let Some(current_index) = self.current_index {
            if let Some(item) = self.items.get(current_index) {
                let path = item.path.clone(); // 克隆路径避免借用问题
                Ok(Some(self.get_audio_file(&path)?))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
    
    /// 获取指定索引项的AudioFile
    /// 
    /// # 参数
    /// * `index` - 播放列表项索引
    /// 
    /// # 返回
    /// AudioFile的引用
    pub fn get_audio_file_by_index(&mut self, index: usize) -> Result<Option<&AudioFile>> {
        if let Some(item) = self.items.get(index) {
            let path = item.path.clone(); // 克隆路径避免借用问题
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
    pub fn get_audio_file_by_current_path(&mut self, file_path: &str) -> Result<Option<&AudioFile>> {
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
    
    /// 清空播放列表和缓存
    pub fn clear(&mut self) {
        self.items.clear();
        self.current_index = None;
        self.audio_cache.clear();
    }
}

/// 播放列表管理器
/// 
/// 管理多个播放列表的缓存，避免重复加载播放列表文件
pub struct PlaylistManager {
    /// 播放列表文件路径 -> Playlist 的映射
    playlists: HashMap<String, Playlist>,
    /// 当前活跃的播放列表路径
    current_playlist_path: Option<String>,
}

impl PlaylistManager {
    /// 创建新的播放列表管理器
    pub fn new() -> Self {
        Self {
            playlists: HashMap::new(),
            current_playlist_path: None,
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
                create_single_file_playlist(playlist_path)?
            };
            
            self.playlists.insert(playlist_path.to_string(), playlist);
        }
        
        Ok(self.playlists.get_mut(playlist_path).unwrap())
    }
    
    /// 设置当前活跃的播放列表
    /// 
    /// # 参数
    /// * `playlist_path` - 播放列表文件路径
    pub fn set_current_playlist(&mut self, playlist_path: &str) -> Result<()> {
        // 确保播放列表已加载
        self.get_or_load_playlist(playlist_path)?;
        self.current_playlist_path = Some(playlist_path.to_string());
        Ok(())
    }
    
    /// 获取当前活跃的播放列表
    /// 
    /// # 返回
    /// 当前播放列表的可变引用
    pub fn current_playlist(&mut self) -> Option<&mut Playlist> {
        if let Some(path) = &self.current_playlist_path.clone() {
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
        if let Some(path) = &self.current_playlist_path {
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
    let mut playlist = Playlist::with_name(extract_filename(file_path));
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
        
        // 创建播放列表项
        let mut item = PlaylistItem::new(file_path.clone());
        
        // 使用M3U中的标题信息
        if let Some((duration, title)) = current_track_info.take() {
            if !title.is_empty() && title != extract_filename(&file_path) {
                item = item.with_name(title);
            }
            if duration > 0 {
                item = item.with_duration(Some(duration as f64));
            }
        }
        
        // 如果M3U中没有时长信息，暂时不获取（延迟加载）
        // 这样可以避免在播放列表解析时重复调用AudioFile::open
        // 时长信息将在实际播放时通过load_audio_file获取
        // if item.duration.is_none() {
        //     if let Ok(audio_info) = AudioFile::get_info(&file_path) {
        //         item = item.with_duration(audio_info.duration);
        //     }
        // }
        
        playlist.add_item(item);
    }
    
    if playlist.is_empty() {
        return Err(PlayerError::PlaylistError("No valid files found in playlist".to_string()));
    }
    
    Ok(playlist)
}

/// 创建单文件播放列表
/// 
/// # 参数
/// * `file_path` - 音频文件路径
/// 
/// # 返回
/// 包含单个文件的播放列表
pub fn create_single_file_playlist(file_path: &str) -> Result<Playlist> {
    let mut playlist = Playlist::new();
    let item = PlaylistItem::new(file_path.to_string());
    
    // 延迟获取音频信息，避免重复调用AudioFile::open
    // 时长信息将在实际播放时通过load_audio_file获取
    // if let Ok(audio_info) = AudioFile::get_info(file_path) {
    //     item = item.with_duration(audio_info.duration);
    // }
    
    playlist.add_item(item);
    playlist.set_current_index(0);
    
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
        assert!(playlist.current_item().is_none());
    }

    #[test]
    fn test_playlist_operations() {
        let mut playlist = Playlist::new();
        
        let item1 = PlaylistItem::new("test1.mp3".to_string());
        let item2 = PlaylistItem::new("test2.mp3".to_string());
        
        playlist.add_item(item1);
        playlist.add_item(item2);
        
        assert_eq!(playlist.len(), 2);
        assert!(!playlist.is_empty());
        
        // 测试导航
        assert!(playlist.next_item().is_some());
        assert_eq!(playlist.current_index(), Some(0));
        
        assert!(playlist.next_item().is_some());
        assert_eq!(playlist.current_index(), Some(1));
        
        assert!(playlist.next_item().is_none()); // 到达末尾
        
        assert!(playlist.previous_item().is_some());
        assert_eq!(playlist.current_index(), Some(0));
    }
    
    #[test]
    fn test_playlist_item_creation() {
        let item = PlaylistItem::new("/path/to/test.mp3".to_string())
            .with_duration(Some(180.0))
            .with_name("Custom Name".to_string());
        
        assert_eq!(item.path, "/path/to/test.mp3");
        assert_eq!(item.name, "Custom Name");
        assert_eq!(item.duration, Some(180.0));
    }
} 