use super::parse::{parse, Tweaks, DEFAULT_TEXT};
use std::sync::{Mutex, OnceLock};

const FILE: &str = "tweaks.toml";
const POLL_SECONDS: f32 = 1.0;

struct State {
    tweaks: Tweaks,
    last_text: Option<String>,
    idle: f32,
}

fn state() -> &'static Mutex<State> {
    static STATE: OnceLock<Mutex<State>> = OnceLock::new();
    STATE.get_or_init(|| {
        let text = crate::storage::read(FILE);
        if text.is_none() {
            crate::storage::write(FILE, DEFAULT_TEXT);
        }
        let (tweaks, warnings) = parse(text.as_deref().unwrap_or(DEFAULT_TEXT));
        warnings.iter().for_each(|w| eprintln!("tweaks: {w}"));
        Mutex::new(State { tweaks, last_text: text.or_else(|| Some(DEFAULT_TEXT.into())), idle: 0.0 })
    })
}

pub fn current() -> Tweaks {
    state().lock().map(|s| s.tweaks).unwrap_or_default()
}

fn refresh(s: &mut State) -> Option<String> {
    let text = crate::storage::read(FILE);
    if text == s.last_text {
        return None;
    }
    let (tweaks, warnings) = parse(text.as_deref().unwrap_or(DEFAULT_TEXT));
    s.tweaks = tweaks;
    s.last_text = text;
    warnings.iter().for_each(|w| eprintln!("tweaks: {w}"));
    Some(match warnings.first() {
        Some(first) => format!("tweaks.toml: {first}"),
        None => "tweaks.toml reloaded".into(),
    })
}

pub fn poll(dt: f32) -> Option<String> {
    let mut s = state().lock().ok()?;
    s.idle += dt;
    if s.idle < POLL_SECONDS {
        return None;
    }
    s.idle = 0.0;
    refresh(&mut s)
}

pub fn reload_now() -> Option<String> {
    let mut s = state().lock().ok()?;
    refresh(&mut s)
}
