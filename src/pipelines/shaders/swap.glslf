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
