mod model;
#[cfg(test)]
mod tests;

pub use model::{Bonus, Catalog, Food, Kind, Placement, StageObjects};
use std::sync::OnceLock;
use ume_engine::matchs::Match;

const DEFAULT_TEXT: &str = include_str!("../../assets/story/objects.toml");
pub const FILE: &str = "story-objects.toml";

pub fn catalog() -> &'static Catalog {
    static CATALOG: OnceLock<Catalog> = OnceLock::new();
    CATALOG.get_or_init(|| {
        crate::storage::read(FILE)
            .and_then(|t| Catalog::parse(&t))
            .or_else(|| Catalog::parse(DEFAULT_TEXT))
            .unwrap_or_default()
    })
}

pub fn spawn_stage(m: &mut Match, def: &super::StageDef, from_fight: usize) {
    m.objects.clear();
    let cat = catalog();
    let Some(stage) = cat.stages.get(def.arena) else {
        return;
    };
    for place in stage
        .objects
        .iter()
        .filter(|p| p.fight >= from_fight && p.fight < super::fights_in(def))
    {
        let center = super::fight_center(def, place.fight);
        if let Some(b) = cat.breakable(place, center, (def.lane_far, def.lane_near)) {
            m.objects.add(b);
        }
    }
}

pub fn spawn_bonus(m: &mut Match, def: &super::StageDef, center: f32) -> bool {
    let cat = catalog();
    let Some(bonus) = cat.bonus(def.arena) else {
        return false;
    };
    m.objects.clear();
    for place in &bonus.objects {
        if let Some(b) = cat.breakable(place, center, (def.lane_far, def.lane_near)) {
            m.objects.add(b);
        }
    }
    if m.objects.breakables.is_empty() {
        return false;
    }
    m.objects.start_break_all_goal(bonus.seconds * 60);
    true
}
