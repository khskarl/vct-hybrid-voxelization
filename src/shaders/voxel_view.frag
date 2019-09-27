#version 450

in vec4 v_color;

out vec4 out_color;

uniform sampler3D volume;

void main() {
	// float voxel = textureLod(volume, vec3(i, j, k), 0).r;
	out_color = v_color;
}
