use crate::ini::{parse_float, parse_i32, split_lines, starts_with_number, strip_comment};
use std::collections::HashMap;

/// Collision box `x1, y1, x2, y2`, normalized so that `x1 <= x2`, `y1 <= y2`.
pub type Box4 = [f32; 4];

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Blend {
    None,
    Add,
    Add1,
    Sub,
}

#[derive(Clone, Debug)]
pub struct AnimFrame {
    pub group: i32,
    pub index: i32,
    pub x: f32,
    pub y: f32,
    /// Ticks; -1 = infinite.
    pub time: i32,
    pub flip_h: bool,
    pub flip_v: bool,
    pub blend: Blend,
    /// `AS<src>D<dst>` alpha values.
    pub alpha: Option<(i32, i32)>,
    pub xscale: f32,
    pub yscale: f32,
    pub angle: f32,
    pub clsn1: Vec<Box4>,
    pub clsn2: Vec<Box4>,
}

#[derive(Clone, Debug)]
pub struct Animation {
    pub id: i32,
    pub frames: Vec<AnimFrame>,
    pub loop_start: usize,
    /// Sum of frame times (-1 if any frame is infinite).
    pub total_time: i32,
}

impl Animation {
    pub fn compute_total(&mut self) {
        let mut t = 0i32;
        for f in &self.frames {
            if f.time < 0 {
                t = -1;
                break;
            }
            t = t.saturating_add(f.time);
        }
        self.total_time = t;
    }
}

#[inline]
fn parse_box4(s: &str) -> Box4 {
    let mut out = [0.0f32; 4];
    for (i, p) in s.split(',').take(4).enumerate() {
        out[i] = parse_float(p.trim()).unwrap_or(0.0) as f32;
    }
    let (x1, y1, x2, y2) = (out[0], out[1], out[2], out[3]);
    [x1.min(x2), y1.min(y2), x1.max(x2), y1.max(y2)]
}

#[inline]
fn is_ws(c: u8) -> bool {
    matches!(c, b' ' | b'\t' | b'\n' | b'\r' | 0x0b | 0x0c)
}

fn skip_ws(b: &[u8], mut i: usize) -> usize {
    while i < b.len() && is_ws(b[i]) {
        i += 1;
    }
    i
}

fn skip_digits(b: &[u8], mut i: usize) -> usize {
    while i < b.len() && b[i].is_ascii_digit() {
        i += 1;
    }
    i
}

/// `/begin\s+action\s+(-?\d+)/i` anywhere in the line.
fn parse_begin_action(line: &str) -> Option<i32> {
    let b = line.as_bytes();
    for (idx, win) in b.windows(5).enumerate() {
        if win.eq_ignore_ascii_case(b"begin") {
            let mut i = idx + 5;
            let j = skip_ws(b, i);
            if j > i && b[j..].len() >= 6 && b[j..j + 6].eq_ignore_ascii_case(b"action") {
                i = j + 6;
                let k = skip_ws(b, i);
                if k > i {
                    let mut d = k;
                    if b.get(d) == Some(&b'-') {
                        d += 1;
                    }
                    let e = skip_digits(b, d);
                    if e > d {
                        return parse_i32(&line[k..e]);
                    }
                }
            }
        }
    }
    None
}

/// `/^clsn([12])(default)?\s*:\s*(\d+)/i` -> (kind, is_default)
fn parse_clsn_header(line: &str) -> Option<(u8, bool)> {
    let b = line.as_bytes();
    if b.len() < 5 || !b[..4].eq_ignore_ascii_case(b"clsn") {
        return None;
    }
    let kind = match b[4] {
        b'1' => 1,
        b'2' => 2,
        _ => return None,
    };
    let mut i = 5;
    let mut is_default = false;
    if b[i..].len() >= 7 && b[i..i + 7].eq_ignore_ascii_case(b"default") {
        is_default = true;
        i += 7;
    }
    i = skip_ws(b, i);
    if b.get(i) != Some(&b':') {
        return None;
    }
    i = skip_ws(b, i + 1);
    if !b.get(i).is_some_and(|c| c.is_ascii_digit()) {
        return None;
    }
    Some((kind, is_default))
}

/// `/^clsn([12])\s*\[\s*\d+\s*\]\s*=\s*(.+)$/i` -> (kind, value start offset)
fn parse_clsn_box(line: &str) -> Option<(u8, usize)> {
    let b = line.as_bytes();
    if b.len() < 5 || !b[..4].eq_ignore_ascii_case(b"clsn") {
        return None;
    }
    let kind = match b[4] {
        b'1' => 1,
        b'2' => 2,
        _ => return None,
    };
    let mut i = skip_ws(b, 5);
    if b.get(i) != Some(&b'[') {
        return None;
    }
    i = skip_ws(b, i + 1);
    let e = skip_digits(b, i);
    if e == i {
        return None;
    }
    i = skip_ws(b, e);
    if b.get(i) != Some(&b']') {
        return None;
    }
    i = skip_ws(b, i + 1);
    if b.get(i) != Some(&b'=') {
        return None;
    }
    i = skip_ws(b, i + 1);
    if i >= b.len() {
        return None;
    }
    Some((kind, i))
}

