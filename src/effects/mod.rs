pub mod animation;
pub mod outline;
pub mod combat;
pub mod condition;
pub mod linker;

use bevy::prelude::*;
use crate::common::loader::LoadingState;
use crate::extensions::cache_scene_entity_lookup_table;
use crate::logic::LogicSet;
use crate::scene::GlobalState;

pub struct EffectsPlugin; impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<animation::AnimationSettings>();
        app.init_resource::<condition::MembershipSettings>();
        app.add_system(condition::setup_membership_materials.in_schedule(OnEnter(LoadingState::Running)));

        app.add_systems((
            animation::extract_unit_animation_trigger,
            animation::update_movement_formation,
            animation::play_unit_animation,
            animation::animate_unit_relocation,
            animation::animate_unit_landing,
            animation::animate_unit_path_traversal.after(animation::update_movement_formation),
            animation::animate_unit_movement.after(animation::update_movement_formation),

            combat::animate_projectile_effect,
            combat::animate_tentacle_effect,
            combat::animate_dome_effect,
            combat::animate_hit_effect,
            combat::animate_impact_effect,
        ).after(LogicSet::PostUpdate)
        .in_set(OnUpdate(GlobalState::Running))
        .in_set(OnUpdate(LoadingState::Running)));

        app.add_systems((
            linker::link_structures,
            
            outline::update_grid_tile_highlight,
            outline::update_grid_group_highlight,
            outline::update_grid_affected_highlight,
            outline::animate_selected_path,
            outline::update_military_range,
        ).after(LogicSet::PostUpdate)
        .in_set(OnUpdate(LoadingState::Running)));

        app.add_systems((
            condition::apply_unit_membership,
            condition::animate_unit_condition_suspended,
            condition::animate_unit_condition_damaged,
            condition::animate_unit_condition_deficit,
            condition::animate_unit_condition_fabricated,
            condition::animate_construction.after(condition::apply_unit_membership),
            condition::animate_reconstruction,
            condition::animate_destruction,
            condition::animate_collector_storage.after(condition::apply_unit_membership),
        ).after(LogicSet::PostUpdate)
        .in_set(OnUpdate(GlobalState::Running))
        .in_set(OnUpdate(LoadingState::Running)));

        app.add_system(combat::reorient_targeting_systems
            .before(bevy::transform::TransformSystem::TransformPropagate)
            .after(bevy::animation::animation_player)
            .in_base_set(CoreSet::PostUpdate)
        );

        app.add_system(cache_scene_entity_lookup_table.after(bevy::scene::scene_spawner_system));
    }
}