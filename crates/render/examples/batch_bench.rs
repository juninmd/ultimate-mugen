use std::time::Instant;
use ume_render::{Batch, Quad, Rect, Vertex};

fn quad(i: usize) -> Quad {
    let mut q = Quad::new(
        [(i % 64) as f32 * 32.0, (i / 64) as f32 * 32.0, 24.0, 24.0],
        Rect {
            x: 0,
            y: 0,
            w: 24,
            h: 24,
        },
        [4096, 4096],
        (i % 16) as u32,
    );
    q.mul = [0.9, 0.8, 0.7, 1.0];
    q.add = [0.05, 0.0, 0.0];
    q
}

fn main() {
    let runs = 400;
    let stride = std::mem::size_of::<Vertex>();
    for quads in [500usize, 2000, 8000] {
        let mut b = Batch::new();
        let mut best = f64::MAX;
        for _ in 0..runs {
            b.clear();
            let t = Instant::now();
            for i in 0..quads {
                b.push(&quad(i));
            }
            best = best.min(t.elapsed().as_secs_f64());
        }
        let vertex_bytes = b.vertices.len() * stride;
        let index_bytes = b.indices.len()
            * if b.fits_u16() {
                std::mem::size_of::<u16>()
            } else {
                std::mem::size_of::<u32>()
            };
        let total = vertex_bytes + index_bytes;
        println!(
            "quads={quads:5} build={:6.3} ms vertices={:6} stride={stride} B vbytes={:8} ibytes={:7} total={:8} {:5} B/quad u16={}",
            best * 1e3,
            b.vertices.len(),
            vertex_bytes,
            index_bytes,
            total,
            total / quads,
            b.fits_u16()
        );
    }
}
