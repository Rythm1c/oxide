#[derive(Debug, Clone, Copy, Default)]
pub struct Material {
    pub albedo: [f32; 3],
    pub metallic: f32,
    pub roughness: f32,
    pub emissive: [f32; 3],
}
