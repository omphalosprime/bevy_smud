use bevy::{
    core_pipeline::Transparent2d,
    prelude::{App, Plugin, Shader, Assets},
    render::{
        render_resource::{BufferDescriptor, BufferUsages, SpecializedPipelines},
        renderer::RenderDevice,
        RenderApp, RenderStage, render_phase::AddRenderCommand,
    },
    ui::TransparentUi,
};

use crate::render::{
    meta::{ShapeMeta, TimeMeta},
    pipeline::SmudPipeline,
    render_command::{DrawSmudShape, DrawSmudUiShape},
    stages::{
        extract::{extract_sdf_shaders, extract_shapes, extract_time, ExtractedShapes, extract_ui_shapes, ExtractedUiShapes},
        prepare::{prepare_time, prepare_ui_shapes},
        queue::{queue_shapes, queue_time_bind_group, queue_ui_shapes},
    },
};

use crate::assets::shader_loading::*;

#[derive(Default)]
pub struct SmudPlugin;

impl Plugin for SmudPlugin {
    fn build(&self, app: &mut App) {
        // All the messy boiler-plate for loading a bunch of shaders
        app.add_plugin(ShaderLoadingPlugin);
        app.add_plugin(TexturedSmudPlugin);
        app.add_plugin(UiShapePlugin);
        let render_device = app.world.get_resource::<RenderDevice>().unwrap();
        let buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("time uniform buffer"),
            size: std::mem::size_of::<f32>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .add_render_command::<Transparent2d, DrawSmudShape>()
                .insert_resource(TimeMeta {
                    buffer,
                    bind_group: None,
                })
                .init_resource::<ExtractedShapes>()
                .init_resource::<ShapeMeta>()
                .init_resource::<SmudPipeline>()
                .init_resource::<SpecializedPipelines<SmudPipeline>>()
                .add_system_to_stage(RenderStage::Extract, extract_time)
                .add_system_to_stage(RenderStage::Extract, extract_shapes)
                .add_system_to_stage(RenderStage::Extract, extract_sdf_shaders)
                .add_system_to_stage(RenderStage::Prepare, prepare_time)
                .add_system_to_stage(RenderStage::Queue, queue_shapes)
                .add_system_to_stage(RenderStage::Queue, queue_time_bind_group);
        }
    }
}

#[derive(Default)]
pub struct UiShapePlugin;

impl Plugin for UiShapePlugin {
    fn build(&self, app: &mut App) {
        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                // re-using command from regular pass... ok?
                .add_render_command::<TransparentUi, DrawSmudUiShape>()
                .init_resource::<ExtractedUiShapes>()
                .add_system_to_stage(RenderStage::Extract, extract_ui_shapes)
                .add_system_to_stage(RenderStage::Prepare, prepare_ui_shapes)
                .add_system_to_stage(RenderStage::Queue, queue_ui_shapes);
        }
    }
}

#[derive(Default)]
pub struct TexturedSmudPlugin;
impl Plugin for TexturedSmudPlugin {
    fn build(&self, app: &mut App) {
        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            // render_app
                // re-using command from regular pass... ok?
                // .add_render_command::<TransparentUi, DrawSmudUiShape>()
                // .init_resource::<ExtractedUiShapes>()
                // .add_system_to_stage(RenderStage::Extract, extract_ui_shapes)
                // .add_system_to_stage(RenderStage::Prepare, prepare_ui_shapes)
                // .add_system_to_stage(RenderStage::Queue, queue_ui_shapes);
        }
    }
}



pub struct ShaderLoadingPlugin;

