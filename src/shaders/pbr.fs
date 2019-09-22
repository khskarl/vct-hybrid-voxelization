#version 330

vec3 light_dir = vec3(0.3, 1.0, 0.4);

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

void main() {
	vec2 uv = vec2(v_uv.x + sin(time) * 0.001, v_uv.y);

	vec3 diffuse = texture2D(albedo, uv).xyz;
	float roughness = texture2D(metaghness, uv).g;
	vec3 my_normal = texture2D(normal, uv).rgb;
	float my_occlusion = texture2D(occlusion, uv).r;

	vec3 color = vec3(0.0, 0.0, 0.0);
	color += diffuse * vec3(0.8, 0.8, 0.7) * dot(normalize(light_dir), v_normal);
	color += diffuse * vec3(0.3, roughness, my_occlusion) + my_normal;
	out_color = vec4(color, 1.0);
}