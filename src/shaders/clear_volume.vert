#version 450 core

uniform layout(binding = 0, rgba8) image3D u_voxel_albedo;
uniform layout(binding = 1, rgba8) image3D u_voxel_normal;
uniform layout(binding = 2, rgba8) image3D u_voxel_emission;
uniform layout(binding = 3, rgba8) image3D u_voxel_radiance;

void main() {
	ivec3 size = imageSize(u_voxel_albedo);

	uint width  = size.x;
	uint height = size.y;
	uint depth  = size.z;

	uint i = gl_VertexID % width;
	uint j = (gl_VertexID / width) % height;
	uint k = (gl_VertexID / width / height) % depth;

	ivec3 coordinate = ivec3(i, j, k);

	imageStore(u_voxel_albedo, coordinate, vec4(0.0, 0.0, 0.0, 0.0));
	imageStore(u_voxel_normal, coordinate, vec4(0.0, 0.0, 0.0, 0.0));
	imageStore(u_voxel_emission, coordinate, vec4(0.0, 0.0, 0.0, 0.0));
	imageStore(u_voxel_radiance, coordinate, vec4(0.0, 0.0, 0.0, 0.0));
}
