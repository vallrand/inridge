use std::time::Duration;
use bevy::prelude::*;
use super::label::{Label, BoxedLabel};
use super::AnimationEvent;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum StateTransitionInterruption {
    Never,
    Always,
    Pause
}

#[derive(Clone, Debug)]
pub struct StateMachineTransition {
    pub state: (BoxedLabel, BoxedLabel),
    pub switch_threshold: f32,
    pub duration: Duration,
    pub interrupt: StateTransitionInterruption,
    start_time: Duration,
    weight: f32,
    normalized: f32,
    offset_time: f64,
}
impl StateMachineTransition {
    pub fn new(start: impl Label, end: impl Label, duration: f32) -> Self {
        Self {
            state: (start.dyn_clone(), end.dyn_clone()),
            switch_threshold: 0.0,
            duration: Duration::from_secs_f32(duration),
            start_time: Duration::ZERO,
            offset_time: 0.0,
            interrupt: StateTransitionInterruption::Always,
            weight: 0.0,
            normalized: 0.0,
        }
    }
}

#[derive(Component, Debug)]
pub struct AnimationStateMachine {
    elapsed: Duration,
    mixer_stack: Vec<usize>,
    transitions: Vec<StateMachineTransition>,
    triggers: bevy::utils::HashSet<(BoxedLabel, BoxedLabel)>,

    last_weight: f32,
    last_state: BoxedLabel,
    last_start_time: Duration,
}

impl AnimationStateMachine {
    pub fn new(transitions: Vec<StateMachineTransition>, initial_state: impl Label) -> Self {
        Self {
            elapsed: Duration::ZERO,
            transitions, triggers: bevy::utils::HashSet::new(),
            mixer_stack: Vec::with_capacity(1),
            last_start_time: Duration::ZERO,
            last_state: initial_state.dyn_clone(),
            last_weight: 1.0,
        }
    }
    pub fn is_idle(&self) -> bool { self.mixer_stack.is_empty() }
    pub fn current_state<T: Label>(&self) -> Option<&T> {
        self.current_state_untyped().as_any().downcast_ref::<T>()
    }
    pub fn current_state_untyped(&self) -> &dyn Label {
        self.mixer_stack.first()
        .map(|&index|&self.transitions[index])
        .map_or(self.last_state.as_ref(), |transition|
            if transition.normalized >= transition.switch_threshold {
                transition.state.1.as_ref()
            } else {
                transition.state.0.as_ref()
            }
        )
    }
    pub fn calculate_weight(&self, state_label: &dyn Label) -> (f32, Duration) {
        if self.last_state.as_ref().eq(state_label) { return (self.last_weight, self.elapsed - self.last_start_time); }
        for &transition in self.mixer_stack.iter() {
            let StateMachineTransition { state, weight, start_time, .. } = &self.transitions[transition];
            if state.1.as_ref().eq(state_label) { return (*weight, self.elapsed - *start_time); }
        }
        (0.0, Duration::ZERO)
    }
    pub fn update(&mut self, delta_time: Duration){
        self.elapsed += delta_time;

        let mut total_weight: f32 = 1.0;
        for i in 0..self.mixer_stack.len() {
            let transition = &mut self.transitions[self.mixer_stack[i]];

            if let StateTransitionInterruption::Always = transition.interrupt {
                let elapsed = (self.elapsed - transition.start_time).as_secs_f64() - transition.offset_time;
                transition.normalized = (elapsed / transition.duration.as_secs_f64()).min(1.0) as f32;
            }
            
            if transition.normalized < 1.0 {
                transition.weight = total_weight * transition.normalized;
                total_weight -= transition.weight;
            }else{
                self.last_state = transition.state.1.clone();
                self.last_start_time = transition.start_time;
                self.mixer_stack.truncate(i);
                break;
            }
        }
        self.last_weight = total_weight;

        let current_state = self.current_state_untyped();
        let mut next_transition: Option<usize> = None;
        for (index, transition) in self.transitions.iter().enumerate() {
            if !self.mixer_stack.is_empty() && (
                self.mixer_stack[0] == index ||
                StateTransitionInterruption::Never == self.transitions[self.mixer_stack[0]].interrupt
            ) { continue; }
            if !current_state.eq(transition.state.0.as_ref()) { continue; }
            if self.triggers.contains(&transition.state) {
                next_transition = Some(index);
                break;
            }
        }
        self.triggers.clear();
        if let Some(index) = next_transition {
            let end_state = &self.transitions[index].state.1;
            let (target_weight, start_time) = 'shift: {
                let mut next = index;
                let mut total_weight: f32 = 1.0;
                for i in 0..self.mixer_stack.len() {
                    let weight = total_weight * self.transitions[i].normalized;
                    total_weight -= weight;

                    std::mem::swap(&mut next, &mut self.mixer_stack[i]);
                    if self.transitions[next].state.1.eq(&end_state) {
                        break 'shift (weight, self.transitions[next].start_time);
                    }
                }
                if self.last_state.eq(&end_state) {
                    self.last_state = self.transitions[next].state.1.clone();
                    (total_weight, self.last_start_time)
                } else {
                    self.mixer_stack.push(next);
                    (0.0, self.elapsed)
                }
            };
            self.transitions[index].weight = target_weight;
            self.transitions[index].start_time = start_time;
            let mut total_weight = 1.0;
            for &index in self.mixer_stack.iter() {
                let transition = &mut self.transitions[index];
                transition.normalized = (transition.weight / total_weight).min(1.0);
                let elapsed = transition.duration.mul_f32(transition.normalized).as_secs_f64();
                transition.offset_time = (self.elapsed - transition.start_time).as_secs_f64() - elapsed;
                total_weight -= transition.weight;
            }
            self.last_weight = total_weight;
        }
    }
    pub fn trigger(&mut self, start: Option<&dyn Label>, end: &dyn Label){
        let start = start.unwrap_or(self.last_state.as_ref());
        self.triggers.insert((start.dyn_clone(), end.dyn_clone()));
    }
}

