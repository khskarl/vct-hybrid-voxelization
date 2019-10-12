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


void discard_if_outside_aabb(vec4 aabb, int resolution) {
	vec2 aabb_min = floor((aabb.xy * 0.5 + 0.5) * resolution);
	vec2 aabb_max = ceil((aabb.zw * 0.5 + 0.5) * resolution);

	if (!(all(greaterThanEqual(gl_FragCoord.xy, aabb_min)) &&
				all(lessThanEqual(gl_FragCoord.xy, aabb_max)))) {
		discard;
	}
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
