//! 音频标签写入模块
//!
//! 处理音频文件标签的写入功能，支持多种音频格式。

use lofty::config::WriteOptions;
use lofty::prelude::*;
use lofty::probe::Probe;
use lofty::tag::Tag;
use std::path::Path;

use crate::error::{PlayerError, Result};

/// 将元数据写入音频文件
pub fn write_metadata_to_file(file_path: &str, metadata: &crate::audio::file::AudioMetadata) -> Result<()> {
    println!("正在写入标签到文件: {}", file_path);
    let path = Path::new(file_path);
    
    // 首先检查文件是否存在
    if !path.exists() {
        return Err(PlayerError::Other(format!("文件不存在: {}", file_path)));
    }
    
    // 使用标准的lofty API链式调用读取音频文件
    let mut tagged_file = Probe::open(path)
        .map_err(|e| PlayerError::Other(format!("无法打开文件: {}", e)))?
        .guess_file_type()
        .map_err(|e| PlayerError::Other(format!("无法识别文件类型: {}", e)))?
        .read()
        .map_err(|e| PlayerError::Other(format!("无法读取音频文件标签: {}", e)))?;
    
    // 获取或创建标签
    let tag = match tagged_file.primary_tag_mut() {
        Some(primary_tag) => primary_tag,
        None => {
            if let Some(first_tag) = tagged_file.first_tag_mut() {
                first_tag
            } else {
                let tag_type = tagged_file.primary_tag_type();
                
                eprintln!("WARN: 没有找到标签，创建新标签类型 `{:?}`", tag_type);
                tagged_file.insert_tag(Tag::new(tag_type));
                
                tagged_file.primary_tag_mut().unwrap()
            }
        },
    };
    
    // 设置标签字段
    if let Some(title) = &metadata.title {
        tag.set_title(title.clone());
    }
    if let Some(artist) = &metadata.artist {
        tag.set_artist(artist.clone());
    }
    if let Some(album) = &metadata.album {
        tag.set_album(album.clone());
    }
    if let Some(genre) = &metadata.genre {
        tag.set_genre(genre.clone());
    }
    if let Some(year) = &metadata.year {
        // 尝试解析年份为数字
        if let Ok(year_num) = year.parse::<u32>() {
            tag.set_year(year_num);
        } else {
            // 如果不是纯数字，尝试提取年份部分
            let year_str = year.chars().filter(|c| c.is_ascii_digit()).collect::<String>();
            if let Ok(year_num) = year_str.parse::<u32>() {
                tag.set_year(year_num);
            }
        }
    }
    if let Some(track_number) = &metadata.track_number {
        // 尝试解析音轨号
        if let Ok(track_num) = track_number.parse::<u32>() {
            tag.set_track(track_num);
        }
    }
    
    // 保存标签到文件
    tag.save_to_path(path, WriteOptions::default())
        .map_err(|e| PlayerError::Other(format!("无法写入音频文件标签: {}", e)))?;
    
    println!("成功写入标签到文件: {}", file_path);
    Ok(())
}

/// 测试标签写入功能
#[cfg(test)]
mod tests {
    use super::*;
    use lofty::tag::TagType;
    
    #[test]
    fn test_tag_creation() {
        // 测试不同类型的标签创建
        let id3v2_tag = Tag::new(TagType::Id3v2);
        let mp4_tag = Tag::new(TagType::Mp4Ilst);
        let vorbis_tag = Tag::new(TagType::VorbisComments);
        
        // 验证标签类型
        assert_eq!(id3v2_tag.tag_type(), TagType::Id3v2);
        assert_eq!(mp4_tag.tag_type(), TagType::Mp4Ilst);
        assert_eq!(vorbis_tag.tag_type(), TagType::VorbisComments);
    }
}