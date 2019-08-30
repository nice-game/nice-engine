#version 450
layout(location = 0) in vec2 pos;
layout(location = 0) out vec2 dir;

void main() {
	dir = pos.xy;
	gl_Position = vec4(pos, 0, 1);
}