pub fn update_state_machine(
    mut events: EventWriter<AnimationEvent>,
    time: Res<Time>,
    mut query: Query<(Entity, &mut AnimationStateMachine)>
){
    for (entity, mut state_machine) in &mut query {
        let was_idle = state_machine.is_idle();
        state_machine.update(time.delta());
        if state_machine.is_idle() && !was_idle {
            events.send(AnimationEvent::TransitionEnd(entity));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Clone, Hash, Debug, PartialEq, Eq)]
    pub enum StateLabel { A, B }
    #[test] pub fn animation_state_machine(){
        let mut state_machine = AnimationStateMachine::new(vec![
            StateMachineTransition::new(StateLabel::A, StateLabel::B, 1.0)
        ], StateLabel::A);
        assert_eq!(state_machine.current_state::<StateLabel>(), Some(&StateLabel::A));
        assert_eq!(state_machine.calculate_weight(&StateLabel::A), (1.0, Duration::ZERO));
        assert_eq!(state_machine.calculate_weight(&StateLabel::B), (0.0, Duration::ZERO));
        state_machine.trigger(None, &StateLabel::B);
        state_machine.update(Duration::from_secs_f32(0.5));
        assert_eq!(state_machine.current_state::<StateLabel>(), Some(&StateLabel::B));
        assert_eq!(state_machine.calculate_weight(&StateLabel::A), (1.0, Duration::from_secs_f32(0.5)));
        assert_eq!(state_machine.calculate_weight(&StateLabel::B), (0.0, Duration::ZERO));
        state_machine.update(Duration::from_secs_f32(0.5));
        assert_eq!(state_machine.current_state::<StateLabel>(), Some(&StateLabel::B));
        assert_eq!(state_machine.calculate_weight(&StateLabel::A), (0.5, Duration::from_secs_f32(1.0)));
        assert_eq!(state_machine.calculate_weight(&StateLabel::B), (0.5, Duration::from_secs_f32(0.5)));
        state_machine.update(Duration::from_secs_f32(1.0));
        assert_eq!(state_machine.is_idle(), true);
        assert_eq!(state_machine.current_state::<StateLabel>(), Some(&StateLabel::B));
        assert_eq!(state_machine.calculate_weight(&StateLabel::A), (0.0, Duration::from_secs_f32(0.0)));
        assert_eq!(state_machine.calculate_weight(&StateLabel::B), (1.0, Duration::from_secs_f32(1.5)));
    }
}