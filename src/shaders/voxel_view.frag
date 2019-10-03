#version 450 core

in vec4 v_color;
in vec3 vw_position;

out vec4 out_color;

layout(binding = 0) uniform sampler3D volume;
uniform int resolution;

void main() {
	// out_color = v_color;
	// out_color = vec4(vw_position / float(resolution), 1.0);
	out_color = vec4(v_color.xyz, 1.0);
}
