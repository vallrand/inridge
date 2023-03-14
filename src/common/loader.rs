use bevy::prelude::*;
use bevy::ecs::system::SystemState;
use bevy::ecs::schedule::ScheduleLabel;
use bevy::asset::{AddAsset, Asset, AssetLoader, AssetPath, BoxedFuture, LoadContext, LoadedAsset};
use std::marker::PhantomData;
use std::collections::HashSet;
use std::any::TypeId;
use std::path::Path;

#[derive(States, Clone, PartialEq, Eq, Hash, Default, Debug)]
pub enum LoadingState {
    #[default] Loading,
    Running
}
pub trait AssetBundleList: Send + Sync + 'static {
    fn from_asset_server(asset_server: &ScopedAssetServer) -> Self;
    fn prepare(&mut self, world: &mut World){}
}

#[derive(Resource)]
pub struct AssetBundle<T: AssetBundleList> {
    delegate: T,
    handles: Vec<HandleUntyped>,
}
impl<T: AssetBundleList> std::ops::Deref for AssetBundle<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target { &self.delegate }
}
impl<T: AssetBundleList> std::ops::DerefMut for AssetBundle<T> {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.delegate }
}

#[derive(ScheduleLabel, Debug, Hash, PartialEq, Eq, Clone)]
struct AssetScheduleLabel(TypeId);

pub struct ScopedAssetServer<'a> {
    handles: std::cell::RefCell<Vec<HandleUntyped>>,
    asset_server: &'a AssetServer
}
impl<'a0> ScopedAssetServer<'a0> {
    pub fn load<'a, T: Asset, P: Into<AssetPath<'a>>>(&self, path: P) -> Handle<T> {
        let handle = self.asset_server.load(path);
        self.handles.borrow_mut().push(handle.clone_untyped());
        handle
    }
    pub fn load_folder_untyped<P: AsRef<Path>>(&self, path: P) -> Vec<HandleUntyped> {
        let handles = self.asset_server.load_folder(path).unwrap();
        self.handles.borrow_mut().extend(handles.iter().map(|handle|handle.clone()));
        handles
    }
    pub fn load_folder<T: Asset, P: AsRef<Path>>(&self, path: P) -> Vec<Handle<T>> {
        self.load_folder_untyped(path).into_iter().map(|handle|handle.typed::<T>()).collect()
    }
}

impl<T: AssetBundleList> FromWorld for AssetBundle<T> {
    fn from_world(world: &mut World) -> Self {
        let asset_server = ScopedAssetServer {
            asset_server: world.resource::<AssetServer>(),
            handles: std::cell::RefCell::new(Vec::new()),
        };
        let delegate: T = T::from_asset_server(&asset_server);
        let handles: Vec<HandleUntyped> = asset_server.handles.take();
        let resource_type = TypeId::of::<T>();

        let mut schedule = Schedule::new();
        schedule.add_system(load_bundle_system::<T>);
        world.resource_mut::<InternalScheduler>().0.push((resource_type, schedule));
        
        // world.add_schedule(schedule, AssetScheduleLabel(resource_type));

        world.resource_mut::<LoadingProgress>().set.insert(resource_type);
        Self{ delegate, handles }
    }
}

fn load_bundle_system<T: AssetBundleList>(
    world: &mut World,
    mut system_state: Local<SystemState<(ResMut<AssetBundle<T>>, ResMut<LoadingProgress>, Res<AssetServer>)>>
){
    use bevy::asset::LoadState;
    let (mut bundle, mut loading, asset_server) = system_state.get_mut(world);
    if bundle.handles.is_empty() {
        return;
    }
    let state = asset_server.get_group_load_state(bundle.handles.iter().map(|h|h.id()));
    match state {
        LoadState::Failed | LoadState::Loaded => {
            bundle.handles.clear();
            let resource_type = TypeId::of::<T>();
            loading.set.remove(&resource_type);
        },
        _ => {}
    }
    if state == LoadState::Loaded {
        world.resource_scope(|world, mut bundle: Mut<AssetBundle<T>>|{
            bundle.delegate.prepare(world);
        });
    }
}


#[derive(Resource, Default)]
struct LoadingProgress {
    set: HashSet<TypeId>
}

#[derive(Resource, Default)]
struct InternalScheduler(Vec<(TypeId, Schedule)>);

fn loading_system(world: &mut World){
    world.resource_scope(|world, mut scheduler: Mut<InternalScheduler>|{
        let keys = &world.resource::<LoadingProgress>().set;
        scheduler.0.retain(|(key, _)|keys.contains(key));
        for (_, stage) in scheduler.0.iter_mut() {
            stage.run(world);
        }
    });
}

fn loading_tracking_system(
    state: Res<State<LoadingState>>,
    mut next_state: ResMut<NextState<LoadingState>>,
    progress: Res<LoadingProgress>,
    schedules: Res<InternalScheduler>
){
    let is_empty = schedules.0.is_empty();
    let is_loading = state.0.eq(&LoadingState::Loading);
    if is_loading && is_empty {
        next_state.set(LoadingState::Running);
    } else if !is_loading && !is_empty {
        next_state.set(LoadingState::Loading);
    }
}

///https://github.com/NiklasEi/bevy_common_assets
pub struct RonAssetPlugin<T> { extensions: Vec<&'static str>, marker: PhantomData<T> }
impl<T> RonAssetPlugin<T>
where for<'a> T: serde::Deserialize<'a> + Asset {
    pub fn new(extension: &'static str) -> Self { Self { extensions: vec![extension], marker: PhantomData } }
}
impl<T> Plugin for RonAssetPlugin<T>
where for<'a> T: serde::Deserialize<'a> + Asset {
    fn build(&self, app: &mut App) {
        app.add_asset::<T>().add_asset_loader(RonAssetLoader::<T> {
            extensions: self.extensions.clone(), marker: PhantomData,
        });
    }
}

struct RonAssetLoader<T> { extensions: Vec<&'static str>, marker: PhantomData<T> }
impl<T> AssetLoader for RonAssetLoader<T>
where for<'a> T: serde::Deserialize<'a> + Asset {
    fn extensions(&self) -> &[&str] { &self.extensions }
    fn load<'a>(
        &'a self, bytes: &'a [u8], load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            use serde_ron::de::from_bytes;
            let asset: T = from_bytes::<T>(bytes)?;
            load_context.set_default_asset(LoadedAsset::new(asset));
            Ok(())
        })
    }
}

pub struct LoaderPlugin; impl Plugin for LoaderPlugin {
    fn build(&self, app: &mut App){
        app.add_state::<LoadingState>()
        .init_resource::<LoadingProgress>()
        .init_resource::<InternalScheduler>()
        .add_system(loading_system)
        .add_system(loading_tracking_system.after(loading_system));
    }
}
