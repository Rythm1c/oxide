#version 450

layout(location = 0) in vec3 pos;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 uv;
layout(location = 3) in vec3 color;

layout(push_constant) uniform PushConstants {
    mat4 proj_view;
    mat4 model;
} pc;

void main() {

    gl_position = pc.proj_view * pc.model * vec4(pos, 1.0);

}