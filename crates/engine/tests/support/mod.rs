#![allow(dead_code)]
use std::collections::HashMap;
use std::sync::Arc;
use ume_engine::chardef::{AnimFrame, Animation, Box4, CharDef, Source};
use ume_engine::cns::parse_states;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::matchs::Match;
use ume_formats::air::Blend;

pub fn frame(time: i32, clsn1: Vec<Box4>, clsn2: Vec<Box4>) -> AnimFrame {
    AnimFrame {
        group: 0,
        index: 0,
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
        clsn1,
        clsn2,
    }
}

pub fn anim(id: i32, frames: Vec<AnimFrame>) -> Animation {
    let mut a = Animation {
        id,
        frames,
        loop_start: 0,
        total_time: 0,
    };
    a.compute_total();
    a
}

pub const HURT: Box4 = [-15.0, -70.0, 15.0, 0.0];
pub const REACH: Box4 = [0.0, -60.0, 60.0, -30.0];

pub fn mugen_char(name: &str, cns: &str) -> Arc<CharDef> {
    let mut d = CharDef::empty(name);
    d.display_name = name.into();
    d.source = Source::Mugen;
    d.anims
        .insert(0, anim(0, vec![frame(-1, vec![], vec![HURT])]));
    d.anims.insert(
        200,
        anim(
            200,
            vec![
                frame(3, vec![], vec![HURT]),
                frame(6, vec![REACH], vec![HURT]),
                frame(6, vec![], vec![HURT]),
            ],
        ),
    );
    d.anims.insert(
        300,
        anim(
            300,
            vec![
                frame(3, vec![], vec![HURT]),
                frame(30, vec![REACH], vec![HURT]),
            ],
        ),
    );
    d.anims
        .insert(5000, anim(5000, vec![frame(-1, vec![], vec![HURT])]));
    d.anims.insert(
        1000,
        anim(
            1000,
            vec![frame(
                -1,
                vec![[-10.0, -10.0, 10.0, 10.0]],
                vec![[-10.0, -10.0, 10.0, 10.0]],
            )],
        ),
    );
    let mut states = HashMap::new();
    parse_states(cns, &mut states, true);
    d.states = states;
    Arc::new(d)
}

pub fn native_char(name: &str, cns: &str) -> Arc<CharDef> {
    let mut d = Arc::try_unwrap(mugen_char(name, cns)).ok().unwrap();
    d.source = Source::Native;
    Arc::new(d)
}

pub fn duel(a: Arc<CharDef>, b: Arc<CharDef>) -> Match {
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(a, 0, 0, 1);
    m.add_player(b, 1, 0, 2);
    m.start_round();
    while m.round_state != 2 {
        m.tick(&[0, 0]);
    }
    m
}

pub fn ticks(m: &mut Match, n: usize) {
    for _ in 0..n {
        m.tick(&[0, 0]);
    }
}

pub fn face_off(m: &mut Match, gap: f32) {
    let c = m.camera.x;
    m.fighters[0].pos.x = c - gap / 2.0;
    m.fighters[1].pos.x = c + gap / 2.0;
    m.fighters[0].facing = 1.0;
    m.fighters[1].facing = -1.0;
}

pub fn enter(m: &mut Match, i: usize, state: i32) {
    let d = m.fighters[i].def.clone();
    m.change_state(i, state, d);
}
