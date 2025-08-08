//! 基于 anim-rs 的动画系统
//! 
//! 提供现代化的动画功能，使用 anim-rs 库来管理动画状态和缓动函数。

use std::time::Duration;
use anim::{Options, Timeline, easing::{self, EasingMode}};
use super::components::ViewType;

/// 视图切换动画状态
#[derive(Debug)]
pub struct ViewTransitionAnimation {
    /// 动画时间轴
    timeline: Option<Timeline<f32>>,
    /// 目标视图类型
    target_view: Option<ViewType>,
    /// 动画是否正在进行
    is_active: bool,
}

impl Default for ViewTransitionAnimation {
    fn default() -> Self {
        Self {
            timeline: None,
            target_view: None,
            is_active: false,
        }
    }
}

impl ViewTransitionAnimation {
    /// 创建新的视图切换动画实例
    pub fn new() -> Self {
        Self::default()
    }
    
    /// 开始新的视图切换动画
    pub fn start_transition(&mut self, target_view: ViewType) {
        // 如果已经在动画中，忽略新的请求
        if self.is_active {
            return;
        }
        
        // 创建从 0.0 到 1.0 的动画，持续 800ms，使用 ease-in-out-quad 缓动以获得更明显的动画效果
        let timeline = Options::new(0.0, 1.0)
            .duration(Duration::from_millis(800))
            .easing(easing::quad_ease().mode(EasingMode::InOut))
            .begin_animation();
            
        self.timeline = Some(timeline);
        self.target_view = Some(target_view);
        self.is_active = true;
    }
    
    /// 更新动画状态
    /// 
    /// 返回 `true` 如果动画已完成，`false` 如果仍在进行中
    pub fn update(&mut self) -> bool {
        if let Some(ref mut timeline) = self.timeline {
            let status = timeline.update();
            
            if status.is_completed() {
                self.finish_animation();
                true
            } else {
                false
            }
        } else {
            false
        }
    }
    
    /// 获取当前动画进度 (0.0 到 1.0)
    pub fn progress(&self) -> f32 {
        if let Some(ref timeline) = self.timeline {
            timeline.value()
        } else {
            0.0
        }
    }
    
    /// 检查动画是否正在进行
    pub fn is_active(&self) -> bool {
        self.is_active
    }
    
    /// 获取目标视图类型
    pub fn target_view(&self) -> Option<&ViewType> {
        self.target_view.as_ref()
    }
    
    /// 完成动画并清理状态
    fn finish_animation(&mut self) {
        self.timeline = None;
        self.target_view = None;
        self.is_active = false;
    }
    
    /// 取消当前动画
    pub fn cancel(&mut self) {
        self.finish_animation();
    }
}

/// 通用数值动画器
/// 
/// 可以用于动画化任何 f32 值，比如透明度、尺寸等
#[derive(Debug)]
pub struct ValueAnimator {
    /// 动画时间轴
    timeline: Option<Timeline<f32>>,
    /// 是否正在动画中
    is_active: bool,
}

impl Default for ValueAnimator {
    fn default() -> Self {
        Self {
            timeline: None,
            is_active: false,
        }
    }
}

impl ValueAnimator {
    /// 创建新的数值动画器
    pub fn new() -> Self {
        Self::default()
    }
    
    /// 开始从 `from` 到 `to` 的动画，持续 `duration`
    pub fn animate_to(&mut self, from: f32, to: f32, duration: Duration) {
        let timeline = Options::new(from, to)
            .duration(duration)
            .easing(easing::quad_ease().mode(EasingMode::InOut))
            .begin_animation();
            
        self.timeline = Some(timeline);
        self.is_active = true;
    }
    
    /// 开始从 `from` 到 `to` 的动画，使用指定的缓动函数
    pub fn animate_to_with_easing(&mut self, from: f32, to: f32, duration: Duration, easing_fn: impl anim::easing::Function + Clone + 'static) {
        let timeline = Options::new(from, to)
            .duration(duration)
            .easing(easing_fn)
            .begin_animation();
            
        self.timeline = Some(timeline);
        self.is_active = true;
    }
    
    /// 更新动画状态
    /// 
    /// 返回 `true` 如果动画已完成
    pub fn update(&mut self) -> bool {
        if let Some(ref mut timeline) = self.timeline {
            let status = timeline.update();
            
            if status.is_completed() {
                self.finish_animation();
                true
            } else {
                false
            }
        } else {
            false
        }
    }
    
    /// 获取当前动画值
    pub fn value(&self) -> f32 {
        if let Some(ref timeline) = self.timeline {
            timeline.value()
        } else {
            0.0
        }
    }
    
    /// 检查是否正在动画中
    pub fn is_active(&self) -> bool {
        self.is_active
    }
    
    /// 完成动画
    fn finish_animation(&mut self) {
        self.timeline = None;
        self.is_active = false;
    }
    
    /// 取消动画
    pub fn cancel(&mut self) {
        self.finish_animation();
    }
}

/// 颜色动画器
/// 
/// 专门用于动画化颜色值
#[derive(Debug)]
pub struct ColorAnimator {
    /// 红色分量动画器
    red: ValueAnimator,
    /// 绿色分量动画器
    green: ValueAnimator,
    /// 蓝色分量动画器
    blue: ValueAnimator,
    /// 透明度分量动画器
    alpha: ValueAnimator,
}

impl Default for ColorAnimator {
    fn default() -> Self {
        Self {
            red: ValueAnimator::new(),
            green: ValueAnimator::new(),
            blue: ValueAnimator::new(),
            alpha: ValueAnimator::new(),
        }
    }
}

impl ColorAnimator {
    /// 创建新的颜色动画器
    pub fn new() -> Self {
        Self::default()
    }
    
    /// 开始颜色动画
    pub fn animate_color(&mut self, from: [f32; 4], to: [f32; 4], duration: Duration) {
        self.red.animate_to(from[0], to[0], duration);
        self.green.animate_to(from[1], to[1], duration);
        self.blue.animate_to(from[2], to[2], duration);
        self.alpha.animate_to(from[3], to[3], duration);
    }
    
    /// 更新所有颜色分量的动画
    /// 
    /// 返回 `true` 如果所有动画都已完成
    pub fn update(&mut self) -> bool {
        let red_done = self.red.update();
        let green_done = self.green.update();
        let blue_done = self.blue.update();
        let alpha_done = self.alpha.update();
        
        red_done && green_done && blue_done && alpha_done
    }
    
    /// 获取当前颜色值 [r, g, b, a]
    pub fn current_color(&self) -> [f32; 4] {
        [
            self.red.value(),
            self.green.value(),
            self.blue.value(),
            self.alpha.value(),
        ]
    }
    
    /// 检查是否有任何颜色分量正在动画中
    pub fn is_active(&self) -> bool {
        self.red.is_active() || self.green.is_active() || self.blue.is_active() || self.alpha.is_active()
    }
    
    /// 取消所有颜色动画
    pub fn cancel(&mut self) {
        self.red.cancel();
        self.green.cancel();
        self.blue.cancel();
        self.alpha.cancel();
    }
} 