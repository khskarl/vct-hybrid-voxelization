#version 450 core

#include <shared.glsl>

uniform int u_width;
uniform int u_height;
uniform int u_depth;

in vec3 gw_position;
in vec2 g_uv;
in flat vec4 g_AABB;
in flat int g_swizzle;

uniform layout(rgba8, binding = 0) image3D u_voxel_diffuse;

layout ( binding = 0, offset = 0 ) uniform atomic_uint u_voxel_frag_count;

// void image_average_rgba8(volatile coherent restrict uimage3D grid, ivec3 coords, vec3 value) {
// 	uint nextUint = packUnorm4x8(vec4(value, 1.0f / 255.0f));
// 	uint prevUint = 0;
// 	uint currUint;

// 	vec4 currVec4;

// 	vec3 average;
// 	uint count;

// 	//"Spin" while threads are trying to change the voxel
// 	while ((currUint = imageAtomicCompSwap(grid, coords, prevUint, nextUint)) != prevUint) {
// 		prevUint = currUint;                 // store packed rgb average and count
// 		currVec4 = unpackUnorm4x8(currUint); // unpack stored rgb average and count

// 		average = currVec4.rgb;            // extract rgb average
// 		count = uint(currVec4.a * 255.0f); // extract count

// 		// Compute the running average
// 		average = (average * count + value) / (count + 1);

// 		// Pack new average and incremented count back into a uint
// 		nextUint = packUnorm4x8(vec4(average, (count + 1) / 255.0f));
// 	}
// }


void main() {
	discard_if_outside_aabb(g_AABB, u_width);

	mat3 swizzle_matrix = mat3(1.0);
	if (g_swizzle == 0) {
		swizzle_matrix = mat3(vec3(0.0, 0.0, 1.0),
													vec3(0.0, 1.0, 0.0),
													vec3(1.0, 0.0, 0.0));
	} else if (g_swizzle == 1) {
		swizzle_matrix = mat3(vec3(1.0, 0.0, 0.0),
													vec3(0.0, 0.0, 1.0),
													vec3(0.0, 1.0, 0.0));
	} else {
		swizzle_matrix = mat3(vec3(1.0, 0.0, 0.0),
													vec3(0.0, 1.0, 0.0),
													vec3(0.0, 0.0, 1.0));
	}
	mat3 swizzle_matrix_inverse = inverse(swizzle_matrix);

	// Voxel position
	ivec3 position = ivec3(swizzle_matrix_inverse * vec3(gl_FragCoord.xy, gl_FragCoord.z * u_depth));

	// vec3 color = vec3(position) / vec3(u_width, u_height, u_depth);
	vec3 color = vec3(g_uv, 1.0);
	imageStore(u_voxel_diffuse, position, vec4(color, 1.0));
}
