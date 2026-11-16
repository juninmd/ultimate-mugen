use crate::gfx::*;
use crate::ui::{hex, Align};
use std::sync::atomic::{AtomicBool, Ordering};
use ume_engine::fighter::Fighter;
use ume_engine::matchs::Match;

static ENABLED: AtomicBool = AtomicBool::new(false);

const PANEL_W: f32 = 112.0;
const LINE_H: f32 = 6.6;
const TOP: f32 = 74.0;
const MARGIN: f32 = 4.0;
const MAX_LISTED_VARS: usize = 6;

pub fn set(on: bool) {
    ENABLED.store(on, Ordering::Relaxed);
}

pub struct Snapshot {
    pub state_no: i32,
    pub prev_state_no: i32,
    pub state_time: i32,
    pub anim_no: i32,
    pub elem: usize,
    pub elem_time: i32,
    pub anim_time: i32,
    pub state_type: char,
    pub move_type: char,
    pub ctrl: bool,
    pub life: f32,
    pub power: f32,
    pub pos: (f32, f32),
    pub vel: (f32, f32),
    pub vars: Vec<(usize, i32)>,
    pub fvars: Vec<(usize, f64)>,
    pub sysvars: [i32; 5],
}

pub fn snapshot(f: &Fighter) -> Snapshot {
    Snapshot {
        state_no: f.state_no,
        prev_state_no: f.prev_state_no,
        state_time: f.state_time,
        anim_no: f.anim_no,
        elem: f.elem + 1,
        elem_time: f.elem_time,
        anim_time: f.anim_time_trig(),
        state_type: f.state_type,
        move_type: f.move_type,
        ctrl: f.ctrl,
        life: f.life,
        power: f.power,
        pos: (f.pos.x, f.pos.y),
        vel: (f.vel.x, f.vel.y),
        vars: f
            .vars
            .iter()
            .copied()
            .enumerate()
            .filter(|(_, v)| *v != 0)
            .collect(),
        fvars: f
            .fvars
            .iter()
            .copied()
            .enumerate()
            .filter(|(_, v)| *v != 0.0)
            .collect(),
        sysvars: f.sysvars,
    }
}

fn listed<T: std::fmt::Display>(prefix: &str, items: &[(usize, T)]) -> String {
    if items.is_empty() {
        return format!("{prefix}: -");
    }
    let shown: Vec<String> = items
        .iter()
        .take(MAX_LISTED_VARS)
        .map(|(i, v)| format!("{i}={v}"))
        .collect();
    let more = items.len().saturating_sub(MAX_LISTED_VARS);
    let tail = if more > 0 {
        format!(" +{more}")
    } else {
        String::new()
    };
    format!("{prefix}: {}{tail}", shown.join(" "))
}

pub fn lines(s: &Snapshot) -> Vec<String> {
    vec![
        format!(
            "state {} (prev {}) t{}",
            s.state_no, s.prev_state_no, s.state_time
        ),
        format!(
            "anim {} elem {} t{} AnimTime {}",
            s.anim_no, s.elem, s.elem_time, s.anim_time
        ),
        format!(
            "{}/{} ctrl {}",
            s.state_type,
            s.move_type,
            if s.ctrl { "1" } else { "0" }
        ),
        format!("life {:.0} power {:.0}", s.life, s.power),
        format!(
            "pos {:.1},{:.1} vel {:.2},{:.2}",
            s.pos.0, s.pos.1, s.vel.0, s.vel.1
        ),
        listed("var", &s.vars),
        listed(
            "fvar",
            &s.fvars
                .iter()
                .map(|(i, v)| (*i, format!("{v:.2}")))
                .collect::<Vec<_>>(),
        ),
        format!("sysvar {:?}", s.sysvars),
    ]
}

pub fn draw<P: Fn(f32, f32) -> Vec2, T: Fn(&str, f32, f32, f32, Color, Align, bool)>(
    p: &P,
    text: &T,
    s: f32,
    w: f32,
    m: &Match,
) {
    if !ENABLED.load(Ordering::Relaxed) {
        return;
    }
    for (slot, x) in [(0usize, MARGIN), (1usize, w - PANEL_W - MARGIN)] {
        let Some(fighter) = m.fighters.get(slot) else {
            continue;
        };
        let rows = lines(&snapshot(fighter));
        let corner = p(x, TOP);
        draw_rectangle(
            corner.x,
            corner.y,
            PANEL_W * s,
            (rows.len() as f32 * LINE_H + MARGIN) * s,
            Color::new(0.02, 0.02, 0.06, 0.62),
        );
        for (i, row) in rows.iter().enumerate() {
            text(
                row,
                x + 3.0,
                TOP + 3.0 + i as f32 * LINE_H,
                4.4,
                hex("#7ff0ff"),
                Align::Left,
                false,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Snapshot {
        Snapshot {
            state_no: 200,
            prev_state_no: 0,
            state_time: 12,
            anim_no: 200,
            elem: 3,
            elem_time: 4,
            anim_time: -2,
            state_type: 'S',
            move_type: 'A',
            ctrl: false,
            life: 900.0,
            power: 1500.0,
            pos: (-40.0, 0.0),
            vel: (2.5, 0.0),
            vars: (0..8).map(|i| (i, i as i32 + 1)).collect(),
            fvars: vec![(1, 0.5)],
            sysvars: [0; 5],
        }
    }

    #[test]
    fn lists_state_anim_and_triggers() {
        let rows = lines(&sample());
        assert_eq!(rows[0], "state 200 (prev 0) t12");
        assert_eq!(rows[1], "anim 200 elem 3 t4 AnimTime -2");
        assert_eq!(rows[2], "S/A ctrl 0");
    }

    #[test]
    fn truncates_long_variable_lists() {
        let rows = lines(&sample());
        assert_eq!(rows[5], "var: 0=1 1=2 2=3 3=4 4=5 5=6 +2");
        assert_eq!(rows[6], "fvar: 1=0.50");
    }

    #[test]
    fn empty_variables_show_a_dash() {
        let mut s = sample();
        s.vars.clear();
        s.fvars.clear();
        let rows = lines(&s);
        assert_eq!(rows[5], "var: -");
        assert_eq!(rows[6], "fvar: -");
    }
}
