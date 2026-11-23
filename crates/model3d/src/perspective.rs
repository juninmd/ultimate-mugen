use crate::model::{Model, Posed};
use crate::scene::Triangle;

const NEAR: f32 = 0.5;

#[derive(Clone, Copy, Debug)]
pub struct Lens {
    pub eye: [f32; 3],
    pub target: [f32; 3],
    pub fov_y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct Look {
    pub toward_light: [f32; 3],
    pub light: [f32; 3],
    pub ambient: [f32; 3],
    pub fog: [f32; 3],
    pub fog_density: f32,
    pub sky: [[f32; 3]; 2],
}

#[derive(Clone, Copy)]
struct Vertex {
    view: [f32; 3],
    color: [f32; 3],
}

struct Basis {
    eye: [f32; 3],
    right: [f32; 3],
    up: [f32; 3],
    forward: [f32; 3],
    focal: f32,
    half: [f32; 2],
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn unit(v: [f32; 3]) -> [f32; 3] {
    let len = dot(v, v).sqrt().max(f32::EPSILON);
    v.map(|c| c / len)
}

fn lerp(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [0, 1, 2].map(|i| a[i] + (b[i] - a[i]) * t)
}

impl Basis {
    fn new(lens: &Lens) -> Basis {
        let forward = unit(sub(lens.target, lens.eye));
        let right = unit(cross(forward, [0.0, 1.0, 0.0]));
        Basis {
            eye: lens.eye,
            right,
            up: cross(right, forward),
            forward,
            focal: lens.height * 0.5 / (lens.fov_y * 0.5).tan(),
            half: [lens.width * 0.5, lens.height * 0.5],
        }
    }

    fn to_view(&self, p: [f32; 3]) -> [f32; 3] {
        let d = sub(p, self.eye);
        [dot(d, self.right), dot(d, self.up), dot(d, self.forward)]
    }

    fn to_screen(&self, v: [f32; 3]) -> [f32; 2] {
        [
            self.half[0] + v[0] / v[2] * self.focal,
            self.half[1] - v[1] / v[2] * self.focal,
        ]
    }
}

fn byte(c: [f32; 3]) -> [u8; 4] {
    let [r, g, b] = c.map(|v| (v * 255.0).clamp(0.0, 255.0) as u8);
    [r, g, b, 255]
}

fn clip_near(tri: [Vertex; 3], out: &mut [Vertex; 4]) -> usize {
    let mut n = 0;
    for i in 0..3 {
        let (a, b) = (tri[i], tri[(i + 1) % 3]);
        let (ain, bin) = (a.view[2] >= NEAR, b.view[2] >= NEAR);
        if ain {
            out[n] = a;
            n += 1;
        }
        if ain != bin {
            let t = (NEAR - a.view[2]) / (b.view[2] - a.view[2]);
            out[n] = Vertex {
                view: lerp(a.view, b.view, t),
                color: lerp(a.color, b.color, t),
            };
            n += 1;
        }
    }
    n
}

fn sky_triangles(lens: &Lens, look: &Look, out: &mut Vec<Triangle>) {
    let [w, h] = [lens.width, lens.height];
    let corner = |x: f32, y: f32| {
        let c = lerp(look.sky[0], look.sky[1], y / h);
        ([x, y], byte(c))
    };
    let (a, b, c, d) = (
        corner(0.0, 0.0),
        corner(w, 0.0),
        corner(w, h),
        corner(0.0, h),
    );
    for tri in [[a, b, c], [a, c, d]] {
        out.push(Triangle {
            points: tri.map(|v| v.0),
            depth: f32::MIN,
            z: [0.0; 3],
            colors: tri.map(|v| v.1),
            uv: [[0.0; 2]; 3],
            texture: None,
        });
    }
}

impl Model {
    pub fn project_scene(&self, posed: &Posed, lens: &Lens, look: &Look, out: &mut Vec<Triangle>) {
        out.clear();
        sky_triangles(lens, look, out);
        let cam = Basis::new(lens);
        let light = unit(look.toward_light);
        for (prim, verts) in self.primitives.iter().zip(&posed.positions) {
            for tri in prim.indices.chunks_exact(3) {
                let world = [0, 1, 2].map(|k| verts[tri[k] as usize]);
                let face = unit(cross(sub(world[1], world[0]), sub(world[2], world[0])));
                let facing = dot(face, sub(cam.eye, world[0])).signum();
                let n = face.map(|c| c * facing);
                let lit = dot(n, light).max(0.0);
                let shade = [0, 1, 2].map(|i| look.ambient[i] + look.light[i] * lit);
                let corners = world.map(|p| {
                    let view = cam.to_view(p);
                    let base = if prim.emissive {
                        prim.color
                    } else {
                        let lit = [0, 1, 2].map(|i| prim.color[i] * shade[i]);
                        let fog = 1.0 - (-view[2].max(0.0) * look.fog_density).exp();
                        lerp(lit, look.fog, fog)
                    };
                    Vertex { view, color: base }
                });
                emit(&cam, corners, out);
            }
        }
    }
}

fn emit(cam: &Basis, corners: [Vertex; 3], out: &mut Vec<Triangle>) {
    let mut poly = [corners[0]; 4];
    let count = if corners.iter().all(|v| v.view[2] >= NEAR) {
        poly[..3].copy_from_slice(&corners);
        3
    } else {
        clip_near(corners, &mut poly)
    };
    for k in 1..count.saturating_sub(1) {
        let tri = [poly[0], poly[k], poly[k + 1]];
        let points = tri.map(|v| cam.to_screen(v.view));
        let outside = |axis: usize, limit: f32| {
            points.iter().all(|p| p[axis] < 0.0) || points.iter().all(|p| p[axis] > limit)
        };
        if outside(0, cam.half[0] * 2.0) || outside(1, cam.half[1] * 2.0) {
            continue;
        }
        let z = tri.map(|v| 1.0 / v.view[2]);
        out.push(Triangle {
            points,
            depth: z[0],
            z,
            colors: tri.map(|v| byte(v.color)),
            uv: [[0.0; 2]; 3],
            texture: None,
        });
    }
}
