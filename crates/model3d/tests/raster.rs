use ume_model3d::{Raster, Triangle};

fn quad(z: f32, color: u8) -> Vec<Triangle> {
    let tri = |points: [[f32; 2]; 3]| Triangle {
        points,
        depth: z,
        z: [z; 3],
        colors: [[color, color, color, 255]; 3],
        uv: [[0.0; 2]; 3],
        texture: None,
    };
    vec![
        tri([[2.0, 2.0], [14.0, 2.0], [14.0, 14.0]]),
        tri([[2.0, 2.0], [14.0, 14.0], [2.0, 14.0]]),
    ]
}

fn pixel(rgba: &[u8], x: usize, y: usize) -> [u8; 4] {
    let at = (y * 16 + x) * 4;
    [rgba[at], rgba[at + 1], rgba[at + 2], rgba[at + 3]]
}

#[test]
fn nearer_surface_wins_whatever_the_draw_order() {
    for flipped in [false, true] {
        let (near, far) = (quad(1.0, 200), quad(-1.0, 40));
        let order: Vec<Triangle> = if flipped {
            near.into_iter().chain(far).collect()
        } else {
            far.into_iter().chain(near).collect()
        };
        let mut raster = Raster::new(16, 16, 2);
        raster.draw(&order, &[]);
        let mut rgba = vec![0; 16 * 16 * 4];
        raster.resolve(&mut rgba);
        assert_eq!(pixel(&rgba, 8, 8), [200, 200, 200, 255]);
        assert_eq!(pixel(&rgba, 0, 0)[3], 0);
    }
}

#[test]
fn redraw_clears_the_previous_frame() {
    let mut raster = Raster::new(16, 16, 1);
    raster.draw(&quad(0.0, 90), &[]);
    raster.draw(&[], &[]);
    let mut rgba = vec![0; 16 * 16 * 4];
    raster.resolve(&mut rgba);
    assert_eq!(pixel(&rgba, 8, 8)[3], 0);
}
