use crate::audio::Audio;
use crate::sound_mix::{decode, Pcm};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Mutex, OnceLock};
use ume_engine::matchs::Match;

static BANK: &[u8] = include_bytes!("../assets/voices/voices.bank");
static GAIN: AtomicU32 = AtomicU32::new(0x3f80_0000);
static CLIPS: Mutex<Option<HashMap<String, Vec<Option<Pcm>>>>> = Mutex::new(None);
static INDEX: OnceLock<HashMap<String, Bank>> = OnceLock::new();

struct Bank {
    ogg: &'static [u8],
    spans: HashMap<String, (u32, u32, i32)>,
}

pub fn set_gain(gain: f32) {
    GAIN.store(gain.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
}

fn gain() -> f32 {
    f32::from_bits(GAIN.load(Ordering::Relaxed))
}

fn take<'a>(data: &mut &'a [u8], n: usize) -> Option<&'a [u8]> {
    let (head, tail) = data.split_at_checked(n)?;
    *data = tail;
    Some(head)
}

fn number(data: &mut &[u8], n: usize) -> Option<u32> {
    let b = take(data, n)?;
    Some(b.iter().rev().fold(0, |acc, &x| acc << 8 | x as u32))
}

fn text(data: &mut &[u8]) -> Option<String> {
    let n = number(data, 1)? as usize;
    Some(String::from_utf8_lossy(take(data, n)?).into_owned())
}

fn parse(bytes: &'static [u8]) -> Option<HashMap<String, Bank>> {
    let mut data = bytes;
    (take(&mut data, 4)? == b"UVB2").then_some(())?;
    number(&mut data, 4)?;
    let banks = number(&mut data, 4)?;
    let mut out = HashMap::new();
    for _ in 0..banks {
        let name = text(&mut data)?;
        let count = number(&mut data, 2)?;
        let mut spans = HashMap::new();
        for index in 0..count {
            let id = text(&mut data)?;
            let start = number(&mut data, 4)?;
            let len = number(&mut data, 4)?;
            spans.insert(id, (start, len, index as i32));
        }
        let size = number(&mut data, 4)? as usize;
        let ogg = take(&mut data, size)?;
        out.insert(name, Bank { ogg, spans });
    }
    Some(out)
}

fn bank_name(def_id: &str) -> &str {
    match def_id.trim_start_matches("umf:") {
        "npc_brute" => "bruno",
        "npc_razor" => "suzume",
        "npc_spitfire" => "akio",
        "npc_colossus" => "daichi",
        "npc_phantom" => "jin",
        other => other,
    }
}

fn move_key(command: &str) -> &str {
    let id = command
        .strip_prefix("m_")
        .or_else(|| command.strip_prefix("fin_"))
        .unwrap_or(command);
    id.strip_prefix("ex_").unwrap_or(id)
}

fn clip(name: &str, id: &str) -> Option<(i32, Pcm)> {
    let bank = INDEX.get_or_init(|| parse(BANK).unwrap_or_default()).get(name)?;
    let &(start, len, index) = bank.spans.get(id)?;
    let mut cache = CLIPS.lock().ok()?;
    let clips = cache.get_or_insert_with(HashMap::new);
    if !clips.contains_key(name) {
        let whole = decode(bank.ogg)?;
        let slots = bank.spans.values().map(|s| s.2).max().map_or(0, |m| m as usize + 1);
        let mut slice = vec![None; slots];
        for &(s, l, i) in bank.spans.values() {
            let frames = whole.frames.get(s as usize..(s + l) as usize)?;
            slice[i as usize] = Some(Pcm { rate: whole.rate, frames: frames.to_vec() });
        }
        clips.insert(name.to_string(), slice);
    }
    clips.get(name)?.get(index as usize)?.clone().map(|p| (index, p))
}

pub fn on_state_change(m: &Match, audio: &mut Audio, fighter: u32) {
    let Some(i) = m.idx(fighter) else { return };
    let f = &m.fighters[i];
    if f.state_no == 5400 {
        audio.sfx("guardbreak");
    }
    if gain() <= 0.0 || f.state_no < 1000 {
        return;
    }
    let Some(info) = f.def.movelist.iter().find(|mv| mv.state == f.state_no) else { return };
    let name = bank_name(&f.def.id);
    if let Some((index, pcm)) = clip(name, move_key(&info.command)) {
        audio.play_clip(name, index, &pcm, fighter, gain());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bank_parses_and_decodes_every_clip() {
        let banks = parse(BANK).expect("bank");
        assert!(banks.len() >= 16);
        for (name, bank) in &banks {
            let pcm = decode(bank.ogg).expect("ogg");
            for (id, &(start, len, _)) in &bank.spans {
                assert!(
                    (start + len) as usize <= pcm.frames.len(),
                    "{name}/{id} overruns the stream"
                );
                assert!(len > 2000, "{name}/{id} too short");
            }
        }
    }

    #[test]
    fn move_commands_map_to_clip_ids() {
        assert_eq!(move_key("m_ex_gale"), "gale");
        assert_eq!(move_key("fin_finish"), "finish");
        assert_eq!(bank_name("umf:npc_brute"), "bruno");
    }
}
