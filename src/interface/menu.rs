use bevy::prelude::*;
use crate::common::loader::AssetBundle;
use crate::scene::{GlobalState, InterfaceAssetBundle};
use crate::interaction::{InteractionEvent, EventTrigger};

pub fn update_menu_screen(
    mut commands: Commands,
    interface_bundle: Res<AssetBundle<InterfaceAssetBundle>>,
    state: Res<State<GlobalState>>,
    mut component: Local<Option<Entity>>,
){
    let menu = match state.0 {
        GlobalState::Menu => true,
        _ => false
    };
    if menu == component.is_some() { return; }
    if let Some(entity) = component.take() {
        commands.entity(entity).despawn_recursive();
    }
    if !menu { return; }
    let entity = commands.spawn(NodeBundle {
        style: Style {
            position_type: PositionType::Absolute, size: Size::all(Val::Percent(100.0)),
            flex_direction: FlexDirection::Column, justify_content: JustifyContent::Center, align_items: AlignItems::Center,
            ..Default::default()
        },
        focus_policy: bevy::ui::FocusPolicy::Block,
        z_index: ZIndex::Local(16), ..Default::default()
    }).id();

    commands.spawn(ButtonBundle {
        style: Style {
            align_items: AlignItems::Center, justify_content: JustifyContent::Center,
            aspect_ratio: Some(4.0), size: Size::height(Val::Px(40.0)),
            ..Default::default()
        },
        background_color: interface_bundle.color_enabled.clone().into(),
        image: interface_bundle.panel_extended.clone().into(),
        ..Default::default()
    })
    .insert(EventTrigger(InteractionEvent::Start(0)))
    .with_children(|parent|{
        parent.spawn(TextBundle {
            text: Text::from_section("PLAY", interface_bundle.text_style_primary.clone()),
            ..Default::default()
        });
    }).set_parent(entity);

    commands.spawn(ButtonBundle {
        style: Style {
            align_items: AlignItems::Center, justify_content: JustifyContent::Center,
            aspect_ratio: Some(4.0), size: Size::height(Val::Px(40.0)),
            ..Default::default()
        },
        background_color: interface_bundle.color_enabled.clone().into(),
        image: interface_bundle.panel_extended.clone().into(),
        ..Default::default()
    })
    .insert(EventTrigger(InteractionEvent::Exit))
    .with_children(|parent|{
        parent.spawn(TextBundle {
            text: Text::from_section("EXIT", interface_bundle.text_style_primary.clone()),
            ..Default::default()
        });
    }).set_parent(entity);

    component.replace(entity);
}