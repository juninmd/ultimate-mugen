use super::{Stage, StageArt, StageKind};
use crate::story::objects::catalog;
use ume_engine::matchs::Match;
use ume_engine::stage_objects::{Breakable, PickupSpec};

const CENTER_CLEARANCE: f32 = 220.0;

struct ArenaProp {
    kind: &'static str,
    x: f32,
    drop: Option<&'static str>,
    hazard: bool,
}

fn layout_for(kind: StageKind) -> &'static [ArenaProp] {
    match kind {
        StageKind::Neon | StageKind::Volcano => &[
            ArenaProp {
                kind: "barrel",
                x: -340.0,
                drop: Some("battery"),
                hazard: true,
            },
            ArenaProp {
                kind: "crate",
                x: -500.0,
                drop: Some("apple"),
                hazard: false,
            },
            ArenaProp {
                kind: "barrel",
                x: 340.0,
                drop: Some("battery"),
                hazard: true,
            },
            ArenaProp {
                kind: "crate",
                x: 500.0,
                drop: Some("apple"),
                hazard: false,
            },
        ],
        StageKind::Dojo | StageKind::Bamboo | StageKind::Spirit => &[
            ArenaProp {
                kind: "crate",
                x: -330.0,
                drop: Some("battery"),
                hazard: false,
            },
            ArenaProp {
                kind: "barrel",
                x: -490.0,
                drop: Some("apple"),
                hazard: true,
            },
            ArenaProp {
                kind: "crate",
                x: 330.0,
                drop: Some("battery"),
                hazard: false,
            },
            ArenaProp {
                kind: "barrel",
                x: 490.0,
                drop: Some("apple"),
                hazard: true,
            },
        ],
        StageKind::Plains | StageKind::Arena => &[
            ArenaProp {
                kind: "crate",
                x: -350.0,
                drop: Some("apple"),
                hazard: false,
            },
            ArenaProp {
                kind: "barrel",
                x: -510.0,
                drop: Some("battery"),
                hazard: true,
            },
            ArenaProp {
                kind: "crate",
                x: 350.0,
                drop: Some("apple"),
                hazard: false,
            },
            ArenaProp {
                kind: "barrel",
                x: 510.0,
                drop: Some("battery"),
                hazard: true,
            },
        ],
        StageKind::Custom => &[],
    }
}

fn enabled() -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::env::var("UME_STAGE_PROPS").map_or(true, |v| v != "0")
    }
    #[cfg(target_arch = "wasm32")]
    {
        true
    }
}

fn pickup_of(name: &str) -> Option<PickupSpec> {
    let cat = catalog();
    let index = cat.pickups.keys().position(|k| k == name)?;
    let food = &cat.pickups[name];
    Some(PickupSpec {
        kind: index as u16,
        heal: food.heal.clamp(0.0, 1.0),
        power: food.power.clamp(0.0, 1.0),
        score: food.score,
    })
}

pub fn populate(m: &mut Match, stage: &Stage) {
    if !enabled() || m.training || m.power_arena || !m.objects.is_idle() {
        return;
    }
    let supported = matches!(
        stage.art,
        StageArt::Procedural { .. } | StageArt::Illustrated { span, .. } if span <= 0.0
    );
    if !supported || stage.kind == StageKind::Custom {
        return;
    }
    let cat = catalog();
    let lo = m.stage.bound_left + 48.0;
    let hi = m.stage.bound_right - 48.0;
    for prop in layout_for(stage.kind) {
        if prop.x.abs() < CENTER_CLEARANCE || prop.x < lo || prop.x > hi {
            continue;
        }
        let Some(kind_idx) = cat.kinds.keys().position(|k| k == prop.kind) else {
            continue;
        };
        let def = &cat.kinds[prop.kind];
        let mut b = Breakable::new(
            kind_idx as u16,
            prop.x,
            0.0,
            def.half_width,
            def.height,
            def.life,
        );
        b.depth = def.depth;
        b.score = def.score;
        b.solid = false;
        b.drop = prop.drop.and_then(pickup_of);
        if prop.hazard {
            b.blast_radius = 96.0;
            b.blast_damage = 75.0;
            b.blast_launch = [4.8, -5.6];
        }
        m.objects.add(b);
    }
    if !m.objects.breakables.is_empty() {
        m.objects.any_collector = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arena_prop_layouts_keep_center_fighting_lane_clear() {
        for kind in [
            StageKind::Dojo,
            StageKind::Neon,
            StageKind::Bamboo,
            StageKind::Plains,
            StageKind::Spirit,
            StageKind::Volcano,
            StageKind::Arena,
        ] {
            let props = layout_for(kind);
            assert!(props.len() >= 4);
            assert!(props.iter().all(|p| p.x.abs() >= CENTER_CLEARANCE));
            assert!(props.iter().any(|p| p.hazard));
        }
        assert!(layout_for(StageKind::Custom).is_empty());
    }
}
