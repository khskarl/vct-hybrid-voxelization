#version 450

layout(location = 0) in vec3 pos;
layout(location = 1) in vec4 color;
layout(location = 0) out vec4 frag_color;

layout(set = 0, binding = 0) uniform Args {
    float my_uniform;
};

void main() {
    frag_color = vec4(color.x, color.y, my_uniform, 1.0);
    gl_Position = vec4(pos, 1.0);
}
