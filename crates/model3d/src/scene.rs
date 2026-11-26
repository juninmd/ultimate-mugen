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
    pub z: [f32; 3],
    pub colors: [[u8; 4]; 3],
    pub uv: [[f32; 2]; 3],
    pub texture: Option<usize>,
}

const LIGHT: [f32; 3] = [-0.38, 0.62, 0.69];
const AMBIENT: f32 = 0.56;
const WRAP: f32 = 0.35;
const RIM: f32 = 0.22;

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

fn normalize(n: [f32; 3]) -> [f32; 3] {
    let len = n
        .iter()
        .map(|v| v * v)
        .sum::<f32>()
        .sqrt()
        .max(f32::EPSILON);
    n.map(|v| v / len)
}

fn shade(normal: [f32; 3], base: [f32; 3]) -> [u8; 4] {
    let mut n = normalize(normal);
    if n[2] < 0.0 {
        n = n.map(|v| -v);
    }
    let lit: f32 = n.iter().zip(LIGHT).map(|(v, l)| v * l).sum();
    let wrap = ((lit + WRAP) / (1.0 + WRAP)).clamp(0.0, 1.0);
    let rim = (1.0 - n[2]).clamp(0.0, 1.0).powi(3) * RIM;
    let k = AMBIENT + (1.0 - AMBIENT) * wrap * wrap;
    let [r, g, bl] = base.map(|v| ((v * k + rim) * 255.0).clamp(0.0, 255.0) as u8);
    [r, g, bl, 255]
}

fn vertex_normals(view: &[[f32; 3]], indices: &[u32], out: &mut Vec<[f32; 3]>) {
    out.clear();
    out.resize(view.len(), [0.0; 3]);
    for tri in indices.chunks_exact(3) {
        let [a, b, c] = [0, 1, 2].map(|k| view[tri[k] as usize]);
        let face = cross(sub(b, a), sub(c, a));
        for &i in tri {
            let n = &mut out[i as usize];
            for axis in 0..3 {
                n[axis] += face[axis];
            }
        }
    }
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
        let (sp, cp) = cam.pitch.sin_cos();
        let reach = cam.lens / cam.span;
        let mut normals = Vec::new();
        for (prim, verts) in self.primitives.iter().zip(&posed.positions) {
            let view: Vec<[f32; 3]> = verts
                .iter()
                .map(|p| {
                    let (x, z) = (p[0] * c + p[2] * s, -p[0] * s + p[2] * c);
                    let dy = p[1] - cam.pivot;
                    let (y, z) = (dy * cp - z * sp, dy * sp + z * cp);
                    let near = if reach == 0.0 { 1.0 } else { 1.0 / (1.0 - z * reach).max(0.35) };
                    [x * near, y * near + cam.pivot, z]
                })
                .collect();
            vertex_normals(&view, &prim.indices, &mut normals);
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
                    z: [a[2], b[2], d[2]],
                    colors: [tri[0], tri[1], tri[2]].map(|i| shade(normals[i as usize], prim.color)),
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
