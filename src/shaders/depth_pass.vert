#version 330

uniform mat4 light_matrix;
// uniform mat4 model;

layout (location = 0) in vec3 aPosition;

void main() {
	gl_Position = light_matrix * vec4(aPosition, 1.0);
}