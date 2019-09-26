#version 330

uniform mat4 proj;
uniform mat4 view;

layout (location = 0) in vec3 aPosition;

out vec3 vw_position;
out vec2 v_uv;
out vec3 v_normal;
out vec4 vl_position;

void main() {
	vw_position = aPosition;
	vl_position = light_matrix * vec4(vw_position, 1.0);
	v_uv = aTexCoord;
	v_normal = aNormal;

	gl_Position = (proj * view) * vec4(aPosition, 1.0);
}
