//! Input macros: the absolute input frames that make a command fire, so a shortcut can perform it.
use super::{match_command, CommandDef};
use crate::input::InputHistory;

fn absolute_dirs(rel: u32, facing: f32) -> u32 {
    let (fwd, back) = if facing >= 0.0 {
        (crate::input::k::R, crate::input::k::L)
    } else {
        (crate::input::k::L, crate::input::k::R)
    };
    let mut b = 0;
    if rel & 1 != 0 {
        b |= crate::input::k::U;
    }
    if rel & 2 != 0 {
        b |= crate::input::k::D;
    }
    if rel & 4 != 0 {
        b |= back;
    }
    if rel & 8 != 0 {
        b |= fwd;
    }
    b
}
fn build_sequence(c: &CommandDef, facing: f32, gap: bool) -> Vec<u32> {
    let mut seq = Vec::with_capacity(c.steps.len() * 3 + 2);
    for (i, s) in c.steps.iter().enumerate() {
        let bits = absolute_dirs(s.dir, facing) | s.btns;
        let frames = if s.release {
            s.charge.max(0) as usize + 2
        } else if s.hold {
            3
        } else if s.btns != 0 {
            3
        } else {
            1
        };
        seq.extend(std::iter::repeat_n(bits, frames));
        if gap && i + 1 < c.steps.len() {
            seq.push(0);
        }
    }
    seq.push(0);
    seq
}
pub(super) fn sequence_matches(c: &CommandDef, seq: &[u32], facing: f32) -> bool {
    let mut h = InputHistory::default();
    let face = if facing >= 0.0 { 1 } else { -1 };
    for _ in 0..4 {
        h.push(0, face);
    }
    seq.iter().any(|b| {
        h.push(*b, face);
        match_command(&h, c)
    })
}

pub fn input_macro(c: &CommandDef, facing: f32) -> Vec<u32> {
    if c.steps.is_empty() {
        return vec![];
    }
    for gap in [false, true] {
        let seq = build_sequence(c, facing, gap);
        if sequence_matches(c, &seq, facing) {
            return seq;
        }
    }
    vec![]
}
