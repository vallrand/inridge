use bevy::prelude::*;
use crate::common::loader::AssetBundle;
use crate::common::geometry::assign_mesh_color;
use crate::logic::{
    MatterBinding,Integrity,UnderConstruction,Suspended,UnitFabrication,
    MapGrid,NetworkGroupList,GroupLink,MilitaryBinding,
};
use crate::interaction::GridSelection;
use crate::scene::InterfaceAssetBundle;
use crate::materials::{MatterIndicatorMaterial};
use super::layout::OverlayLayout;

pub fn update_indicator_display(
    fixed_time: Res<FixedTime>,
    layout: Res<OverlayLayout>,
    mut commands: Commands,
    mut materials: ResMut<Assets<MatterIndicatorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut matter_component: Local<Option<IndicatorBarComponent>>,
    mut components: Local<Vec<IndicatorSingleComponent>>,
    interface_asset_bundle: Res<AssetBundle<InterfaceAssetBundle>>,
    query_grid: Query<(&MapGrid, &NetworkGroupList, &GridSelection)>,
    query_unit: Query<(
        &GroupLink, &Integrity, Option<&MatterBinding>, Option<&UnderConstruction>,
        Option<&Suspended>, Option<&UnitFabrication>, Option<&MilitaryBinding>,
    )>,
){
    let Ok((grid, groups, selection)) = query_grid.get_single() else { return };
    let fraction = fixed_time.accumulated().as_secs_f32() / fixed_time.period.as_secs_f32();
    let matter_component = matter_component.get_or_insert_with(||
        IndicatorBarComponent::new(&mut commands, &layout, &interface_asset_bundle, 0)
    );

    let mut offset = 0;
    if let Some((
        group,
        integrity, matter, construction,
        suspended, fabrication, military,
    )) = grid.tiles[selection.0].reference.and_then(|entity|query_unit.get(entity).ok()) {
        let Some(group) = group.0.map(|index|&groups[index]) else { return };

        commands.entity(matter_component.text).insert(Text::from_section(
            format!("{}/{}+{}", group.summary.matter_consumption, group.summary.matter_production, group.summary.matter_reservation),
            interface_asset_bundle.text_style_primary.clone()));

        let material = materials.get_mut(&interface_asset_bundle.matter_material).unwrap();
        material.fraction = (
            group.summary.matter_consumption as f32 / (group.summary.matter_production + group.summary.matter_reservation) as f32
        ).min(1.0);

        if components.len() <= offset { components.push(IndicatorSingleComponent::new(&mut commands, &layout, &mut meshes, &interface_asset_bundle, offset)); }

        match (suspended, construction, matter) {
            (None, Some(construction), _) => {
                components[offset].update_as_bar(
                    &mut commands, &mut meshes,
                    &interface_asset_bundle.icon_build, construction.calculate(fraction), construction.tier(),
                );
                matter_component.update_flow_direction(&mut commands, -1);
            },
            (None, None, Some(MatterBinding::Production(production))) => {
                components[offset].update_as_value(&mut commands, production.extracted);
                matter_component.update_flow_direction(&mut commands, 1);
            },
            (None, None, Some(MatterBinding::Consumption(consumption))) => {
                components[offset].update_as_value(&mut commands, -consumption.calculated);
                matter_component.update_flow_direction(&mut commands, -1);
            },
            (None, None, Some(MatterBinding::Collection(storage))) => {
                components[offset].update_as_bar(
                    &mut commands, &mut meshes,
                    &interface_asset_bundle.icon_matter, storage.calculate(fraction), storage.tier(),
                );
                matter_component.update_flow_direction(&mut commands, -storage.delta());
            },
            _ => {
                components[offset].update_as_empty(&mut commands);
                matter_component.update_flow_direction(&mut commands, 0);
            }
        }

        offset += 1;
        if components.len() <= offset { components.push(IndicatorSingleComponent::new(&mut commands, &layout, &mut meshes, &interface_asset_bundle, offset)); }

        let construction_percent = construction.map_or(1.0,|construction|construction.calculate(fraction));
        components[offset].update_as_bar(
            &mut commands, &mut meshes,
            &interface_asset_bundle.icon_shield, integrity.calculate(fraction, construction_percent),integrity.tier(),
        );
        offset += 1;

        if let Some(fabrication) = fabrication {
            if components.len() <= offset { components.push(IndicatorSingleComponent::new(&mut commands, &layout, &mut meshes, &interface_asset_bundle, offset)); }
            components[offset].update_as_bar(
                &mut commands, &mut meshes,
                &interface_asset_bundle.icon_radius, fabrication.calculate(fraction), fabrication.total_metric(),
            );
            offset += 1;
        }

        if let Some((value, metric)) = match military {
            Some(MilitaryBinding::Trajectory { cooldown_timer, .. }) => Some((cooldown_timer.percent(), 1)),
            Some(MilitaryBinding::Connection { limit, released, .. }) => Some((*released as f32, *limit)),
            _ => None
        } {
            if components.len() <= offset { components.push(IndicatorSingleComponent::new(&mut commands, &layout, &mut meshes, &interface_asset_bundle, offset)); }
            components[offset].update_as_bar(
                &mut commands, &mut meshes,
                &interface_asset_bundle.icon_rate, value, metric,
            );
            offset += 1;
        }
    }
    for i in offset..components.len() {
        components[i].update_as_empty(&mut commands);
    }
}

