#version 450
#define MAX_LIGHTS 4

#include <shared.glsl>

uniform vec3 light_direction[MAX_LIGHTS];
uniform vec3 light_position[MAX_LIGHTS];
uniform vec3 light_color[MAX_LIGHTS];
uniform float light_range[MAX_LIGHTS];

uniform int num_lights;

uniform float time;
uniform vec3 camera_position;

uniform vec3 u_volume_center;
uniform vec3 u_volume_scale;
uniform int u_width;

uniform sampler2D albedo_map;
uniform sampler2D metaghness_map;
uniform sampler2D normal_map;
uniform sampler2D occlusion_map;
uniform sampler2D shadow_map;
uniform layout(binding = 5) sampler3D u_radiance;

in vec3 vw_position;
in vec2 v_uv;
in vec4 vl_position;
in mat3 v_TBN;

out vec4 out_color;

float distribution_ggx(vec3 N, vec3 H, float a) {
	float a2     = pow(a, 2.0);
	float NdotH2 = pow(max(dot(N, H), 0.0), 2.0);

	float denom = pow(NdotH2 * (a2 - 1) + 1, 2.0) * PI;

	return a2 / denom;
}

float geometry_smith_ggx(float NdotV, float k) {
	float nom   = NdotV;
	float denom = NdotV * (1.0 - k) + k;

	return nom / denom;
}

float geometry_smith(vec3 N, vec3 V, vec3 L, float k) {
	float NdotV = max(dot(N, V), 0.0);
	float NdotL = max(dot(N, L), 0.0);
	float ggx1 = geometry_smith_ggx(NdotV, k);
	float ggx2 = geometry_smith_ggx(NdotL, k);

	return ggx1 * ggx2;
}

vec3 fresnelSchlick(float cosTheta, vec3 F0) {
	return F0 + (1.0 - F0) * pow(1.0 - cosTheta, 5.0);
}

const vec3 propagationDirections[] = {
	vec3(0.0, 0.0, 1.0),
	vec3(0.0, 0.866025, 0.5),
	vec3(0.754996, -0.4330128, 0.5),
	vec3(-0.754996, -0.4330128, 0.5)
};

vec3 radiance_coordinate(vec3 w_position) {
	vec3 volume_corner = u_volume_center - u_volume_scale * 0.505;
	return (((w_position - volume_corner) / (u_volume_scale)));
}

vec4 ConeTrace(sampler3D voxels, vec3 P,vec3 N, vec3 direction, float aperture) {
	// const float voxel_size = (u_volume_scale.x / float(u_width));
	const float voxel_size = 0.8 + u_width * 0.001;
	const float MAX_DIST = 20.0;
	vec3 offset = N * voxel_size;
	vec3 origin = P + offset;

	const float maxDistance = MAX_DIST * voxel_size;
	vec3 color = vec3(0.0);
	float alpha = 0.0;
	float t = 2.0 * voxel_size;
	while (t < maxDistance && alpha < 1.0) {
		float diameter = max(voxel_size, 2 * aperture * t);
		float mip = log2(diameter / voxel_size);

		vec3 position = origin + direction * t;
		position = radiance_coordinate(position);

		if (mip >= 9)
			break;

		vec4 radiance = texture(voxels, position, mip);

		float a = 1 - alpha;
		color += a * radiance.rgb;
		alpha += a * radiance.a;

		t += diameter * 0.15;
	}

	return vec4(color, alpha);
}

vec3 direct_lighting(vec3 Li, vec3 Lc, vec3 albedo, float roughness, float metalness, vec3 normal, float occlusion, vec3 V, vec3 F0) {
	vec3 L = -normalize(Li);
	vec3 H = normalize(V + L);

	vec3 N = normal;
	float NDF = distribution_ggx(N, H, roughness);
	float G   = geometry_smith(N, V, L, roughness);
	vec3 F    = fresnelSchlick(max(dot(H, V), 0.0), F0);
	vec3 radiance = Lc;

	float NdotL = max(dot(N, L), 0.0);

	vec3  nom   = NDF * G * F;
	float denom = 4 * max(dot(N, V), 0.0) * NdotL + 0.001;
	vec3 specular = nom / denom;

	vec3 kS = F;
	vec3 kD = vec3(1.0) - kS;
	kD *= 1.0 - metalness;

	return (kD * albedo / PI + specular) * radiance * NdotL;
}


void main() {

	vec2 uv = vec2(v_uv.x + sin(time) * 0.001, v_uv.y);

	vec3 albedo = texture(albedo_map, uv).xyz;
	float roughness = texture(metaghness_map, uv).g;
	float metalness = texture(metaghness_map, uv).b;
	vec3 normal = texture(normal_map, uv).rgb;
	normal = normalize(normal * 2.0 - 1.0);
	normal = normalize(v_TBN * normal);

	float occlusion = texture(occlusion_map, uv).r;

	vec3 V = normalize(camera_position - vw_position.xyz);

	vec3 F0 = vec3(0.04);
	F0 = mix(F0, albedo, metalness);

	float shadow = shadow_visilibity_pcf(shadow_map, vl_position);

	vec3 direct = vec3(0.0);
	for(int i = 0; i < min(1, num_lights); i++) {
		vec3 radiance = direct_lighting(
			light_direction[i],
			light_color[i],
			albedo,
			roughness,
			metalness,
			normal,
			occlusion,
			V,
			F0
		);

		direct += radiance * (1.0 - shadow);
	}
	for(int i = 1; i < num_lights; i++) {
		vec3 Li = vw_position.xyz - light_position[i];
		float dist = length(Li);
		float attenuation = 0.1 * dist * dist;

		vec3 radiance = direct_lighting(
			Li,
			light_color[i],
			albedo,
			roughness,
			metalness,
			normal,
			occlusion,
			V,
			F0
		);

		direct += radiance / attenuation;
	}
	vec3 coordinate = radiance_coordinate(vw_position);

	vec4 radiance = vec4(0.0);
	for(int i = 0; i < 4; i++) {
		vec3 cone_dir = normalize(v_TBN * propagationDirections[i]);
		radiance += ConeTrace(u_radiance, vw_position, normal, cone_dir, tan(PI * 0.5 * 0.23));
	}
	radiance /= 4.0;
	// vec3 radiance = texelFetch(u_radiance, coordinate, 0).rgb;
	vec3 ambient_radiance = radiance.rgb + vec3(0.1, 0.07, 0.05) * 0.002;
	vec3 ambient = albedo * ambient_radiance * occlusion;
	vec3 color = (direct + ambient);
	out_color = vec4(color, 1.0);
}
