use super::{parse_route, parse_unlocks, UnlockRules};
use std::sync::OnceLock;

const BUNDLED_UNLOCKS: &str = include_str!("../../assets/routes/unlocks.toml");

pub fn route_text(character_id: &str) -> Option<String> {
    crate::storage::read(&format!("route_{character_id}.toml"))
}

pub fn load_route(character_id: &str) -> Option<super::plan::Route> {
    let text = route_text(character_id)?;
    match parse_route(&text) {
        Ok(route) => Some(route),
        Err(error) => {
            eprintln!("route {character_id}: {error}");
            None
        }
    }
}

fn parse_or_report(label: &str, text: &str) -> Option<UnlockRules> {
    match parse_unlocks(text) {
        Ok(rules) => Some(rules),
        Err(error) => {
            eprintln!("{label}: {error}");
            None
        }
    }
}

pub fn unlock_rules() -> &'static UnlockRules {
    static RULES: OnceLock<UnlockRules> = OnceLock::new();
    RULES.get_or_init(|| {
        let user = crate::storage::read("unlocks.toml")
            .and_then(|text| parse_or_report("unlocks.toml", &text));
        user.or_else(|| parse_or_report("bundled unlocks", BUNDLED_UNLOCKS))
            .unwrap_or_default()
    })
}
