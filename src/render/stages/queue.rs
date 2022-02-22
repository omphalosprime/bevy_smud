use std::cmp::Ordering;

use bevy::{
    asset::HandleId,
    core::FloatOrd,
    prelude::*,
    reflect::Uuid,
    render::{
        render_resource::{
            BindGroupDescriptor, BindGroupEntry, CachedPipelineId, PrimitiveTopology,
            RenderPipelineCache, SpecializedPipelines,
        },
        renderer::{RenderDevice, RenderQueue}, view::{VisibleEntities, ViewUniforms}, render_phase::{RenderPhase, DrawFunctions},
    },
    sprite::Mesh2dPipelineKey, core_pipeline::Transparent2d, ui::TransparentUi, math::Vec3Swizzles,
};

use crate::render::{
    meta::{ShapeMeta, TimeMeta},
    pipeline::{SmudPipeline, SmudPipelineKey},
    stages::{ShapeVertex, UiShapeBatch}, render_command::{DrawSmudShape, DrawSmudUiShape},
};

use super::{extract::ExtractedShapes, ShapeBatch};

pub fn queue_time_bind_group(
    render_device: Res<RenderDevice>,
    mut time_meta: ResMut<TimeMeta>,
    pipeline: Res<SmudPipeline>,
) {
    let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: &pipeline.time_bind_group_layout,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: time_meta.buffer.as_entire_binding(),
        }],
    });
    time_meta.bind_group = Some(bind_group);
}

pub fn queue_shapes(
    mut commands: Commands,
    mut views: Query<(&mut RenderPhase<Transparent2d>, &VisibleEntities)>,
    mut pipelines: ResMut<SpecializedPipelines<SmudPipeline>>,
    mut pipeline_cache: ResMut<RenderPipelineCache>,
    mut extracted_shapes: ResMut<ExtractedShapes>, // todo needs mut?
    mut shape_meta: ResMut<ShapeMeta>,
    transparent_draw_functions: Res<DrawFunctions<Transparent2d>>,
    render_device: Res<RenderDevice>,
    smud_pipeline: Res<SmudPipeline>,
    msaa: Res<Msaa>,
    view_uniforms: Res<ViewUniforms>,
    render_queue: Res<RenderQueue>,
) {
    // Clear the vertex buffer
    shape_meta.vertices.clear();

    let view_binding = match view_uniforms.uniforms.binding() {
        Some(binding) => binding,
        None => return,
    };

    shape_meta.view_bind_group = Some(render_device.create_bind_group(&BindGroupDescriptor {
        entries: &[BindGroupEntry {
            binding: 0,
            resource: view_binding,
        }],
        label: Some("smud_shape_view_bind_group"),
        layout: &smud_pipeline.view_layout,
    }));

    // Vertex buffer index
    let mut index = 0;

    let draw_smud_shape = transparent_draw_functions
        .read()
        .get_id::<DrawSmudShape>()
        .unwrap();

    let shape_meta = &mut shape_meta;

    // Iterate over each view (a camera is a view)
    for (mut transparent_phase, _visible_entities) in views.iter_mut() {
        // todo: check visible entities?

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

        let mesh_key = Mesh2dPipelineKey::from_msaa_samples(msaa.samples)
            | Mesh2dPipelineKey::from_primitive_topology(PrimitiveTopology::TriangleStrip);

        // Impossible starting values that will be replaced on the first iteration
        let mut current_batch = ShapeBatch {
            shader: (
                HandleId::Id(Uuid::nil(), u64::MAX),
                HandleId::Id(Uuid::nil(), u64::MAX),
            ),
        };
        let mut current_batch_entity = Entity::from_raw(u32::MAX);
        let mut current_batch_pipeline = CachedPipelineId::INVALID;

        // Add a phase item for each shape, and detect when successive items can be batched.
        // Spawn an entity with a `ShapeBatch` component for each possible batch.
        // Compatible items share the same entity.
        // Batches are merged later (in `batch_phase_system()`), so that they can be interrupted
        // by any other phase item (and they can interrupt other items from batching).
        for extracted_shape in extracted_shapes.iter() {
            let new_batch = ShapeBatch {
                shader: (
                    extracted_shape.sdf_shader.id,
                    extracted_shape.fill_shader.id,
                ),
            };

            if new_batch != current_batch {
                current_batch_entity = commands.spawn_bundle((current_batch,)).id();

                current_batch = new_batch;

                if let Some(_shader) = smud_pipeline.shaders.0.get(&current_batch.shader) {
                    // todo pass the shader into specialize
                    let specialize_key = SmudPipelineKey {
                        mesh: mesh_key,
                        shader: current_batch.shader,
                    };
                    current_batch_pipeline =
                        pipelines.specialize(&mut pipeline_cache, &smud_pipeline, specialize_key);
                }
            }

            if current_batch_pipeline == CachedPipelineId::INVALID {
                debug!("Shape not ready yet, skipping");
                continue; // skip shapes that are not ready yet
            }

            // let mesh_z = mesh2d_uniform.transform.w_axis.z;

            // let color = extracted_shape.color.as_linear_rgba_f32();
            // // encode color as a single u32 to save space
            // let color = (color[0] * 255.0) as u32
            //     | ((color[1] * 255.0) as u32) << 8
            //     | ((color[2] * 255.0) as u32) << 16
            //     | ((color[3] * 255.0) as u32) << 24;

            let color = extracted_shape.color.as_linear_rgba_f32();

            let position = extracted_shape.transform.translation;
            let z = position.z;
            let position = position.into();

            let rotation = extracted_shape.transform.rotation * Vec3::X;
            let rotation = rotation.xy().into();

            let vertex = ShapeVertex {
                position,
                color,
                rotation,
                scale: extracted_shape.transform.scale.x,
                frame: extracted_shape.frame,
            };
            shape_meta.vertices.push(vertex);

            let item_start = index;
            index += 1;
            let item_end = index;

            transparent_phase.add(Transparent2d {
                entity: current_batch_entity,
                draw_function: draw_smud_shape,
                pipeline: current_batch_pipeline,
                sort_key: FloatOrd(z),
                batch_range: Some(item_start..item_end),
            });
        }
    }

    shape_meta
        .vertices
        .write_buffer(&render_device, &render_queue);
}

