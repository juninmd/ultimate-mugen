use super::Match;
use crate::chardef::Source;
use crate::fighter::HitDefData;

pub const KNOCKDOWN_PRESETS: [KnockdownRule; 5] = [
    KnockdownRule::OFF,
    KnockdownRule { scope: KnockdownScope::Native, threshold_percent: 8.0 },
    KnockdownRule { scope: KnockdownScope::Native, threshold_percent: 16.0 },
    KnockdownRule { scope: KnockdownScope::Native, threshold_percent: 24.0 },
    KnockdownRule { scope: KnockdownScope::All, threshold_percent: 16.0 },
];
pub const DEFAULT_KNOCKDOWN_PRESET: usize = 2;
const COMBO_ENDER_MULTIPLE: f32 = 3.0;
const MAX_PUSH_BONUS: f32 = 0.75;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KnockdownScope {
    Off,
    Native,
    All,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KnockdownRule {
    pub scope: KnockdownScope,
    pub threshold_percent: f32,
}

impl Default for KnockdownRule {
    fn default() -> Self {
        KNOCKDOWN_PRESETS[DEFAULT_KNOCKDOWN_PRESET]
    }
}

impl KnockdownRule {
    pub const OFF: KnockdownRule = KnockdownRule { scope: KnockdownScope::Off, threshold_percent: 0.0 };

    pub fn covers(&self, attacker: Source) -> bool {
        match self.scope {
            KnockdownScope::Off => false,
            KnockdownScope::Native => attacker == Source::Native,
            KnockdownScope::All => true,
        }
    }

    pub fn push_multiplier(&self, hit_percent: f32) -> f32 {
        let share = (hit_percent / self.threshold_percent.max(1.0)).clamp(0.0, 1.0);
        1.0 + MAX_PUSH_BONUS * share
    }
}

fn is_hyper(hd: &HitDefData) -> bool {
    hd.attr_kind.iter().any(|k| k.starts_with('H'))
}

impl Match {
    pub(super) fn standing_push(&self, a: usize, d: usize, hd: &HitDefData, dmg: f32, was_hit: bool) -> Option<f32> {
        let rule = self.knockdown;
        let (att, def) = (&self.fighters[a], &self.fighters[d]);
        if !rule.covers(att.def.source) || !hd.fall || hd.force_no_fall || hd.p2stateno >= 0 || is_hyper(hd) { return None; }
        if hd.down_bounce || hd.wall_bounce || hd.ground_type == 3 { return None; }
        if def.state_type == 'A' || def.state_type == 'L' || def.air_height() < -1.0 || def.ghv.fall { return None; }
        let max = def.life_max.max(1.0);
        let hit_percent = dmg / max * 100.0;
        let combo_before = if was_hit { att.combo_damage } else { 0.0 };
        let combo_percent = (combo_before + dmg) / max * 100.0;
        if hit_percent >= rule.threshold_percent || combo_percent >= rule.threshold_percent * COMBO_ENDER_MULTIPLE { return None; }
        if def.life - dmg < 1.0 { return None; }
        Some(rule.push_multiplier(hit_percent))
    }
}