/// `/AS(\d+)D(\d+)/` anywhere in a string.
fn parse_as_d(bl: &str) -> Option<(i32, i32)> {
    let b = bl.as_bytes();
    for (idx, win) in b.windows(2).enumerate() {
        if win.eq_ignore_ascii_case(b"as") {
            let s = idx + 2;
            let e = skip_digits(b, s);
            if e > s && b.get(e).is_some_and(|&c| c == b'd' || c == b'D') {
                let e2 = skip_digits(b, e + 1);
                if e2 > e + 1 {
                    return Some((parse_i32(&bl[s..e])?, parse_i32(&bl[e + 1..e2])?));
                }
            }
        }
    }
    None
}

fn finish(cur: &mut Option<Animation>, anims: &mut HashMap<i32, Animation>) {
    if let Some(mut a) = cur.take() {
        a.compute_total();
        anims.insert(a.id, a);
    }
}

pub fn parse_air(text: &str) -> HashMap<i32, Animation> {
    let mut anims: HashMap<i32, Animation> = HashMap::new();
    let mut cur: Option<Animation> = None;
    let mut def1: Vec<Box4> = Vec::new();
    let mut def2: Vec<Box4> = Vec::new();
    let mut next1: Option<Vec<Box4>> = None;
    let mut next2: Option<Vec<Box4>> = None;
    let mut pending: Option<(u8, bool)> = None;

    for raw in split_lines(text) {
        let line = strip_comment(raw).trim();
        if line.is_empty() {
            continue;
        }
        if line.as_bytes().starts_with(b"[") {
            finish(&mut cur, &mut anims);
            if let Some(id) = parse_begin_action(line) {
                cur = Some(Animation {
                    id,
                    frames: Vec::new(),
                    loop_start: 0,
                    total_time: 0,
                });
            }
            def1.clear();
            def2.clear();
            next1 = None;
            next2 = None;
            pending = None;
            continue;
        }
        let Some(anim) = cur.as_mut() else { continue };
        if let Some((kind, is_default)) = parse_clsn_header(line) {
            pending = Some((kind, is_default));
            match (kind, is_default) {
                (1, true) => def1 = Vec::new(),
                (2, true) => def2 = Vec::new(),
                (1, false) => next1 = Some(Vec::new()),
                _ => next2 = Some(Vec::new()),
            }
            continue;
        }
        if let Some((kind, vs)) = parse_clsn_box(line) {
            let bx = parse_box4(&line[vs..]);
            match pending {
                Some((k, true)) if k == kind => {
                    if kind == 1 {
                        def1.push(bx)
                    } else {
                        def2.push(bx)
                    }
                }
                Some((k, false)) if k == kind => {
                    if kind == 1 {
                        next1.get_or_insert_with(Vec::new).push(bx)
                    } else {
                        next2.get_or_insert_with(Vec::new).push(bx)
                    }
                }
                _ => {
                    if kind == 1 {
                        next1.get_or_insert_with(Vec::new).push(bx)
                    } else {
                        next2.get_or_insert_with(Vec::new).push(bx)
                    }
                }
            }
            continue;
        }
        if line.len() >= 9 && line[..9].eq_ignore_ascii_case("loopstart") {
            anim.loop_start = anim.frames.len();
            continue;
        }
        if line.len() >= 11 && line[..11].eq_ignore_ascii_case("interpolate") {
            continue;
        }
        if starts_with_number(line) {
            let mut p = [""; 11];
            let mut count = 0;
            for s in line.split(',') {
                if count < p.len() {
                    p[count] = s.trim();
                    count += 1;
                }
            }
            let at = |i: usize| p.get(i).copied().unwrap_or("");
            let fl = at(5);
            let bl = at(6);
            let mut blend = Blend::None;
            let mut alpha = None;
            if bl.len() >= 2 && bl[..2].eq_ignore_ascii_case("a1") {
                blend = Blend::Add1;
            } else if bl.len() >= 2 && bl[..2].eq_ignore_ascii_case("as") {
                blend = Blend::Add;
                alpha = parse_as_d(bl);
            } else if bl.starts_with(['A', 'a']) {
                blend = Blend::Add;
            } else if bl.starts_with(['S', 's']) {
                blend = Blend::Sub;
            }
            let fnum = |s: &str, d: f32| match parse_float(s) {
                Some(v) if v != 0.0 && !v.is_nan() => v as f32,
                _ => d,
            };
            let opt = |i: usize, d: f32| if at(i).is_empty() { d } else { fnum(at(i), d) };
            let frame = AnimFrame {
                group: parse_i32(at(0)).unwrap_or(0),
                index: parse_i32(at(1)).unwrap_or(0),
                x: fnum(at(2), 0.0),
                y: fnum(at(3), 0.0),
                time: parse_i32(at(4)).unwrap_or(1),
                flip_h: fl.chars().any(|c| c.eq_ignore_ascii_case(&'h')),
                flip_v: fl.chars().any(|c| c.eq_ignore_ascii_case(&'v')),
                blend,
                alpha,
                xscale: opt(7, 1.0),
                yscale: opt(8, 1.0),
                angle: opt(9, 0.0),
                clsn1: next1.take().unwrap_or_else(|| def1.clone()),
                clsn2: next2.take().unwrap_or_else(|| def2.clone()),
            };
            anim.frames.push(frame);
        }
    }
    finish(&mut cur, &mut anims);
    anims
}
