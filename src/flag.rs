pub(crate) mod catalog;
mod designs;
mod paint;
#[cfg(test)]
mod tests;

pub(crate) const ASPECT: f32 = 1.5;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Shape {
    Rect {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        color: u32,
    },
    Disc {
        x: f32,
        y: f32,
        r: f32,
        color: u32,
    },
    Half {
        x: f32,
        y: f32,
        r: f32,
        top: bool,
        color: u32,
    },
    Star {
        x: f32,
        y: f32,
        r: f32,
        points: u8,
        color: u32,
    },
    Poly {
        points: &'static [[f32; 2]],
        color: u32,
    },
}

pub(crate) fn design(code: &str) -> Option<&'static [Shape]> {
    designs::all()
        .find(|(known, _)| known.eq_ignore_ascii_case(code))
        .map(|(_, shapes)| shapes)
}

pub(crate) fn height_for(width: f32) -> f32 {
    width / ASPECT
}
