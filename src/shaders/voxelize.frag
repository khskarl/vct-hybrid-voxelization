#version 450 core

#include <shared.glsl>

uniform int u_width;
uniform int u_height;
uniform int u_depth;

in vec3 gw_position;
in vec3 gw_normal;
in vec2 g_uv;
in flat vec4 g_AABB;
in flat int g_swizzle;

layout(binding = 0, r32ui) uniform volatile coherent restrict uimage3D u_voxel_albedo;
layout(binding = 1, r32ui) uniform volatile coherent restrict uimage3D u_voxel_normal;
layout(binding = 2, r32ui) uniform volatile coherent restrict uimage3D u_voxel_emission;

layout(binding = 0) uniform sampler2D albedo_map;

void main() {
	if(is_outside_aabb(g_AABB, gl_FragCoord.xy, u_width)) {
		discard;
	}

	mat3 swizzle_matrix_inverse = inverse(swizzleLUT[g_swizzle]);

	// Voxel position
	ivec3 position = ivec3(swizzle_matrix_inverse * vec3(gl_FragCoord.xy, gl_FragCoord.z * u_depth));

	vec3 albedo = texture(albedo_map, g_uv).rgb;
	vec3 normal = encode_normal(gw_normal);

	image_average_rgba8(u_voxel_albedo, position, albedo);
	image_average_rgba8(u_voxel_normal, position, normal);
	vec3 emission = vec3(1.0) - normal.yyy;
	if(emission.y < 0.9) {
		emission.rgb = vec3(0.0);
	}

	image_average_rgba8(u_voxel_emission, position, emission * (vec3(position) / u_width));
}
