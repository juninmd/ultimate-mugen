use crate::support;
use support::*;
use ume_engine::matchs::Event;

#[test]
fn high_speed_corner_slam_triggers_wall_shatter_bonus_and_meter() {
    let mut m = duel(mugen_char("p1", ""), mugen_char("p2", ""));
    m.stage.bound_left = -240.0;
    m.stage.bound_right = 240.0;
    m.camera.x = 240.0;

    let p1_id = m.fighters[0].id;
    m.fighters[0].power = 100.0;
    m.fighters[0].pos.x = 260.0;
    let stage_right = m.stage.bound_right + m.view_w / 2.0;
    m.fighters[1].pos.x = stage_right - 10.0;
    m.fighters[1].pos.y = -20.0;
    m.fighters[1].vel.x = -12.0;
    m.fighters[1].state_no = 5050;
    m.fighters[1].move_type = 'H';
    m.fighters[1].physics = 'N';
    m.fighters[1].life = 500.0;
    m.fighters[1].last_hit_by = Some(p1_id);

    let mut saw_shatter = false;
    for _ in 0..4 {
        m.tick(&[0, 0]);
        for ev in &m.events {
            if let Event::WallShatter {
                right_wall,
                bonus_damage,
                ..
            } = ev
            {
                assert!(
                    *right_wall,
                    "impact on right corner must set right_wall=true"
                );
                assert!(
                    *bonus_damage >= 34.0,
                    "expected >= 34 bonus damage, got {bonus_damage}"
                );
                saw_shatter = true;
            }
        }
    }

    assert!(
        saw_shatter,
        "high-speed corner slam must emit Event::WallShatter"
    );
    assert!(
        m.fighters[1].life <= 466.0,
        "defender must take WallShatter bonus damage"
    );
    assert!(
        m.fighters[0].power >= 249.0,
        "attacker must gain WallShatter meter bonus"
    );
    assert!(
        m.fighters[1].vel.x * m.fighters[1].facing < 0.0,
        "defender must rebound away from right wall in world space"
    );
    assert!(
        m.fighters[1].ghv.wall_bounced,
        "wall_bounced flag prevents double shatter"
    );
}

#[test]
fn wall_shatter_is_non_lethal_at_one_hp() {
    let mut m = duel(mugen_char("p1", ""), mugen_char("p2", ""));
    m.stage.bound_left = -240.0;
    m.stage.bound_right = 240.0;
    m.camera.x = -240.0;

    let stage_left = m.stage.bound_left - m.view_w / 2.0;
    m.fighters[1].pos.x = stage_left + 10.0;
    m.fighters[1].pos.y = -20.0;
    m.fighters[1].vel.x = 11.0;
    m.fighters[1].state_no = 5050;
    m.fighters[1].move_type = 'H';
    m.fighters[1].physics = 'N';
    m.fighters[1].life = 12.0;

    m.tick(&[0, 0]);
    assert!(
        (m.fighters[1].life - 1.0).abs() < 0.01,
        "WallShatter must clamp defender life at 1.0, got {}",
        m.fighters[1].life
    );
}
