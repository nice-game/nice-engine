#version 450
#include "util.glsl"

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
