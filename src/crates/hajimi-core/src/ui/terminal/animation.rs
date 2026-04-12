//! AnimationEngine - B15-01: 60fps Terminal Animation Core
//! Target: 16ms frame time (60fps), max 32 concurrent animations, 0-10s duration
use std::time::{Duration, Instant};
use super::layout::Rect;

const MAX_ANIMATIONS: usize = 32;
const MAX_DURATION_SECS: f32 = 10.0;
const FRAME_BUDGET_MS: u64 = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Easing { Linear, QuadOut, QuadIn, CubicOut, CubicIn }

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Animation {
    pub id: u64, pub start_time: Instant, pub duration: Duration,
    pub easing: Easing, pub from_value: f32, pub to_value: f32, pub target_region: Rect,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnimationError { TooManyAnimations, InvalidDuration, InvalidValue }

#[derive(Debug, Clone)]
pub struct AnimationEngine {
    animations: Vec<Animation>, dirty_regions: Vec<Rect>, enabled: bool, last_tick: Instant,
}

impl AnimationEngine {
    pub fn new() -> Self {
        Self { animations: Vec::with_capacity(MAX_ANIMATIONS), dirty_regions: Vec::new(), enabled: true, last_tick: Instant::now() }
    }

    pub fn tick(&mut self, dt: Duration) -> Vec<Rect> {
        if !self.enabled || self.animations.is_empty() { return Vec::new(); }
        let now = self.last_tick.checked_add(dt).unwrap_or_else(Instant::now);
        self.last_tick = now; self.dirty_regions.clear();
        let mut i = 0;
        while i < self.animations.len() {
            let anim = &self.animations[i];
            let elapsed = now.saturating_duration_since(anim.start_time);
            if elapsed >= anim.duration || anim.duration.is_zero() { self.animations.swap_remove(i); continue; }
            self.dirty_regions.push(anim.target_region); i += 1;
        }
        self.dirty_regions.clone()
    }

    pub fn add_animation(&mut self, anim: Animation) -> Result<(), AnimationError> {
        if self.animations.len() >= MAX_ANIMATIONS { return Err(AnimationError::TooManyAnimations); }
        if anim.duration > Duration::from_secs_f32(MAX_DURATION_SECS) { return Err(AnimationError::InvalidDuration); }
        if !Self::is_valid_f32(anim.from_value) || !Self::is_valid_f32(anim.to_value) { return Err(AnimationError::InvalidValue); }
        self.dirty_regions.push(anim.target_region); self.animations.push(anim); Ok(())
    }

    pub fn remove_animation(&mut self, id: u64) -> bool {
        if let Some(idx) = self.animations.iter().position(|a| a.id == id) {
            self.dirty_regions.push(self.animations[idx].target_region);
            self.animations.swap_remove(idx); true
        } else { false }
    }

    pub fn calculate_easing(easing: Easing, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match easing {
            Easing::Linear => t,
            Easing::QuadOut => 1.0 - (1.0 - t) * (1.0 - t),
            Easing::QuadIn => t * t,
            Easing::CubicOut => { let t1 = 1.0 - t; 1.0 - t1 * t1 * t1 }
            Easing::CubicIn => t * t * t,
        }
    }

    pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
        if !Self::is_valid_f32(a) || !Self::is_valid_f32(b) || !Self::is_valid_f32(t) { return 0.0; }
        a + (b - a) * t.clamp(0.0, 1.0)
    }

    fn is_valid_f32(v: f32) -> bool { !v.is_nan() && !v.is_infinite() }
    pub fn set_enabled(&mut self, enabled: bool) { self.enabled = enabled; }
    pub fn is_enabled(&self) -> bool { self.enabled }
    pub fn animation_count(&self) -> usize { self.animations.len() }
    pub fn dirty_region_count(&self) -> usize { self.dirty_regions.len() }
    pub fn clear(&mut self) { self.animations.clear(); self.dirty_regions.clear(); }
    pub fn frame_budget() -> Duration { Duration::from_millis(FRAME_BUDGET_MS) }
}

