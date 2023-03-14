use bevy::prelude::*;
use bevy::utils::FloatOrd;
use std::time::Duration;
use super::ease::{Ease,lerp};
use super::state::AnimationStateMachine;
use super::label::{Label,BoxedLabel};
use super::AnimationEvent;

pub trait AnimatedProperty: Sync + Send + 'static {
    type Component: Component;
    const DEFAULT: Self;
    fn lerp(&self, rhs: &Self, fraction: f32) -> Self;
    fn blend(&self, weight: f32, target: &mut Self::Component);
}

impl AnimatedProperty for BackgroundColor {
    type Component = BackgroundColor;
    const DEFAULT: Self = BackgroundColor(Color::NONE);
    fn lerp(&self, rhs: &Self, fraction: f32) -> Self {
        let prev = self.0.as_linear_rgba_f32();
        let next = rhs.0.as_linear_rgba_f32();
        Color::rgba_linear(
            lerp(prev[0], next[0], fraction),
            lerp(prev[1], next[1], fraction),
            lerp(prev[2], next[2], fraction),
            lerp(prev[3], next[3], fraction),
        ).into()
    }
    fn blend(&self, weight: f32, target: &mut Self::Component) {
        target.0 = Self::lerp(&target.0.into(), self, weight).0;
    }
}

pub struct Track<T: AnimatedProperty> {
    state: Option<BoxedLabel>,
    repeat: i32,
    duration: f32,
    keyframes: Vec<(T, f32, Ease)>
}
impl<T: AnimatedProperty> Track<T> {
    pub fn from_frames(mut keyframes: Vec<(T, f32, Ease)>, repeat: i32) -> Self {
        keyframes.sort_by_key(|frame|FloatOrd(frame.1));
        let mut track = Self { keyframes, duration: 0.0, repeat, state: None };
        track.duration = track.end_time() - track.start_time();
        track
    }
    pub fn from_static(value: T) -> Self {
        Self { keyframes: vec![(value, 0.0, Ease::Linear)], duration: 0.0, repeat: 0, state: None }
    }
    pub fn with_state(mut self, state: impl Label) -> Self {
        self.state = Some(state.dyn_clone());
        self
    }
    pub fn start_time(&self) -> f32 { self.keyframes.first().map_or(0.0, |frame|frame.1) }
    pub fn end_time(&self) -> f32 { self.keyframes.last().map_or(0.0, |frame|frame.1) }
    pub fn duration(&self) -> f32 {
        if self.repeat >= 0 {
            (self.repeat + 1) as f32 * self.duration
        } else {
            f32::INFINITY
        }
    }
    pub fn just_finished(&self, delta_time: Duration, elapsed: Duration) -> bool {
        let prev_elapsed = elapsed.checked_sub(delta_time).unwrap_or_default();
        let total_duration = self.duration();
        prev_elapsed.as_secs_f32() < total_duration && elapsed.as_secs_f32() >= total_duration
    }
    pub fn apply(&self, weight: f32, _delta_time: Duration, elapsed: Duration, mut target: Mut<T::Component>){
        let start_time = self.start_time();
        let total_duration = self.duration();
        let mut elapsed_time = (elapsed.as_secs_f32() - start_time).clamp(0.0, total_duration);
        let repeated = (elapsed_time / self.duration).floor().min(self.repeat as f32);
        elapsed_time = start_time + elapsed_time - repeated * self.duration;

        let (Ok(index) | Err(index)) = self.keyframes.binary_search_by(
            |frame| frame.1.total_cmp(&elapsed_time)
        );
        if index == 0 {
            self.keyframes[0].0.blend(weight, &mut target);
        } else if index == self.keyframes.len() {
            self.keyframes[self.keyframes.len()-1].0.blend(weight, &mut target);
        } else {
            let prev_frame = &self.keyframes[index-1];
            let next_frame = &self.keyframes[index];
            let ratio = (elapsed_time - prev_frame.1) / (next_frame.1 - prev_frame.1);
            T::lerp(&prev_frame.0, &next_frame.0, next_frame.2.calculate(ratio))
            .blend(weight, &mut target);
        }
    }
}

