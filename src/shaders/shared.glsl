vec3 my_color () {
  return vec3(0.0, 1.0, 0.0);
}

void discard_if_outside_aabb(vec4 aabb, int resolution) {
	vec2 aabb_min = floor((aabb.xy * 0.5 + 0.5) * resolution);
	vec2 aabb_max = ceil((aabb.zw * 0.5 + 0.5) * resolution);

	if (!(all(greaterThanEqual(gl_FragCoord.xy, aabb_min)) &&
				all(lessThanEqual(gl_FragCoord.xy, aabb_max)))) {
		discard;
	}
}
