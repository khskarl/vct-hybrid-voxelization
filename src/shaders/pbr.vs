#version 330

uniform mat4 proj;
uniform mat4 view;

layout (location = 0) in vec3 aPosition;
layout (location = 1) in vec2 aTexCoord;
layout (location = 2) in vec3 aNormal;

out vec2 v_uv;
out vec3 v_normal;

void main() {
	gl_Position = (proj * view) * vec4(aPosition, 1.0);
	v_uv = aTexCoord;
	v_normal = aNormal;
}