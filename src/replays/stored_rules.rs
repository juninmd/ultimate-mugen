use serde_json::{json, Map, Value};
use ume_engine::matchs::{KnockdownRule, KnockdownScope};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StoredRules {
    pub rounds_to_win: i32,
    pub round_ticks: i32,
    pub life_percent: u32,
    pub single_round: bool,
    pub classic_pushbox: bool,
    pub knockdown: KnockdownRule,
    pub team_life_share: bool,
    pub game_speed: f32,
}

impl Default for StoredRules {
    fn default() -> Self {
        StoredRules {
            rounds_to_win: 0,
            round_ticks: 0,
            life_percent: 0,
            single_round: false,
            classic_pushbox: false,
            knockdown: KnockdownRule::OFF,
            team_life_share: false,
            game_speed: 0.0,
        }
    }
}

fn scope_name(scope: KnockdownScope) -> &'static str {
    match scope {
        KnockdownScope::Off => "off",
        KnockdownScope::Native => "native",
        KnockdownScope::All => "all",
    }
}

fn scope_of(name: &str) -> KnockdownScope {
    match name {
        "native" => KnockdownScope::Native,
        "all" => KnockdownScope::All,
        _ => KnockdownScope::Off,
    }
}

fn int(o: Option<&Map<String, Value>>, key: &str, default: i32) -> i32 {
    o.and_then(|o| o.get(key)).and_then(Value::as_i64).map(|v| v as i32).unwrap_or(default)
}

fn flag(o: Option<&Map<String, Value>>, key: &str, default: bool) -> bool {
    o.and_then(|o| o.get(key)).and_then(Value::as_bool).unwrap_or(default)
}

fn real(o: Option<&Map<String, Value>>, key: &str, default: f32) -> f32 {
    o.and_then(|o| o.get(key)).and_then(Value::as_f64).map(|f| f as f32).unwrap_or(default)
}

impl StoredRules {
    pub fn to_json(&self) -> Value {
        json!({
            "rounds_to_win": self.rounds_to_win,
            "round_ticks": self.round_ticks,
            "life_percent": self.life_percent,
            "single_round": self.single_round,
            "classic_pushbox": self.classic_pushbox,
            "knockdown_scope": scope_name(self.knockdown.scope),
            "knockdown_threshold": self.knockdown.threshold_percent,
            "team_life_share": self.team_life_share,
            "game_speed": self.game_speed,
        })
    }

    pub fn from_json(r: Option<&Map<String, Value>>) -> Self {
        let scope = scope_of(r.and_then(|r| r.get("knockdown_scope")).and_then(Value::as_str).unwrap_or("off"));
        StoredRules {
            rounds_to_win: int(r, "rounds_to_win", 2),
            round_ticks: int(r, "round_ticks", 99 * 60),
            life_percent: int(r, "life_percent", 100).max(0) as u32,
            single_round: flag(r, "single_round", false),
            classic_pushbox: flag(r, "classic_pushbox", true),
            knockdown: KnockdownRule { scope, threshold_percent: real(r, "knockdown_threshold", 0.0) },
            team_life_share: flag(r, "team_life_share", true),
            game_speed: real(r, "game_speed", 1.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replays_saved_before_the_rule_existed_replay_with_classic_knockdowns() {
        let old = json!({ "rounds_to_win": 2, "classic_pushbox": true });
        let rules = StoredRules::from_json(old.as_object());
        assert_eq!(rules.knockdown.scope, KnockdownScope::Off);
    }

    #[test]
    fn the_knockdown_rule_survives_a_save_and_load() {
        let rules = StoredRules { knockdown: KnockdownRule { scope: KnockdownScope::All, threshold_percent: 24.0 }, ..StoredRules::from_json(None) };
        let back = StoredRules::from_json(rules.to_json().as_object());
        assert_eq!(back.knockdown, rules.knockdown);
    }
}
