#version 450 core

in VSOUT {
	vec3 w_position;
	vec3 w_normal;
	vec2 uv;
} v_in[];

uniform int u_width;
uniform int u_height;
uniform int u_depth;

uniform mat4 pv;

out vec3 gw_position;
out vec3 gw_normal;
out vec2 g_uv;
out flat vec4 g_AABB;
out flat int g_swizzle;

layout (triangles) in;
layout (triangle_strip, max_vertices = 3) out;

vec4[3] enlarge_triangle(vec4 s_position[3]) {
	vec2 edge[3] = {
		s_position[1].xy - s_position[0].xy,
		s_position[2].xy - s_position[1].xy,
		s_position[0].xy - s_position[2].xy
	};

	vec2 edge_normal[3];
	edge_normal[0] = normalize(edge[0]);
	edge_normal[1] = normalize(edge[1]);
	edge_normal[2] = normalize(edge[2]);
	edge_normal[0] = vec2(-edge_normal[0].y, edge_normal[0].x);
	edge_normal[1] = vec2(-edge_normal[1].y, edge_normal[1].x);
	edge_normal[2] = vec2(-edge_normal[2].y, edge_normal[2].x);

	// Flip back facing triangles, otherwise they will shrink instead of grow
	vec3 a = normalize(s_position[1].xyz - s_position[0].xyz);
	vec3 b = normalize(s_position[2].xyz - s_position[0].xyz);
	vec3 clip_space_normal = cross(a, b);
	if (clip_space_normal.z < 0.0) {
		edge_normal[0] *= -1.0;
		edge_normal[1] *= -1.0;
		edge_normal[2] *= -1.0;
	}

	vec3 edge_distance;
	edge_distance.x = dot(edge_normal[0], s_position[0].xy);
	edge_distance.y = dot(edge_normal[1], s_position[1].xy);
	edge_distance.z = dot(edge_normal[2], s_position[2].xy);

	float pixel_diagonal = 1.4142135637309 / float(u_width);
	s_position[0].xy = s_position[0].xy - pixel_diagonal * (edge[2] / dot(edge[2], edge_normal[0]) +
																																		edge[0] / dot(edge[0], edge_normal[2]));
	s_position[1].xy = s_position[1].xy - pixel_diagonal * (edge[0] / dot(edge[0], edge_normal[1]) +
																																		edge[1] / dot(edge[1], edge_normal[0]));
	s_position[2].xy = s_position[2].xy - pixel_diagonal * (edge[1] / dot(edge[1], edge_normal[2]) +
																																		edge[2] / dot(edge[2], edge_normal[1]));

	return s_position;
}

void main() {
	vec3 e1 = normalize(v_in[1].w_position - v_in[0].w_position);
	vec3 e2 = normalize(v_in[2].w_position - v_in[0].w_position);
	vec3 normal = abs(cross(e1, e2));

	float dominant_axis = max(normal.x, max(normal.y, normal.z));

	int swizzle_axis = 0;
	mat3 swizzle_matrix = mat3(1.0);
	if (dominant_axis == normal.x) {
		swizzle_axis = 0;
		swizzle_matrix = mat3(vec3(0.0, 0.0, 1.0),
													vec3(0.0, 1.0, 0.0),
													vec3(1.0, 0.0, 0.0));
	} else if (dominant_axis == normal.y) {
		swizzle_axis = 1;
		swizzle_matrix = mat3(vec3(1.0, 0.0, 0.0),
													vec3(0.0, 0.0, 1.0),
													vec3(0.0, 1.0, 0.0));
	} else {
		swizzle_axis = 2;
		swizzle_matrix = mat3(vec3(1.0, 0.0, 0.0),
													vec3(0.0, 1.0, 0.0),
													vec3(0.0, 0.0, 1.0));
	}

	vec4 s_position[3]= {
		vec4(swizzle_matrix * v_in[0].w_position, 1.0),
		vec4(swizzle_matrix * v_in[1].w_position, 1.0),
		vec4(swizzle_matrix * v_in[2].w_position, 1.0)
	};

	// Calculate clipping region
	float pixel_diagonal = 1.4142135637309 / float(u_width);
	vec4 AABB;
	AABB.xy = min(s_position[0].xy, min(s_position[1].xy, s_position[2].xy));
	AABB.zw = max(s_position[0].xy, max(s_position[1].xy, s_position[2].xy));
	AABB.xy -= vec2(pixel_diagonal);
	AABB.zw += vec2(pixel_diagonal);

	s_position = enlarge_triangle(s_position);

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