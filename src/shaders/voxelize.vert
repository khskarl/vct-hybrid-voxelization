#version 450 core

layout (location = 0) in vec3 aPosition;
layout (location = 1) in vec2 aTexCoord;
layout (location = 2) in vec3 aNormal;

out VSOUT {
	vec3 w_position;
	vec2 uv;
} v_out;

uniform int u_width;
uniform int u_height;
uniform int u_depth;

uniform mat4 pv;
uniform mat4 model;

void main() {
	vec4 w_position = model * vec4(aPosition, 1.0);
  gl_Position = w_position;

  v_out.w_position = w_position.xyz;
  v_out.uv = aTexCoord;
}
