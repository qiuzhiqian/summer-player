//! 用户界面模块
//! 
//! 基于iced框架的图形用户界面实现。

pub mod app;
pub mod messages;
pub mod components;
pub mod animation;

// 重新导出主要类型
pub use app::PlayerApp;
pub use messages::Message;
pub use components::ViewType; 