use super::model::{Labels, Tweaks};
use super::parse::{parse, DEFAULT_TEXT};
use crate::settings::Rules;
use std::sync::{Mutex, OnceLock};

const FILE: &str = "tweaks.toml";
const POLL_SECONDS: f32 = 1.0;

struct State {
    tweaks: Tweaks,
    labels: Labels,
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
        let (tweaks, labels, warnings) = parse(text.as_deref().unwrap_or(DEFAULT_TEXT));
        warnings.iter().for_each(|w| eprintln!("tweaks: {w}"));
        Mutex::new(State { tweaks, labels, last_text: text.or_else(|| Some(DEFAULT_TEXT.into())), idle: 0.0 })
    })
}

pub fn current() -> Tweaks {
    state().lock().map(|s| s.tweaks).unwrap_or_default()
}

pub fn label(key: &str) -> Option<String> {
    let s = state().lock().ok()?;
    s.labels.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone())
}

pub fn apply_rules(mut rules: Rules) -> Rules {
    let o = current().rules;
    rules.rounds_to_win = o.rounds_to_win.unwrap_or(rules.rounds_to_win);
    rules.round_ticks = o.round_seconds.map_or(rules.round_ticks, |s| s as i32 * 60);
    rules.life_percent = o.life_percent.unwrap_or(rules.life_percent);
    rules.game_speed = o.game_speed.unwrap_or(rules.game_speed);
    rules
}

fn refresh(s: &mut State) -> Option<String> {
    let text = crate::storage::read(FILE);
    if text == s.last_text {
        return None;
    }
    let (tweaks, labels, warnings) = parse(text.as_deref().unwrap_or(DEFAULT_TEXT));
    s.tweaks = tweaks;
    s.labels = labels;
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
