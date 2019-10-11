#version 450 core

uniform int u_width;
uniform int u_height;
uniform int u_depth;

uniform layout(rgba8, binding = 0) image3D u_voxel_albedo;
uniform layout(rgba8, binding = 1) image3D u_voxel_normal;
uniform layout(rgba8, binding = 2) image3D u_voxel_emission;

void main() {
	uint i = gl_VertexID % u_width;
	uint j = (gl_VertexID / u_width) % u_height;
	uint k = (gl_VertexID / u_width / u_height) % u_depth;
	ivec3 coordinate = ivec3(i, j, k);

	imageStore(u_voxel_albedo, coordinate, vec4(0.0, 0.0, 0.0, 0.0));
	imageStore(u_voxel_normal, coordinate, vec4(0.0, 0.0, 0.0, 0.0));
	imageStore(u_voxel_emission, coordinate, vec4(0.0, 0.0, 0.0, 0.0));
}
