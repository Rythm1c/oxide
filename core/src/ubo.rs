#[repr(C)]
#[derive(Copy, Clone)]
pub struct MaterialUbo{
    pub metallic   : f32,
    pub roughness  : f32,
    pub ao         : f32,

    pub _pad0: f32,
    // checker board stuff
    pub use_checker: f32,  // 0.0=solid, 1.0=checker
    pub divisions  : f32,   // number of checher boxes per face
    pub factor     : f32,    // darkness of the checker boxes(0.0 - 1.0)

    pub _pad1 :f32
}

impl Default for MaterialUbo {
    fn default() -> Self {
        Self { 
            metallic   :0.5,
            roughness  :0.5,
            ao         :0.05,
            _pad0      :0.0,
            use_checker:0.0,
            divisions  :0.0,
            factor     :1.0,
            _pad1      :0.0
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CameraUbo {
    pub view    : [[f32; 4]; 4],
    pub proj    : [[f32; 4]; 4],
    pub view_dir: [f32; 4],
}

impl Default for CameraUbo {
    fn default() -> Self {
        let id = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        let view_dir = [0.1, 0.1, 0.1, 0.0];
        Self {
            view_dir,
            view: id,
            proj: id,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct LightUbo {
    pub light_dir  : [f32; 4],
    pub light_color: [f32; 4],
}

impl Default for LightUbo {
    fn default() -> Self {
        Self {
            light_dir  : [0.0, -1.0, 0.0, 0.0],
            light_color: [1.0,  1.0, 1.0, 1.0],
        }
    }
}