use crate::support;
use support::*;
use ume_engine::stage_objects::{ObjectEvent, StageZone, StageZoneKind};

fn build_match() -> ume_engine::matchs::Match {
    let mut m = duel(mugen_char("p1", ""), mugen_char("p2", ""));
    m.stage.bound_left = -480.0;
    m.stage.bound_right = 480.0;
    m
}

#[test]
fn power_well_recharges_meter_and_emits_zone_event() {
    let mut m = build_match();
    m.fighters[0].pos.x = 0.0;
    m.fighters[0].power = 100.0;
    m.fighters[1].pos.x = 260.0;
    m.fighters[1].power = 100.0;
    m.objects.add_zone(StageZone::new(
        StageZoneKind::PowerWell {
            meter_per_tick: 6.0,
        },
        0.0,
        48.0,
    ));

    let mut saw_zone = false;
    for _ in 0..30 {
        m.tick(&[0, 0]);
        if m.objects
            .events
            .iter()
            .any(|e| matches!(e, ObjectEvent::ZoneTriggered { index: 0, .. }))
        {
            saw_zone = true;
        }
        m.objects.events.clear();
    }

    assert!(
        m.fighters[0].power > 250.0,
        "P1 inside PowerWell must gain meter"
    );
    assert!(
        (m.fighters[1].power - 100.0).abs() < 1.0,
        "P2 outside PowerWell must not gain zone meter"
    );
    assert!(saw_zone, "PowerWell must emit ZoneTriggered events");
}

#[test]
fn burn_hazard_damages_grounded_fighter_without_ko_underflow() {
    let mut m = build_match();
    m.fighters[0].pos.x = 120.0;
    m.fighters[0].pos.y = 0.0;
    m.fighters[0].life = 15.0;
    m.fighters[1].pos.x = -200.0;
    m.objects.add_zone(StageZone::new(
        StageZoneKind::BurnHazard {
            damage: 8.0,
            interval: 6,
        },
        120.0,
        40.0,
    ));

    for _ in 0..24 {
        m.tick(&[0, 0]);
        m.objects.events.clear();
    }

    assert!(
        (m.fighters[0].life - 1.0).abs() < 0.01,
        "BurnHazard should drain life down to 1.0 floor, got {}",
        m.fighters[0].life
    );
}

#[test]
fn updraft_lifts_airborne_fighter_higher_than_baseline() {
    let mut plain = build_match();
    let mut windy = build_match();
    for m in [&mut plain, &mut windy] {
        m.fighters[0].pos.x = 0.0;
        m.fighters[0].pos.y = -20.0;
        m.fighters[0].vel.y = -4.0;
        m.fighters[0].state_type = 'A';
        m.fighters[0].physics = 'A';
        m.fighters[1].pos.x = 300.0;
    }
    windy.objects.add_zone(StageZone::new(
        StageZoneKind::Updraft { lift: 0.45 },
        0.0,
        64.0,
    ));

    let mut min_y_plain = 0.0_f32;
    let mut min_y_windy = 0.0_f32;
    for _ in 0..20 {
        plain.tick(&[0, 0]);
        windy.tick(&[0, 0]);
        min_y_plain = min_y_plain.min(plain.fighters[0].pos.y);
        min_y_windy = min_y_windy.min(windy.fighters[0].pos.y);
    }

    assert!(
        min_y_windy < min_y_plain - 6.0,
        "Updraft must lift airborne fighter higher (windy={min_y_windy}, plain={min_y_plain})"
    );
}

#[test]
fn ice_floor_boosts_horizontal_velocity() {
    let mut m = build_match();
    m.fighters[0].pos.x = 0.0;
    m.fighters[0].pos.y = 0.0;
    m.fighters[0].vel.x = 4.0;
    m.fighters[0].physics = 'N';
    m.fighters[1].pos.x = 300.0;
    m.objects.add_zone(StageZone::new(
        StageZoneKind::IceFloor { boost: 0.2 },
        0.0,
        120.0,
    ));

    let before_vx = m.fighters[0].vel.x;
    m.tick(&[0, 0]);
    assert!(
        m.fighters[0].vel.x > before_vx,
        "IceFloor should boost horizontal slide velocity (before={before_vx}, after={})",
        m.fighters[0].vel.x
    );
}