pub struct IndicatorSingleComponent {
    panel: Entity,
    empty: Entity,
    icon: Entity,
    text: Entity,
    text_style: TextStyle,
    mesh_handle: Handle<Mesh>,
}
impl IndicatorSingleComponent {
    pub fn new(
        commands: &mut Commands,
        layout: &OverlayLayout,
        meshes: &mut Assets<Mesh>,
        interface_asset_bundle: &AssetBundle<InterfaceAssetBundle>,
        offset: usize,
    ) -> Self {
        let size = 12.0;
        let angle = std::f32::consts::PI - (1.0 - offset as f32 / 4.0).asin();

        let panel = commands.spawn(ImageBundle {
            style: Style {
                position_type: PositionType::Absolute, aspect_ratio: Some(1.0),
                size: Size::new(Val::Auto, Val::Percent(size)),
                position: layout.radial_placement(layout.inner_radius, angle, size, 2),
                justify_content: JustifyContent::FlexStart, align_items: AlignItems::Center,
                ..Default::default()
            },
            background_color: interface_asset_bundle.color_secondary.into(),
            image: interface_asset_bundle.panel_single.clone().into(),
            ..Default::default()
        }).set_parent(layout.quadrants[2]).id();
        let icon = commands.spawn(ImageBundle {
            style: Style {
                align_items: AlignItems::Center, justify_content: JustifyContent::Center,
                aspect_ratio: Some(1.0), size: Size::height(Val::Percent(100.0)), flex_shrink: 0.0,
                ..Default::default()
            },
            background_color: interface_asset_bundle.color_enabled.into(),
            image: interface_asset_bundle.icon_shield.clone().into(),
            ..Default::default()
        }).set_parent(panel).id();
    
        let mut quad_mesh = Mesh::from(shape::Quad::new(Vec2::ONE));
        assign_mesh_color(&mut quad_mesh, Color::NONE);
        let mesh_handle = meshes.add(quad_mesh);
    
        commands.spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                aspect_ratio: Some(1.0), size: Size::height(Val::Percent(200.0)),
                ..Default::default()
            }, ..Default::default()
        }).insert((
            interface_asset_bundle.radial_material.clone(),
            bevy::sprite::Mesh2dHandle::from(mesh_handle.clone()),
        )).set_parent(icon);

        let empty = commands.spawn(ImageBundle {
            style: Style {
                position_type: PositionType::Absolute, aspect_ratio: Some(1.0),
                size: Size::new(Val::Auto, Val::Percent(size + 6.0)),
                position: layout.radial_placement(layout.inner_radius, angle, size + 6.0, 2),
                justify_content: JustifyContent::Center, align_items: AlignItems::Center,
                ..Default::default()
            },
            background_color: interface_asset_bundle.color_secondary.into(),
            image: interface_asset_bundle.panel_single.clone().into(),
            ..Default::default()
        }).set_parent(layout.quadrants[2]).id();
        let text = commands.spawn(TextBundle::default()).set_parent(empty).id();
    
        Self { panel, icon, empty, text, mesh_handle, text_style: interface_asset_bundle.text_style_secondary.clone() }
    }
    pub fn update_as_bar(
        &mut self, commands: &mut Commands, meshes: &mut Assets<Mesh>,
        icon: &Handle<Image>, value: f32, metric: i32,
    ){
        commands.entity(self.panel).insert(Visibility::Inherited);
        commands.entity(self.empty).insert(Visibility::Hidden);

        commands.entity(self.icon).insert(UiImage::from(icon.clone()));

        if let Some(mesh) = meshes.get_mut(&self.mesh_handle) {
            assign_mesh_color(mesh, Color::rgb(metric as f32, value as f32, 0.0));
        }
    }
    pub fn update_as_value(
        &mut self, commands: &mut Commands,
        value: i32,
    ){
        commands.entity(self.panel).insert(Visibility::Hidden);
        commands.entity(self.empty).insert(Visibility::Inherited);

        commands.entity(self.text).insert(Text::from_section(
            format!("{}", value), self.text_style.clone())
            .with_alignment(TextAlignment::Center)
        );
    }
    pub fn update_as_empty(&mut self, commands: &mut Commands){
        commands.entity(self.panel).insert(Visibility::Hidden);
        commands.entity(self.empty).insert(Visibility::Hidden);
    }
}

