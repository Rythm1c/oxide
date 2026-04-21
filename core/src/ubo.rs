pub struct CameraUBO {
    pub view: [[f32; 4]; 4],
    pub proj: [[f32; 4]; 4],
}

pub struct LightUBO {
    pub view_dir: [f32; 4],
    pub light_dir: [f32; 4],
    pub light_color: [f32; 4],
}
