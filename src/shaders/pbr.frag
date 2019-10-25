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

uniform layout(binding = 0) sampler2D albedo_map;
uniform layout(binding = 1) sampler2D metaghness_map;
uniform layout(binding = 2) sampler2D normal_map;
uniform layout(binding = 3) sampler2D occlusion_map;
uniform layout(binding = 4) sampler3D u_radiance;

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

const vec3 CONES[] =
{
	vec3(0.57735, 0.57735, 0.57735),
	vec3(0.57735, -0.57735, -0.57735),
	vec3(-0.57735, 0.57735, -0.57735),
	vec3(-0.57735, -0.57735, 0.57735),
	vec3(-0.903007, -0.182696, -0.388844),
	vec3(-0.903007, 0.182696, 0.388844),
	vec3(0.903007, -0.182696, 0.388844),
	vec3(0.903007, 0.182696, -0.388844),
	vec3(-0.388844, -0.903007, -0.182696),
};

vec3 radiance_coordinate(vec3 w_position) {
	vec3 volume_corner = u_volume_center - u_volume_scale * 0.505;
	return (((w_position - volume_corner) / (u_volume_scale)));
}

vec4 ConeTrace(sampler3D voxels, vec3 P,vec3 N, vec3 direction, float aperture) {
	P = radiance_coordinate(P);
	const float voxel_size = 1.0 / float(u_width);
	vec3 origin = P + N * voxel_size * 2.0 * 1.414213;

	const float maxDistance = 3.0;
	vec3 color = vec3(0.0);
	float alpha = 0.0;
	float t = voxel_size;
	while (t < maxDistance && alpha < 1.0) {
		float diameter = max(voxel_size, 2.0 * aperture * t);
		float mip = log2(diameter * voxel_size * 500.0);

		vec3 tc = origin + direction * t;
		vec4 radiance = textureLod(voxels, tc, min(mip, 6.0));

		float a = 1 - alpha;
		color += a * radiance.rgb;
		alpha += a * radiance.a;

		t += diameter * 0.5;
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

	vec3 direct = vec3(0.0);
	for(int i = 0; i < num_lights; i++) {
		vec3 Li = vw_position.xyz - light_position[i];
		float dist = length(Li);
		float attenuation = 0.05 * dist * dist;

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

	vec4 radiance = vec4(0.0);
	for(int i = 0; i < 9; i++) {
		vec3 cone_dir = normalize(CONES[i] + normal);
		cone_dir *= dot(cone_dir, normal) < 0 ? -1 : 1;
		radiance += ConeTrace(u_radiance, vw_position, normal, cone_dir, tan(PI * 0.5 * 0.33));
	}
	radiance /= 9.0;
	vec3 ambient_radiance = radiance.rgb + vec3(0.1, 0.07, 0.05) * 0.0002;
	vec3 ambient = albedo * ambient_radiance * occlusion;
	vec3 color = (direct + ambient * 2.0);
	out_color = vec4(color, 1.0);
}
