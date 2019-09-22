#version 450
const float M_PI = 3.1415926535897932384626433832795;

vec3 light_dir[3] = vec3[](
	vec3(0.1, 1.0, 0.3),
	vec3(0.2, 1.0, -0.4),
	vec3(-0.4, 1.0, 0.1)
);

vec3 light_color[3] = vec3[](
	vec3(0.5, 0.6, 0.7),
	vec3(0.1, 0.2, 0.4),
	vec3(0.1, 0.3, 0.3)
);

uniform mat4 proj;
uniform mat4 view;
uniform float time;

uniform sampler2D albedo;
uniform sampler2D metaghness;
uniform sampler2D normal;
uniform sampler2D occlusion;

in vec2 v_uv;
in vec3 v_normal;

out vec4 out_color;

float distribution_ggx(vec3 N, vec3 H, float a) {
	float a2 = pow(a, 2.0);
	float NdotH2 = pow(max(dot(N, H), 0.0), 2.0);

	float denom = pow(NdotH2 * (a2 - 1) + 1, 2.0) * M_PI;

	return a2 / denom;
}

void main() {
	vec2 uv = vec2(v_uv.x + sin(time) * 0.001, v_uv.y);

	vec3 diffuse = texture2D(albedo, uv).xyz;
	float roughness = texture2D(metaghness, uv).g;
	vec3 my_normal = texture2D(normal, uv).rgb;
	float my_occlusion = texture2D(occlusion, uv).r;

	vec3 direct = vec3(0.0);
	for(int i = 0; i < 3; i++) {
		direct += diffuse * light_color[i] * dot(normalize(light_dir[i]), v_normal);
	}

	vec3 ambient = diffuse * vec3(0.1, roughness * 0.1, my_occlusion * 0.1) * my_occlusion + my_normal * 0.01;

	vec3 color = direct + ambient;
	out_color = vec4(color, 1.0);
}