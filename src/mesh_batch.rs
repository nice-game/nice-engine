use crate::{surface::SWAP_FORMAT, Context};
use std::sync::Arc;
use vulkano::{
	command_buffer::DynamicState,
	framebuffer::{RenderPassAbstract, Subpass},
	pipeline::{GraphicsPipeline, GraphicsPipelineAbstract},
};

pub struct MeshBatch {
}
impl MeshBatch {
	pub fn new(ctx: &Context) -> Self {


		Self { }
	}
}

#[derive(Default, Debug, Clone)]
pub(crate) struct Vertex {
	position: [f32; 2],
}
vulkano::impl_vertex!(Vertex, position);

pub(crate) mod vs {
	vulkano_shaders::shader! {
		ty: "vertex",
		src: "
#version 450
layout(location = 0) in vec2 position;
void main() {
	gl_Position = vec4(position, 0.0, 1.0);
}"
	}
}

pub(crate) mod fs {
	vulkano_shaders::shader! {
		ty: "fragment",
		src: "
#version 450
layout(location = 0) out vec4 f_color;
void main() {
	f_color = vec4(1.0, 0.0, 0.0, 1.0);
}
"
	}
}
