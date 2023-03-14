use bevy::prelude::*;
use crate::interaction::{EventTrigger, InteractionEvent};

pub fn dispatch_interaction_events(
    mut interaction_events: EventWriter<InteractionEvent>,
    mut interaction_query: Query<(&Interaction, &EventTrigger<InteractionEvent>), (Changed<Interaction>, With<Button>)>,
){
    for (&interaction, action) in interaction_query.iter_mut() {
        match interaction {
            Interaction::Clicked => {
                interaction_events.send(action.0.clone());
            },
            Interaction::Hovered => {},
            Interaction::None => {}
        }
    }
}