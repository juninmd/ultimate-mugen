use super::rig::{art_res, RigRender};
use super::types::{Box4, NativeFrame};
use crate::chardef::ImageData;

const OPAQUE: u8 = 40;
const MIN_SIDE: f32 = 16.0;
const FALLBACK_SHARE: f32 = 0.4;
const BACK_CENTER: f32 = -0.3;

pub struct SpriteMask<'a> {
    alpha: &'a [u8],
    width: usize,
    height: usize,
    left: f32,
    top: f32,
    unit: f32,
}

impl<'a> SpriteMask<'a> {
    pub fn new(image: &'a ImageData, axis: [f32; 2], scale: f32, offset: [f32; 2]) -> Self {
        let res = art_res();
        SpriteMask {
            alpha: &image.rgba,
            width: image.w as usize,
            height: image.h as usize,
            left: offset[0] - axis[0] * scale,
            top: offset[1] - axis[1] * scale,
            unit: res * scale,
        }
    }

    fn solid(&self, x: usize, y: usize) -> bool {
        self.alpha
            .get((y * self.width + x) * 4 + 3)
            .is_some_and(|a| *a >= OPAQUE)
    }

    fn pixels(&self, b: Box4) -> [usize; 4] {
        let cell = |value: f32, origin: f32, limit: usize, up: bool| {
            let at = (value - origin) / self.unit;
            (if up { at.ceil() } else { at.floor() }).clamp(0.0, limit as f32) as usize
        };
        [
            cell(b[0], self.left, self.width, false),
            cell(b[1], self.top, self.height, false),
            cell(b[2], self.left, self.width, true),
            cell(b[3], self.top, self.height, true),
        ]
    }

    fn units(&self, p: [usize; 4]) -> Box4 {
        [
            self.left + p[0] as f32 * self.unit,
            self.top + p[1] as f32 * self.unit,
            self.left + p[2] as f32 * self.unit,
            self.top + p[3] as f32 * self.unit,
        ]
    }

    pub fn bounds(&self) -> Option<Box4> {
        self.trim_pixels([0, 0, self.width, self.height])
            .map(|p| self.units(p))
    }

    pub fn trim(&self, b: Box4) -> Option<Box4> {
        self.trim_pixels(self.pixels(b)).map(|p| self.units(p))
    }

    fn trim_pixels(&self, [x0, y0, x1, y1]: [usize; 4]) -> Option<[usize; 4]> {
        let cols = (x0..x1).filter(|x| (y0..y1).any(|y| self.solid(*x, y)));
        let (first, last) = (cols.clone().next()?, cols.last()?);
        let rows = (y0..y1).filter(|y| (first..=last).any(|x| self.solid(x, *y)));
        Some([first, rows.clone().next()?, last + 1, rows.last()? + 1])
    }
}

fn fit_strike(mask: &SpriteMask, bounds: Box4, declared: Box4) -> Box4 {
    let forward =
        (declared[0] + declared[2]) * 0.5 >= BACK_CENTER * (bounds[3] - bounds[1]).max(1.0);
    let mut reach = declared;
    if forward {
        reach[2] = reach[2].max(bounds[2]);
    } else {
        reach[0] = reach[0].min(bounds[0]);
    }
    let solid = |b: &Box4| b[2] - b[0] >= MIN_SIDE && b[3] - b[1] >= MIN_SIDE;
    mask.trim(reach)
        .filter(solid)
        .or_else(|| {
            let width = (bounds[2] - bounds[0]) * FALLBACK_SHARE;
            mask.trim(if forward {
                [bounds[2] - width, bounds[1], bounds[2], bounds[3]]
            } else {
                [bounds[0], bounds[1], bounds[0] + width, bounds[3]]
            })
        })
        .unwrap_or(bounds)
}

const SLICE_UNITS: f32 = 12.0;
const MIN_SLICES: usize = 3;
const MAX_SLICES: usize = 8;

fn slices(mask: &SpriteMask, whole: Box4, along_x: bool) -> Vec<Box4> {
    let (lo, hi) = if along_x { (0, 2) } else { (1, 3) };
    let span = whole[hi] - whole[lo];
    let wanted = ((span / SLICE_UNITS).round() as usize).clamp(MIN_SLICES, MAX_SLICES);
    let mut best = vec![whole];
    for count in wanted..=MAX_SLICES * 2 {
        let step = span / count as f32;
        let parts: Vec<Box4> = (0..count)
            .filter_map(|i| {
                let mut part = whole;
                part[lo] = whole[lo] + step * i as f32;
                part[hi] = whole[lo] + step * (i + 1) as f32;
                mask.trim(part)
            })
            .collect();
        if parts.len() >= MIN_SLICES {
            return parts;
        }
        if parts.len() > best.len() {
            best = parts;
        }
    }
    best
}

pub(super) fn fit_boxes(render: &mut RigRender, frame: &NativeFrame) {
    let Some(image) = render.image.clone() else {
        return;
    };
    let offset = frame.offset.unwrap_or([0.0, 0.0]);
    let mask = SpriteMask::new(&image, [render.axis_x, render.axis_y], render.scale, offset);
    let Some(bounds) = mask.bounds() else { return };
    render.hurt = render
        .hurt
        .iter()
        .flat_map(|hurt| slices(&mask, mask.trim(*hurt).unwrap_or(bounds), false))
        .collect();
    render.hit = render
        .hit
        .iter()
        .flat_map(|hit| slices(&mask, fit_strike(&mask, bounds, *hit), true))
        .collect();
}
