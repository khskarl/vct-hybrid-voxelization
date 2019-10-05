#version 450 core

layout (location = 0) in vec3 aPosition;
layout (location = 1) in vec2 aTexCoord;
layout (location = 2) in vec3 aNormal;

out vec3 v_position;
out vec2 v_uv;
out vec3 v_normal;

out VSOUT {
	vec3 position;
	vec2 uv;
	vec3 normal;
} v_out;

uniform layout(rgba8, binding = 0) image3D voxel_color;

void main() {
	v_out.uv = aTexCoord;
	v_out.normal = aNormal;

	uint i = gl_VertexID % 16;
	uint j = (gl_VertexID / 16) % 16;
	uint k = (gl_VertexID / 16 / 16) % 16;
	ivec3 coordinate = ivec3(i, j, k);

	gl_Position = vec4(coordinate - vec3(0.0, 2.0, 0.0), 1.0);

	v_out.position = coordinate;
	imageStore(voxel_color, coordinate, vec4(0.0, 1.0, 1.0, 1.0));
}
