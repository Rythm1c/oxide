#version 450

layout(location = 0) in vec3 o_normal;
layout(location = 1) in vec2 o_uv;
layout(location = 2) in vec3 o_color;
layout(location = 3) in vec3 view_dir;

layout(location = 0) out vec4 uFragColor;

layout(binding = 1) uniform LightUBO {
    vec4 light_dir;
    vec4 light_color;
} lightUBO;

void main() {

    //directional lighting

    //ambient
    vec3 ambient = vec3(0.1, 0.1, 0.1) * lightUBO.light_color.xyz;

    //diffuse
    vec3 normal = normalize(o_normal);
    float diff = max(dot(normal, lightUBO.light_dir.xyz), 0.0);
    vec3 diffuse = lightUBO.light_color.xyz * diff;

    //specular
    vec3 viewDir = normalize(view_dir);
    vec3 reflectDir = reflect(-lightUBO.light_dir.xyz, normal);
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), 32.0);
    vec3 specular = lightUBO.light_color.xyz * spec;

    //final color
    vec3 finalColor = (diffuse + specular) * o_color + ambient;

    uFragColor = vec4(finalColor, 1.0);
}