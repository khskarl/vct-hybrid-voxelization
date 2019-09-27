// Credits: Solomon Medland
// https://stackoverflow.com/questions/28375338/cube-using-single-gl-triangle-strip

#version 330

in VSOUT{
  vec4 position;
  int index;
	vec4 color;
} v_in[];

out vec4 v_color;

layout (points) in;
layout (points, max_vertices = 1) out;

uniform sampler3D volume;
uniform mat4 mvp;

void main() {
	int id = v_in[0].index;

	int k = id % 10;
	int j = k % 10;
	int i = j % 10;

	vec4 voxel = texelFetch(volume, ivec3(i, j, k), 0);

	vec4 center = v_in[0].position;
	v_color = v_in[0].color;

	gl_Position = center;
	EmitVertex();
}
