use bevy::prelude::*;

#[derive(Component, serde::Deserialize, Clone, Copy, PartialEq, Eq, Hash, Default, Debug)]
pub enum Agent {
    #[default] Player,
    AI(u8)
}