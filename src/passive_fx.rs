use ume_engine::fighter::Fighter;
use ume_engine::potential::PotentialLevel;

const RAGE_COLOR: [f32; 3] = [1.0, 0.3, 0.14];
const TIME_COLOR: [f32; 3] = [0.8, 0.2, 1.0];
const AURA_STRENGTH: f32 = 0.85;

const POTENTIAL_COLOR: [f32; 3] = [0.45, 0.85, 1.0];
const FURY_COLOR: [f32; 3] = [1.0, 0.3, 0.2];

pub fn aura(f: &Fighter) -> f32 {
    let passive = f.passive_view().map_or(0.0, |view| view.aura * AURA_STRENGTH);
    passive.max(f.potential.level.intensity())
}

pub fn color(f: &Fighter) -> [f32; 3] {
    match f.potential.level {
        PotentialLevel::Fury => return FURY_COLOR,
        PotentialLevel::Potential => return POTENTIAL_COLOR,
        PotentialLevel::Idle => {}
    }
    match f.passive_view() {
        Some(view) if view.rage.is_some_and(|r| r > 0.0) && view.stacks == 0 => RAGE_COLOR,
        Some(view) if view.time.is_some_and(|t| t > 0.0) && view.stacks == 0 => TIME_COLOR,
        _ => crate::vfx::Vfx::form_color(f),
    }
}
