#version 450

layout(location = 0) in vec3 pos;
layout(location = 1) in vec2 uv;

layout(std140, set = 0, binding = 0) uniform Args {
	mat4 proj;
	mat4 view;
	float time;
};

layout(location = 0) out vec2 frag_uv;

void main() {
	frag_uv = uv;
	gl_Position = (proj * view) * vec4(pos, 1.0);
}
