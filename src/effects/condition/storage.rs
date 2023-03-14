use bevy::prelude::*;
use crate::extensions::EntityLookupTable;
use crate::logic::MatterBinding;
use crate::materials::MatterEffectMaterial;

pub fn animate_collector_storage(
    fixed_time: Res<FixedTime>,
    mut query: Query<(Entity, &mut MatterBinding, Option<&EntityLookupTable>)>,
    children: Query<&Children>,
    mut query_transform: Query<(&mut Transform, &mut Visibility), (With<Handle<Mesh>>, With<Handle<MatterEffectMaterial>>)>,
){
    let fraction = fixed_time.accumulated().as_secs_f32() / fixed_time.period.as_secs_f32();
    
    for (entity, mut matter, lookup) in query.iter_mut() {
        let MatterBinding::Collection(storage) = matter.as_mut() else { continue };
        let height = 0.5 * storage.calculate(fraction) * storage.tier() as f32;
        
        let Some((mut transform, mut visibility)) = lookup
        .and_then(|lookup|lookup.get(&storage.key).cloned())
        .or_else(||
            children.iter_descendants(entity).find(|&entity|query_transform.contains(entity))
        ).and_then(|entity|query_transform.get_mut(entity).ok()) else { continue };

        transform.scale.y = height;
        if visibility.eq(&Visibility::Hidden) && height > 0.0 {
            *visibility = Visibility::Inherited;
        } else if !visibility.eq(&Visibility::Hidden) && height == 0.0 {
            *visibility = Visibility::Hidden;
        }
    }
}