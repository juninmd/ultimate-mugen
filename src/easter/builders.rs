use super::{ActiveEgg, EggSprite, EggStyle};

pub(crate) fn ribbon(sprite: EggSprite, title: &str, sub: &str, color: [f32; 3], side: usize) -> ActiveEgg {
    ActiveEgg {
        style: EggStyle::TopRibbon,
        sprite,
        title: title.to_string(),
        sub: sub.to_string(),
        speaker: String::new(),
        color,
        side,
        t: 0.0,
        dur: 2.8,
        flip: false,
    }
}

pub(crate) fn corner_pop(sprite: EggSprite, title: &str, sub: &str, color: [f32; 3]) -> ActiveEgg {
    ActiveEgg {
        style: EggStyle::CornerPop,
        sprite,
        title: title.to_string(),
        sub: sub.to_string(),
        speaker: String::new(),
        color,
        side: 1,
        t: 0.0,
        dur: 2.3,
        flip: false,
    }
}

pub(crate) fn floor_roll(sprite: EggSprite, title: &str, sub: &str, color: [f32; 3], flip: bool) -> ActiveEgg {
    ActiveEgg {
        style: EggStyle::FloorRoll,
        sprite,
        title: title.to_string(),
        sub: sub.to_string(),
        speaker: String::new(),
        color,
        side: 0,
        t: 0.0,
        dur: 3.4,
        flip,
    }
}

pub(crate) fn flyby(sprite: EggSprite, title: &str, sub: &str, color: [f32; 3], flip: bool) -> ActiveEgg {
    ActiveEgg {
        style: EggStyle::Flyby,
        sprite,
        title: title.to_string(),
        sub: sub.to_string(),
        speaker: String::new(),
        color,
        side: 0,
        t: 0.0,
        dur: 3.8,
        flip,
    }
}

pub(crate) fn speech(speaker: &str, line: &str, color: [f32; 3], side: usize) -> ActiveEgg {
    ActiveEgg {
        style: EggStyle::SpeechBubble,
        sprite: EggSprite::None,
        title: line.to_string(),
        sub: String::new(),
        speaker: speaker.to_string(),
        color,
        side,
        t: 0.0,
        dur: 2.9,
        flip: side == 1,
    }
}
