use super::spec::{ExactAnim, ExactFrame};
use std::collections::{BTreeMap, HashMap};
use ume_formats::air::{AnimFrame, Animation, Blend};

fn blend_name(b: Blend) -> &'static str {
    match b {
        Blend::None => "",
        Blend::Add => "A",
        Blend::Add1 => "A1",
        Blend::Sub => "S",
    }
}

fn blend_of(name: &str) -> Blend {
    match name {
        "A" => Blend::Add,
        "A1" => Blend::Add1,
        "S" => Blend::Sub,
        _ => Blend::None,
    }
}

pub fn frame_of(f: &AnimFrame) -> ExactFrame {
    let flip = format!("{}{}", if f.flip_h { "H" } else { "" }, if f.flip_v { "V" } else { "" });
    ExactFrame {
        g: f.group,
        i: f.index,
        x: f.x,
        y: f.y,
        t: f.time,
        flip,
        blend: blend_name(f.blend).into(),
        alpha: f.alpha.map(|(s, d)| [s, d]),
        scale: (f.xscale != 1.0 || f.yscale != 1.0).then_some([f.xscale, f.yscale]),
        angle: f.angle,
        clsn1: f.clsn1.clone(),
        clsn2: f.clsn2.clone(),
    }
}

pub fn anim_frame(f: &ExactFrame) -> AnimFrame {
    let [xscale, yscale] = f.scale.unwrap_or([1.0, 1.0]);
    AnimFrame {
        group: f.g,
        index: f.i,
        x: f.x,
        y: f.y,
        time: f.t,
        flip_h: f.flip.contains('H'),
        flip_v: f.flip.contains('V'),
        blend: blend_of(&f.blend),
        alpha: f.alpha.map(|[s, d]| (s, d)),
        xscale,
        yscale,
        angle: f.angle,
        clsn1: f.clsn1.clone(),
        clsn2: f.clsn2.clone(),
    }
}

pub fn export_anims(anims: &HashMap<i32, Animation>) -> BTreeMap<i32, ExactAnim> {
    anims.iter().map(|(&id, a)| (id, ExactAnim { loop_start: a.loop_start, frames: a.frames.iter().map(frame_of).collect() })).collect()
}

pub fn import_anims(anims: &BTreeMap<i32, ExactAnim>) -> HashMap<i32, Animation> {
    anims
        .iter()
        .map(|(&id, a)| {
            let mut an = Animation { id, frames: a.frames.iter().map(anim_frame).collect(), loop_start: a.loop_start, total_time: 0 };
            an.compute_total();
            (id, an)
        })
        .collect()
}
