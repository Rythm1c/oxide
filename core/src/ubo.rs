#[repr(C)]
#[derive(Copy, Clone)]
pub struct CameraUbo {
    pub view: [[f32; 4]; 4],
    pub proj: [[f32; 4]; 4],
}

impl Default for CameraUbo {
    fn default() -> Self {
        let id = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        Self {
            view: id,
            proj: id,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct LightUbo {
    pub view_dir   : [f32; 4],
    pub light_dir  : [f32; 4],
    pub light_color: [f32; 4],
}

impl Default for LightUbo {
    fn default() -> Self {
        Self {
            light_dir  : [0.0, -1.0, 0.0, 0.0],
            light_color: [1.0,  1.0, 1.0, 1.0],
            view_dir   : [0.1,  0.1, 0.1, 0.0],
        }
    }
}