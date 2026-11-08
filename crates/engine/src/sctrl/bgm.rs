use crate::chardef::Controller;
use crate::matchs::{BgmAction, Event, Match};

pub fn play_bgm(m: &mut Match, i: usize, sc: &Controller) {
    let Some(raw) = sc.raw("bgm") else { return };
    let file = ume_formats::ini::unquote(raw);
    if file.trim().is_empty() {
        stop_bgm(m);
        return;
    }
    let n = |m: &Match, k: &str, d: f64| m.num(i, sc, k, d, 0);
    let looping = n(m, "loop", 1.0) != 0.0;
    let volume = (n(m, "volume", 100.0) / 100.0).clamp(0.0, 2.0) as f32;
    let loop_start = n(m, "loopstart", 0.0).max(0.0) as usize;
    let loop_end = n(m, "loopend", 0.0).max(0.0) as usize;
    let freq_mul = n(m, "freqmul", 1.0).clamp(0.05, 8.0) as f32;
    m.events.push(Event::Bgm(BgmAction::Play { file, looping, volume, loop_start, loop_end, freq_mul }));
}

pub fn modify_bgm(m: &mut Match, i: usize, sc: &Controller) {
    let n = |m: &Match, k: &str, d: f64| m.num(i, sc, k, d, 0);
    if sc.has("volume", 0) {
        let volume = (n(m, "volume", 100.0) / 100.0).clamp(0.0, 2.0) as f32;
        m.events.push(Event::Bgm(BgmAction::Volume(volume)));
    }
    if sc.has("loop", 0) {
        let looping = n(m, "loop", 1.0) != 0.0;
        m.events.push(Event::Bgm(BgmAction::Loop(looping)));
    }
    if sc.has("loopstart", 0) || sc.has("loopend", 0) {
        let start = n(m, "loopstart", 0.0).max(0.0) as usize;
        let end = n(m, "loopend", 0.0).max(0.0) as usize;
        m.events.push(Event::Bgm(BgmAction::LoopPoints { start, end }));
    }
    if sc.has("freqmul", 0) {
        let rate = n(m, "freqmul", 1.0).clamp(0.05, 8.0) as f32;
        m.events.push(Event::Bgm(BgmAction::Rate(rate)));
    }
}

pub fn stop_bgm(m: &mut Match) {
    m.events.push(Event::Bgm(BgmAction::Stop));
}
