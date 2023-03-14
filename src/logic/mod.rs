mod agent;
mod group;
mod event;

mod foundation;
mod economy;
mod distribution;
mod military;
mod fabrication;
mod movement;
mod terrain;
mod strategy;

pub use agent::*;
use bevy::time::common_conditions::on_timer;
pub use group::*;
pub use event::*;

pub use foundation::*;
pub use economy::*;
pub use distribution::*;
pub use terrain::grid::*;
pub use terrain::generation::*;
pub use terrain::lookup::*;
pub use fabrication::*;
pub use military::*;
pub use movement::*;
pub use strategy::*;

use bevy::prelude::*;
use bevy::time::fixed_timestep::run_fixed_update_schedule;
use crate::common::loader::LoadingState;
use crate::scene::GlobalState;

#[derive(SystemSet, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum LogicSet {
    PreUpdate,
    PreFixedUpdate,
    PreFixedUpdateFlush,
    FixedUpdate,
    PostFixedUpdate,
    PostUpdate,
}
pub struct LogicPlugin; impl Plugin for LogicPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<GlobalState>();
        app.configure_set(LogicSet::PreUpdate
            .run_if(in_state(LoadingState::Running))
            .in_base_set(CoreSet::FixedUpdate)
            .before(run_fixed_update_schedule));
        app.configure_set(LogicSet::PostUpdate
            .in_set(OnUpdate(LoadingState::Running))
            .after(bevy::scene::scene_spawner_system));

        app.edit_schedule(CoreSchedule::FixedUpdate, |schedule|{
            schedule.configure_sets((
                LogicSet::PreFixedUpdate.run_if(in_state(LoadingState::Running)).run_if(in_state(GlobalState::Running)),
                LogicSet::PreFixedUpdateFlush.run_if(in_state(LoadingState::Running)).run_if(in_state(GlobalState::Running)),
                LogicSet::FixedUpdate.run_if(in_state(LoadingState::Running)).run_if(in_state(GlobalState::Running)),
                LogicSet::PostFixedUpdate.run_if(in_state(LoadingState::Running)).run_if(in_state(GlobalState::Running)),
            ).chain());
            schedule.add_system(apply_system_buffers.in_set(LogicSet::PreFixedUpdateFlush));
        });

        app.add_event::<event::ConstructionEvent>();
        app.add_event::<event::CombatEvent>();

        app.add_systems((
            movement::execute_structure_relocation,
            movement::execute_probe_landing,
            movement::execute_movement_directives,
            military::redirect_unit_directive,
            foundation::construction_phase,
            foundation::destruction_phase,
            fabrication::expiration_phase,
        ).chain().in_set(LogicSet::PreFixedUpdate).in_schedule(CoreSchedule::FixedUpdate));

        app.add_systems((
            economy::reset_economy_phase,
            economy::production_phase,
            economy::reservation_phase,
            economy::resource_allocation_phase,
            economy::collection_phase,
            foundation::reconstruction_phase,
            fabrication::fabrication_phase,

            distribution::expiration_phase,
            distribution::propagation_phase,

            military::apply_degradation_effect,
            military::propagate_degradation_effect,

            apply_system_buffers,
            military::resupply_military_phase,
        ).chain().in_set(LogicSet::FixedUpdate).in_schedule(CoreSchedule::FixedUpdate));

        app.init_resource::<terrain::lookup::SpatialLookupGrid<Entity>>();
        app.init_resource::<economy::GlobalEconomy>();
        app.add_system(terrain::lookup::update_spatial_lookup_grid::<(With<GridTileIndex>, With<Integrity>)>
            .in_base_set(CoreSet::First));
        app.add_systems((
            military::update_military_targeting,
            military::apply_combat_damage,
        ).chain().in_set(OnUpdate(LoadingState::Running)));

        app.add_system(group::relink_network_group.in_set(LogicSet::PreUpdate));
        app.add_system(group::relink_network_group.in_schedule(CoreSchedule::FixedUpdate)
        .after(LogicSet::PreFixedUpdateFlush).before(LogicSet::FixedUpdate));

        app.init_resource::<StrategySettings>();
        app.add_system(strategy::strategical_planning_phase
            .in_set(LogicSet::PostFixedUpdate)
            .run_if(on_timer(std::time::Duration::from_secs_f32(5.0)))
        );

        app.insert_resource(FixedTime::new_from_secs(0.5 * 1.0));
    }
}