#version 450 core

uniform int u_width;
uniform int u_height;
uniform int u_depth;

in vec3 gw_position;
in flat vec4 g_AABB;
in flat int g_swizzle;

uniform layout(rgba8, binding = 0) image3D u_voxel_diffuse;

void discard_if_outside_aabb(vec4 aabb, int resolution) {
	vec2 aabb_min = floor((aabb.xy * 0.5 + 0.5) * resolution);
	vec2 aabb_max = ceil((aabb.zw * 0.5 + 0.5) * resolution);

	if (!(all(greaterThanEqual(gl_FragCoord.xy, aabb_min)) &&
				all(lessThanEqual(gl_FragCoord.xy, aabb_max)))) {
		discard;
	}
}

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

	vec3 color = vec3(position) / vec3(u_width, u_height, u_depth);
	imageStore(u_voxel_diffuse, position, vec4(color, 1.0));
}
