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
