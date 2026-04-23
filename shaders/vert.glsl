#version 400
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout(location = 0) in vec3 pos;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 uv;
layout(location = 3) in vec3 color;


layout(location = 0) out vec3 o_normal;
layout(location = 1) out vec2 o_uv;
layout(location = 2) out vec3 o_color;
layout(location = 3) out vec3 view_dir;

layout(set = 0, binding = 0) uniform CameraUBO {
    vec4 view_dir;
    mat4 view;
    mat4 projection;
} camUBO;

layout(push_constant) uniform PushConstants {
    mat4  model;
} pc;

void main() {
    o_normal = normalize(mat3(transpose(inverse(pc.model))) * normal);
    o_uv = uv;
    o_color = color;
    view_dir = camUBO.view_dir.xyz;

    vec4 worldPos = pc.model * vec4(pos, 1.0);
    vec4 viewPos = camUBO.view * worldPos;
    gl_Position = camUBO.projection * viewPos;

}