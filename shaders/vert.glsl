#version 400
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout(location = 0) in vec3 pos;
layout(location = 1) in vec3 normal;

layout(location = 0) out vec3 o_normal;

layout(binding = 0) uniform UBO {
    mat4 model;
    mat4 view;
    mat4 projection;
} ubo;

void main() {
    o_normal = normal;

    vec4 worldPos = ubo.model * vec4(pos, 1.0);
    vec4 viewPos = ubo.view * worldPos;
    gl_Position = ubo.projection * viewPos;

}