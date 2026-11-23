use ume_formats::air::{AnimFrame, Animation};

pub(super) fn frame_at(animation: &Animation, tick: i32) -> Option<&AnimFrame> {
    if animation.frames.is_empty() {
        return None;
    }
    let tick = if animation.total_time > 0 {
        tick.rem_euclid(animation.total_time)
    } else {
        tick.max(0)
    };
    let mut elapsed = 0;
    for frame in &animation.frames {
        if frame.time < 0 || tick < elapsed + frame.time {
            return Some(frame);
        }
        elapsed += frame.time;
    }
    animation.frames.last()
}

#[cfg(test)]
mod tests {
    use super::frame_at;
    use ume_formats::air::{AnimFrame, Animation, Blend};

    fn frame(index: i32, time: i32) -> AnimFrame {
        AnimFrame {
            group: 0,
            index,
            x: 0.0,
            y: 0.0,
            time,
            flip_h: false,
            flip_v: false,
            blend: Blend::None,
            alpha: None,
            xscale: 1.0,
            yscale: 1.0,
            angle: 0.0,
            clsn1: vec![],
            clsn2: vec![],
        }
    }

    #[test]
    fn storyboard_animation_loops_and_holds_infinite_frames() {
        let looping = Animation {
            id: 0,
            frames: vec![frame(0, 2), frame(1, 3)],
            loop_start: 0,
            total_time: 5,
        };
        assert_eq!(frame_at(&looping, 0).unwrap().index, 0);
        assert_eq!(frame_at(&looping, 2).unwrap().index, 1);
        assert_eq!(frame_at(&looping, 5).unwrap().index, 0);
        let hold = Animation {
            id: 1,
            frames: vec![frame(2, -1)],
            loop_start: 0,
            total_time: -1,
        };
        assert_eq!(frame_at(&hold, 900).unwrap().index, 2);
    }
}
