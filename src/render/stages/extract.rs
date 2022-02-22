use bevy::{
    asset::HandleId,
    prelude::*,
    reflect::Uuid,
    render::{render_resource::ShaderImport, RenderWorld},
    utils::HashMap,
};
use copyless::VecHelper;

use crate::{
    prelude::{Frame, SmudShape},
    render::pipeline::SmudPipeline,
};

// extract the passed time into a resource in the render world
pub fn extract_time(mut commands: Commands, time: Res<Time>) {
    commands.insert_resource(ExtractedTime {
        seconds_since_startup: time.seconds_since_startup() as f32,
    });
}

#[derive(Default)]
pub struct ExtractedTime {
    pub seconds_since_startup: f32,
}

#[derive(Component, Clone, Debug)]
pub struct ExtractedShape {
    pub color: Color,
    pub frame: f32,
    pub sdf_shader: Handle<Shader>,  // todo could be HandleId?
    pub fill_shader: Handle<Shader>, // todo could be HandleId?
    pub transform: GlobalTransform,
}

#[derive(Default)]
pub struct ShapeShaders(pub HashMap<(HandleId, HandleId), Handle<Shader>>);

pub fn extract_sdf_shaders(
    mut render_world: ResMut<RenderWorld>,
    shapes: Query<&SmudShape>, //, Changed<SmudShape>>, // does changed help? need to make sure it is not racy then!
    mut shaders: ResMut<Assets<Shader>>,
) {
    let mut pipeline = render_world.get_resource_mut::<SmudPipeline>().unwrap();

    for shape in shapes.iter() {
        let shader_key = (shape.sdf.id, shape.fill.id);
        if pipeline.shaders.0.contains_key(&shader_key) {
            continue;
        }

        // todo use asset events instead?
        let sdf_import_path = match shaders.get_mut(&shape.sdf.clone()) {
            Some(shader) => match shader.import_path() {
                Some(ShaderImport::Custom(p)) => p.to_owned(),
                _ => {
                    let id = Uuid::new_v4();
                    let path = format!("bevy_smud::generated::{id}");
                    shader.set_import_path(&path);
                    path
                }
            },
            None => {
                debug!("Waiting for sdf to load");
                continue;
            }
        };

        let fill_import_path = match shaders.get_mut(&shape.fill.clone()) {
            Some(shader) => match shader.import_path() {
                Some(ShaderImport::Custom(p)) => p.to_owned(),
                _ => {
                    let id = Uuid::new_v4();
                    let path = format!("bevy_smud::generated::{id}");
                    shader.set_import_path(&path);
                    path
                }
            },
            None => {
                debug!("Waiting for fill to load");
                continue;
            }
        };

        info!("Generating shader");
        let generated_shader = Shader::from_wgsl(format!(
            r#"
#import bevy_smud::vertex
#import {sdf_import_path}
#import {fill_import_path}
#import bevy_smud::fragment
"#
        ));

        // todo does this work, or is it too late?
        let generated_shader_handle = shaders.add(generated_shader);

        pipeline
            .shaders
            .0
            .insert(shader_key, generated_shader_handle);
    }
}

#[derive(Default, Debug)]
pub struct ExtractedShapes(pub Vec<ExtractedShape>);

pub fn extract_shapes(
    mut render_world: ResMut<RenderWorld>,
    query: Query<(&SmudShape, &ComputedVisibility, &GlobalTransform)>,
) {
    let mut extracted_shapes = render_world.get_resource_mut::<ExtractedShapes>().unwrap();
    extracted_shapes.0.clear();

    for (shape, computed_visibility, transform) in query.iter() {
        if !computed_visibility.is_visible {
            continue;
        }

        let frame = match shape.frame {
            Frame::Quad(s) => s,
        };

        extracted_shapes.0.alloc().init(ExtractedShape {
            color: shape.color,
            transform: *transform,
            sdf_shader: shape.sdf.clone_weak(),
            fill_shader: shape.fill.clone_weak(),
            frame
            // rect: None,
            // // Pass the custom size
            // custom_size: shape.custom_size,
        });
    }
}

#[derive(Default, Debug)]
pub struct ExtractedUiShapes(pub Vec<ExtractedShape>);

pub fn extract_ui_shapes(
    mut render_world: ResMut<RenderWorld>,
    query: Query<(&Node, &GlobalTransform, &SmudShape, &Visibility, &UiColor)>,
) {
    let mut extracted_shapes = render_world
        .get_resource_mut::<ExtractedUiShapes>()
        .unwrap();
    extracted_shapes.0.clear();

    for (node, transform, shape, visibility, color) in query.iter() {
        if !visibility.is_visible {
            continue;
        }

        let size = node.size.x; // TODO: Also pass on the height value
        let frame = size / 2.;

        extracted_shapes.0.alloc().init(ExtractedShape {
            color: shape.color * Vec4::from(color.0),
            transform: *transform,
            sdf_shader: shape.sdf.clone_weak(),
            fill_shader: shape.fill.clone_weak(),
            frame,
        });
    }
}
