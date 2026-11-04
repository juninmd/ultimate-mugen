use crate::chardef::Controller;
use crate::hitdef::parse_sound_ref;
use crate::matchs::{Event, Match, PauseState};

pub fn pause(m: &mut Match, i: usize, sc: &Controller) {
    let time = m.num(i, sc, "time", 0.0, 0) as i32;
    let move_time = m.num(i, sc, "movetime", 0.0, 0) as i32;
    let end_cmd_buf_time = m.num(i, sc, "endcmdbuftime", 0.0, 0) as i32;
    let pause_bg = m.num(i, sc, "pausebg", 1.0, 0) != 0.0;
    let by = m.fighters[i].id;
    m.pause = Some(PauseState { by, time, move_time, end_cmd_buf_time, pause_bg, ..Default::default() });
}

pub fn super_pause(m: &mut Match, i: usize, sc: &Controller) {
    let n = |m: &Match, k: &str, d: f64, idx: usize| m.num(i, sc, k, d, idx);
    let l = m.fighters[i].lscl;
    let time = n(m, "time", 30.0, 0) as i32;
    let move_time = n(m, "movetime", 0.0, 0) as i32;
    let darken = n(m, "darken", 1.0, 0) != 0.0;
    let unhittable = n(m, "unhittable", 1.0, 0) != 0.0;
    let p2_def_mul = n(m, "p2defmul", 1.5, 0) as f32;
    let end_cmd_buf_time = n(m, "endcmdbuftime", 0.0, 0) as i32;
    let by = m.fighters[i].id;
    m.pause = Some(PauseState { by, time, move_time, is_super: true, darken, unhittable, p2_def_mul, end_cmd_buf_time, pause_bg: true });
    if unhittable { m.fighters[i].unhittable = time + move_time.max(0); }
    if p2_def_mul > 0.0 && p2_def_mul != 1.0 {
        for t in m.fighters[i].targets.iter().filter_map(|t| m.idx(t.0)).collect::<Vec<_>>() { m.fighters[t].super_defence_mul = p2_def_mul; }
    }
    let power = n(m, "poweradd", 0.0, 0) as f32;
    m.add_power(i, power);
    let anim = sc.raw("anim").map(|s| s.trim().to_string());
    let own = anim.as_ref().is_some_and(|a| a.starts_with(['s', 'S']));
    let no = anim.as_ref().and_then(|a| a.trim_start_matches(['s', 'S', 'f', 'F']).trim().parse::<i32>().ok()).unwrap_or(30);
    let (px, py) = (n(m, "pos", 0.0, 0) as f32 * l, n(m, "pos", 0.0, 1) as f32 * l);
    if no >= 0 { m.spawn_super_fx(i, px, py, if own { no } else { -1 }); }
    let snd = sc.raw("sound").map(|s| s.to_string());
    let pan = crate::sctrl::sound_pan(m, i, 0.0, false);
    let f = &m.fighters[i];
    let ev = match parse_sound_ref(snd.as_deref()) {
        Some((g, idx, own)) if g >= 0 => Event::Sound { def: Some(f.def.clone()), group: g, index: idx, own, volume: 1.0, channel: -1, owner: f.id, builtin: None, pan, looping: false, freq_mul: 1.0, low_priority: false },
        _ => Event::Sound { def: None, group: -1, index: 0, own: false, volume: 1.0, channel: -1, owner: f.id, builtin: Some("super"), pan, looping: false, freq_mul: 1.0, low_priority: false },
    };
    let (side, fighter) = (f.side, f.root);
    m.events.push(ev);
    m.events.push(Event::Super { side, fighter });
}
