use std::sync::{Arc, Mutex};
use ume_engine::chardef::ImageData;

pub const SEGMENT: f32 = 836.0;
pub const SEGMENTS: usize = 5;
pub const STRIP: f32 = SEGMENT * SEGMENTS as f32;
pub const SEAM_BAND: f32 = 70.0;

const CORES: [[f32; 4]; 5] = [
    [210.0, 120.0, 330.0, 190.0],
    [200.0, 160.0, 180.0, 170.0],
    [200.0, 150.0, 175.0, 190.0],
    [210.0, 160.0, 185.0, 175.0],
    [200.0, 150.0, 180.0, 190.0],
];
const GRADES: [[f32; 3]; 5] = [
    [0.82, 0.72, 0.95],
    [0.85, 0.8, 1.05],
    [0.85, 0.95, 0.85],
    [1.1, 0.8, 0.7],
    [0.85, 0.78, 1.0],
];

static BOUND: Mutex<[u64; 5]> = Mutex::new([0; 5]);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Run {
    pub at: f32,
    pub width: f32,
    pub source: f32,
    pub flipped: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Piece {
    pub offset: f32,
    pub width: f32,
    pub source: f32,
    pub flipped: bool,
}

#[derive(Clone)]
pub struct Backdrop {
    pub runs: Vec<Run>,
    pub grade: [f32; 3],
}

pub fn register(index: usize, image: &Arc<ImageData>) {
    BOUND.lock().unwrap()[index] = image.id;
}

pub fn backdrop(image: &ImageData) -> Option<Backdrop> {
    let index = BOUND
        .lock()
        .unwrap()
        .iter()
        .position(|id| *id == image.id)?;
    Some(Backdrop {
        runs: runs(CORES[index]),
        grade: GRADES[index],
    })
}

pub fn runs(core: [f32; 4]) -> Vec<Run> {
    let mut out = vec![Run {
        at: 0.0,
        width: SEGMENT,
        source: 0.0,
        flipped: false,
    }];
    for (slot, start) in core[..3].iter().enumerate() {
        let span = core[3].min(SEGMENT - start);
        let origin = SEGMENT * (slot + 1) as f32;
        let (mut x, mut flipped) = (0.0, false);
        while x < SEGMENT {
            let width = span.min(SEGMENT - x);
            let source = if flipped {
                start + span - width
            } else {
                *start
            };
            out.push(Run {
                at: origin + x,
                width,
                source,
                flipped,
            });
            x += span;
            flipped = !flipped;
        }
    }
    out.push(Run {
        at: SEGMENT * (SEGMENTS - 1) as f32,
        width: SEGMENT,
        source: 0.0,
        flipped: true,
    });
    out
}

pub fn pieces(runs: &[Run], from: f32, to: f32, mirrored: bool) -> Vec<Piece> {
    let mut out = Vec::new();
    for run in runs {
        let (a, b) = (from.max(run.at), to.min(run.at + run.width));
        if b <= a {
            continue;
        }
        let source = if run.flipped {
            run.source + run.width - (b - run.at)
        } else {
            run.source + a - run.at
        };
        let offset = if mirrored { to - b } else { a - from };
        out.push(Piece {
            offset,
            width: b - a,
            source,
            flipped: run.flipped != mirrored,
        });
    }
    out
}

#[cfg(test)]
pub fn sample(runs: &[Run], x: f32) -> Option<f32> {
    let run = runs
        .iter()
        .find(|run| x >= run.at && x < run.at + run.width)?;
    let local = x - run.at;
    Some(if run.flipped {
        run.source + run.width - 1.0 - local
    } else {
        run.source + local
    })
}

pub fn grade_at(grade: [f32; 3], x: f32) -> [f32; 3] {
    let ramp = (x / (STRIP - 1.0)).clamp(0.0, 1.0);
    grade.map(|end| 1.0 + (end - 1.0) * ramp)
}

pub fn seam_mix(offset: f32) -> f32 {
    let t = ((offset + SEAM_BAND) / (2.0 * SEAM_BAND - 1.0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
#[path = "story_loop_tests.rs"]
mod tests;
