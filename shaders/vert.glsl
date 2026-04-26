#version 450

layout(location = 0) in vec3 pos;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 uv;
layout(location = 3) in vec3 color;

layout(location = 0) out vec3 o_normal;
layout(location = 1) out vec2 o_uv;
layout(location = 2) out vec3 o_color;
layout(location = 3) out vec3 fragWorldPos;

layout(set = 0, binding = 0) uniform CameraUBO {
    mat4 view;
    mat4 projection;
} camera;

layout(push_constant) uniform PushConstants {
    mat4 model;
} pc;

void main() {
    vec4 worldPos = pc.model * vec4(pos, 1.0);

    o_normal     = normalize(mat3(transpose(inverse(pc.model))) * normal);
    o_uv         = uv;
    o_color      = color;
    fragWorldPos = worldPos.xyz;  

    gl_Position = camera.projection * camera.view * worldPos;
}
