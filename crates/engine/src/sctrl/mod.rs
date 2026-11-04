pub mod clsn;
pub mod debug;
mod hitmod;
mod pause;
mod state;
pub(crate) mod sound;

pub use debug::format_clipboard;
pub use sound::sound_pan;
use crate::chardef::Controller;
use crate::matchs::{Event, Match};

pub fn run(m: &mut Match, i: usize, sc: &Controller) -> bool {
    match sc.kind.as_str() {
        "null" => {}
        "runstate" => state::run_state(m, i, sc),
        "height" => state::height(m, i, sc),
        "gethitvarset" => hitmod::get_hit_var_set(m, i, sc),
        "modifyhitdef" => hitmod::modify_hitdef(m, i, sc),
        "modifyreversaldef" => hitmod::modify_reversaldef(m, i, sc),
        "overrideclsn" | "transformclsn" | "resizeclsn" => clsn::run(m, i, sc),
        "offset" => offset(m, i, sc),
        "victoryquote" => { let v = m.num(i, sc, "value", -1.0, 0) as i32; m.fighters[i].victory_quote = v; }
        "forcefeedback" => force_feedback(m, i, sc),
        "envshake" => env_shake(m, i, sc),
        "pause" => pause::pause(m, i, sc),
        "superpause" => pause::super_pause(m, i, sc),
        "stopsnd" => sound::stop(m, i, sc),
        "sndpan" => sound::pan(m, i, sc),
        "displaytoclipboard" => debug::display(m, i, sc, false),
        "appendtoclipboard" => debug::display(m, i, sc, true),
        "clearclipboard" => m.fighters[i].clipboard.clear(),
        _ => return false,
    }
    true
}

fn offset(m: &mut Match, i: usize, sc: &Controller) {
    let l = m.fighters[i].lscl;
    let x = if sc.has("x", 0) { Some(m.num(i, sc, "x", 0.0, 0) as f32 * l) } else { None };
    let y = if sc.has("y", 0) { Some(m.num(i, sc, "y", 0.0, 0) as f32 * l) } else { None };
    let f = &mut m.fighters[i];
    if let Some(x) = x { f.draw_offset.x = x; }
    if let Some(y) = y { f.draw_offset.y = y; }
}

fn force_feedback(m: &mut Match, i: usize, sc: &Controller) {
    let time = m.num(i, sc, "time", 60.0, 0) as i32;
    let ampl = m.num(i, sc, "ampl", 128.0, 0) as f32;
    let to_self = m.num(i, sc, "self", 1.0, 0) != 0.0;
    let waveform = sc.raw("waveform").unwrap_or("sine").trim().to_ascii_lowercase();
    let target = if to_self { Some(m.fighters[i].root) } else { m.enemy(i, 0).map(|e| m.fighters[e].id) };
    let Some(fighter) = target else { return };
    let strength = (ampl.abs() / 255.0).clamp(0.0, 1.0);
    let (strong, weak) = match waveform.as_str() { "square" | "sinesquare" => (strength, strength), "off" => (0.0, 0.0), _ => (strength * 0.5, strength) };
    m.events.push(Event::Rumble { fighter, strong, weak, ms: (time.max(0) as u32) * 1000 / 60 });
}

fn env_shake(m: &mut Match, i: usize, sc: &Controller) {
    let time = m.num(i, sc, "time", 1.0, 0) as i32;
    let freq = m.num(i, sc, "freq", 60.0, 0).clamp(0.0, 180.0) as f32;
    let ampl = m.num(i, sc, "ampl", -4.0, 0) as f32 * m.fighters[i].lscl;
    let phase = m.num(i, sc, "phase", if freq >= 90.0 { 90.0 } else { 0.0 }, 0) as f32;
    m.shake(time, freq, ampl, phase);
}