pub fn queue_ui_shapes(
    transparent_draw_functions: Res<DrawFunctions<TransparentUi>>,
    view_uniforms: Res<ViewUniforms>,
    mut shape_meta: ResMut<ShapeMeta>, // TODO: make UI meta?
    render_device: Res<RenderDevice>,
    smud_pipeline: Res<SmudPipeline>,
    ui_shape_batches: Query<(Entity, &UiShapeBatch)>,
    mut views: Query<&mut RenderPhase<TransparentUi>>,
) {
    // TODO: look at both the shape renderer and the
    // ui renderer and figure out which part to copy here!!!

    let view_binding = match view_uniforms.uniforms.binding() {
        Some(binding) => binding,
        None => return,
    };

    // TODO: maybe redundant? (also done in regular pass)
    shape_meta.view_bind_group = Some(render_device.create_bind_group(&BindGroupDescriptor {
        entries: &[BindGroupEntry {
            binding: 0,
            resource: view_binding,
        }],
        label: Some("smud_shape_view_bind_group"),
        layout: &smud_pipeline.view_layout,
    }));

    let draw_smud_ui_shape = transparent_draw_functions
        .read()
        // TODO: compare with ui draw command
        .get_id::<DrawSmudUiShape>()
        .unwrap();

    for mut transparent_phase in views.iter_mut() {
        for (entity, batch) in ui_shape_batches.iter() {
            // TODO: specializing seems to normally be done in queue. Move it here?
            let pipeline = batch.pipeline;
            transparent_phase.add(TransparentUi {
                draw_function: draw_smud_ui_shape,
                pipeline,
                entity,
                sort_key: batch.z,
            });
        }
    }
}