#[derive(Component, Default)]
pub struct Animator<T: AnimatedProperty> {
    pub enabled: bool,
    pub rate: f32,
    elapsed: Duration,
    tracks: Vec<Track<T>>,
}
impl<T: AnimatedProperty> From<Track<T>> for Animator<T> {
    fn from(track: Track<T>) -> Self {
        Self { enabled: true, rate: 1.0, elapsed: Duration::ZERO, tracks: vec![track] }
    }
}
impl<T: AnimatedProperty> Animator<T> {
    pub fn new() -> Self { Self { enabled: true, rate: 1.0, elapsed: Duration::ZERO, tracks: Vec::new() } }
    pub fn with_rate(mut self, rate: f32) -> Self { self.rate = rate; self }
    pub fn set_elapsed(&mut self, elapsed: f32){
        self.elapsed = Duration::from_secs_f32(elapsed);
    }
    pub fn is_playing(&self) -> bool { self.enabled }
    pub fn clear(&mut self){
        self.tracks.clear();
        self.elapsed = Duration::ZERO;
    }
    pub fn add(mut self, track: Track<T>) -> Self {
        self.tracks.push(track);
        self
    }
}

#[derive(Component, Clone)]
pub struct AnimationTimeline {
    rate: f32,
    manual: bool,
    delta: Duration,
    pub elapsed: Duration,
    cleanup: Option<Duration>,
}
impl Default for AnimationTimeline {
    fn default() -> Self { Self { rate: 1.0, manual: false, delta: Duration::ZERO, elapsed: Duration::ZERO, cleanup: None } }
}
impl AnimationTimeline {
    pub fn with_rate(mut self, rate: f32, manual: bool) -> Self {
        self.rate = rate;
        self.manual = manual;
        self
    }
    pub fn with_cleanup(mut self, duration: f32) -> Self {
        self.cleanup = Some(Duration::from_secs_f32(duration));
        self
    }
    pub fn set_elapsed(&mut self, elapsed: Duration){
        self.delta = elapsed - self.elapsed;
        self.elapsed = elapsed;
    }
}

pub fn update_timeline(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut AnimationTimeline)>
){
    for (entity, mut timeline) in query.iter_mut() {
        if timeline.manual { continue; }
        let delta_time = time.delta().mul_f32(timeline.rate);
        timeline.elapsed += delta_time;
        timeline.delta = delta_time;

        if timeline.cleanup.map_or(false, |end_time|timeline.elapsed >= end_time) {
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub fn animate_component<T: AnimatedProperty>(
    mut events: EventWriter<AnimationEvent>,
    time: Res<Time>,
    mut query: Query<(Entity, &mut T::Component, &mut Animator<T>, Option<&AnimationTimeline>)>,
    query_state: Query<(Option<&Parent>, Option<&AnimationStateMachine>), Or<(With<Parent>, With<AnimationStateMachine>)>>
){
    for (mut entity, mut target, mut animator, timeline) in &mut query {
        if !animator.enabled { continue; }
        let state_machine: Option<&AnimationStateMachine> = loop {
            entity = match query_state.get(entity) {
                Ok((_, Some(state_machine))) => break Some(state_machine),
                Ok((Some(parent), None)) => parent.get(),
                _ => break None,
            };
        };

        let (delta_time, elapsed_time) = if let Some(timeline) = timeline {
            (timeline.delta, timeline.elapsed)
        } else {
            let delta_time = time.delta().mul_f32(animator.rate);
            animator.elapsed += delta_time;
            (delta_time, animator.elapsed)
        };

        T::blend(&T::DEFAULT, 1.0, &mut target);
        animator.tracks.retain_mut(|track|{
            let (weight, elapsed) = track.state.as_ref()
            .map_or((1.0, elapsed_time), |state_label|state_machine
            .map_or((0.0, Duration::ZERO), |states|states.calculate_weight(state_label.as_ref())));
            if weight > 0.0 {
                track.apply(weight, delta_time, elapsed, target.reborrow());
                if track.just_finished(delta_time, elapsed) {
                    events.send(AnimationEvent::AnimationEnd);
                }
            }
            true
        });
        animator.enabled = !animator.tracks.is_empty();
    }
}