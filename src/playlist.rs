//! 播放列表模块
//! 
//! 处理播放列表的解析、管理和操作。

use std::{
    fs,
    io::{BufRead, BufReader},
    path::Path,
};

use crate::error::{PlayerError, Result};
use crate::utils::{extract_filename, normalize_path};
use crate::audio::AudioFile;

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

/// 播放列表
#[derive(Debug, Clone, Default)]
pub struct Playlist {
    /// 播放列表项
    items: Vec<PlaylistItem>,
    /// 当前播放索引
    current_index: Option<usize>,
    /// 播放列表名称
    name: Option<String>,
}

impl Playlist {
    /// 创建新的空播放列表
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            current_index: None,
            name: None,
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
    
    /// 获取播放列表名称
    /// 
    /// # 返回
    /// 播放列表名称的引用，如果没有则返回None
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
    
    /// 清空播放列表
    pub fn clear(&mut self) {
        self.items.clear();
        self.current_index = None;
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
        
        // 如果M3U中没有时长信息，尝试从文件获取
        if item.duration.is_none() {
            if let Ok(audio_info) = AudioFile::get_info(&file_path) {
                item = item.with_duration(audio_info.duration);
            }
        }
        
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
    let mut item = PlaylistItem::new(file_path.to_string());
    
    // 尝试获取音频信息
    if let Ok(audio_info) = AudioFile::get_info(file_path) {
        item = item.with_duration(audio_info.duration);
    }
    
    playlist.add_item(item);
    playlist.set_current_index(0);
    
    Ok(playlist)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_playlist_creation() {
        let mut playlist = Playlist::new();
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