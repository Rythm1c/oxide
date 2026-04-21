#version 400
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout(location = 0) in vec3 o_normal;

layout(location = 0) out vec4 uFragColor;

layout(binding = 1) uniform FragmentUBO {
    vec3 o_color;
    vec3 view_dir;
    vec3 light_dir;
    vec3 light_color;
} frag_ubo;

void main() {

    //directional lighting

    //ambient
    vec3 ambient = vec3(0.1, 0.1, 0.1) * frag_ubo.light_color;

    //diffuse
    vec3 normal = normalize(o_normal);
    float diff = max(dot(normal, frag_ubo.light_dir), 0.0);
    vec3 diffuse = frag_ubo.light_color * diff;

    //specular
    vec3 viewDir = normalize(frag_ubo.view_dir);
    vec3 reflectDir = reflect(-frag_ubo.light_dir, normal);
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), 32.0);
    vec3 specular = frag_ubo.light_color * spec;

    //final color
    vec3 finalColor = (diffuse + specular) * frag_ubo.o_color + ambient;

    uFragColor = vec4(finalColor, 1.0);
}