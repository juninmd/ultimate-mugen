use crate::camera::Camera;
use crate::model::{Model, Posed};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Facing {
    Right,
    Left,
}

#[derive(Clone, Copy, Debug)]
pub struct Triangle {
    pub points: [[f32; 2]; 3],
    pub depth: f32,
    pub color: [u8; 4],
    pub uv: [[f32; 2]; 3],
    pub texture: Option<usize>,
}

const LIGHT: [f32; 3] = [-0.35, 0.65, 0.68];

fn wrapped_uv(uv: [[f32; 2]; 3]) -> [[f32; 2]; 3] {
    let shift = [0, 1].map(|axis| ((uv[0][axis] + uv[1][axis] + uv[2][axis]) / 3.0).floor());
    uv.map(|p| [0, 1].map(|axis| (p[axis] - shift[axis]).clamp(0.002, 0.998)))
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn shade(a: [f32; 3], b: [f32; 3], c: [f32; 3], base: [f32; 3]) -> [u8; 4] {
    let mut n = cross(sub(b, a), sub(c, a));
    if n[2] < 0.0 {
        n = n.map(|v| -v);
    }
    let len = n
        .iter()
        .map(|v| v * v)
        .sum::<f32>()
        .sqrt()
        .max(f32::EPSILON);
    let lit: f32 = n.iter().zip(LIGHT).map(|(v, l)| v / len * l).sum();
    let k = 0.42 + 0.58 * lit.max(0.0);
    let [r, g, bl] = base.map(|v| (v * k * 255.0).clamp(0.0, 255.0) as u8);
    [r, g, bl, 255]
}

impl Model {
    pub fn project(
        &self,
        posed: &Posed,
        cam: &Camera,
        facing: Facing,
        origin: [f32; 2],
        out: &mut Vec<Triangle>,
    ) {
        out.clear();
        let yaw = cam.yaw_offset
            + match facing {
                Facing::Right => std::f32::consts::FRAC_PI_2,
                Facing::Left => -std::f32::consts::FRAC_PI_2,
            };
        let (s, c) = yaw.sin_cos();
        for (prim, verts) in self.primitives.iter().zip(&posed.positions) {
            let view: Vec<[f32; 3]> = verts
                .iter()
                .map(|p| [p[0] * c + p[2] * s, p[1], -p[0] * s + p[2] * c])
                .collect();
            for tri in prim.indices.chunks_exact(3) {
                let [a, b, d] = [
                    view[tri[0] as usize],
                    view[tri[1] as usize],
                    view[tri[2] as usize],
                ];
                let to_screen = |p: [f32; 3]| {
                    [
                        origin[0] + p[0] * cam.pixels_per_unit,
                        origin[1] - (p[1] - cam.floor) * cam.pixels_per_unit,
                    ]
                };
                out.push(Triangle {
                    points: [to_screen(a), to_screen(b), to_screen(d)],
                    depth: (a[2] + b[2] + d[2]) / 3.0,
                    color: shade(a, b, d, prim.color),
                    uv: wrapped_uv(
                        [tri[0], tri[1], tri[2]]
                            .map(|i| prim.uvs.get(i as usize).copied().unwrap_or([0.0; 2])),
                    ),
                    texture: prim.texture,
                });
            }
        }
        out.sort_by(|x, y| x.depth.total_cmp(&y.depth));
    }
}
