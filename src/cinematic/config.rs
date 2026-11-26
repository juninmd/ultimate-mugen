use super::beat::Beat;
use serde::Deserialize;
use std::sync::OnceLock;

const DEFAULT_TOML: &str = include_str!("../../assets/cinematic.toml");
const USER_FILE: &str = "cinematic.toml";

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
struct Raw {
    enabled: Option<bool>,
    intro: Vec<Beat>,
    victory: Vec<Beat>,
}

#[derive(Clone, Debug)]
pub struct Config {
    pub enabled: bool,
    pub intro: Vec<Beat>,
    pub victory: Vec<Beat>,
}

fn clean(beats: Vec<Beat>) -> Vec<Beat> {
    beats.into_iter().map(Beat::sanitized).collect()
}

impl Config {
    pub fn parse(base: &str, user: Option<&str>) -> Config {
        let raw: Raw = toml::from_str(base).unwrap_or_default();
        let mut config = Config {
            enabled: raw.enabled.unwrap_or(true),
            intro: clean(raw.intro),
            victory: clean(raw.victory),
        };
        if let Some(extra) = user.and_then(|text| toml::from_str::<Raw>(text).ok()) {
            config.enabled = extra.enabled.unwrap_or(config.enabled);
            if !extra.intro.is_empty() {
                config.intro = clean(extra.intro);
            }
            if !extra.victory.is_empty() {
                config.victory = clean(extra.victory);
            }
        }
        config
    }
}

pub fn config() -> &'static Config {
    static CONFIG: OnceLock<Config> = OnceLock::new();
    CONFIG.get_or_init(|| Config::parse(DEFAULT_TOML, crate::storage::read(USER_FILE).as_deref()))
}
