#version 450 core

in vec4 v_color;
in vec3 vw_position;

out vec4 out_color;

layout(binding = 0) uniform sampler3D volume;

void main() {
	out_color = v_color;
	// out_color = vec4(v_color / 16.0, 1.0) + v_color;
	out_color = vec4(v_color) + v_color;
}