impl Plugin for ShaderLoadingPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "smud_shader_hot_reloading")]
        {
            let mut hot_shaders = {
                let asset_server = app.world.get_resource::<AssetServer>().unwrap();
                HotShaders::<Self> {
                    shaders: [
                        ("prelude.wgsl", PRELUDE_SHADER_IMPORT, PRELUDE_SHADER_HANDLE),
                        ("shapes.wgsl", SHAPES_SHADER_IMPORT, SHAPES_SHADER_HANDLE),
                        (
                            "colorize.wgsl",
                            COLORIZE_SHADER_IMPORT,
                            COLORIZE_SHADER_HANDLE,
                        ),
                        ("smud.wgsl", SMUD_SHADER_IMPORT, SMUD_SHADER_HANDLE),
                        ("vertex.wgsl", VERTEX_SHADER_IMPORT, VERTEX_SHADER_HANDLE),
                        (
                            "fragment.wgsl",
                            FRAGMENT_SHADER_IMPORT,
                            FRAGMENT_SHADER_HANDLE,
                        ),
                        // Hot-loading is borked-ish for these for some reason, so always load normally
                        // (
                        //     "fills/cubic_falloff.wgsl",
                        //     DEFAULT_FILL_IMPORT,
                        //     DEFAULT_FILL_HANDLE,
                        // ),
                        // ("fills/simple.wgsl", SIMPLE_FILL_IMPORT, SIMPLE_FILL_HANDLE),
                    ]
                    .into_iter()
                    .map(|(path, import_path, untyped_handle)| HotShader {
                        strong_handle: asset_server.load(path),
                        untyped_handle: Some(untyped_handle),
                        import_path: import_path.into(),
                        loaded: false,
                    })
                    .collect(),
                    ..Default::default()
                }
            };
            let mut shader_assets = app.world.get_resource_mut::<Assets<Shader>>().unwrap();

            for hot_shader in hot_shaders.shaders.iter_mut() {
                let untyped_handle = hot_shader.untyped_handle.take().unwrap();
                shader_assets.add_alias(hot_shader.strong_handle.clone(), untyped_handle);
            }

            app.insert_resource(hot_shaders);
            app.add_system(setup_shader_imports::<Self>);
        }

        #[cfg(not(feature = "smud_shader_hot_reloading"))]
        {
            let mut shaders = app.world.get_resource_mut::<Assets<Shader>>().unwrap();

            let prelude = Shader::from_wgsl(include_str!("../assets/prelude.wgsl"))
                .with_import_path(PRELUDE_SHADER_IMPORT);
            shaders.set_untracked(PRELUDE_SHADER_HANDLE, prelude);

            let shapes = Shader::from_wgsl(include_str!("../assets/shapes.wgsl"))
                .with_import_path(SHAPES_SHADER_IMPORT);
            shaders.set_untracked(SHAPES_SHADER_HANDLE, shapes);

            let colorize = Shader::from_wgsl(include_str!("../assets/colorize.wgsl"))
                .with_import_path(COLORIZE_SHADER_IMPORT);
            shaders.set_untracked(COLORIZE_SHADER_HANDLE, colorize);

            let smud = Shader::from_wgsl(include_str!("../assets/smud.wgsl"))
                .with_import_path(SMUD_SHADER_IMPORT);
            shaders.set_untracked(SMUD_SHADER_HANDLE, smud);

            let vertex = Shader::from_wgsl(include_str!("../assets/vertex.wgsl"))
                .with_import_path(VERTEX_SHADER_IMPORT);
            shaders.set_untracked(VERTEX_SHADER_HANDLE, vertex);

            let fragment = Shader::from_wgsl(include_str!("../assets/fragment.wgsl"))
                .with_import_path(FRAGMENT_SHADER_IMPORT);
            shaders.set_untracked(FRAGMENT_SHADER_HANDLE, fragment);
        }

        // Hot-loading is borked-ish for these for some reason, so always load normally
        {
            let mut shaders = app.world.get_resource_mut::<Assets<Shader>>().unwrap();
            let fill = Shader::from_wgsl(include_str!("../assets/fills/cubic_falloff.wgsl"))
                .with_import_path(DEFAULT_FILL_IMPORT);
            shaders.set_untracked(DEFAULT_FILL_HANDLE, fill);

            let simple_fill = Shader::from_wgsl(include_str!("../assets/fills/simple.wgsl"))
                .with_import_path(SIMPLE_FILL_IMPORT);
            shaders.set_untracked(SIMPLE_FILL_HANDLE, simple_fill);
        }
    }
}

