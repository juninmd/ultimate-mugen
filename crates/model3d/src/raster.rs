use crate::image_read::Image;
use crate::scene::Triangle;

const ALPHA_CUTOFF: u8 = 128;

pub struct Raster {
    width: usize,
    height: usize,
    samples: usize,
    color: Vec<[u8; 4]>,
    depth: Vec<f32>,
    bounds: [usize; 4],
}

fn edge(a: [f32; 2], b: [f32; 2], p: [f32; 2]) -> f32 {
    (b[0] - a[0]) * (p[1] - a[1]) - (b[1] - a[1]) * (p[0] - a[0])
}

fn texel(image: &Image, u: f32, v: f32) -> [u8; 4] {
    let x = ((u.max(0.0) * image.width as f32) as u32).min(image.width - 1);
    let y = ((v.max(0.0) * image.height as f32) as u32).min(image.height - 1);
    let k = ((y * image.width + x) * 4) as usize;
    [
        image.rgba[k],
        image.rgba[k + 1],
        image.rgba[k + 2],
        image.rgba[k + 3],
    ]
}

impl Raster {
    pub fn new(width: usize, height: usize, samples: usize) -> Raster {
        let samples = samples.max(1);
        let cells = width * height * samples * samples;
        Raster {
            width,
            height,
            samples,
            color: vec![[0; 4]; cells],
            depth: vec![f32::NEG_INFINITY; cells],
            bounds: [0, 0, 0, 0],
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    fn clear(&mut self) {
        let stride = self.width * self.samples;
        let [x0, y0, x1, y1] = self.bounds;
        for y in y0..y1 {
            let row = y * stride;
            self.color[row + x0..row + x1].fill([0; 4]);
            self.depth[row + x0..row + x1].fill(f32::NEG_INFINITY);
        }
    }

    pub fn draw(&mut self, triangles: &[Triangle], images: &[Option<Image>]) {
        self.clear();
        let k = self.samples as f32;
        let (sw, sh) = (self.width * self.samples, self.height * self.samples);
        let mut bounds = [sw, sh, 0, 0];
        for t in triangles {
            let p = t.points.map(|q| [q[0] * k, q[1] * k]);
            let area = edge(p[0], p[1], p[2]);
            if area.abs() < 1e-6 {
                continue;
            }
            let lo =
                |axis: usize| p[0][axis].min(p[1][axis]).min(p[2][axis]).floor().max(0.0) as usize;
            let hi = |axis: usize, limit: usize| {
                (p[0][axis].max(p[1][axis]).max(p[2][axis]).ceil().max(0.0) as usize).min(limit)
            };
            let (x0, y0, x1, y1) = (lo(0), lo(1), hi(0, sw), hi(1, sh));
            if x0 >= x1 || y0 >= y1 {
                continue;
            }
            bounds = [
                bounds[0].min(x0),
                bounds[1].min(y0),
                bounds[2].max(x1),
                bounds[3].max(y1),
            ];
            let image = t
                .texture
                .and_then(|i| images.get(i))
                .and_then(|i| i.as_ref());
            let inv = 1.0 / area;
            let plane = |i: usize, j: usize| {
                let (a, b) = (p[i], p[j]);
                let dx = -(b[1] - a[1]) * inv;
                let dy = (b[0] - a[0]) * inv;
                (dx, dy, -(dx * a[0] + dy * a[1]))
            };
            let planes = [plane(1, 2), plane(2, 0), plane(0, 1)];
            let attr = |values: [f32; 3]| {
                let mut out = [0.0f32; 3];
                for (i, (dx, dy, c)) in planes.iter().enumerate() {
                    out[0] += dx * values[i];
                    out[1] += dy * values[i];
                    out[2] += c * values[i];
                }
                out
            };
            let mut gradients = [[0.0f32; 3]; 6];
            gradients[0] = attr(t.z);
            gradients[1] = attr([t.uv[0][0], t.uv[1][0], t.uv[2][0]]);
            gradients[2] = attr([t.uv[0][1], t.uv[1][1], t.uv[2][1]]);
            for c in 0..3 {
                gradients[3 + c] = attr(t.colors.map(|v| v[c] as f32 * (1.0 / 255.0)));
            }
            for y in y0..y1 {
                let fy = y as f32 + 0.5;
                let (mut from, mut to) = (x0 as f32 + 0.5, x1 as f32 - 0.5);
                for (dx, dy, c) in planes {
                    let r = dy * fy + c;
                    if dx.abs() < 1e-12 {
                        if r < 0.0 {
                            to = f32::NEG_INFINITY;
                        }
                    } else if dx > 0.0 {
                        from = from.max(-r / dx);
                    } else {
                        to = to.min(-r / dx);
                    }
                }
                if from > to {
                    continue;
                }
                let first = ((from - 0.5).ceil().max(x0 as f32)) as usize;
                let last = ((to - 0.5).floor().min((x1 - 1) as f32)) as usize;
                if first > last {
                    continue;
                }
                let fx = first as f32 + 0.5;
                let mut value = gradients.map(|g| g[0] * fx + g[1] * fy + g[2]);
                let step = gradients.map(|g| g[0]);
                let row = y * sw;
                let depth = &mut self.depth[row + first..=row + last];
                let color = &mut self.color[row + first..=row + last];
                for (d, c) in depth.iter_mut().zip(color.iter_mut()) {
                    let z = value[0];
                    if z > *d {
                        let texture = match image {
                            Some(image) => texel(image, value[1], value[2]),
                            None => [255; 4],
                        };
                        if texture[3] >= ALPHA_CUTOFF {
                            *d = z;
                            let lit = |k: usize| (value[3 + k] * texture[k] as f32) as u8;
                            *c = [lit(0), lit(1), lit(2), 255];
                        }
                    }
                    for k in 0..6 {
                        value[k] += step[k];
                    }
                }
            }
        }
        self.bounds = if bounds[0] < bounds[2] {
            bounds
        } else {
            [0, 0, 0, 0]
        };
    }

    pub fn resolve(&self, out: &mut [u8]) {
        let n = self.samples;
        let stride = self.width * n;
        let inv = 1.0 / (n * n) as f32;
        for y in 0..self.height {
            for x in 0..self.width {
                let mut acc = [0.0f32; 4];
                for sy in 0..n {
                    for sx in 0..n {
                        let c = self.color[(y * n + sy) * stride + x * n + sx];
                        let a = c[3] as f32 / 255.0;
                        for ch in 0..3 {
                            acc[ch] += c[ch] as f32 * a;
                        }
                        acc[3] += a;
                    }
                }
                let o = (y * self.width + x) * 4;
                let alpha = acc[3] * inv;
                for ch in 0..3 {
                    out[o + ch] = if acc[3] > 0.0 {
                        (acc[ch] / acc[3]) as u8
                    } else {
                        0
                    };
                }
                out[o + 3] = (alpha * 255.0) as u8;
            }
        }
    }
}
