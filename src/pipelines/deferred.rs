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
// const ALT_DEPTH_FORMAT: Format = Format::X8_D24UnormPack32;
const COLOR_FORMAT: Format = Format::A2B10G10R10UnormPack32;
const POSITION_FORMAT: Format = Format::R32G32B32A32Sfloat;
const NORMAL_FORMAT: Format = Format::R32G32B32A32Sfloat;
const LIGHT_FORMAT: Format = Format::R32G32B32A32Sfloat;

pub(crate) struct DeferredPipelineDef;
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
layout(location = 1) out vec4 out_texc;
layout(location = 2) out vec3 out_pos;

layout(push_constant) uniform PushConsts {
	vec4 cam_proj;
	vec4 cam_pos;
	vec4 cam_rot;
	vec4 mesh_pos;
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

	vec3 pos_ws = quat_mul(mesh_rot, pos) + pc.mesh_pos.xyz;
	vec3 pos_cs = quat_mul(quat_inv(cam_rot), pos_ws - pc.cam_pos.xyz);
	vec3 pos_es = vec3(pos_cs.x, -pos_cs.z, -pos_cs.y);

	out_nor = quat_mul(mesh_rot, nor);
	out_pos = pos_ws;
	out_texc = vec4(texc, lmap);
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
layout(location = 1) in vec4 texc;
layout(location = 2) in vec3 pos;

layout(location = 0) out vec4 out_color;
layout(location = 1) out vec4 out_light;
layout(location = 2) out vec4 out_normal;
layout(location = 3) out vec4 out_position;

layout(set = 0, binding = 0) uniform sampler2D color;
layout(set = 0, binding = 1) uniform sampler2D finish;
layout(set = 0, binding = 2) uniform sampler2D ambient_occlusion;
layout(set = 0, binding = 3) uniform sampler2D lightmap_flat;
layout(set = 0, binding = 4) uniform sampler2D lightmap_angle0;
layout(set = 0, binding = 5) uniform sampler2D lightmap_angle1;
layout(set = 0, binding = 6) uniform sampler2D lightmap_angle2;

void main() {
	vec4 color = texture(color, texc.xy);
	if (color.w < 0.125) discard;
	//color.rgb = sqrt(color.rgb); // FIXME: do this for srgb or linear textures, skip it for quadratic textures.
	out_color = vec4(color.rgb, 0);
	out_light = texture(ambient_occlusion, texc.zw) * 0.2 * out_color;
	out_normal = vec4(nor, 0);
	out_position = vec4(pos, 0);
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
layout(location = 0) out vec2 dir;

void main() {
	dir = pos.xy;
	gl_Position = vec4(pos, 0, 1);
}"
	}
}

