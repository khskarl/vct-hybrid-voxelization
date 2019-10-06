#version 450 core

in VSOUT {
	vec3 position;
	vec2 uv;
} v_in[];

layout (triangles) in;
layout (triangle_strip, max_vertices = 3) out;

void main() {
	gl_Position = vec4(v_in[0].position, 1.0);
	EmitVertex();
	gl_Position = vec4(v_in[1].position, 1.0);
	EmitVertex();
	gl_Position = vec4(v_in[2].position, 1.0);
	EmitVertex();
}