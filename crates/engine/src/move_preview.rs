use crate::chardef::{Animation, CharDef, MoveInfo, StateDef};
use crate::move_cost::{literal, target_state};

fn first_literal(sd: &StateDef, key: &str) -> Option<i32> {
    sd.param(key)
        .and_then(|p| p.exprs.first())
        .and_then(|e| literal(&e.node))
        .map(|v| v as i32)
}

fn state_anim(sd: &StateDef) -> Option<i32> {
    first_literal(sd, "anim").or_else(|| {
        sd.controllers
            .iter()
            .filter(|c| c.kind == "changeanim")
            .find_map(|c| {
                c.param("value")
                    .and_then(|p| p.exprs.first())
                    .and_then(|e| literal(&e.node))
                    .map(|v| v as i32)
            })
    })
}

pub fn anim_id(def: &CharDef, mv: &MoveInfo) -> Option<i32> {
    let state = if mv.state >= 0 {
        mv.state
    } else {
        target_state(&def.states, &mv.command)
            .filter(|&s| def.states.contains_key(&s))
            .unwrap_or(0)
    };
    // Se o estado caiu para 0, certifique-se de que ele existe; caso contrário, tente o próximo estado válido
    let state = if state == 0 && !def.states.contains_key(&0) {
        // Procura pelo primeiro estado positivo presente
        def.states.keys().find(|&&s| s > 0).copied().unwrap_or(0)
    } else {
        state
    };
    state_anim(def.states.get(&state)?)
}

pub fn preview_anim<'a>(def: &'a CharDef, mv: &MoveInfo) -> Option<&'a Animation> {
    let anim = def.anims.get(&anim_id(def, mv)?)?;
    (!anim.frames.is_empty()).then_some(anim)
}

pub fn frame_at(anim: &Animation, ticks: u32) -> usize {
    const HOLD: i32 = 24;
    let span = |t: i32| if t < 0 { HOLD } else { t.max(1) } as u32;
    let total: u32 = anim.frames.iter().map(|f| span(f.time)).sum::<u32>().max(1);
    let mut left = ticks % total;
    for (i, f) in anim.frames.iter().enumerate() {
        let n = span(f.time);
        if left < n {
            return i;
        }
        left -= n;
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chardef::{AnimFrame, Blend};

    fn states(text: &str) -> std::collections::HashMap<i32, StateDef> {
        let mut m = std::collections::HashMap::new();
        crate::cns::parse_states(text, &mut m, true);
        m
    }

    #[test]
    fn anim_follows_the_command_target_state() {
        let s = states("[Statedef -1]\n[State -1, a]\ntype = ChangeState\ntrigger1 = command = \"QCF_x\"\nvalue = 210\n\n[Statedef 210]\ntype = S\nanim = 1230\n");
        let target = target_state(&s, "QCF_x").unwrap();
        assert_eq!(state_anim(&s[&target]), Some(1230));
    }

    #[test]
    fn anim_falls_back_to_the_first_change_anim() {
        let s = states("[Statedef 300]\ntype = S\n[State 300, a]\ntype = ChangeAnim\ntrigger1 = Time = 0\nvalue = 3100\n");
        assert_eq!(state_anim(&s[&300]), Some(3100));
    }

    #[test]
    fn frames_loop_by_their_tick_durations() {
        let mk = |time| AnimFrame {
            group: 0, index: 0, x: 0.0, y: 0.0, time, flip_h: false, flip_v: false,
            blend: Blend::None, alpha: None, xscale: 1.0, yscale: 1.0, angle: 0.0,
            clsn1: vec![], clsn2: vec![],
        };
        let a = Animation { id: 1, frames: vec![mk(2), mk(3), mk(-1)], loop_start: 0, total_time: -1 };
        let seq: Vec<usize> = (0..6).map(|t| frame_at(&a, t)).collect();
        assert_eq!(seq, vec![0, 0, 1, 1, 1, 2]);
        assert_eq!(frame_at(&a, 2 + 3 + 24), 0);
    }
}