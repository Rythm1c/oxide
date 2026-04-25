use engine_core::vertex::Vertex;

pub fn generate_uv_sphere(
    radius: f32,
    segments: u32,
    rings: u32,
    color: Option<[f32; 3]>,
) -> (Vec<Vertex>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let vertical_step = std::f32::consts::PI / rings as f32;
    let horizontal_step = 2.0 * std::f32::consts::PI / segments as f32;

    // y = radius * sin(phi)
    // xz_plane_radius = radius * cos(phi)
    // x = xz_plane_radius * cos(theta)
    // z = xz_plane_radius * sin(theta)
    for i in 0..=rings {
        let phi= std::f32::consts::PI / 2.0 - i as f32 * vertical_step;
        //let phi = i as f32 * vertical_step;
        let y = radius * phi.sin();
        let xz_plane_radius = radius * phi.cos();
        for j in 0..=segments {
            let theta = j as f32 * horizontal_step;
            let x = xz_plane_radius * theta.cos();
            let z = xz_plane_radius * theta.sin();

            let position = [x, y, z];
            let normal = [x / radius, y / radius, z / radius];
            let uv = [j as f32 / segments as f32, i as f32 / rings as f32];
            let vertex_color = color.unwrap_or([1.0, 1.0, 1.0]);
            vertices.push(Vertex::new(position, normal, uv, vertex_color));
        }
    }

    for i in 0..rings {
        for j in 0..segments {
            let first = i * (segments + 1) + j;
            let second = first + segments + 1;

            indices.push(first);
            indices.push(first + 1);
            indices.push(second);

            indices.push(second);
            indices.push(first + 1);
            indices.push(second + 1);
        }
    }

    (vertices, indices)
}