mod swap_fshader {
	vulkano_shaders::shader! {
		ty: "fragment",
		src: "
#version 450
layout(location = 0) in vec2 dir;
layout(location = 0) out vec4 pixel;

layout(input_attachment_index = 0, set = 0, binding = 0) uniform subpassInput g_depth;
layout(input_attachment_index = 1, set = 0, binding = 1) uniform subpassInput g_color;
layout(input_attachment_index = 2, set = 0, binding = 2) uniform subpassInput g_normal;
layout(input_attachment_index = 3, set = 0, binding = 3) uniform subpassInput g_position;
layout(input_attachment_index = 4, set = 0, binding = 4) uniform subpassInput g_light;

layout(set = 1, binding = 0) uniform sampler2D sky;

layout(push_constant) uniform PushConsts {
	vec4 inv_proj;
	vec4 cam_rot;
} pc;

const float M_PI = 3.141592653589793;

vec3 eye_ray(vec4 inv_proj, vec2 texc) {
	return -normalize(inv_proj.xyz * vec3(2.0 * texc - 1.0, 1.0));
}

vec3 quat_mul(vec4 quat, vec3 vec) {
	return cross(quat.xyz, cross(quat.xyz, vec) + vec * quat.w) * 2.0 + vec;
}

vec3 skybox(vec4 cam_rot) {
	vec3 skydir = -normalize(pc.inv_proj.xyz * vec3(dir, 1.0));
	skydir = quat_mul(cam_rot, vec3(skydir.x, -skydir.z, -skydir.y));
	vec2 uv = vec2(atan(skydir.x, -skydir.y) / 2.0, acos(skydir.z)) / M_PI;
	return textureLod(sky, uv, 0).rgb;
}

void main() {
	// stupid math library puts w first, so we flip it here
	vec4 cam_rot = pc.cam_rot.yzwx;

	float depth = subpassLoad(g_depth).x;
	vec3 color = subpassLoad(g_light).rgb;
	if (depth == 1.0) color = skybox(cam_rot);
	color /= 1.0 + length(color);
	// color = (color * (2.51 * color + 0.03)) / (color * (2.43 * color + 0.59) + 0.14); // ACES
	pixel = vec4(color, 0); // Don't gamma correct! Output framebuffer has hardware sRGB encoding.
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

void main() {
	gl_Position = vec4(pos, 0, 1);
}"
	}
}

mod light_fshader {
	vulkano_shaders::shader! {
		ty: "fragment",
		src: "
#version 450
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
	vec4 CameraOffset;
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

vec3 oren_nayar(vec3 lightDirection, vec3 viewDirection, vec3 surfaceNormal, float rough, vec3 albedo) {
	float LdotV = dot(lightDirection, viewDirection);
	float NdotL = dot(lightDirection, surfaceNormal);
	float NdotV = dot(surfaceNormal, viewDirection);
	float s = LdotV - NdotL * NdotV;
	float t = mix(1.0, max(NdotL, NdotV), step(0.0, s));
	float sigma2 = rough * rough;
	vec3 A = 1.0 + sigma2 * (albedo / (sigma2 + 0.13) + 0.5 / (sigma2 + 0.33));
	float B = 0.45 * sigma2 / (sigma2 + 0.09);
	return albedo * max(0.0, NdotL) * (A + B * s / t) / 3.14159265;
}

void main() {
	vec3 position = subpassLoad(g_position).xyz;
	vec3 normal = subpassLoad(g_normal).xyz;
	vec3 color = subpassLoad(g_color).rgb;
	color *= color;

	float metal = 0.0;
	float rough = 0.2;

	/*
	bool grid = false;
	if (mod(position.x, 1.0) < 0.5) grid = !grid;
	if (mod(position.y, 1.0) < 0.5) grid = !grid;
	if (mod(position.z, 1.0) < 0.5) grid = !grid;
	if (grid) metal = 1.0;
	*/

	float specularExponent = 128.0 * pow(2.0, 4.0 - 8.0 * rough);
	float specularNorm = specularExponent * 0.03978873577297383 + 0.2785211504108169;
	vec3 diffuseColor = color * (1.0 - metal);

	float lightRadiusSquaredTimesCutoff = pc.LightColor.w;
	float lightRadiusSquaredInverse = pc.LightPosition.w;
	vec3 lightOffset = pc.LightPosition.xyz - position;
	float lightDistanceSquared = dot(lightOffset, lightOffset);
	float lightFalloff = min(1.0, lightDistanceSquared * lightRadiusSquaredInverse);
	lightFalloff *= lightFalloff; lightFalloff *= lightFalloff;
	lightFalloff = 1.0 - lightFalloff;
	lightFalloff = mix(lightFalloff * lightFalloff, lightFalloff, 0.3095096836885878);
	lightFalloff *= max(0.0, dot(normal, normalize(lightOffset)));
	lightFalloff /= 1.0 + lightDistanceSquared;
	lightFalloff *= lightRadiusSquaredTimesCutoff;
	vec3 lightPower = pc.LightColor.rgb * lightFalloff;
	float specularPower = pow(max(0.0, dot(normalize(normalize(pc.LightPosition.xyz - position) + normalize(pc.CameraOffset.xyz - position)), normal)), specularExponent) * specularNorm;
	vec3 specularColor = mix(vec3(0.04), color, metal) * specularPower;
	pixel = vec4((diffuseColor + specularColor) * lightPower, 1);
}
"
	}
}
