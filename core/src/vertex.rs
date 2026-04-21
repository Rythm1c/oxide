use std::mem::offset_of;

#[repr(C)]
#[derive(Debug, Clone, Default, Copy)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal  : [f32; 3],
    pub uv      : [f32; 2],
    pub color   : [f32; 3],
}

impl Vertex {
    pub fn new(position: [f32; 3], normal: [f32; 3], uv: [f32; 2], color: [f32; 3]) -> Self {
        Vertex { position, normal, uv, color }
    }

    pub fn get_binding_description() -> ash::vk::VertexInputBindingDescription {
        ash::vk::VertexInputBindingDescription {
            binding: 0,
            stride: std::mem::size_of::<Vertex>() as u32,
            input_rate: ash::vk::VertexInputRate::VERTEX,
        }
    }

    pub fn get_attribute_descriptions() -> Vec<ash::vk::VertexInputAttributeDescription> {
        vec![
            ash::vk::VertexInputAttributeDescription {
                location: 0,
                binding: 0,
                format: ash::vk::Format::R32G32B32_SFLOAT,
                offset: offset_of!(Vertex, position) as u32,
            },
            ash::vk::VertexInputAttributeDescription {
                location: 1,
                binding: 0,
                format: ash::vk::Format::R32G32B32_SFLOAT,
                offset: offset_of!(Vertex, normal) as u32,
            },
            ash::vk::VertexInputAttributeDescription {
                location: 2,
                binding: 0,
                format: ash::vk::Format::R32G32_SFLOAT,
                offset: offset_of!(Vertex, uv) as u32,
            },
            ash::vk::VertexInputAttributeDescription {
                location: 3,
                binding: 0,
                format: ash::vk::Format::R32G32B32_SFLOAT,
                offset: offset_of!(Vertex, color) as u32,
            },
        ]
    }
}
