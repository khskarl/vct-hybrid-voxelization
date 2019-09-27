#version 330

layout (location = 0) in vec3 aPosition;

out VSOUT{
  vec4 position;
  int index;
	vec4 color;
} vsout;

uniform sampler3D volume;
uniform mat4 mvp;

void main() {
	int id = gl_VertexID;
	int k = id % 16;
	int j = k % 16;
	int i = j % 16;

	vec4 voxel = texelFetch(volume, ivec3(i, j, k), 0);

	//
	gl_Position = mvp * vec4(aPosition, 1.0);

	vsout.position = gl_Position;
	vsout.index = gl_VertexID;
	vsout.color = voxel;

	gl_PointSize = 10;
}
