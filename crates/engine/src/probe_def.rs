use crate::chardef::CharDef;
use ume_formats::air::{AnimFrame, Animation, Blend};

pub fn probe_def() -> CharDef {
    let mut d = CharDef::empty("diagnostics-probe");
    let mut a = Animation {
        id: 0,
        frames: vec![AnimFrame {
            group: 0,
            index: 0,
            x: 0.0,
            y: 0.0,
            time: 1,
            flip_h: false,
            flip_v: false,
            blend: Blend::None,
            alpha: None,
            xscale: 1.0,
            yscale: 1.0,
            angle: 0.0,
            clsn1: vec![],
            clsn2: vec![],
        }],
        loop_start: 0,
        total_time: 0,
    };
    a.compute_total();
    d.anims.insert(0, a);
    d
}
