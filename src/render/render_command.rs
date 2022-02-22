
use bevy::{prelude::*, render::{view::ViewUniformOffset, render_phase::{EntityRenderCommand, SetItemPipeline, TrackedRenderPass, RenderCommandResult, BatchedPhaseItem, RenderCommand}}, ecs::system::{lifetimeless::{SQuery, Read, SRes}, SystemParamItem}};

use super::{meta::{ShapeMeta, TimeMeta}, stages::{ShapeBatch, UiShapeBatch}};



// order matters....
pub type DrawSmudShape = (
    SetItemPipeline,
    SetShapeViewBindGroup<0>,
    SetTimeBindGroup<1>,
    DrawShapeBatch,
);
pub struct SetShapeViewBindGroup<const I: usize>;
impl<const I: usize> EntityRenderCommand for SetShapeViewBindGroup<I> {
    type Param = (SRes<ShapeMeta>, SQuery<Read<ViewUniformOffset>>);

    fn render<'w>(
        view: Entity,
        _item: Entity,
        (shape_meta, view_query): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let view_uniform = view_query.get(view).unwrap();
        pass.set_bind_group(
            I,
            shape_meta.into_inner().view_bind_group.as_ref().unwrap(),
            &[view_uniform.offset],
        );
        RenderCommandResult::Success
    }
}

pub struct DrawShapeBatch;
impl<P: BatchedPhaseItem> RenderCommand<P> for DrawShapeBatch {
    type Param = (SRes<ShapeMeta>, SQuery<Read<ShapeBatch>>);

    fn render<'w>(
        _view: Entity,
        item: &P,
        (shape_meta, _query_batch): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        // let shape_batch = query_batch.get(item.entity()).unwrap();
        let shape_meta = shape_meta.into_inner();
        pass.set_vertex_buffer(0, shape_meta.vertices.buffer().unwrap().slice(..));
        pass.draw(0..4, item.batch_range().as_ref().unwrap().clone());
        RenderCommandResult::Success
    }
}

pub struct SetTimeBindGroup<const I: usize>;
impl<const I: usize> EntityRenderCommand for SetTimeBindGroup<I> {
    type Param = SRes<TimeMeta>;

    fn render<'w>(
        _view: Entity,
        _item: Entity,
        time_meta: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let time_bind_group = time_meta.into_inner().bind_group.as_ref().unwrap();

        pass.set_bind_group(I, time_bind_group, &[]);

        RenderCommandResult::Success
    }
}





pub type DrawSmudUiShape = (SetItemPipeline, SetShapeViewBindGroup<0>, DrawUiShapeNode);
pub struct DrawUiShapeNode;
impl EntityRenderCommand for DrawUiShapeNode {
    type Param = (SRes<ShapeMeta>, SQuery<Read<UiShapeBatch>>);

    fn render<'w>(
        _view: Entity,
        item: Entity,
        (ui_shape_meta, query_batch): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let batch = query_batch.get(item).unwrap();

        pass.set_vertex_buffer(
            0,
            ui_shape_meta
                .into_inner()
                .ui_vertices
                .buffer()
                .unwrap()
                .slice(..),
        );
        pass.draw(0..4, batch.range.clone());
        RenderCommandResult::Success
    }
}




// struct DrawQuad;
// impl EntityRenderCommand for DrawQuad {
//     type Param = SRes<SmudPipeline>;
//     #[inline]
//     fn render<'w>(
//         _view: Entity,
//         _item: Entity,
//         pipeline: SystemParamItem<'w, '_, Self::Param>,
//         pass: &mut TrackedRenderPass<'w>,
//     ) -> RenderCommandResult {
//         let gpu_mesh = &pipeline.into_inner().quad;
//         pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
//         match &gpu_mesh.buffer_info {
//             GpuBufferInfo::Indexed {
//                 buffer,
//                 index_format,
//                 count,
//             } => {
//                 pass.set_index_buffer(buffer.slice(..), 0, *index_format);
//                 pass.draw_indexed(0..*count, 0, 0..1);
//             }
//             GpuBufferInfo::NonIndexed { vertex_count } => {
//                 pass.draw(0..*vertex_count, 0..1);
//             }
//         }
//         RenderCommandResult::Success
//     }
// }