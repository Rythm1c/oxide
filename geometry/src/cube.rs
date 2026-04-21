use engine_core::vertex::Vertex;

pub fn cube(size: f32, color: Option<[f32; 3]>) -> (Vec<Vertex>, Vec<u32>) {
    let half_size = size / 2.0;
    //vertices for with normals for each face(+X, -X, +Y, -Y, +Z, -Z)
    // Each face has its own 4 vertices (with normals pointing outwards), so 24 vertices total.
    // Indices define 2 triangles per face (6 faces, 12 triangles, 36 indices).

    let face_colors = [
        [0.0, 0.0, 1.0], // +X face - blue
        [0.0, 0.0, 1.0], // -X face - blue
        [1.0, 1.0, 0.0], // -Y face - yellow
        [1.0, 1.0, 0.0], // -Y face - yellow
        [1.0, 0.0, 0.0], // +Z face - red
        [1.0, 0.0, 0.0], // -Z face - red
    ];

    let positions = [
        // +X face
        ([half_size, -half_size, -half_size], [1.0, 0.0, 0.0]),
        ([half_size, half_size, -half_size], [1.0, 0.0, 0.0]),
        ([half_size, half_size, half_size], [1.0, 0.0, 0.0]),
        ([half_size, -half_size, half_size], [1.0, 0.0, 0.0]),
        // -X face
        ([-half_size, -half_size, half_size], [-1.0, 0.0, 0.0]),
        ([-half_size, half_size, half_size], [-1.0, 0.0, 0.0]),
        ([-half_size, half_size, -half_size], [-1.0, 0.0, 0.0]),
        ([-half_size, -half_size, -half_size], [-1.0, 0.0, 0.0]),
        // +Y face
        ([-half_size, half_size, -half_size], [0.0, 1.0, 0.0]),
        ([-half_size, half_size, half_size], [0.0, 1.0, 0.0]),
        ([half_size, half_size, half_size], [0.0, 1.0, 0.0]),
        ([half_size, half_size, -half_size], [0.0, 1.0, 0.0]),
        // -Y face
        ([-half_size, -half_size, half_size], [0.0, -1.0, 0.0]),
        ([-half_size, -half_size, -half_size], [0.0, -1.0, 0.0]),
        ([half_size, -half_size, -half_size], [0.0, -1.0, 0.0]),
        ([half_size, -half_size, half_size], [0.0, -1.0, 0.0]),
        // +Z face
        ([half_size, -half_size, half_size], [0.0, 0.0, 1.0]),
        ([half_size, half_size, half_size], [0.0, 0.0, 1.0]),
        ([-half_size, half_size, half_size], [0.0, 0.0, 1.0]),
        ([-half_size, -half_size, half_size], [0.0, 0.0, 1.0]),
        // -Z face
        ([-half_size, -half_size, -half_size], [0.0, 0.0, -1.0]),
        ([-half_size, half_size, -half_size], [0.0, 0.0, -1.0]),
        ([half_size, half_size, -half_size], [0.0, 0.0, -1.0]),
        ([half_size, -half_size, -half_size], [0.0, 0.0, -1.0]),
    ];

    // Create vertices with position, normal, uv, and color and generate indices for each face (2 triangles per face)
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    for i in 0..6 {
        let base_index = i * 4;
        let face_color = color.unwrap_or(face_colors[i]);
        vertices.push(Vertex::new(positions[base_index + 0].0, positions[base_index + 0].1, [0.0, 0.0], face_color));
        vertices.push(Vertex::new(positions[base_index + 1].0, positions[base_index + 1].1, [0.0, 1.0], face_color));
        vertices.push(Vertex::new(positions[base_index + 2].0, positions[base_index + 2].1, [1.0, 1.0], face_color));
        vertices.push(Vertex::new(positions[base_index + 3].0, positions[base_index + 3].1, [1.0, 0.0], face_color));

        indices.push(base_index as u32);
        indices.push((base_index + 1) as u32);
        indices.push((base_index + 2) as u32);
        indices.push(base_index as u32);
        indices.push((base_index + 2) as u32);
        indices.push((base_index + 3) as u32);
    }

    (vertices, indices)
}
