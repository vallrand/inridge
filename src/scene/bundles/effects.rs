use bevy::prelude::*;
use bevy_hanabi::prelude::*;
use crate::common::loader::{AssetBundleList, ScopedAssetServer};
use crate::common::geometry::cylinder::Cylinder;
use crate::materials::{
    UnlitMaterial, BillboardEffectMaterial, HairBall,
    ScanlineEffectMaterial, ExplosionMatterial, TentacleEffectMaterial,
    BarrierEffectMaterial, LightningEffectMaterial, Billboard, ProjectileTrailMaterial,
};
use crate::materials::displacement::DisplacementMaterial;
use super::texture::TextureSamplerOptions;

pub struct EffectAssetBundle {
    pub border: Handle<Image>,
    pub glow: Handle<Image>,
    pub wave: Handle<Image>,
    pub flash: Handle<Image>,
    pub ring: Handle<Image>,
    pub spark: Handle<Image>,
    pub material_fresnel: Handle<UnlitMaterial>,
    pub material_flash: Handle<BillboardEffectMaterial>,
    pub material_ring: Handle<BillboardEffectMaterial>,
    pub material_deficit: Handle<ScanlineEffectMaterial>,
    pub material_reload: Handle<ScanlineEffectMaterial>,
    pub material_explosion_fire: Handle<ExplosionMatterial>,
    pub material_explosion_toxic: Handle<ExplosionMatterial>,
    pub material_wave: Handle<DisplacementMaterial>,
    pub material_warp: Handle<DisplacementMaterial>,
    pub material_tentacle: Handle<TentacleEffectMaterial>,
    pub material_lightning: Handle<LightningEffectMaterial>,
    pub material_barrier: Handle<BarrierEffectMaterial>,
    pub material_vines: Handle<ProjectileTrailMaterial>,
    pub mesh_quad: Handle<Mesh>,
    pub mesh_sphere: Handle<Mesh>,
    pub mesh_tube: Handle<Mesh>,
    pub mesh_fuzz: Handle<Mesh>,
    pub particle_explosion: Handle<EffectAsset>,
    pub particle_hit: Handle<EffectAsset>,
}
impl AssetBundleList for EffectAssetBundle {
    fn from_asset_server(asset_server: &ScopedAssetServer) -> Self { Self {
        border: asset_server.load("textures/border_mask.png"),
        glow: asset_server.load("textures/effect_glow.png"),
        wave: asset_server.load("textures/effect_wave.png"),
        flash: asset_server.load("textures/effect_flash.png"),
        ring: asset_server.load("textures/effect_ring.png"),
        spark: asset_server.load("textures/effect_spark.png"),
        material_fresnel: Default::default(),
        material_flash: Default::default(),
        material_ring: Default::default(),
        material_deficit: Default::default(),
        material_reload: Default::default(),
        material_explosion_fire: Default::default(),
        material_explosion_toxic: Default::default(),
        material_wave: Default::default(),
        material_warp: Default::default(),
        material_tentacle: Default::default(),
        material_lightning: Default::default(),
        material_barrier: Default::default(),
        material_vines: Default::default(),
        mesh_quad: Default::default(),
        mesh_sphere: Default::default(),
        mesh_tube: Default::default(),
        mesh_fuzz: Default::default(),
        particle_explosion: Default::default(),
        particle_hit: Default::default(),
    } }
    fn prepare(&mut self, world: &mut World) {
        let mut images = world.resource_mut::<Assets<Image>>();
        TextureSamplerOptions::DEFAULT.apply(images.get_mut(&self.border).unwrap());
        TextureSamplerOptions::DEFAULT.apply(images.get_mut(&self.glow).unwrap());
        TextureSamplerOptions::DEFAULT.apply(images.get_mut(&self.wave).unwrap());
        TextureSamplerOptions::DEFAULT.apply(images.get_mut(&self.flash).unwrap());
        TextureSamplerOptions::DEFAULT.apply(images.get_mut(&self.ring).unwrap());

        let mut quad = Mesh::from(shape::Quad::new(Vec2::ONE));
        quad.generate_tangents().unwrap();
        self.mesh_quad = world.resource_mut::<Assets<Mesh>>().add(quad);

        self.mesh_sphere = world.resource_mut::<Assets<Mesh>>().add(shape::UVSphere {
            radius: 0.5, sectors: 36, stacks: 18,
        }.into());
        
        self.mesh_tube = world.resource_mut::<Assets<Mesh>>().add(Cylinder{
            cap_lower: false, cap_upper: false,
            ..Default::default()
        }.into());

        self.mesh_fuzz = world.resource_mut::<Assets<Mesh>>().add(HairBall {
            seed: 1, radius: 1.0, width: 0.2, quantity: 32, hemisphere: true
        }.into());

        self.material_fresnel = world.resource_mut::<Assets<UnlitMaterial>>().add(UnlitMaterial {
            color: Color::WHITE, fresnel_mask: 1.0, depth_mask: 2.0,
            diffuse: None, depth_bias: 0, double_sided: true
        });
        self.material_flash = world.resource_mut::<Assets<BillboardEffectMaterial>>().add(BillboardEffectMaterial {
            diffuse_texture: Some(self.flash.clone()),
            diffuse: Color::WHITE, uv_transform: Vec4::new(0.0, 0.0, 1.0, 1.0),
            alpha_threshold: 0.2, billboard: Some(Billboard::SphericalScreen),
            alpha_mode: AlphaMode::Add
        });
        self.material_ring = world.resource_mut::<Assets<BillboardEffectMaterial>>().add(BillboardEffectMaterial {
            diffuse_texture: Some(self.ring.clone()),
            diffuse: Color::WHITE, uv_transform: Vec4::new(0.0, 0.0, 1.0, 1.0),
            alpha_threshold: 0.2, billboard: None,
            alpha_mode: AlphaMode::Add
        });
        self.material_deficit = world.resource_mut::<Assets<ScanlineEffectMaterial>>().add(ScanlineEffectMaterial {
            color: Color::rgb(1.6,0.0,0.2),
            uv_transform: Vec4::new(0.0, 0.0, 8.0, 8.0),
            vertical_fade: Vec2::new(0.2, 0.8),
            line_width: Vec4::new(0.2, 0.8, 1.0, -0.5),
            alpha_mode: AlphaMode::Add, ..Default::default()
        });
        self.material_reload = world.resource_mut::<Assets<ScanlineEffectMaterial>>().add(ScanlineEffectMaterial {
            color: Color::rgb(1.8,2.4,0.8),
            uv_transform: Vec4::new(0.0, 0.0, 16.0, 16.0),
            vertical_fade: Vec2::new(0.2, 0.8),
            line_width: Vec4::new(0.5, 1.0, 3.0, 2.0),
            alpha_mode: AlphaMode::Premultiplied, ..Default::default()
        });
        self.material_explosion_fire = world.resource_mut::<Assets<ExplosionMatterial>>().add(ExplosionMatterial {
            color: Color::rgba(1.0,0.2,0.1,1.0),
            uv_transform: Vec4::new(0.0, 0.0, 1.0, 1.0)
        });
        self.material_explosion_toxic = world.resource_mut::<Assets<ExplosionMatterial>>().add(ExplosionMatterial {
            color: Color::rgba(0.2,1.0,0.1,1.0),
            uv_transform: Vec4::new(0.0, 0.0, 1.0, 1.0)
        });
        self.material_wave = world.resource_mut::<Assets<DisplacementMaterial>>().add(DisplacementMaterial {
            displacement: 0.5,
            chromatic_aberration: 0.02,
            displacement_map: Some(self.wave.clone()),
            fresnel: false, mask: true,
        });
        self.material_warp = world.resource_mut::<Assets<DisplacementMaterial>>().add(DisplacementMaterial {
            displacement: 0.1, chromatic_aberration: 0.0,
            displacement_map: None, fresnel: true, mask: true
        });
        self.material_tentacle = world.resource_mut::<Assets<TentacleEffectMaterial>>().add(TentacleEffectMaterial{
            color: Color::rgb(0.8,1.0,0.6), uv_transform: Vec4::new(0.0,0.0,1.0,1.0),
            reflectance: 0.5, metallic: 0.6, roughness: 0.2,
            ..Default::default()
        });
        self.material_lightning = world.resource_mut::<Assets<LightningEffectMaterial>>().add(LightningEffectMaterial {
            color: Color::rgb(1.0, 0.2, 0.4),
            uv_transform: Vec4::new(0.0, 0.0, 1.0, 1.0),
        });
        self.material_barrier = world.resource_mut::<Assets<BarrierEffectMaterial>>().add(BarrierEffectMaterial {
            color: Color::rgb(4.0, 0.8, 1.6),
            fresnel_mask: 1.0,
        });
        self.material_vines = world.resource_mut::<Assets<ProjectileTrailMaterial>>().add(ProjectileTrailMaterial {
            color: Color::rgb(1.0, 1.0, 1.0), head_color: Color::NONE, time_scale: 0.5,
            uv_transform: Vec4::new(0.0, 1.0, 1.0, -1.0), iterations: 1,
            billboard: true, blend_mode: AlphaMode::Premultiplied, vertical_fade: Vec2::new(0.1, 1.0),
        });

        let mut color_gradient_explosion = Gradient::new();
        color_gradient_explosion.add_key(0.0, Vec4::new(16.0, 8.0, 8.0, 1.0));
        color_gradient_explosion.add_key(0.1, Vec4::new(8.0, 2.0, 1.0, 1.0));
        color_gradient_explosion.add_key(0.8, Vec4::new(2.0, 0.0, 0.0, 1.0));
        color_gradient_explosion.add_key(1.0, Vec4::new(1.0, 0.0, 0.0, 0.0));

        let mut size_gradient_explosion = Gradient::new();
        size_gradient_explosion.add_key(0.0, Vec2::new(0.8, 0.1));
        size_gradient_explosion.add_key(0.2, Vec2::new(0.2, 0.1));
        size_gradient_explosion.add_key(1.0, Vec2::ZERO);

        self.particle_explosion = world.resource_mut::<Assets<EffectAsset>>().add(EffectAsset {
            capacity: 2048,
            spawner: Spawner::new(Value::Single(128.0), Value::Single(0.4), f32::INFINITY.into()),
            ..Default::default()
        }.init(InitPositionSphereModifier {
            center: Vec3::ZERO, radius: 0.1, dimension: ShapeDimension::Volume,
        }).init(InitVelocitySphereModifier {
            center: Vec3::ZERO, speed: Value::Uniform((1.0, 6.0))
        })
        .init(InitSizeModifier { size: Value::<f32>::Uniform((0.1, 1.0)).into() })
        .init(InitLifetimeModifier { lifetime: Value::Uniform((0.4, 1.6)) })
        .init(InitAgeModifier { age: Value::Uniform((0.0, 0.1)) })
        .update(AccelModifier::constant(Vec3::new(0.0, -1.0, 0.0)))
        .update(LinearDragModifier { drag: 2.0 })
        .render(ColorOverLifetimeModifier { gradient: color_gradient_explosion })
        .render(SizeOverLifetimeModifier { gradient: size_gradient_explosion })
        .render(ParticleTextureModifier { texture: self.spark.clone() })
        .render(BillboardModifier)
        .render(OrientAlongVelocityModifier));

        let mut color_gradient_hit = Gradient::new();
        color_gradient_hit.add_key(0.0, Vec4::new(12.0, 16.0, 8.0, 1.0));
        color_gradient_hit.add_key(0.9, Vec4::new(2.0, 4.0, 0.5, 1.0));
        color_gradient_hit.add_key(0.6, Vec4::new(0.8,1.0,0.2,1.0));
        color_gradient_hit.add_key(1.0, Vec4::ZERO);

        let mut size_gradient_hit = Gradient::new();
        size_gradient_hit.add_key(0.0, Vec2::new(0.4, 0.2));
        size_gradient_hit.add_key(0.3, Vec2::new(0.1, 0.1));
        size_gradient_hit.add_key(1.0, Vec2::ZERO);

        self.particle_hit = world.resource_mut::<Assets<EffectAsset>>().add(EffectAsset {
            capacity: 1024,
            spawner: Spawner::new(Value::Single(32.0), Value::Single(0.2), f32::INFINITY.into()),
            ..Default::default()
        }
        .init(InitPositionSphereModifier { center: Vec3::ZERO, radius: 0.1, dimension: ShapeDimension::Surface })
        .init(InitVelocitySphereModifier { center: Vec3::ZERO, speed: Value::Uniform((1.0, 10.0)) })
        .init(InitSizeModifier { size: Value::<f32>::Uniform((0.1, 0.4)).into() })
        .init(InitLifetimeModifier { lifetime: Value::Uniform((0.4, 1.0)) })
        .update(AccelModifier::constant(Vec3::new(0.0, -1.6, 0.0)))
        .update(LinearDragModifier { drag: 4.0 })
        .render(ColorOverLifetimeModifier { gradient: color_gradient_hit })
        .render(SizeOverLifetimeModifier { gradient: size_gradient_hit })
        .render(ParticleTextureModifier { texture: self.spark.clone() })
        .render(BillboardModifier)
        .render(OrientAlongVelocityModifier));

    }
}