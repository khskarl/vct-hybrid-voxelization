const float PI = 3.14159265f;
const float EPSILON = 1e-30;

const mat3 swizzleLUT[] = {
	mat3(0, 0, 1,
			 0, 1, 0,
			 1, 0, 0),
	mat3(1, 0, 0,
			 0, 0, 1,
			 0, 1, 0),
	mat3(1, 0, 0,
			 0, 1, 0,
			 0, 0, 1)
};

vec3 encode_normal(vec3 normal) {
	return normal * 0.5f + vec3(0.5f);
}

vec3 decode_normal(vec3 normal) {
	return normal * 2.0f - vec3(1.0f);
}

bool is_outside_aabb(vec4 aabb, vec2 coord, int resolution) {
	vec2 aabb_min = floor((aabb.xy * 0.5 + 0.5) * resolution);
	vec2 aabb_max = ceil((aabb.zw * 0.5 + 0.5) * resolution);

	return !(all(greaterThanEqual(coord.xy, aabb_min)) &&
					 all(lessThanEqual(coord.xy, aabb_max)));
}

vec4 rgba8_to_vec4(uint val) {
	return vec4 (
		float((val & 0x000000FF)),
		float((val & 0x0000FF00) >>  8U),
		float((val & 0x00FF0000) >> 16U),
		float((val & 0xFF000000) >> 24U));
}

uint vec4_to_rgba8( vec4 val) {
	return
		(uint(val.w) & 0x000000FF) << 24U |
		(uint(val.z) & 0x000000FF) << 16U |
		(uint(val.y) & 0x000000FF) <<  8U |
		(uint(val.x) & 0x000000FF);
}

void image_average_rgba8(layout(r32ui) volatile coherent restrict uimage3D img, ivec3 coords, vec3 value) {
	uint next_val = packUnorm4x8(vec4(value, 1.0f / 255.0f));
	uint prev_val = 0;
	uint curr_val;

	while((curr_val = imageAtomicCompSwap(img, coords, prev_val, next_val)) != prev_val) {
		prev_val = curr_val;
		vec4 currVec4 = unpackUnorm4x8(curr_val);

		vec3 average = currVec4.rgb;
		uint count = uint(currVec4.a * 255.0f);

		average = (average * count + value) / (count + 1);

		next_val = packUnorm4x8(vec4(average, (count + 1) / 255.0f));
	}
}

float shadow_visilibity(sampler2D shadow_map, vec4 light_pos) {
	vec3 proj_coords = light_pos.xyz / light_pos.w;
	proj_coords = proj_coords * 0.5 + 0.5;

	float bias = 0.00001;
	float current_depth = proj_coords.z;

	float shadow = 0.0;
	float pcf_depth = texture(shadow_map, proj_coords.xy).r;
	shadow = current_depth - bias > pcf_depth ? 1.0 : 0.0;

	return shadow /= (13.0 * 13.0);
}

float shadow_visilibity_pcf(sampler2D shadow_map, vec4 light_pos) {
	vec3 proj_coords = light_pos.xyz / light_pos.w;
	proj_coords = proj_coords * 0.5 + 0.5;

	float bias = 0.00001;
	float current_depth = proj_coords.z;

	float shadow = 0.0;
	vec2 texel_size = 1.0 / textureSize(shadow_map, 0);
	for(int x = -6; x <= 6; ++x) {
		for(int y = -6; y <= 6; ++y) {
			float pcf_depth = texture(shadow_map, proj_coords.xy + vec2(x, y) * texel_size).r;
			shadow += current_depth - bias > pcf_depth ? 1.0 : 0.0;
		}
	}

	return shadow /= (13.0 * 13.0);
}

vec4[3] enlarge_triangle(vec4 s_position[3], ivec3 resolution) {
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

	float pixel_diagonal = 1.4142135637309 / float(resolution.x);
	s_position[0].xy = s_position[0].xy - pixel_diagonal * (edge[2] / dot(edge[2], edge_normal[0]) +
																																		edge[0] / dot(edge[0], edge_normal[2]));
	s_position[1].xy = s_position[1].xy - pixel_diagonal * (edge[0] / dot(edge[0], edge_normal[1]) +
																																		edge[1] / dot(edge[1], edge_normal[0]));
	s_position[2].xy = s_position[2].xy - pixel_diagonal * (edge[1] / dot(edge[1], edge_normal[2]) +
																																		edge[2] / dot(edge[2], edge_normal[1]));

	return s_position;
}