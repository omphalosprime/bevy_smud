use bevy::render::render_resource::{BindGroup, Buffer, BufferUsages, BufferVec};

use super::stages::ShapeVertex;

pub struct TimeMeta {
    pub buffer: Buffer,
    pub bind_group: Option<BindGroup>,
}
pub struct TexturedTimeMeta {
    pub buffer: Buffer,
    pub bind_group: Option<BindGroup>,
}

pub struct ShapeMeta {
    pub vertices: BufferVec<ShapeVertex>,
    pub ui_vertices: BufferVec<ShapeVertex>,
    pub view_bind_group: Option<BindGroup>,
}

impl Default for ShapeMeta {
    fn default() -> Self {
        Self {
            vertices: BufferVec::new(BufferUsages::VERTEX),
            ui_vertices: BufferVec::new(BufferUsages::VERTEX),
            view_bind_group: None,
        }
    }
}
pub struct TexturedShapeMeta {
    pub vertices: BufferVec<ShapeVertex>,
    pub ui_vertices: BufferVec<ShapeVertex>,
    pub view_bind_group: Option<BindGroup>,
    pub material_bind_group: Option<BindGroup>
}

impl Default for TexturedShapeMeta {
    fn default() -> Self {
        Self {
            vertices: BufferVec::new(BufferUsages::VERTEX),
            ui_vertices: BufferVec::new(BufferUsages::VERTEX),
            view_bind_group: None,
            material_bind_group: None,
            
        }
    }
}
