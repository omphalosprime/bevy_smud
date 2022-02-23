use super::stages::extract::ShapeShaders;
use bevy::asset::HandleId;
use bevy::prelude::*;
use bevy::render::render_resource::std140::AsStd140;
use bevy::render::render_resource::{
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BlendState, BufferBindingType,
    BufferSize, ColorTargetState, ColorWrites, Face, FragmentState, FrontFace, MultisampleState,
    PolygonMode, PrimitiveState, RenderPipelineDescriptor, SamplerBindingType, ShaderStages,
    SpecializedPipeline, TextureFormat, TextureSampleType, TextureViewDimension, VertexAttribute,
    VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
};
use bevy::render::texture::BevyDefault;
use bevy::render::view::ViewUniform;
use bevy::{prelude::FromWorld, render::render_resource::BindGroupLayout};
use bevy::{render::renderer::RenderDevice, sprite::Mesh2dPipelineKey};

fn shape_view_layout(render_device: &RenderDevice) -> BindGroupLayout {
    render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        entries: &[BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: true,
                min_binding_size: BufferSize::new(ViewUniform::std140_size_static() as u64),
            },
            count: None,
        }],

        label: Some("shape_view_layout"),
    })
}

fn time_layout(render_device: &RenderDevice) -> BindGroupLayout {
    render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("time layout"),
        entries: &[BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: BufferSize::new(std::mem::size_of::<f32>() as u64),
            },
            count: None,
        }],
    })
}

fn material_layout(render_device: &RenderDevice) -> BindGroupLayout {
    render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        entries: &[
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Texture {
                    multisampled: false,
                    sample_type: TextureSampleType::Float { filterable: true },
                    view_dimension: TextureViewDimension::D2,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Sampler(SamplerBindingType::Filtering),
                count: None,
            },
        ],
        label: Some("texture_material_layout"),
    })
}

pub struct SmudPipeline {
    pub view_layout: BindGroupLayout,
    pub time_bind_group_layout: BindGroupLayout,
    pub shaders: ShapeShaders,
}

impl FromWorld for SmudPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.get_resource::<RenderDevice>().unwrap();
        let view_layout = shape_view_layout(render_device);
        let time_bind_group_layout = time_layout(render_device);
        Self {
            view_layout,
            shaders: Default::default(),
            time_bind_group_layout,
        }
    }
}

fn smud_vertex_data() ->  (Vec<VertexAttribute>, u64){
      // Customize how to store the meshes' vertex attributes in the vertex buffer
        // Our meshes only have position and color
        let vertex_attributes = vec![
            // (NEED TO KNOW: GOTCHA! attributes are sorted alphabetically, and offsets need to reflect this)
            // Color
            VertexAttribute {
                format: VertexFormat::Float32x4,
                offset: 0,
                shader_location: 1,
            },
            // Frame
            VertexAttribute {
                format: VertexFormat::Float32,
                offset: (4) * 4,
                shader_location: 4,
            },
            // Position
            VertexAttribute {
                format: VertexFormat::Float32x3,
                offset: (4 + 1) * 4,
                shader_location: 0,
            },
            // Rotation
            VertexAttribute {
                format: VertexFormat::Float32x2,
                offset: (4 + 1 + 3) * 4,
                shader_location: 2,
            },
            // Scale
            VertexAttribute {
                format: VertexFormat::Float32,
                offset: (4 + 1 + 3 + 2) * 4,
                shader_location: 3,
            },
        ];
        // This is the sum of the size of the attributes above
        let vertex_array_stride = (4 + 1 + 3 + 2 + 1) * 4;
        (vertex_attributes, vertex_array_stride)

}


#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct SmudPipelineKey {
    pub mesh: Mesh2dPipelineKey,
    pub shader: (HandleId, HandleId),
}

impl SpecializedPipeline for SmudPipeline {
    type Key = SmudPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        let shader = self.shaders.0.get(&key.shader).unwrap();
        info!("specializing for {shader:?}");
        let (vertex_attributes, vertex_array_stride) = smud_vertex_data();
  
