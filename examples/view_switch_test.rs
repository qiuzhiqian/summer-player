//! 视图切换动画测试
//! 
//! 专门测试修复后的视图切换动画是否能正确切换到目标视图

use std::time::Duration;
use player::ui::animation::ViewTransitionAnimation;
use player::ui::components::ViewType;

fn main() {
    println!("=== 视图切换动画测试 ===\n");
    
    test_playlist_to_lyrics();
    test_lyrics_to_playlist();
}

fn test_playlist_to_lyrics() {
    println!("测试 1: 播放列表 -> 歌词");
    
    let mut current_view = ViewType::Playlist;
    let mut view_animation = ViewTransitionAnimation::new();
    
    println!("   初始视图: {:?}", current_view);
    
    // 开始动画
    view_animation.start_transition(ViewType::Lyrics);
    println!("   开始动画: 播放列表 -> 歌词");
    
    // 模拟动画循环
    let mut step = 0;
    while view_animation.is_active() && step < 20 {
        // 在更新动画之前获取目标视图（模拟应用中的修复）
        let target_view = view_animation.target_view().cloned();
        
        // 更新动画
        if view_animation.update() {
            // 动画完成时切换视图
            if let Some(target_view) = target_view {
                current_view = target_view;
                println!("   动画完成! 视图已切换到: {:?}", current_view);
            } else {
                println!("   错误: 目标视图为空!");
            }
        } else {
            let progress = view_animation.progress();
            println!("   步骤 {}: 进度 {:.2}, 当前视图: {:?}", step + 1, progress, current_view);
        }
        
        step += 1;
        std::thread::sleep(Duration::from_millis(15)); // 模拟 ~60fps
    }
    
    // 验证结果
    if current_view == ViewType::Lyrics {
        println!("   ✅ 测试成功: 视图已正确切换到歌词界面");
    } else {
        println!("   ❌ 测试失败: 视图仍然是 {:?}", current_view);
    }
    
    println!();
}

fn test_lyrics_to_playlist() {
    println!("测试 2: 歌词 -> 播放列表");
    
    let mut current_view = ViewType::Lyrics;
    let mut view_animation = ViewTransitionAnimation::new();
    
    println!("   初始视图: {:?}", current_view);
    
    // 开始动画
    view_animation.start_transition(ViewType::Playlist);
    println!("   开始动画: 歌词 -> 播放列表");
    
    // 模拟动画循环
    let mut step = 0;
    while view_animation.is_active() && step < 20 {
        // 在更新动画之前获取目标视图（模拟应用中的修复）
        let target_view = view_animation.target_view().cloned();
        
        // 更新动画
        if view_animation.update() {
            // 动画完成时切换视图
            if let Some(target_view) = target_view {
                current_view = target_view;
                println!("   动画完成! 视图已切换到: {:?}", current_view);
            } else {
                println!("   错误: 目标视图为空!");
            }
        } else {
            let progress = view_animation.progress();
            println!("   步骤 {}: 进度 {:.2}, 当前视图: {:?}", step + 1, progress, current_view);
        }
        
        step += 1;
        std::thread::sleep(Duration::from_millis(15)); // 模拟 ~60fps
    }
    
    // 验证结果
    if current_view == ViewType::Playlist {
        println!("   ✅ 测试成功: 视图已正确切换到播放列表界面");
    } else {
        println!("   ❌ 测试失败: 视图仍然是 {:?}", current_view);
    }
    
    println!();
    
    println!("=== 测试结论 ===");
    println!("如果以上两个测试都显示 ✅，说明视图切换动画修复成功！");
} 