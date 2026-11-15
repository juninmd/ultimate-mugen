use std::cell::RefCell;
use ume_engine::input::k;

pub const MAX_FRAMES: usize = 600;

#[derive(Default)]
struct Tape {
    frames: Vec<u32>,
    recording: bool,
    cursor: usize,
}

thread_local! {
    static TAPE: RefCell<Tape> = RefCell::new(Tape::default());
}

fn face_forward(bits: u32, facing: f32) -> u32 {
    if facing >= 0.0 {
        return bits;
    }
    let (left, right) = (bits & k::L != 0, bits & k::R != 0);
    let rest = bits & !(k::L | k::R);
    rest | if left { k::R } else { 0 } | if right { k::L } else { 0 }
}

pub fn start_recording() {
    TAPE.with(|t| {
        let mut t = t.borrow_mut();
        t.frames.clear();
        t.recording = true;
        t.cursor = 0;
    });
}

pub fn stop_recording() {
    TAPE.with(|t| {
        let mut t = t.borrow_mut();
        t.recording = false;
        t.cursor = 0;
    });
}

pub fn record(bits: u32, facing: f32) -> u32 {
    TAPE.with(|t| {
        let mut t = t.borrow_mut();
        if t.recording && t.frames.len() < MAX_FRAMES {
            t.frames.push(face_forward(bits, facing));
        }
    });
    bits
}

pub fn playback(facing: f32) -> u32 {
    TAPE.with(|t| {
        let mut t = t.borrow_mut();
        if t.frames.is_empty() {
            return 0;
        }
        let bits = t.frames[t.cursor % t.frames.len()];
        t.cursor = (t.cursor + 1) % t.frames.len();
        face_forward(bits, facing)
    })
}

pub fn len() -> usize {
    TAPE.with(|t| t.borrow().frames.len())
}

pub fn is_recording() -> bool {
    TAPE.with(|t| t.borrow().recording)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recorded_inputs_replay_mirrored_for_a_rival_facing_the_other_way() {
        start_recording();
        record(k::R | k::X, 1.0);
        record(k::D, 1.0);
        stop_recording();
        assert_eq!(len(), 2);
        assert_eq!(playback(-1.0), k::L | k::X);
        assert_eq!(playback(-1.0), k::D);
        assert_eq!(playback(1.0), k::R | k::X);
    }

    #[test]
    fn recording_stops_at_the_frame_cap_and_idle_tape_is_silent() {
        start_recording();
        for _ in 0..MAX_FRAMES + 50 {
            record(k::X, 1.0);
        }
        stop_recording();
        assert_eq!(len(), MAX_FRAMES);
        start_recording();
        stop_recording();
        assert_eq!(playback(1.0), 0);
    }
}
