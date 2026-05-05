use std::path::Path;
use std::process::Command;

fn main() {
    let shaders = [
        ("shaders/vert.glsl", "shaders/vert.spv", "vert"),
        ("shaders/frag.glsl", "shaders/frag.spv", "frag"),
        ("shaders/shadow.glsl", "shaders/shadow.spv", "vert"),
    ];

    // Tell cargo to re-run this script if ANY shader source changes.
    // This must be declared before any early returns or panics.
    for (src, _, _) in &shaders {
        println!("cargo:rerun-if-changed={}", src);
    }
    // Also re-run if the build script itself changes.
    println!("cargo:rerun-if-changed=build.rs");

    // Ensure the output directory exists.
    std::fs::create_dir_all("shaders").expect("Failed to create shaders directory");

    for (src_path, out_path, stage) in &shaders {
        // Skip if source doesn't exist yet (e.g. first checkout before files are created).
        if !Path::new(src_path).exists() {
            eprintln!("cargo:warning=Shader source not found, skipping: {}", src_path);
            continue;
        }

        let status = Command::new("glslc")
            .arg(format!("-fshader-stage={}", stage))
            .arg(src_path)
            .arg("-o")
            .arg(out_path)
            .status()
            .unwrap_or_else(|e| panic!("Failed to run glslc (is it installed?): {}", e));

        if !status.success() {
            panic!("glslc failed to compile {}", src_path);
        }

        println!("cargo:warning=Compiled {} -> {}", src_path, out_path);
    }
}