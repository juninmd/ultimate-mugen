use crate::chardef::{Box4, Controller};
use crate::matchs::Match;

const MAX_MODS: usize = 32;

#[derive(Clone, Debug, PartialEq)]
pub enum ClsnOp {
    Override { index: i32, rect: Box4 },
    Resize { index: i32, delta: Box4 },
    Transform { scale: [f32; 2], offset: [f32; 2] },
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClsnMod {
    pub kind: u8,
    pub time: i32,
    pub op: ClsnOp,
}

impl ClsnMod {
    fn same_slot(&self, other: &ClsnMod) -> bool {
        self.kind == other.kind
            && match (&self.op, &other.op) {
                (ClsnOp::Override { index: a, .. }, ClsnOp::Override { index: b, .. }) => a == b,
                (ClsnOp::Resize { index: a, .. }, ClsnOp::Resize { index: b, .. }) => a == b,
                (ClsnOp::Transform { .. }, ClsnOp::Transform { .. }) => true,
                _ => false,
            }
    }
}

fn normalized(b: Box4) -> Box4 {
    [
        b[0].min(b[2]),
        b[1].min(b[3]),
        b[0].max(b[2]),
        b[1].max(b[3]),
    ]
}

pub fn apply(mods: &[ClsnMod], kind: u8, boxes: &[Box4]) -> Vec<Box4> {
    let mut out = boxes.to_vec();
    for m in mods.iter().filter(|m| m.kind == kind) {
        match &m.op {
            ClsnOp::Override { index, rect } => {
                if *index < 0 {
                    out = vec![*rect];
                } else if (*index as usize) < out.len() {
                    out[*index as usize] = *rect;
                } else {
                    out.push(*rect);
                }
            }
            ClsnOp::Resize { index, delta } => {
                for (k, b) in out.iter_mut().enumerate() {
                    if *index < 0 || *index as usize == k {
                        *b = normalized([
                            b[0] + delta[0],
                            b[1] + delta[1],
                            b[2] + delta[2],
                            b[3] + delta[3],
                        ]);
                    }
                }
            }
            ClsnOp::Transform { scale, offset } => {
                for b in out.iter_mut() {
                    *b = normalized([
                        b[0] * scale[0] + offset[0],
                        b[1] * scale[1] + offset[1],
                        b[2] * scale[0] + offset[0],
                        b[3] * scale[1] + offset[1],
                    ]);
                }
            }
        }
    }
    out
}

fn quad(m: &Match, i: usize, sc: &Controller, key: &str, default: f64) -> Option<Box4> {
    if !sc.has(key, 3) {
        return None;
    }
    Some([0, 1, 2, 3].map(|k| m.num(i, sc, key, default, k) as f32))
}

fn push(m: &mut Match, i: usize, new: ClsnMod) {
    let f = &mut m.fighters[i];
    f.clsn_mods.retain(|c| !c.same_slot(&new));
    if f.clsn_mods.len() < MAX_MODS {
        f.clsn_mods.push(new);
    }
}

pub fn run(m: &mut Match, i: usize, sc: &Controller) {
    let kind = if m.num(i, sc, "clsn", 2.0, 0) as i32 == 1 {
        1
    } else {
        2
    };
    let time = match m.num(i, sc, "time", 1.0, 0) as i32 {
        0 => 1,
        t => t,
    };
    let index = m.num(i, sc, "index", -1.0, 0) as i32;
    let op = match sc.kind.as_str() {
        "overrideclsn" => match quad(m, i, sc, "value", 0.0) {
            Some(v) => ClsnOp::Override {
                index,
                rect: normalized(v),
            },
            None => return,
        },
        "resizeclsn" => match quad(m, i, sc, "value", 0.0) {
            Some(delta) => ClsnOp::Resize { index, delta },
            None => return,
        },
        _ => {
            let scale = [
                m.num(i, sc, "scale", 1.0, 0) as f32,
                if sc.has("scale", 1) {
                    m.num(i, sc, "scale", 1.0, 1) as f32
                } else {
                    m.num(i, sc, "scale", 1.0, 0) as f32
                },
            ];
            let offset = [
                m.num(i, sc, "offset", 0.0, 0) as f32,
                m.num(i, sc, "offset", 0.0, 1) as f32,
            ];
            ClsnOp::Transform { scale, offset }
        }
    };
    push(m, i, ClsnMod { kind, time, op });
}
