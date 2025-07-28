//! 动画系统演示示例
//! 
//! 展示如何使用基于 anim-rs 的新动画系统

use std::time::Duration;
use player::ui::animation::{ViewTransitionAnimation, ValueAnimator, ColorAnimator};
use player::ui::components::ViewType;

fn main() {
    println!("=== 动画系统演示 ===\n");
    
    // 演示视图切换动画
    demo_view_transition();
    
    // 演示数值动画器
    demo_value_animator();
    
    // 演示颜色动画器
    demo_color_animator();
}

fn demo_view_transition() {
    println!("1. 视图切换动画演示");
    println!("   - 使用 anim-rs 库的 cubic ease-in-out 缓动");
    println!("   - 280ms 动画持续时间");
    
    let mut view_animation = ViewTransitionAnimation::new();
    
    // 开始从播放列表到歌词的切换动画
    view_animation.start_transition(ViewType::Lyrics);
    
    println!("   开始动画: 播放列表 -> 歌词");
    
    // 模拟动画更新循环
    let mut steps = 0;
    while view_animation.is_active() && steps < 10 {
        let is_complete = view_animation.update();
        let progress = view_animation.progress();
        
        println!("   步骤 {}: 进度 {:.2}", steps + 1, progress);
        
        if is_complete {
            println!("   动画完成！");
            break;
        }
        
        steps += 1;
        std::thread::sleep(Duration::from_millis(30)); // 模拟帧更新
    }
    
    println!();
}

fn demo_value_animator() {
    println!("2. 数值动画器演示");
    println!("   - 从 0.0 动画到 100.0，持续 500ms");
    println!("   - 使用 quad ease-in-out 缓动");
    
    let mut value_animator = ValueAnimator::new();
    
    // 开始数值动画
    value_animator.animate_to(0.0, 100.0, Duration::from_millis(500));
    
    println!("   开始动画: 0.0 -> 100.0");
    
    // 模拟动画更新循环
    let mut steps = 0;
    while value_animator.is_active() && steps < 8 {
        let is_complete = value_animator.update();
        let value = value_animator.value();
        
        println!("   步骤 {}: 值 {:.2}", steps + 1, value);
        
        if is_complete {
            println!("   动画完成！");
            break;
        }
        
        steps += 1;
        std::thread::sleep(Duration::from_millis(65)); // 模拟帧更新
    }
    
    println!();
}

fn demo_color_animator() {
    println!("3. 颜色动画器演示");
    println!("   - 从红色 [1.0, 0.0, 0.0, 1.0] 动画到蓝色 [0.0, 0.0, 1.0, 1.0]");
    println!("   - 持续 300ms");
    
    let mut color_animator = ColorAnimator::new();
    
    // 开始颜色动画：红色到蓝色
    let red = [1.0, 0.0, 0.0, 1.0];
    let blue = [0.0, 0.0, 1.0, 1.0];
    color_animator.animate_color(red, blue, Duration::from_millis(300));
    
    println!("   开始动画: 红色 -> 蓝色");
    
    // 模拟动画更新循环
    let mut steps = 0;
    while color_animator.is_active() && steps < 6 {
        let is_complete = color_animator.update();
        let color = color_animator.current_color();
        
        println!("   步骤 {}: RGBA [{:.2}, {:.2}, {:.2}, {:.2}]", 
                steps + 1, color[0], color[1], color[2], color[3]);
        
        if is_complete {
            println!("   动画完成！");
            break;
        }
        
        steps += 1;
        std::thread::sleep(Duration::from_millis(50)); // 模拟帧更新
    }
    
    println!();
} 