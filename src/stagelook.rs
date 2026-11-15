use crate::stages::StageKind;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::OnceLock;

const DEFAULT_TEXT: &str = include_str!("../assets/stagelook.toml");
const FILE: &str = "stagelook.toml";

static CURRENT: AtomicU8 = AtomicU8::new(6);

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(default)]
pub struct Look {
    pub rim: [f32; 3],
    pub rim_strength: f32,
    pub rim_side: f32,
    pub tint: [f32; 3],
    pub saturation: f32,
    pub vignette: f32,
    pub pulse: f32,
    pub pulse_period: f32,
}

impl Default for Look {
    fn default() -> Self {
        Look {
            rim: [1.0; 3],
            rim_strength: 0.0,
            rim_side: 1.0,
            tint: [1.0; 3],
            saturation: 0.0,
            vignette: 0.0,
            pulse: 0.0,
            pulse_period: 4.0,
        }
    }
}

type Table = HashMap<String, Look>;

fn parse(text: &str) -> Option<Table> {
    let mut table: toml::Table = toml::from_str(text).ok()?;
    let base: Look = table
        .remove("default")
        .and_then(|v| v.try_into().ok())
        .unwrap_or_default();
    let mut out = Table::new();
    for (name, value) in table {
        let Some(section) = value.as_table() else {
            continue;
        };
        let mut merged = toml::Table::new();
        if let Ok(b) = toml::Table::try_from(base) {
            merged.extend(b);
        }
        merged.extend(section.clone());
        if let Ok(look) = toml::Value::Table(merged).try_into::<Look>() {
            out.insert(name, look);
        }
    }
    Some(out)
}

fn table() -> &'static Table {
    static TABLE: OnceLock<Table> = OnceLock::new();
    TABLE.get_or_init(|| {
        crate::storage::read(FILE)
            .and_then(|t| parse(&t))
            .or_else(|| parse(DEFAULT_TEXT))
            .unwrap_or_default()
    })
}

fn name(kind: StageKind) -> &'static str {
    match kind {
        StageKind::Dojo => "dojo",
        StageKind::Neon => "neon",
        StageKind::Bamboo => "bamboo",
        StageKind::Plains => "plains",
        StageKind::Spirit => "spirit",
        StageKind::Volcano => "volcano",
        StageKind::Custom => "custom",
    }
}

pub fn select(kind: StageKind) {
    CURRENT.store(kind as u8, Ordering::Relaxed);
}

fn current_kind() -> StageKind {
    [
        StageKind::Dojo,
        StageKind::Neon,
        StageKind::Bamboo,
        StageKind::Plains,
        StageKind::Spirit,
        StageKind::Volcano,
        StageKind::Custom,
    ]
    .get(CURRENT.load(Ordering::Relaxed) as usize)
    .copied()
    .unwrap_or_default()
}

pub fn look(kind: StageKind) -> Look {
    table().get(name(kind)).copied().unwrap_or_default()
}

pub fn rim() -> Option<(Look, f32)> {
    let level = crate::vfx::quality::level();
    let l = look(current_kind());
    (level >= 1 && l.rim_strength > 0.0)
        .then_some((l, l.rim_strength * if level >= 2 { 1.0 } else { 0.6 }))
}

pub fn grade(p: &mut crate::vfx::FxParams, kind: StageKind, game_time: f32) {
    let level = crate::vfx::quality::level();
    if level == 0 {
        return;
    }
    let l = look(kind);
    let wave = if l.pulse > 0.0 {
        1.0 + l.pulse * (game_time / 60.0 * std::f32::consts::TAU / l.pulse_period.max(0.5)).sin()
    } else {
        1.0
    };
    p.saturation += l.saturation;
    p.vignette += l.vignette;
    (0..3).for_each(|k| p.tint[k] *= l.tint[k] * wave);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_parse_and_inherit() {
        let t = parse(DEFAULT_TEXT).unwrap();
        assert!(t["volcano"].rim_strength > 0.5);
        assert_eq!(t["custom"].pulse, 0.0);
        assert_eq!(t.len(), 7);
    }

    #[test]
    fn bad_text_is_rejected() {
        assert!(parse("[dojo\nrim=").is_none());
    }
}
