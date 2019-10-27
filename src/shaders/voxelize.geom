#version 450 core

#include <shared.glsl>

in VSOUT {
	vec3 w_position;
	vec3 w_normal;
	vec2 uv;
} v_in[];

layout(location = 0) uniform ivec3 u_resolution;
layout(location = 1) uniform mat4 pv;
layout(location = 2) uniform bool u_expand_triangle;

out vec3 gw_position;
out vec3 gw_normal;
out vec2 g_uv;
out flat vec4 g_AABB;
out flat int g_swizzle;

layout (triangles) in;
layout (triangle_strip, max_vertices = 3) out;

void main() {
	vec3 e1 = normalize(v_in[1].w_position - v_in[0].w_position);
	vec3 e2 = normalize(v_in[2].w_position - v_in[0].w_position);
	vec3 normal = abs(cross(e1, e2));

	float dominant_axis = max(normal.x, max(normal.y, normal.z));

	int swizzle_axis = 0;
	if (dominant_axis == normal.x) {
		swizzle_axis = 0;
	} else if (dominant_axis == normal.y) {
		swizzle_axis = 1;
	} else {
		swizzle_axis = 2;
	}

	mat3 swizzle_matrix = swizzleLUT[swizzle_axis];

	vec4 s_position[3]= {
		vec4(swizzle_matrix * v_in[0].w_position, 1.0),
		vec4(swizzle_matrix * v_in[1].w_position, 1.0),
		vec4(swizzle_matrix * v_in[2].w_position, 1.0)
	};

	// Calculate clipping region
	float pixel_diagonal = 1.4142135637309 / float(u_resolution.x);
	vec4 AABB = vec4(0.0);
	if (u_expand_triangle == true) {
		AABB.xy = min(s_position[0].xy, min(s_position[1].xy, s_position[2].xy));
		AABB.zw = max(s_position[0].xy, max(s_position[1].xy, s_position[2].xy));
		AABB.xy -= vec2(pixel_diagonal);
		AABB.zw += vec2(pixel_diagonal);
		s_position = enlarge_triangle(s_position, u_resolution);
	}

	for(int i = 0; i < 3; i++) {
		gl_Position = s_position[i];
		gw_position = v_in[i].w_position;
		gw_normal = v_in[i].w_normal;
		g_uv = v_in[i].uv;
		g_AABB = AABB;
		g_swizzle = swizzle_axis;

		EmitVertex();
	}

	EndPrimitive();
}