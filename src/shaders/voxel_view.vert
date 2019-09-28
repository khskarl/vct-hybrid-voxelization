#version 450 core

layout (location = 0) in vec3 aPosition;

out VSOUT{
	vec4 position;
	vec3 w_position;
	vec4 color;
} v_out;

layout (binding = 0) uniform sampler3D volume;
uniform mat4 mvp;

void main() {
	int k = gl_VertexID % 16;
	int j = k % 16;
	int i = j % 16;

	float voxel = texelFetch(volume, ivec3(i, j, k), 0).r;

	gl_Position = mvp * vec4(aPosition, 1.0);

	v_out.position = gl_Position;
	v_out.w_position = aPosition;
	v_out.color = vec4(voxel);

	gl_PointSize = gl_Position.z;
}
