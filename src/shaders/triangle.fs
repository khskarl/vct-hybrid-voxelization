#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(early_fragment_tests) in;

layout(location = 0) in vec4 frag_color;
layout(location = 0) out vec4 color;

layout(set = 0, binding = 0) uniform Args {
    float my_uniform;
};

void main() {
    color = frag_color;
}
