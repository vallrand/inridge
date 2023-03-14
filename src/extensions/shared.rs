use bevy::prelude::*;
use bevy::render::render_resource::ShaderRef;

///TODO: Should be a utility of ShaderRef
pub fn load_shader(asset_server: &AssetServer, shader: ShaderRef) -> Option<Handle<Shader>> {
    match shader {
        ShaderRef::Default => None,
        ShaderRef::Handle(handle) => Some(handle),
        ShaderRef::Path(path) => Some(asset_server.load(path)),
    }
}