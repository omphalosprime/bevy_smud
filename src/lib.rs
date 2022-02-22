mod assets;
mod ecs;
mod plugin;
mod render;

pub mod prelude {
    pub use crate::{
        assets::sdf_assets::SdfAssets,
        assets::shader_loading::{DEFAULT_FILL_HANDLE, SIMPLE_FILL_HANDLE},
        ecs::bundle::{ShapeBundle, UiShapeBundle},
        ecs::components::{Frame, SmudShape},
        plugin::SmudPlugin,
    };
}

// TODO: is RenderAsset asking too much?
// pub trait SdfShapeShader: 'static + Send + Sync {
//     /// Shader must include a handle to a shader with a wgsl function with signature fn distance(pos: vec2<f32>) -> f32
//     fn shader(asset_server: &AssetServer) -> Handle<Shader>;
// }

// /// Adds the necessary ECS resources and render logic to enable rendering entities using the given [`SdfShapeShader`]
// /// asset type
// pub struct SdfShapePlugin<S: SdfShapeShader>(PhantomData<S>);

// impl<S: SdfShapeShader> Default for SdfShapePlugin<S> {
//     fn default() -> Self {
//         Self(Default::default())
//     }
// }

// impl<S: SdfShapeShader> Plugin for SdfShapePlugin<S> {
//     fn build(&self, app: &mut App) {
//         // TODO:
//         // app.add_asset::<S>()
//         //     .add_plugin(ExtractComponentPlugin::<Handle<S>>::default())
//         //     .add_plugin(RenderAssetPlugin::<S>::default());
//         // if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
//         //     render_app
//         //         .add_render_command::<Transparent3d, DrawMaterial<S>>()
//         //         .add_render_command::<Opaque3d, DrawMaterial<S>>()
//         //         .add_render_command::<AlphaMask3d, DrawMaterial<S>>()
//         //         .init_resource::<MaterialPipeline<S>>()
//         //         .init_resource::<SpecializedPipelines<MaterialPipeline<S>>>()
//         //         .add_system_to_stage(RenderStage::Queue, queue_material_meshes::<S>);
//         // }
//     }
// }
