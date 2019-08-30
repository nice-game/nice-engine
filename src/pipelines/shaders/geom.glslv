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
}
