#version 450

layout(location = 0) in vec2 frag_uv;

layout(location = 0) out vec4 color;

layout(std140, set = 0, binding = 0) uniform Args {
  float time;
  mat4 proj;
  mat4 view;
};

layout(set = 0, binding = 1) uniform texture2D tex;
layout(set = 0, binding = 2) uniform sampler tex_sampler;

void main() {
  color.rgb = texture(sampler2D(tex, tex_sampler), frag_uv).rgb;
  color.a = 1.0;
//   color = vec4(frag_uv, 0.1, 1.0);
}
