use std::ops::Range;

use bevy::{
    asset::HandleId,
    core::FloatOrd,
    prelude::{Component, Handle, Image},
    render::render_resource::{BindGroup, CachedPipelineId},
    utils::HashMap,
};
use bytemuck::{Pod, Zeroable};

pub mod extract;
pub mod prepare;
pub mod queue;

#[derive(Component, Eq, PartialEq, Copy, Clone)]
pub struct ShapeBatch {
    pub shader: (HandleId, HandleId),
}
#[derive(Component, Eq, PartialEq, Copy, Clone)]
pub struct TexturedShapeBatch {
    pub shader: (HandleId, HandleId),
    pub image_handle_id: HandleId,
}

#[derive(Component, Eq, PartialEq, Clone)]
pub struct UiShapeBatch {
    pub range: Range<u32>,
    pub shader_key: (HandleId, HandleId),
    pub z: FloatOrd,
    pub pipeline: CachedPipelineId,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct ShapeVertex {
    pub color: [f32; 4],
    pub frame: f32,
    pub position: [f32; 3],
    pub rotation: [f32; 2],
    pub scale: f32,
    // pub uv: [f32; 2],
}

#[derive(Default)]
pub struct ImageBindGroups {
    pub values: HashMap<Handle<Image>, BindGroup>,
}
