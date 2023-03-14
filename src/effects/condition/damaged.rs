use bevy::prelude::*;
use crate::extensions::EntityLookupTable;
use crate::materials::{ColorUniform, ModelEffectLayeredMaterial};
use crate::logic::{Integrity, UnderConstruction};

pub fn animate_unit_condition_damaged(
    fixed_time: Res<FixedTime>,
    query_unit: Query<(Entity, &Integrity, Option<&UnderConstruction>), Or<(Changed<Integrity>, Changed<EntityLookupTable>)>>,
    children: Query<&Children>,
    mut query: Query<&mut ColorUniform, With<Handle<ModelEffectLayeredMaterial>>>,
){
    let fraction = fixed_time.accumulated().as_secs_f32() / fixed_time.period.as_secs_f32();
    for (entity, integrity, construction) in query_unit.iter() {
        let construction_percent = construction.map_or(1.0,|construction|construction.calculate(fraction));
        let damage_percent = integrity.calculate_damage(fraction, construction_percent);

        for entity in children.iter_descendants(entity) {
            let Ok(mut uniform) = query.get_mut(entity) else { continue };
            uniform.color.set_g(damage_percent.min(1.0));
        }
    }
}