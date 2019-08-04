mod context;
mod pipeline;

use self::context::DeferredPipelineContext;
use vulkano::sync::GpuFuture;

use crate::pipelines::{PipelineContext, PipelineDef};
use std::sync::Arc;
use vulkano::{
	device::{Device, Queue},
	format::Format,
};

const DIFFUSE_FORMAT: Format = Format::A2B10G10R10UnormPack32;
const NORMAL_FORMAT: Format = Format::R16G16B16A16Sfloat;
const DEPTH_FORMAT: Format = Format::D16Unorm;

pub struct DeferredPipelineDef;
impl PipelineDef for DeferredPipelineDef {
	fn make_context(device: &Arc<Device>, queue: &Arc<Queue>) -> (Box<dyn PipelineContext>, Box<dyn GpuFuture>) {
		let (pctx, future) = DeferredPipelineContext::new(device, queue);
		(Box::new(pctx), Box::new(future))
	}
}

#[derive(Default, Debug, Clone, Copy)]
#[repr(C)]
struct Vert2D {
	pos: [f32; 2],
	texc: [f32; 2],
}
vulkano::impl_vertex!(Vert2D, pos, texc);

mod geom_vshader {
	vulkano_shaders::shader! {
		ty: "vertex",
		src: "
#version 450
layout(location = 0) in vec3 pos;
layout(location = 1) in vec3 nor;
layout(location = 2) in vec2 texc;
layout(location = 3) in vec2 lmap;

layout(location = 0) out vec3 out_nor;
layout(location = 1) out vec2 out_texc;

layout(push_constant) uniform PushConsts {
	vec4 cam_proj;
	vec3 cam_pos;
	vec4 cam_rot;
	vec3 mesh_pos;
	vec4 mesh_rot;
} pc;

vec4 perspective(vec4 proj, vec3 pos) {
	return vec4(pos.xy * proj.xy, pos.z * proj.z + proj.w, -pos.z);
}

vec4 quat_inv(vec4 quat) {
	return vec4(-quat.xyz, quat.w) / dot(quat, quat);
}

vec3 quat_mul(vec4 quat, vec3 vec) {
	return cross(quat.xyz, cross(quat.xyz, vec) + vec * quat.w) * 2.0 + vec;
}

void main() {
	// stupid math library puts w first, so we flip it here
	vec4 cam_rot = pc.cam_rot.yzwx;
	vec4 mesh_rot = pc.mesh_rot.yzwx;

	vec3 pos_ws = quat_mul(mesh_rot, pos) + pc.mesh_pos;
	vec3 pos_cs = quat_mul(quat_inv(cam_rot), pos_ws - pc.cam_pos);
	vec3 pos_es = vec3(pos_cs.x, -pos_cs.z, -pos_cs.y);

	out_nor = nor;
	out_texc = texc;
	gl_Position = perspective(pc.cam_proj, pos_es);
}"
	}
}

mod geom_fshader {
	vulkano_shaders::shader! {
		ty: "fragment",
		src: "
#version 450
layout(location = 0) in vec3 nor;
layout(location = 1) in vec2 texc;

layout(location = 0) out vec4 color;
layout(location = 1) out vec4 normal;

layout(set = 0, binding = 0) uniform sampler2D tex;

void main() {
	color = vec4(sqrt(texture(tex, texc).rgb), 0);
	normal = vec4(nor, 0);
}
"
	}
}

mod swap_vshader {
	vulkano_shaders::shader! {
		ty: "vertex",
		src: "
#version 450
layout(location = 0) in vec2 pos;
layout(location = 2) in vec2 texc;

layout(location = 0) out vec2 out_texc;

void main() {
	out_texc = texc;
	gl_Position = vec4(pos, 0, 1);
}"
	}
}

mod swap_fshader {
	vulkano_shaders::shader! {
		ty: "fragment",
		src: "
#version 450
layout(location = 0) in vec2 texc;

layout(location = 0) out vec4 pixel;

layout(input_attachment_index = 0, set = 0, binding = 0) uniform subpassInput g_color;
layout(input_attachment_index = 1, set = 0, binding = 1) uniform subpassInput g_normal;
layout(input_attachment_index = 2, set = 0, binding = 2) uniform subpassInput g_depth;

void main() {
	vec4 layer0 = subpassLoad(g_color);
	vec4 layer1 = subpassLoad(g_normal);
	vec3 color = layer0.rgb * layer0.rgb;
	vec3 normal = layer1.xyz;
	vec3 light = vec3(0.02);
	light += vec3(1.0, 0.9, 0.8) * max(0, dot(normal.xyz, -normalize(vec3(1,2,3))));
	light += vec3(0.8, 0.9, 1.0) * max(0, dot(normal.xyz, normalize(vec3(1.75,1.25,3))));
	pixel = vec4(color * light, 0);
}
"
	}
}
