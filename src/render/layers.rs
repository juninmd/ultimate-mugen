use crate::gfx::*;
pub(super) fn draw_transformed_layer_quad(
    texture: &Texture2D,
    pivot: [f32; 2],
    axis: [f32; 2],
    size: [f32; 2],
    angle_deg: f32,
    x_shear: f32,
    color: Color,
) {
    let (sin, cos) = (-angle_deg.to_radians()).sin_cos();
    let corners = [
        [0.0, 0.0],
        [size[0], 0.0],
        [size[0], size[1]],
        [0.0, size[1]],
    ];
    let uvs = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
    let vertices = (0..4)
        .map(|i| {
            let (rx, ry) = (corners[i][0] - axis[0], corners[i][1] - axis[1]);
            let sheared = rx - x_shear * ry;
            Vertex::new(
                pivot[0] + sheared * cos - ry * sin,
                pivot[1] + sheared * sin + ry * cos,
                0.0,
                uvs[i][0],
                uvs[i][1],
                color,
            )
        })
        .collect();
    draw_mesh(&Mesh {
        vertices,
        indices: vec![0, 1, 2, 0, 2, 3],
        texture: Some(texture.clone()),
    });
}
