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
const POSITION_FORMAT: Format = Format::R32G32B32A32Sfloat;
const NORMAL_FORMAT: Format = Format::R16G16B16A16Sfloat;
const DEPTH_FORMAT: Format = Format::D32Sfloat;
const ALT_DEPTH_FORMAT: Format = Format::X8_D24UnormPack32;

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
layout(location = 2) out vec3 out_pos;

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

	out_nor = quat_mul(mesh_rot, nor);
	out_pos = pos_ws;
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
layout(location = 2) in vec3 pos;

layout(location = 0) out vec4 color;
layout(location = 1) out vec4 normal;
layout(location = 2) out vec4 position;

layout(set = 0, binding = 0) uniform sampler2D tex;

void main() {
	vec4 tex = texture(tex, texc);
	if (tex.w < 0.125) discard;
	//tex.rgb = sqrt(tex.rgb); // FIXME: do this for srgb or linear textures, skip it for quadratic textures.
	color = vec4(tex.rgb, 0);
	normal = vec4(nor, 0);
	position = vec4(pos, 0);
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

layout(input_attachment_index = 0, set = 0, binding = 0) uniform subpassInput g_depth;
layout(input_attachment_index = 1, set = 0, binding = 1) uniform subpassInput g_color;
layout(input_attachment_index = 2, set = 0, binding = 2) uniform subpassInput g_normal;
layout(input_attachment_index = 3, set = 0, binding = 3) uniform subpassInput g_position;

layout(push_constant) uniform PushConsts {
	vec4 Resolution;
	vec4 Projection;
	vec4 CameraRotation;
	vec3 CameraOffset;
} pc;

vec3 quat_mul(vec4 quat, vec3 vec) {
	return cross(quat.xyz, cross(quat.xyz, vec) + vec * quat.w) * 2.0 + vec;
}

vec4 quat_inv(vec4 quat) {
	return vec4(-quat.xyz, quat.w) / dot(quat, quat);
}

vec3 inv_perspective(vec4 Projection, vec3 Position) {
	//vec4 InverseProjection = vec4(projection.w / projection.x, projection.w / projection.y, -projection.w, projection.z);
	//return vec3(Position.xy * InverseProjection.xy, InverseProjection.z) / (Position.z + InverseProjection.w);

	return Projection.w * vec3(Position.xy / Projection.xy, 1.0) / (Position.z + Projection.z);
}

void main() {
	//vec2 Resolution = vec2(1440.0, 810.0, 1.0/1440.0, 1.0/810.0);

	vec4 layer0 = subpassLoad(g_depth);
	vec4 layer1 = subpassLoad(g_color);
	vec4 layer2 = subpassLoad(g_normal);
	vec4 layer3 = subpassLoad(g_position);

	vec3 color = layer1.rgb * layer1.rgb;	
	vec3 normal_ws = layer2.xyz;
	vec3 position_ws = layer3.xyz;
	
	//vec3 position_ds = vec3(gl_FragCoord.xy * pc.Resolution.zw, layer0.x) * 2.0 - 1.0;
	//vec3 position_es = inv_perspective(pc.Projection, position_ds);
	//vec3 position_cs = vec3(position_es.x, -position_es.z, -position_es.y);
	//vec3 position_ws = quat_mul(pc.CameraRotation.yzwx, position_cs) + pc.CameraOffset;
	
	vec3 dlight = vec3(0.02);
	vec3 slight = vec3(0.0);

//	dlight += vec3(0.10, 0.09, 0.08) * max(0, dot(normal_ws.xyz, -normalize(vec3(1,2,3))));
//	dlight += vec3(0.08, 0.09, 0.10) * max(0, dot(normal_ws.xyz, normalize(vec3(1.75,1.25,3))));
	
	float gridSize = 1.0;
	bool grid = (mod(position_ws.x, gridSize) > gridSize*0.5);
	if (mod(position_ws.y, gridSize) > gridSize*0.5) grid = !grid;
	if (mod(position_ws.z, gridSize) > gridSize*0.5) grid = !grid;

	vec3 specColor = vec3(0.04); // IRL everything except metal reflects about 4%
	float specExponent = 20.0;
	//if (grid) specExponent = 500.0;
	float specNorm = specExponent * 0.03978873577297383 + 0.2785211504108169;
	//float specNorm = (specExponent + 2) * (specExponent + 4) / ((8 * 3.14159) * (specExponent + 1.0 / pow(2.0, specExponent / 2.0))); // exact


	float lightCutoff = 0.01;
	vec3 grayWeights = vec3(0.2126, 0.7152, 0.0722);

	vec3 lightColor = vec3(10.0, 7.5, 5.625);
	float lightRadius = sqrt(pow(dot(lightColor, grayWeights), 1.0/3.0) / lightCutoff);
	vec3 lightPos = vec3(23.0, 18.0, -12.0);
	vec3 lightOffset = lightPos - position_ws;
	float lightFalloff = max(0.0, dot(normal_ws, normalize(lightOffset))) / (1.0 + dot(lightOffset, lightOffset));
	lightFalloff *= sqrt(1.0 - min(1.0, length(lightOffset) / lightRadius));
	float lightSpecular = pow(max(0.0, dot(normalize(normalize(lightPos - position_ws) + normalize(pc.CameraOffset - position_ws)), normal_ws)), specExponent);
	dlight += lightColor * lightFalloff;
	slight += lightColor * lightFalloff * lightSpecular * specNorm * specColor;
	
	lightColor = vec3(5.625, 7.5, 10.0);
	lightRadius = sqrt(pow(dot(lightColor, grayWeights), 1.0/3.0) / lightCutoff);
	lightPos = vec3(5.2, 21.8, -12.0);
	lightOffset = lightPos - position_ws;
	lightFalloff = max(0.0, dot(normal_ws, normalize(lightOffset))) / (1.0 + dot(lightOffset, lightOffset));
	lightFalloff *= sqrt(1.0 - min(1.0, length(lightOffset) / lightRadius));
	lightSpecular = pow(max(0.0, dot(normalize(normalize(lightPos - position_ws) + normalize(pc.CameraOffset - position_ws)), normal_ws)), specExponent);
	dlight += lightColor * lightFalloff;
	slight += lightColor * lightFalloff * lightSpecular * specNorm * specColor;

	pixel = vec4(color * dlight + slight, 0);
	//pixel = vec4(normal_ws, 0);
}
"
	}
}
