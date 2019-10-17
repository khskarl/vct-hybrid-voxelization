#version 450 core

vec3 positions[24] = vec3[](
  // (0, 0, 0) to neighbors
  vec3(0.0, 0.0, 0.0),
  vec3(1.0, 0.0, 0.0),
  vec3(0.0, 0.0, 0.0),
  vec3(0.0, 0.0, 1.0),
  vec3(0.0, 0.0, 0.0),
  vec3(0.0, 1.0, 0.0),
  // (1, 1, 1) to neighbors
  vec3(1.0, 1.0, 1.0),
  vec3(0.0, 1.0, 1.0),
  vec3(1.0, 1.0, 1.0),
  vec3(1.0, 0.0, 1.0),
  vec3(1.0, 1.0, 1.0),
  vec3(1.0, 1.0, 0.0),
  // (1, 0, 1)
  vec3(1.0, 0.0, 1.0),
  vec3(1.0, 0.0, 0.0),
  vec3(1.0, 0.0, 1.0),
  vec3(0.0, 0.0, 0.0),
  // (1, 1, 0)
  vec3(1.0, 1.0, 0.0),
  vec3(1.0, 0.0, 0.0),
  vec3(1.0, 1.0, 0.0),
  vec3(0.0, 1.0, 0.0),
  // (0, 1, 1)
  vec3(0.0, 1.0, 1.0),
  vec3(0.0, 1.0, 0.0),
  vec3(0.0, 1.0, 1.0),
  vec3(0.0, 0.0, 1.0)
);

uniform mat4 mvp;

void main() {
  gl_Position = mvp * vec4(positions[gl_VertexID], 1.0);
}