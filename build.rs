use std::process::Command;

fn main() {
    // compile shaders
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let shaders = [
        ("shaders/vert.glsl", "vert.spv", "vert"),
        ("shaders/frag.glsl", "frag.spv", "frag"),
    ];

    for (src_path, out_name, stage) in shaders.iter() {
        let out_path = format!("{}/{}", out_dir, out_name);
        let status = Command::new("glslc")
            .arg(format!("-fshader-stage={}", stage))
            .arg(src_path)
            .arg("-o")
            .arg(&out_path)
            .status()
            .expect("Failed to run glslc");
        if !status.success() {
            panic!("glslc failed to compile {}", src_path);
        }
        println!("cargo:rerun-if-changed={}", src_path);
    }
}
