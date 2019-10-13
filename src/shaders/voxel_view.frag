#version 450 core

in vec3 v_color;
in vec3 vw_position;

out vec4 out_color;

layout(binding = 0) uniform sampler3D volume;
uniform int resolution;

void main() {
	out_color = vec4(v_color, 1.0);
}
