use crate::Context;

pub struct MeshBatch {}
impl MeshBatch {
	pub fn new(_ctx: &Context) -> Self {
		Self {}
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
