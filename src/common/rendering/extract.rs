use bevy::app::{App, IntoSystemAppConfig, Plugin};
use bevy::asset::{AssetEvent, Assets, Handle};
use bevy::prelude::{Deref, DerefMut};
use bevy::ecs::prelude::*;
use bevy::ecs::system::SystemParamItem;
use bevy::render::render_asset::PrepareAssetSet;
use bevy::render::{Extract, ExtractSchedule, RenderApp, RenderSet};
use std::marker::PhantomData;
use bevy::asset::Asset;
use bevy::ecs::system::{StaticSystemParam, SystemParam};
use bevy::utils::{HashMap, HashSet};

pub enum PrepareAssetError<E: Send + Sync + 'static> { RetryNextUpdate(E) }

pub trait RenderAsset: Send + Sync + Sized + 'static {
    type SourceAsset: Asset;
    type ExtractedAsset: Send + Sync + 'static;
    type Param: SystemParam;
    fn extract_asset(source_asset: &Self::SourceAsset) -> Self::ExtractedAsset;
    fn prepare_asset(
        extracted_asset: Self::ExtractedAsset,
        param: &mut SystemParamItem<Self::Param>,
    ) -> Result<Self, PrepareAssetError<Self::ExtractedAsset>>;
}

pub struct ExtractAssetPlugin<A: RenderAsset>(PhantomData<fn() -> A>);
impl<A: RenderAsset> Default for ExtractAssetPlugin<A> { fn default() -> Self { Self(PhantomData) } }
impl<A: RenderAsset> Plugin for ExtractAssetPlugin<A> {
    fn build(&self, app: &mut App) {
        app.sub_app_mut(RenderApp)
        .configure_sets(
            (PrepareAssetSet::PreAssetPrepare, PrepareAssetSet::AssetPrepare, PrepareAssetSet::PostAssetPrepare)
                .chain().in_set(RenderSet::Prepare),
        )
        .init_resource::<ExtractedAssets<A>>()
        .init_resource::<PreparedRenderAssets<A>>()
        .add_system(extract_render_asset::<A>.in_schedule(ExtractSchedule))
        .add_system(prepare_assets::<A>.in_set(PrepareAssetSet::AssetPrepare));
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct PreparedRenderAssets<A: RenderAsset>(HashMap<Handle<A::SourceAsset>, A>);
impl<A: RenderAsset> Default for PreparedRenderAssets<A> { fn default() -> Self { Self(Default::default()) } }

#[derive(Resource)]
pub struct ExtractedAssets<A: RenderAsset> {
    extracted: Vec<(Handle<A::SourceAsset>, A::ExtractedAsset)>,
    removed: Vec<Handle<A::SourceAsset>>,
}
impl<A: RenderAsset> Default for ExtractedAssets<A> {
    fn default() -> Self { Self { extracted: Default::default(), removed: Default::default() } }
}

fn extract_render_asset<A: RenderAsset>(
    mut commands: Commands,
    mut events: Extract<EventReader<AssetEvent<A::SourceAsset>>>,
    assets: Extract<Res<Assets<A::SourceAsset>>>,
) {
    let mut changed_assets = HashSet::default();
    let mut removed = Vec::new();
    for event in events.iter() {
        match event {
            AssetEvent::Created { handle } | AssetEvent::Modified { handle } => {
                changed_assets.insert(handle.clone_weak());
            }
            AssetEvent::Removed { handle } => {
                changed_assets.remove(handle);
                removed.push(handle.clone_weak());
            }
        }
    }
    let mut extracted_assets = Vec::new();
    for handle in changed_assets.drain() {
        if let Some(asset) = assets.get(&handle) {
            extracted_assets.push((handle, A::extract_asset(&asset)));
        }
    }
    commands.insert_resource(ExtractedAssets::<A> { extracted: extracted_assets, removed });
}

pub fn prepare_assets<A: RenderAsset>(
    mut extracted_assets: ResMut<ExtractedAssets<A>>,
    mut render_assets: ResMut<PreparedRenderAssets<A>>,
    mut prepare_next_frame: Local<Vec<(Handle<A::SourceAsset>, A::ExtractedAsset)>>,
    param: StaticSystemParam<<A as RenderAsset>::Param>,
){
    let mut param = param.into_inner();
    let queued_assets = std::mem::take(std::ops::DerefMut::deref_mut(&mut prepare_next_frame));
    for (handle, extracted_asset) in queued_assets {
        match A::prepare_asset(extracted_asset, &mut param) {
            Ok(prepared_asset) => {
                render_assets.insert(handle, prepared_asset);
            },
            Err(PrepareAssetError::RetryNextUpdate(extracted_asset)) => {
                prepare_next_frame.push((handle, extracted_asset));
            }
        }
    }
    for removed in std::mem::take(&mut extracted_assets.removed) {
        render_assets.remove(&removed);
    }
    for (handle, extracted_asset) in std::mem::take(&mut extracted_assets.extracted) {
        match A::prepare_asset(extracted_asset, &mut param) {
            Ok(prepared_asset) => {
                render_assets.insert(handle, prepared_asset);
            },
            Err(PrepareAssetError::RetryNextUpdate(extracted_asset)) => {
                prepare_next_frame.push((handle, extracted_asset));
            }
        }
    }
}
