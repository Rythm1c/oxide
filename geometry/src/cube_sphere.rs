use engine_core::vertex::Vertex;

pub fn generate_cube_sphere(
    radius: f32,
    subdivisions: u32,
    color: Option<[f32; 3]>,
) -> (Vec<Vertex>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let face_colors = [
        [0.1, 0.2, 0.9], // +X face - blue
        [0.1, 0.2, 0.9], // -X face - blue
        [0.9, 0.9, 0.1], // +Y face - yellow
        [0.9, 0.9, 0.1], // -Y face - yellow
        [1.0, 0.1, 0.1], // +Z face - red
        [1.0, 0.1, 0.1], // -Z face - red
    ];

    // +X face
    vertices.extend(subdivide_face(
        [0.5, 0.5, 0.5],
        [0.0, 0.0, -1.0],
        [0.0, -1.0, 0.0],
        subdivisions,
        radius,
        color.unwrap_or(face_colors[0]),
    ));
    // -X face
    vertices.extend(subdivide_face(
        [-0.5, 0.5, -0.5],
        [0.0, 0.0, 1.0],
        [0.0, -1.0, 0.0],
        subdivisions,
        radius,
        color.unwrap_or(face_colors[1]),
    ));
    // +Y face
    vertices.extend(subdivide_face(
        [-0.5, 0.5, -0.5],
        [1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0],
        subdivisions,
        radius,
        color.unwrap_or(face_colors[2]),
    ));
    // -Y face
    vertices.extend(subdivide_face(
        [0.5, -0.5, -0.5],
        [-1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0],
        subdivisions,
        radius,
        color.unwrap_or(face_colors[3]),
    ));
    //+Z face
    vertices.extend(subdivide_face(
        [-0.5, 0.5, 0.5],
        [1.0, 0.0, 0.0],
        [0.0, -1.0, 0.0],
        subdivisions,
        radius,
        color.unwrap_or(face_colors[4]),
    ));
    //-Z face
    vertices.extend(subdivide_face(
        [0.5, 0.5, -0.5],
        [-1.0, 0.0, 0.0],
        [0.0, -1.0, 0.0],
        subdivisions,
        radius,
        color.unwrap_or(face_colors[5]),
    ));

    for i in 0..6 {
        let base_index = i * (subdivisions + 1) * (subdivisions + 1);
        for row in 0..subdivisions {
            for col in 0..subdivisions {
                let idx1 = base_index + row * (subdivisions + 1) + col;
                let idx2 = base_index + (row + 1) * (subdivisions + 1) + col;
                let idx3 = base_index + row * (subdivisions + 1) + col + 1;

                let idx4 = base_index + row * (subdivisions + 1) + col + 1;
                let idx5 = base_index + (row + 1) * (subdivisions + 1) + col;
                let idx6 = base_index + (row + 1) * (subdivisions + 1) + col + 1;

                indices.push(idx1 as u32);
                indices.push(idx2 as u32);
                indices.push(idx3 as u32);

                indices.push(idx4 as u32);
                indices.push(idx5 as u32);
                indices.push(idx6 as u32);
            }
        }
    }

    (vertices, indices)
}

fn subdivide_face(
    start: [f32; 3],
    h_sweep: [f32; 3],
    v_sweep: [f32; 3],
    subdivisions: u32,
    radius: f32,
    color: [f32; 3],
) -> Vec<Vertex> {
    let mut vertices = Vec::new();

    for i in 0..=subdivisions {
        let v_factor = i as f32 / subdivisions as f32;
        for j in 0..=subdivisions {
            let h_factor = j as f32 / subdivisions as f32;
            let pos = [
                start[0] + h_sweep[0] * h_factor + v_sweep[0] * v_factor,
                start[1] + h_sweep[1] * h_factor + v_sweep[1] * v_factor,
                start[2] + h_sweep[2] * h_factor + v_sweep[2] * v_factor,
            ];
            let projected_pos = project_to_sphere(pos, radius);
            let normal = normalize(projected_pos);
            vertices.push(Vertex::new(
                projected_pos,
                normal,
                [h_factor, v_factor],
                color,
            ));
        }
    }

    vertices
}

fn normalize(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    [v[0] / len, v[1] / len, v[2] / len]
}

fn project_to_sphere(pos: [f32; 3], radius: f32) -> [f32; 3] {
    let norm = normalize(pos);
    [norm[0] * radius, norm[1] * radius, norm[2] * radius]
}
