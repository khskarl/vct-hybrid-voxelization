#version 450 core

layout (location = 0) in vec3 aPosition;

out vec4 v_color;
out vec3 vw_position;

layout (binding = 0) uniform sampler3D volume;
uniform mat4 mvp;

void main() {
	int id = gl_VertexID;
	int k = id % 16;
	int j = k % 16;
	int i = j % 16;

	vec4 voxel = texelFetch(volume, ivec3(i, j, k), 0);

	gl_Position = mvp * vec4(aPosition, 1.0);

	// vsout.color = voxel;
	vw_position = aPosition;
	v_color = voxel;

	gl_PointSize = gl_Position.z;
}