pub struct IndicatorBarComponent {
    arrow_texture: Handle<Image>,
    text: Entity,
    panel: Entity,
    arrow: Entity,
}
impl IndicatorBarComponent {
    pub fn update_flow_direction(&self, commands: &mut Commands, direction: i32){
        if direction > 0 {
            commands.entity(self.arrow).insert(Visibility::Inherited).insert(UiImage{ texture: self.arrow_texture.clone(), flip_x: true, flip_y: false });
        } else if direction < 0 {
            commands.entity(self.arrow).insert(Visibility::Inherited).insert(UiImage::new(self.arrow_texture.clone()));
        } else {
            commands.entity(self.arrow).insert(Visibility::Hidden);
        }
    }
    pub fn new(
        commands: &mut Commands,
        layout: &OverlayLayout,
        interface_asset_bundle: &AssetBundle<InterfaceAssetBundle>,
        offset: usize,
    ) -> Self {
        let size: f32 = 18.0;
        let angle = std::f32::consts::PI - (1.0 - offset as f32 / 5.0).asin();
        let panel = commands.spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute, aspect_ratio: Some(1.0),
                size: Size::new(Val::Auto, Val::Percent(size)),
                position: layout.radial_placement(layout.inner_radius, angle, size, 2),
                padding: UiRect::right(Val::Percent(100.0)),
                justify_content: JustifyContent::FlexEnd,
                ..Default::default()
            }, ..Default::default()
        }).set_parent(layout.quadrants[2]).id();
        let bar = commands.spawn(ImageBundle {
            style: Style {
                align_items: AlignItems::Center, justify_content: JustifyContent::Center,
                aspect_ratio: Some(4.0), size: Size::height(Val::Percent(100.0)),
                flex_shrink: 0.0,
                ..Default::default()
            },
            background_color: interface_asset_bundle.color_enabled.into(),
            image: interface_asset_bundle.panel_extended.clone().into(),
            ..Default::default()
        }).insert((
            interface_asset_bundle.matter_material.clone(),
            bevy::sprite::Mesh2dHandle::from(interface_asset_bundle.quad_mesh.clone()),
        )).set_parent(panel).id();
        let text = commands.spawn(TextBundle {
            text: Text::from_section("", interface_asset_bundle.text_style_primary.clone()),
            ..Default::default()
        }).set_parent(bar).id();
        let arrow = commands.spawn(ImageBundle {
            style: Style {
                aspect_ratio: Some(1.0), size: Size::height(Val::Percent(100.0)), flex_shrink: 0.0,
                margin: UiRect::horizontal(Val::Percent(-16.0)),
                ..Default::default()
            },
            background_color: interface_asset_bundle.color_enabled.into(),
            image: interface_asset_bundle.arrow.clone().into(),
            ..Default::default()
        })
        .insert(Visibility::Hidden)
        .set_parent(panel).id();

        Self{ panel, arrow, text, arrow_texture: interface_asset_bundle.arrow.clone() }
    }
}