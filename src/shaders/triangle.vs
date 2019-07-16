#version 450

layout(location = 0) in vec3 pos;
layout(location = 1) in vec4 color;

layout(location = 0) out vec4 frag_color;

layout(std140, set = 0, binding = 0) uniform Args {
    mat4 proj;
    mat4 view;
    float time;
};

void main() {
    frag_color = vec4(color.x, color.y, color.z, 1.0);
    gl_Position = (proj * view) * vec4(pos, 1.0);
}
