use bevy::prelude::*;
use bevy::ecs::system::{Command, EntityCommands};
use std::ops::AddAssign;
pub trait CommandsExtension {
    fn insert_add<T: Component + AddAssign<T>>(&mut self, component: T) -> &mut Self;
}
impl<'w, 's, 'a> CommandsExtension for EntityCommands<'w, 's, 'a> {
    fn insert_add<T: Component + AddAssign<T>>(&mut self, component: T) -> &mut Self {
        let entity = self.id();
        self.commands().add(InsertAdd { entity, component });
        self
    }
}

pub struct InsertAdd<T: Component>{
    pub entity: Entity,
    pub component: T,
}
impl<T: Component + AddAssign<T>> Command for InsertAdd<T> {
    fn write(self, world: &mut World) {
        let Some(mut entity) = world.get_entity_mut(self.entity) else { return };
        if let Some(mut component) = entity.get_mut::<T>() {
            component.add_assign(self.component);
        } else {
            entity.insert(self.component);
        }
    }
}