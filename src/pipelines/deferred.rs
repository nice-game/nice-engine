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

const DEPTH_FORMAT: Format = Format::D32Sfloat;
const ALT_DEPTH_FORMAT: Format = Format::X8_D24UnormPack32;

const DIFFUSE_FORMAT: Format = Format::A2B10G10R10UnormPack32;
const POSITION_FORMAT: Format = Format::R32G32B32A32Sfloat;
const NORMAL_FORMAT: Format = Format::R32G32B32A32Sfloat;
const LIGHT_FORMAT: Format = Format::R32G32B32A32Sfloat;

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
layout(input_attachment_index = 4, set = 0, binding = 4) uniform subpassInput g_light;

layout(push_constant) uniform PushConsts {
	float Exposure;
} pc;

void main() {
	float depth = subpassLoad(g_depth).x;
	vec3 color = subpassLoad(g_light).rgb;
	color *= pc.Exposure;
	color /= 1.0 + color;
	pixel = vec4(color, 0);
}
"
	}
}

mod light_vshader {
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

mod light_fshader {
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
layout(input_attachment_index = 4, set = 0, binding = 4) uniform subpassInput g_light;

layout(push_constant) uniform PushConsts {
	vec4 Resolution;
	vec4 Projection;
	vec4 CameraRotation;
	vec3 CameraOffset;
	vec4 LightPosition;
	vec4 LightColor;
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
	vec3 position = subpassLoad(g_position).xyz;
	vec3 normal = subpassLoad(g_normal).xyz;
	vec3 color = subpassLoad(g_color).rgb;
	color *= color;

	float metalness = 0.0;
	float smoothness = 0.5;

	vec3 diffuseColor = color * (1.0 - metalness);
	vec3 specularColor = mix(vec3(0.04), color, metalness);
	float specularExponent = 128.0 * pow(2.0, 8.0 * smoothness - 4.0);
	float specularNorm = specularExponent * 0.03978873577297383 + 0.2785211504108169;

	float lightRadiusSquaredTimesCutoff = pc.LightColor.w;
	float lightRadiusSquaredInverse = pc.LightPosition.w;
	vec3 lightOffset = pc.LightPosition.xyz - position;
	float lightDistanceSquared = dot(lightOffset, lightOffset);
	float lightFalloff = min(1.0, lightDistanceSquared * lightRadiusSquaredInverse);
	lightFalloff *= lightFalloff; lightFalloff *= lightFalloff;
	lightFalloff = 1.0 - lightFalloff;
	lightFalloff = mix(lightFalloff * lightFalloff, lightFalloff, 0.3095096836885878);
	lightFalloff *= max(0.0, dot(normal, normalize(lightOffset))) / (1.0 + lightDistanceSquared);
	lightFalloff *= lightRadiusSquaredTimesCutoff;
	vec3 lightPower = pc.LightColor.rgb * lightFalloff;
	float lightSpecular = pow(max(0.0, dot(normalize(normalize(pc.LightPosition.xyz - position) + normalize(pc.CameraOffset - position)), normal)), specularExponent) * specularNorm;
	vec3 diffuse = lightPower * diffuseColor;
	vec3 specular = lightPower * specularColor * lightSpecular;
	pixel = vec4(diffuse + specular, 0);
}
"
	}
}
