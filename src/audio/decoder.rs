//! 音频解码器模块
//! 
//! 处理音频解码器的创建和管理。

use symphonia::core::{
    codecs::{Decoder, DecoderOptions},
    formats::Track,
};
use symphonia::default;

use crate::error::{PlayerError, Result};

/// 创建音频解码器
/// 
/// # 参数
/// * `track` - 音频轨道
/// 
/// # 返回
/// 成功时返回解码器，失败时返回错误
pub fn create_decoder(track: &Track) -> Result<Box<dyn Decoder>> {
    default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|e| PlayerError::DecodingError(e.to_string()))
} 