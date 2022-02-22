use std::cmp::Ordering;

use bevy::{prelude::*, render::{renderer::{RenderQueue, RenderDevice}, render_resource::{SpecializedPipelines, RenderPipelineCache, CachedPipelineId, PrimitiveTopology}}, asset::HandleId, reflect::Uuid, sprite::Mesh2dPipelineKey, core::FloatOrd, math::Vec3Swizzles};

use crate::render::{meta::{TimeMeta, ShapeMeta}, pipeline::{SmudPipeline, SmudPipelineKey}, stages::{UiShapeBatch, ShapeVertex}};

use super::extract::{ExtractedTime, ExtractedUiShapes};
pub fn prepare_time(
    time: Res<ExtractedTime>,
    time_meta: ResMut<TimeMeta>,
    render_queue: Res<RenderQueue>,
) {
    render_queue.write_buffer(
        &time_meta.buffer,
        0,
        bevy::core::cast_slice(&[time.seconds_since_startup]),
    );
}








pub fn prepare_ui_shapes(
    mut commands: Commands,
    mut pipelines: ResMut<SpecializedPipelines<SmudPipeline>>,
    mut pipeline_cache: ResMut<RenderPipelineCache>,
    mut extracted_shapes: ResMut<ExtractedUiShapes>,
    mut shape_meta: ResMut<ShapeMeta>, // TODO: make UI meta?
    render_device: Res<RenderDevice>,
    smud_pipeline: Res<SmudPipeline>,
    render_queue: Res<RenderQueue>,
) {
    shape_meta.ui_vertices.clear();

    let extracted_shapes = &mut extracted_shapes.0;
    // Sort shapes by z for correct transparency and then by handle to improve batching
    extracted_shapes.sort_unstable_by(|a, b| {
        match a
            .transform
            .translation
            .z
            .partial_cmp(&b.transform.translation.z)
        {
            Some(Ordering::Equal) | None => {
                (&a.sdf_shader, &a.fill_shader).cmp(&(&b.sdf_shader, &b.fill_shader))
            }
            Some(other) => other,
        }
    });

    let shape_meta = &mut shape_meta;

    let mut start = 0;
    let mut end = 0;
    let mut current_batch_shaders = (
        HandleId::Id(Uuid::nil(), u64::MAX),
        HandleId::Id(Uuid::nil(), u64::MAX),
    );
    let mut last_z = 0.;
    let mut current_batch_pipeline = CachedPipelineId::INVALID;

    // todo: how should msaa be handled for ui?
    // would perhaps be solved if I move this to queue?
    let mesh_key = Mesh2dPipelineKey::from_msaa_samples(1)
        | Mesh2dPipelineKey::from_primitive_topology(PrimitiveTopology::TriangleStrip);

    for extracted_shape in extracted_shapes.iter() {
        let shader_key = (
            extracted_shape.sdf_shader.id,
            extracted_shape.fill_shader.id,
        );
        let position = extracted_shape.transform.translation;
        let z = position.z;

        // We also split by z, so other ui systems can get their stuff in the middle
        if current_batch_shaders != shader_key || z != last_z {
            if start != end {
                commands.spawn_bundle((UiShapeBatch {
                    range: start..end,
                    shader_key: current_batch_shaders,
                    pipeline: current_batch_pipeline,
                    z: FloatOrd(last_z),
                },));
                start = end;
            }
            current_batch_shaders = shader_key;

            current_batch_pipeline = match smud_pipeline.shaders.0.get(&shader_key) {
                Some(_shader) => {
                    // todo pass the shader into specialize
                    let specialize_key = SmudPipelineKey {
                        mesh: mesh_key,
                        shader: shader_key,
                    };
                    pipelines.specialize(&mut pipeline_cache, &smud_pipeline, specialize_key)
                }
                None => CachedPipelineId::INVALID,
            }
        }

        if current_batch_pipeline == CachedPipelineId::INVALID {
            debug!("Shape not ready yet, skipping");
            continue; // skip shapes that are not ready yet
        }

        let color = extracted_shape.color.as_linear_rgba_f32();

        let position = position.into();
        // let position = Vec3::ZERO.into();

        let rotation = extracted_shape.transform.rotation * Vec3::X;
        let rotation = rotation.xy().into();

        let vertex = ShapeVertex {
            position,
            color,
            rotation,
            scale: extracted_shape.transform.scale.x,
            frame: extracted_shape.frame,
        };
        debug!("{vertex:?}");
        shape_meta.ui_vertices.push(vertex);
        last_z = z;
        end += 1;
    }

    // if start != end, there is one last batch to process
    if start != end {
        commands.spawn_bundle((UiShapeBatch {
            range: start..end,
            shader_key: current_batch_shaders,
            z: FloatOrd(last_z),
            pipeline: current_batch_pipeline,
        },));
    }

    shape_meta
        .ui_vertices
        .write_buffer(&render_device, &render_queue);
}