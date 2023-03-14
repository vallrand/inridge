use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use crate::common::animation::{AnimationStateMachine, StateMachineTransition, Animator, Track};
use crate::interaction::{EventTrigger, InteractionEvent, SelectionState};
use super::layout::OverlayLayout;

#[derive(Clone, Default)]
pub struct ControlComponentDescriptor {
    pub quadrant: u8,
    pub size: f32,
    pub angle: f32,
    pub image_panel: Handle<Image>,
    pub image_icon: Handle<Image>,
    pub color_enabled: Color,
    pub color_active: Option<Color>,
    pub text_style: Option<TextStyle>
}

pub struct ControlComponent {
    pub panel: Entity,
    pub icon: Entity,
    pub label: Option<Entity>,
    pub state: SelectionState,
    descriptor: ControlComponentDescriptor,
    trigger: Option<InteractionEvent>,
}

impl ControlComponent {
    pub fn set_trigger(&mut self, commands: &mut Commands, event: InteractionEvent){
        if self.trigger.as_ref().map_or(true,|e|!event.eq(e)) {
            self.trigger = Some(event.clone());
            commands.entity(self.panel).insert(EventTrigger(event));
        }
    }
    pub fn clear_trigger(&mut self, commands: &mut Commands){
        if self.trigger.is_some() {
            self.trigger = None;
            commands.entity(self.panel).remove::<EventTrigger<InteractionEvent>>();
        }
    }
    pub fn set_state(&mut self, commands: &mut Commands, next_state: SelectionState){
        if self.state == next_state { return; }
        self.state = next_state;
        if self.descriptor.color_active.is_some() {
            commands.entity(self.panel).insert(match self.state {
                SelectionState::Active => SelectionState::Active,
                _ => SelectionState::Enabled
            });
        }
        commands.entity(self.panel).insert(match next_state {
            SelectionState::Active | SelectionState::Enabled => Visibility::Inherited,
            _ => Visibility::Hidden
        });
    }
    pub fn set_label(&mut self, commands: &mut Commands, label: String){
        let (Some(entity), Some(text_style)) = (self.label, self.descriptor.text_style.as_ref()) else { return };
        commands.entity(entity).insert(Text::from_section(label, text_style.clone()));
    }
    pub fn new(
        commands: &mut Commands, layout: &OverlayLayout, descriptor: ControlComponentDescriptor
    ) -> Self {
        let panel = commands.spawn(ButtonBundle {
            style: Style {
                position_type: PositionType::Absolute, aspect_ratio: Some(1.0),
                size: Size::new(Val::Auto, Val::Percent(descriptor.size)),
                position: layout.radial_placement(layout.inner_radius, descriptor.angle, descriptor.size, descriptor.quadrant),
                justify_content: JustifyContent::End, align_items: AlignItems::Center,
                ..Default::default()
            },
            focus_policy: FocusPolicy::Block,
            background_color: descriptor.color_enabled.into(),
            image: descriptor.image_panel.clone().into(),
            ..Default::default()
        })
        .insert(Visibility::Hidden)
        .set_parent(layout.quadrants[descriptor.quadrant as usize]).id();

        let label = descriptor.text_style.as_ref().map(|text_style|{
            commands.spawn(TextBundle{
                style: Style{ margin: UiRect::right(Val::Px(16.0)), ..Default::default() },
                text: Text::from_section("", text_style.clone()),
                ..Default::default()
            }).set_parent(panel).id()
        });

        let icon = commands.spawn(ImageBundle {
            style: Style { aspect_ratio: Some(1.0), size: Size::height(Val::Percent(100.0)), ..Default::default() },
            background_color: descriptor.color_enabled.into(),
            image: descriptor.image_icon.clone().into(),
            ..Default::default()
        }).set_parent(panel).id();

        if let Some(color_active) = descriptor.color_active {
            commands.entity(panel).insert((
                SelectionState::Enabled,
                AnimationStateMachine::new(vec![
                    StateMachineTransition::new(SelectionState::Enabled, SelectionState::Active, 0.2),
                    StateMachineTransition::new(SelectionState::Active, SelectionState::Enabled, 0.2),
                ], SelectionState::Enabled),
                Animator::<BackgroundColor>::new()
                    .add(Track::from_static(color_active.into()).with_state(SelectionState::Active))
                    .add(Track::from_static(descriptor.color_enabled.into()).with_state(SelectionState::Enabled)),
            ));
            commands.entity(icon).insert((
                Animator::<BackgroundColor>::new()
                .add(Track::from_static(color_active.into()).with_state(SelectionState::Active))
                .add(Track::from_static(descriptor.color_enabled.into()).with_state(SelectionState::Enabled)),
            ));
        }

        Self { panel, icon, label, state: SelectionState::Disabled, descriptor, trigger: None }
    }
}