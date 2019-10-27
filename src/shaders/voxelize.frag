#version 450 core

#include <shared.glsl>

in vec3 gw_position;
in vec3 gw_normal;
in vec2 g_uv;
in flat vec4 g_AABB;
in flat int g_swizzle;

layout(binding = 0, r32ui) uniform volatile coherent restrict uimage3D u_voxel_albedo;
layout(binding = 1, r32ui) uniform volatile coherent restrict uimage3D u_voxel_normal;
layout(binding = 2, r32ui) uniform volatile coherent restrict uimage3D u_voxel_emission;

layout(binding = 0) uniform sampler2D albedo_map;

layout(location = 0) uniform ivec3 u_resolution;
layout(location = 2) uniform bool u_expand_triangle;

void main() {
	if(u_expand_triangle == true) {
		if(is_outside_aabb(g_AABB, gl_FragCoord.xy, u_resolution.x)) {
			discard;
		}
	}

	mat3 swizzle_matrix_inverse = inverse(swizzleLUT[g_swizzle]);

	// Voxel position
	vec3 pos = vec3(gl_FragCoord.xy, gl_FragCoord.z * u_resolution.z);
	ivec3 position = ivec3(swizzle_matrix_inverse * pos);

	vec3 albedo = texture(albedo_map, g_uv).rgb;
	// vec3 albedo = texture(albedo_map, g_uv).rgb * 0.00001 + vec3(1.0, 0.0, 0.0);
	vec3 normal = encode_normal(gw_normal);
	vec3 emission = vec3(0.0);

	image_average_rgba8(u_voxel_albedo, position, albedo);
	image_average_rgba8(u_voxel_normal, position, normal);
	image_average_rgba8(u_voxel_emission, position, emission);
}