        RenderPipelineDescriptor {
            vertex: VertexState {
                shader: shader.clone_weak(),
                // shader: SMUD_SHADER_HANDLE.typed::<Shader>(),
                entry_point: "vertex".into(),
                shader_defs: Vec::new(),
                buffers: vec![VertexBufferLayout {
                    array_stride: vertex_array_stride,
                    step_mode: VertexStepMode::Instance,
                    attributes: vertex_attributes,
                }],
            },
            fragment: Some(FragmentState {
                shader: shader.clone_weak(),
                // shader: SMUD_SHADER_HANDLE.typed::<Shader>(),
                entry_point: "fragment".into(),
                shader_defs: Vec::new(),
                targets: vec![ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                }],
            }),
            layout: Some(vec![
                // Bind group 0 is the view uniform
                self.view_layout.clone(),
                self.time_bind_group_layout.clone(),
            ]),
            primitive: PrimitiveState {
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back), // TODO: incorrect? sprite pipeline set to None
                unclipped_depth: false,      // What is this?
                polygon_mode: PolygonMode::Fill,
                conservative: false,                     // What is this?
                topology: key.mesh.primitive_topology(), // resolves to none?
                strip_index_format: None,                // TODO: what does this do?
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: key.mesh.msaa_samples(),
                mask: !0,                         // what does the mask do?
                alpha_to_coverage_enabled: false, // what is this?
            },
            label: Some("bevy_smud_pipeline".into()),
        }
    }
}

pub struct SmudTexturedPipeline {
    pub view_layout: BindGroupLayout,
    pub time_bind_group_layout: BindGroupLayout,
    pub material_layout: BindGroupLayout,
    pub shaders: ShapeShaders,
}

impl FromWorld for SmudTexturedPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.get_resource::<RenderDevice>().unwrap();
        let view_layout = shape_view_layout(render_device);
        let time_bind_group_layout = time_layout(render_device);
        let material_layout = material_layout(render_device);
        Self {
            view_layout,
            shaders: Default::default(),
            time_bind_group_layout,
            material_layout
        }
    }
}


#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct SmudTexturedPipelineKey {
    pub mesh: Mesh2dPipelineKey,
    pub shader: (HandleId, HandleId),
}

impl SpecializedPipeline for SmudTexturedPipeline {
    type Key = SmudTexturedPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        let shader = self.shaders.0.get(&key.shader).unwrap();
        info!("specializing for {shader:?}");
        let (vertex_attributes, vertex_array_stride) = smud_vertex_data();
  
        RenderPipelineDescriptor {
            vertex: VertexState {
                shader: shader.clone_weak(),
                // shader: SMUD_SHADER_HANDLE.typed::<Shader>(),
                entry_point: "vertex".into(),
                shader_defs: Vec::new(),
                buffers: vec![VertexBufferLayout {
                    array_stride: vertex_array_stride,
                    step_mode: VertexStepMode::Instance,
                    attributes: vertex_attributes,
                }],
            },
            fragment: Some(FragmentState {
                shader: shader.clone_weak(),
                // shader: SMUD_SHADER_HANDLE.typed::<Shader>(),
                entry_point: "fragment".into(),
                shader_defs: Vec::new(),
                targets: vec![ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                }],
            }),
            layout: Some(vec![
                // Bind group 0 is the view uniform
                self.view_layout.clone(),
                self.time_bind_group_layout.clone(),
                self.material_layout.clone()
            ]),
            primitive: PrimitiveState {
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back), // TODO: incorrect? sprite pipeline set to None
                unclipped_depth: false,      // What is this?
                polygon_mode: PolygonMode::Fill,
                conservative: false,                     // What is this?
                topology: key.mesh.primitive_topology(), // resolves to none?
                strip_index_format: None,                // TODO: what does this do?
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: key.mesh.msaa_samples(),
                mask: !0,                         // what does the mask do?
                alpha_to_coverage_enabled: false, // what is this?
            },
            label: Some("bevy_smud_pipeline".into()),
        }
    }
}