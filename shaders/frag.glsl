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
    float metallic;    // offset  0
    float roughness;   // offset  4
    float ao;          // offset  8
    float _pad0;       // offset 12

    float useChecker;  // offset 16
    float divisions;   // offset 20
    float factor;      // offset 24
    float _pad1;       // offset 28
} material;

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

float distributionGGX(vec3 N, vec3 H, float roughness) {
    float a     = roughness * roughness;
    float a2    = a * a;
    float NdotH = max(dot(N, H), 0.0);
    float denom = NdotH * NdotH * (a2 - 1.0) + 1.0;
    return a2 / (3.14159265 * denom * denom);
}

float geometrySchlickGGX(float NdotX, float roughness) {
    float k = (roughness + 1.0);
    k = (k * k) / 8.0;
    return NdotX / (NdotX * (1.0 - k) + k);
}

float geometrySmith(float NdotL, float NdotV, float roughness) {
    return geometrySchlickGGX(NdotL, roughness)
         * geometrySchlickGGX(NdotV, roughness);
}

vec3 fresnelSchlick(float cosTheta, vec3 F0) {
    return F0 + (1.0 - F0) * pow(clamp(1.0 - cosTheta, 0.0, 1.0), 5.0);
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

void main() {
    // Apply checker pattern if enabled
    float check  = (material.useChecker > 0.5) ? checker(o_uv) : 1.0;
    vec3  albedo = o_color * check;

    // Vectors
    vec3 N = normalize(o_normal);
    vec3 V = normalize(lightUBO.cameraPos.xyz - fragWorldPos);
    vec3 L = normalize(-lightUBO.light_dir.xyz);  // negate: toward light
    vec3 H = normalize(L + V);

    float NdotL = max(dot(N, L), 0.0);
    float NdotV = max(dot(N, V), 0.0);

    // F0: base reflectivity
    // Dielectrics: 0.04 (most plastics/stone)
    // Metals: use albedo (metals have coloured specular)
    vec3 F0 = mix(vec3(0.04), albedo, material.metallic);

    // Cook-Torrance specular BRDF
    float D = distributionGGX(N, H, material.roughness);
    float G = geometrySmith(NdotL, NdotV, material.roughness);
    vec3  F = fresnelSchlick(max(dot(H, V), 0.0), F0);

    vec3 specular = (D * G * F) / max(4.0 * NdotL * NdotV, 0.001);

    // Energy conservation
    vec3 ks = F;
    vec3 kd = (vec3(1.0) - ks) * (1.0 - material.metallic);

    // Lambertian diffuse (divided by PI for energy conservation)
    vec3 diffuse = kd * albedo / 3.14159265;

    // Incoming radiance: color × intensity
    vec3 Li = lightUBO.light_color.xyz * lightUBO.light_color.w;

    // Direct lighting
    vec3 Lo = (diffuse + specular) * Li * NdotL;

    // Ambient (approximation of indirect light, scaled by AO)
    vec3 ambient = lightUBO.ambient.xyz * albedo * material.ao;

    vec3 color = ambient + Lo;

    // Reinhard tone mapping — compresses HDR to [0,1]
    // Without this, bright specular highlights blow out to pure white
   // color = color / (color + vec3(1.0));

    // Gamma correction — convert linear to sRGB for display
    // Without this, colours look washed out and too bright
    //color = pow(color, vec3(1.0 / 2.2));

    uFragColor = vec4(color, 1.0);
}
