#version 450 core

layout (location = 0) in vec3 aPosition;

out VSOUT{
	vec4 position;
	vec3 w_position;
	vec4 color;
} v_out;

layout (binding = 0) uniform sampler3D volume;
uniform mat4 mvp;
uniform int resolution;

void main() {
	float voxel_size = (1.0 / float(resolution));

	uint i = gl_VertexID % resolution;
	uint j = (gl_VertexID / resolution) % resolution;
	uint k = (gl_VertexID / resolution / resolution) % resolution;

	ivec3 texel_position = ivec3(i, j, k) + ivec3(aPosition);
	vec4 color = texelFetch(volume, texel_position, 0);

	gl_Position = mvp * vec4(texel_position * voxel_size + vec3(voxel_size / 2.0), 1.0);

	v_out.position = gl_Position;
	v_out.w_position = texel_position;
	v_out.color = color;
}
