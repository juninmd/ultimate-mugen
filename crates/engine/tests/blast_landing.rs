use crate::support;
use support::*;
use ume_engine::stage_objects::Breakable;

fn blast_duel() -> ume_engine::matchs::Match {
    let mut m = duel(mugen_char("p1", ""), mugen_char("p2", ""));
    m.stage.bound_left = -680.0;
    m.stage.bound_right = 680.0;
    m.stage.bound_high = -140.0;
    let mut barrel = Breakable::new(1, 320.0, 0.0, 18.0, 50.0, 60.0);
    barrel.blast_radius = 96.0;
    barrel.blast_damage = 75.0;
    barrel.blast_launch = [4.8, -5.6];
    m.objects.add(barrel);
    m.camera.x = 300.0;
    m.fighters[0].pos.x = 260.0;
    m.fighters[1].pos.x = 306.0;
    m.fighters[1].state_no = 5050;
    m.fighters[1].physics = 'N';
    m.fighters[1].move_type = 'H';
    m.fighters[1].ghv.fall = true;
    m.fighters[1].vel.x = 5.5;
    m
}

#[test]
fn standing_fighter_launched_by_a_blast_falls_back_to_the_floor() {
    let mut m = blast_duel();
    m.tick(&[0, 0]);
    assert!(m.fighters[0].state_no == 5030 || m.fighters[0].state_no == 5020, "launch must enter a hit-fall state, got {}", m.fighters[0].state_no);
    let mut peak = 0.0f32;
    for _ in 0..240 {
        m.tick(&[0, 0]);
        peak = peak.min(m.fighters[0].pos.y);
    }
    assert!(peak < -20.0, "the blast must actually lift the fighter, peak {peak}");
    let f = &m.fighters[0];
    assert!(f.pos.y >= -0.01, "fighter left floating at y={} state={}", f.pos.y, f.state_no);
}

#[test]
fn watchdog_frees_a_fighter_hovering_in_a_hit_state() {
    let mut m = blast_duel();
    m.objects = Default::default();
    let f = &mut m.fighters[0];
    f.state_no = 5050;
    f.physics = 'N';
    f.move_type = 'H';
    f.state_type = 'A';
    f.ghv.fall = true;
    f.ghv.yaccel = 0.0;
    f.pos.y = -120.0;
    f.vel.y = 0.0;
    m.round_state = 2;
    for _ in 0..600 {
        m.tick(&[0, 0]);
    }
    let f = &m.fighters[0];
    assert!(f.pos.y >= -0.01, "hovering fighter never recovered: y={} state={}", f.pos.y, f.state_no);
}
