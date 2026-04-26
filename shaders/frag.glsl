#version 450

layout(location = 0) in vec3 o_normal;
layout(location = 1) in vec2 o_uv;
layout(location = 2) in vec3 o_color;
layout(location = 3) in vec3 view_dir;

layout(location = 0) out vec4 uFragColor;

layout(set = 0, binding = 1) uniform LightUBO {
    vec4 light_dir;
    vec4 light_color;
} lightUBO;

layout(set = 1, binding = 0) uniform MaterialUBO {
    float roughness;   // 0 = mirror, 1 = completely diffuse
    float metallic;    // 0 = dielectric (plastic/stone), 1 = metal
    float ao;          // ambient occlusion (1.0 = no occlusion)

    float _pad0;       // padding
    
    /* checker board pattern configaration*/
    float useChecker;  // 0.0=solid, 1.0=checker
    float divisions;   // number of checher boxes per face
    float factor;      // darkness of the checker boxes

    float _pad1;   

} material;

// checker board pattern generator  
float checker() {
    return 0.0;
}

void main() {

    //directional lighting

    //ambient
    vec3 ambient = vec3(0.01, 0.01, 0.01) * lightUBO.light_color.xyz;

    //diffuse
    vec3 normal   = normalize(o_normal);
    vec3 lightDir = normalize(lightUBO.light_dir.xyz);
    float diff    = max(dot(normal, lightDir), 0.0);
    vec3 diffuse  = lightUBO.light_color.xyz * diff;

    //specular
    vec3 viewDir    = normalize(-view_dir);  // Negate because view_dir points away from camera
    vec3 reflectDir = reflect(-lightDir, normal);
    float spec      = pow(max(dot(viewDir, reflectDir), 0.0), 8.0);
    vec3 specular   = lightUBO.light_color.xyz * spec;

    //final color
    vec3 finalColor = (diffuse + specular) * o_color + ambient;

    uFragColor = vec4(finalColor, 1.0);
}