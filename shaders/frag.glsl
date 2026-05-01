#version 450

layout(location = 0) in vec3 o_normal;
layout(location = 1) in vec2 o_uv;
layout(location = 2) in vec3 o_color;
layout(location = 3) in vec3 fragWorldPos;

layout(location = 0) out vec4 uFragColor;

// set = 0, binding = 1 — matches LightUbo Rust struct field order
layout(set = 0, binding = 1) uniform LightUBO {
    vec4 cameraPos;    // offset  0 — xyz = camera world position
    vec4 ambient;      // offset 16 — xyz = ambient color
    vec4 light_dir;    // offset 32 — xyz = light direction (toward scene)
    vec4 light_color;  // offset 48 — xyz = color, w = intensity
} lightUBO;

// set = 1, binding = 0 — matches MaterialUbo Rust struct field order:
//   roughness, metallic, ao, _pad0, useChecker, divisions, factor, _pad1
layout(set = 1, binding = 0) uniform MaterialUBO {
    float roughness;   // offset  0
    float metallic;    // offset  4
    float ao;          // offset  8
    float _pad0;       // offset 12

    float useChecker;  // offset 16
    float divisions;   // offset 20
    float factor;      // offset 24
    float _pad1;       // offset 28
} material;

const float PI = 3.14159265359;

// ---------------------------------------------------------------------------
// Checker board pattern
// Returns a blend factor: 1.0 for the primary color, `factor` for the dark square.
// ---------------------------------------------------------------------------
float checker(vec2 uv) {
    // Scale UV by divisions to control box count per face
    vec2 scaled = uv * material.divisions;
    // floor + mod gives alternating 0/1 pattern
    float cx = mod(floor(scaled.x), 2.0);
    float cy = mod(floor(scaled.y), 2.0);
    // XOR: same parity = white square, different = dark square
    float isBlack = abs(cx - cy);
    // Blend between 1.0 (full color) and factor (darkened)
    return mix(1.0, material.factor, isBlack);
}

// ---------------------------------------------------------------------------
// PBR helper functions
// ---------------------------------------------------------------------------

float DistributionGGX(vec3 N, vec3 H, float roughness) {
    float a      = roughness * roughness;
    float a2     = a * a;
    float NdotH  = max(dot(N, H), 0.0);
    float NdotH2 = NdotH * NdotH;

    float nom = a2;
    float denom = (NdotH2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;

    return nom / denom;
}

float GeometrySchlickGGX(float NdotV, float roughness) {
    float r = (roughness + 1.0);
    float k = (r * r) / 8.0;

    float nom = NdotV;
    float denom = NdotV * (1.0 - k) + k;

    return nom / denom;
}

float GeometrySmith(vec3 N, vec3 V, vec3 L, float roughness) {
    float NdotV = max(dot(N, V), 0.0);
    float NdotL = max(dot(N, L), 0.0);
    float ggx2  = GeometrySchlickGGX(NdotV, roughness);
    float ggx1  = GeometrySchlickGGX(NdotL, roughness);

    return ggx1 * ggx2;
}

vec3 fresnelSchlick(float cosTheta, vec3 F0) {
    return F0 + (1.0 - F0) * pow(clamp(1.0 - cosTheta, 0.0, 1.0), 5.0);
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

void main() {
    // Apply checker pattern if enabled
    float check = (material.useChecker > 0.5) ? checker(o_uv) : 1.0;
    vec3 albedo = o_color * check;
    //vec3 albedo = vec3(0.0, 1.0, 1.0) * check;

    vec3 N = normalize(o_normal);
    vec3 V = normalize(lightUBO.cameraPos.xyz - fragWorldPos);

    vec3 F0 = vec3(0.04);
    F0 = mix(F0, albedo, material.metallic);

    // reflectance equation
    vec3 Lo = vec3(0.0);

        // calculate per-light radiance
    vec3 L = normalize(-lightUBO.light_dir.xyz);
    vec3 H = normalize(V + L);
    //float distance = length(lightPositions[i] - WorldPos);
    //float attenuation = 1.0 / (distance * distance);
    vec3 radiance = lightUBO.light_color.xyz;// * attenuation;

        // Cook-Torrance BRDF
    float NDF = DistributionGGX(N, H, material.roughness);
    float G   = GeometrySmith(N, V, L, material.roughness);
    vec3 F    = fresnelSchlick(clamp(dot(H, V), 0.0, 1.0), F0);

    vec3 numerator = NDF * G * F;
    float denominator = 4.0 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0) + 0.0001; // + 0.0001 to prevent divide by zero
    vec3 specular = numerator / denominator;

        // kS is equal to Fresnel
    vec3 kS = F;
    
    vec3 kD = vec3(1.0) - kS;

    kD *= 1.0 - material.metallic;	  

        // scale light by NdotL
    float NdotL = max(dot(N, L), 0.0);        

        // add to outgoing radiance Lo
    Lo += (kD * albedo / PI + specular) * radiance * NdotL;  

    // ambient lighting 
    vec3 ambient = vec3(0.03) * albedo * material.ao;

    vec3 color = ambient + Lo;

    // HDR tonemapping
    color = color / (color + vec3(1.0));
    // gamma correct
    //color = pow(color, vec3(1.0 / 2.2));

    uFragColor = vec4(color, 1.0);
}