impl Default for AnimationEngine { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    fn make_anim(id: u64, dur_ms: u64, x: u16, y: u16) -> Animation {
        Animation { id, start_time: Instant::now(), duration: Duration::from_millis(dur_ms), easing: Easing::Linear, from_value: 0.0, to_value: 100.0, target_region: Rect { x, y, width: 10, height: 10 } }
    }

    #[test] fn test_easing_linear() {
        assert_eq!(AnimationEngine::calculate_easing(Easing::Linear, 0.0), 0.0);
        assert_eq!(AnimationEngine::calculate_easing(Easing::Linear, 0.5), 0.5);
        assert_eq!(AnimationEngine::calculate_easing(Easing::Linear, 1.0), 1.0);
    }

    #[test] fn test_easing_quad() {
        let out = AnimationEngine::calculate_easing(Easing::QuadOut, 0.5);
        assert!(out > 0.5 && out < 1.0);
        let inp = AnimationEngine::calculate_easing(Easing::QuadIn, 0.5);
        assert!(inp > 0.0 && inp < 0.5);
    }

    #[test] fn test_animation_tick() {
        let mut engine = AnimationEngine::new();
        assert!(engine.add_animation(make_anim(1, 100, 0, 0)).is_ok());
        assert_eq!(engine.tick(Duration::from_millis(50)).len(), 1);
        assert_eq!(engine.tick(Duration::from_millis(100)).len(), 0);
        assert_eq!(engine.animation_count(), 0);
    }

    #[test] fn test_dirty_regions() -> Result<(), AnimationError> {
        let mut engine = AnimationEngine::new();
        engine.add_animation(make_anim(1, 500, 0, 0))?;
        engine.add_animation(make_anim(2, 500, 20, 20))?;
        assert_eq!(engine.tick(Duration::from_millis(16)).len(), 2);
        Ok(())
    }

    #[test] fn test_memory_limit() {
        let mut engine = AnimationEngine::new();
        for i in 0..=MAX_ANIMATIONS {
            let r = engine.add_animation(make_anim(i as u64, 100, 0, 0));
            if i < MAX_ANIMATIONS { assert!(r.is_ok()); } else { assert_eq!(r, Err(AnimationError::TooManyAnimations)); }
        }
    }

    #[test] fn test_zero_duration() -> Result<(), AnimationError> {
        let mut engine = AnimationEngine::new();
        engine.add_animation(make_anim(1, 0, 0, 0))?;
        assert!(engine.tick(Duration::ZERO).is_empty());
        assert_eq!(engine.animation_count(), 0);
        Ok(())
    }

    #[test] fn test_invalid_duration() {
        let mut engine = AnimationEngine::new();
        let anim = Animation { id: 1, start_time: Instant::now(), duration: Duration::from_secs_f32(15.0), easing: Easing::Linear, from_value: 0.0, to_value: 100.0, target_region: Rect { x: 0, y: 0, width: 10, height: 10 } };
        assert_eq!(engine.add_animation(anim), Err(AnimationError::InvalidDuration));
    }

    #[test] fn test_lerp() {
        assert_eq!(AnimationEngine::lerp(0.0, 100.0, 0.0), 0.0);
        assert_eq!(AnimationEngine::lerp(0.0, 100.0, 0.5), 50.0);
        assert_eq!(AnimationEngine::lerp(0.0, 100.0, 1.0), 100.0);
        assert_eq!(AnimationEngine::lerp(0.0, 100.0, f32::NAN), 0.0);
        assert_eq!(AnimationEngine::lerp(0.0, 100.0, f32::INFINITY), 0.0);
    }

    #[test] fn test_remove_animation() -> Result<(), AnimationError> {
        let mut engine = AnimationEngine::new();
        engine.add_animation(make_anim(1, 500, 0, 0))?;
        assert!(engine.remove_animation(1));
        assert!(!engine.remove_animation(1));
        assert_eq!(engine.animation_count(), 0);
        Ok(())
    }

    #[test] fn test_disabled_engine() -> Result<(), AnimationError> {
        let mut engine = AnimationEngine::new();
        engine.set_enabled(false);
        engine.add_animation(make_anim(1, 500, 0, 0))?;
        assert!(engine.tick(Duration::from_millis(16)).is_empty());
        engine.set_enabled(true);
        assert_eq!(engine.tick(Duration::from_millis(16)).len(), 1);
        Ok(())
    }
}
