use bevy::prelude::*;
use crate::scene::UnitBlueprint;
use crate::logic::{MatterBinding, MilitaryBinding, FabricationGate, Suspended, UnderConstruction, Integrity, LandingProbe};

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum AnimationState {
    Disabled,
    Deficit,
    Enabled,
    Active,
}

pub fn extract_unit_animation_trigger(
    mut commands: Commands,
    mut query_unit: ParamSet<(
        Query<Entity, (With<AnimationState>, Or<(With<UnderConstruction>, Without<Integrity>)>)>,
        Query<(
            Entity, Option<&AnimationState>,
            Option<&MatterBinding>, Option<&MilitaryBinding>, Option<&Suspended>,
            Option<&FabricationGate>, Option<&LandingProbe>
        ), (Without<UnderConstruction>, With<Integrity>, With<Handle<UnitBlueprint>>)>,
    )>    
){
    for entity in query_unit.p0().iter() {
        commands.entity(entity).remove::<AnimationState>();
    }
    for (
        entity, condition,
        matter, military, suspended,
        gate, landing
    ) in query_unit.p1().iter() {
        let next_condition = if suspended.is_some() {
            AnimationState::Disabled
        } else if match matter {
            Some(MatterBinding::Consumption(consumption)) => !consumption.active(),
            _ => false
        } {
            AnimationState::Deficit
        } else if landing.is_some() || gate.is_some() || match military {
            Some(MilitaryBinding::Connection { released, .. }) => *released > 0,
            _ => false 
        } {
            AnimationState::Active
        } else {
            AnimationState::Enabled
        };

        if condition.map_or(true,|prev_condition|!next_condition.eq(prev_condition)) {
            commands.entity(entity).insert(next_condition);
        }
    }
}