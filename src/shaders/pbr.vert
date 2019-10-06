#version 330

uniform mat4 pv;
uniform mat4 light_matrix;
uniform mat4 model;

layout (location = 0) in vec3 aPosition;
layout (location = 1) in vec2 aTexCoord;
layout (location = 2) in vec3 aNormal;

out vec3 vw_position;
out vec2 v_uv;
out vec3 v_normal;
out vec4 vl_position;

void main() {
	vec3 w_position = vec3(model * vec4(aPosition, 1.0));
	vw_position = w_position;
	vl_position = light_matrix * vec4(w_position, 1.0);
	v_uv = aTexCoord;
	v_normal = transpose(inverse(mat3(model))) * aNormal;

	gl_Position = pv * model * vec4(aPosition, 1.0);
}
