//! Shortcut slots: a move bound to keys 1-4 or L1/L2/R1/R2 runs its input macro, and the Mugen macros used by touch specials.
use crate::controls::Controls;
use ume_engine::commands::{input_macro, CommandDef};
use ume_engine::input::k;

/// Consumes the slot bits pressed this frame and queues the bound move's input frames for `side`.
pub(super) fn take_slots(
    c: &mut Controls,
    side: usize,
    bits: u32,
    name: &str,
    commands: &[CommandDef],
    facing: f32,
) -> u32 {
    let slots = bits & k::SLOT_MASK;
    let fresh = slots & !c.shortcuts.prev[side];
    c.shortcuts.prev[side] = slots;
    if let Some(slot) = k::SLOTS.iter().position(|bit| fresh & bit != 0) {
        if c.touch.macro_q[side].is_empty() {
            let frames = c
                .shortcuts
                .command(name, slot)
                .and_then(|cmd| commands.iter().find(|d| d.name == cmd))
                .map(|d| input_macro(d, facing))
                .unwrap_or_default();
            c.touch.macro_q[side].extend(frames);
        }
    }
    bits & !k::SLOT_MASK
}

pub(super) fn mugen_macro(facing: f32, which: u32) -> Vec<u32> {
    let (f, b, d) = (
        if facing >= 0.0 { k::R } else { k::L },
        if facing >= 0.0 { k::L } else { k::R },
        k::D,
    );
    let qcf = [d, d, d | f, d | f, f, f];
    match which {
        1 => {
            let mut v = qcf.to_vec();
            v.extend([f | k::X, 0]);
            v
        }
        2 => vec![f, f, d, d, d | f, d | f | k::X, 0],
        3 => vec![d, d, d | b, d | b, b, b | k::A, 0],
        _ => {
            let mut v = qcf.to_vec();
            v.push(0);
            v.extend(qcf);
            v.extend([f | k::X, 0]);
            v
        }
    }
}
