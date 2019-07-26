#version 450

layout(early_fragment_tests) in;

layout(location = 0) in vec4 frag_color;

layout(location = 0) out vec4 color;

layout(std140, set = 0, binding = 0) uniform Args {
  float time;
  mat4 proj;
  mat4 view;
};

void main() {
  color = frag_color;
}
