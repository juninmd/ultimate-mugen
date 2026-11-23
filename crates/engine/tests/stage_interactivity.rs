use crate::support;
use support::*;
use ume_engine::stage_objects::{Breakable, ObjectEvent, PickupSpec, PICKUP_ARM_TICKS};

#[test]
fn knockback_slam_shatters_breakable_and_explosive_hazard_launches_fighters() {
    let mut m = duel(mugen_char("p1", ""), mugen_char("p2", ""));
    m.stage.bound_left = -680.0;
    m.stage.bound_right = 680.0;
    m.stage.bound_high = -140.0;
    m.objects.any_collector = true;

    let mut barrel = Breakable::new(1, 320.0, 0.0, 18.0, 50.0, 60.0);
    barrel.blast_radius = 96.0;
    barrel.blast_damage = 75.0;
    barrel.blast_launch = [4.8, -5.6];
    barrel.drop = Some(PickupSpec {
        kind: 4,
        heal: 0.0,
        power: 0.34,
        score: 150.0,
    });
    m.objects.add(barrel);

    m.camera.x = 300.0;
    m.fighters[0].pos.x = 260.0;
    m.fighters[1].pos.x = 306.0;
    m.fighters[1].state_no = 5050;
    m.fighters[1].physics = 'N';
    m.fighters[1].move_type = 'H';
    m.fighters[1].ghv.fall = true;
    m.fighters[1].vel.x = 5.5;
    let p2_life_before = m.fighters[1].life;

    let mut saw_broken = false;
    let mut saw_blast = false;
    let mut saw_drop = false;
    for _ in 0..8 {
        m.tick(&[0, 0]);
        for ev in m.objects.events.drain(..) {
            match ev {
                ObjectEvent::Broken { index: 0 } => saw_broken = true,
                ObjectEvent::Blast { radius, damage, .. } if radius >= 90.0 && damage >= 70.0 => {
                    saw_blast = true;
                }
                ObjectEvent::Dropped { kind: 4, .. } => saw_drop = true,
                _ => {}
            }
        }
    }

    assert!(
        saw_broken,
        "knocked-back fighter slam must shatter breakable"
    );
    assert!(saw_blast, "explosive barrel must emit ObjectEvent::Blast");
    assert!(saw_drop, "shattered barrel must drop its pickup");
    assert!(
        m.fighters[1].life < p2_life_before && m.fighters[1].life >= 1.0,
        "blast damages nearby fighter without cheap KO below 1 HP"
    );
    assert!(
        m.fighters[1].vel.y < 0.0 || m.fighters[1].state_type == 'A',
        "blast launches nearby fighter airborne"
    );
    assert!(m.shake_time > 0, "blast shakes the camera");
}

#[test]
fn any_collector_allows_p2_to_pick_up_arena_drops() {
    let mut m = duel(mugen_char("p1", ""), mugen_char("p2", ""));
    m.stage.bound_left = -680.0;
    m.stage.bound_right = 680.0;
    m.objects.any_collector = true;

    let mut crate_prop = Breakable::new(0, 340.0, 0.0, 20.0, 42.0, 40.0);
    crate_prop.drop = Some(PickupSpec {
        kind: 0,
        heal: 0.2,
        power: 0.34,
        score: 100.0,
    });
    m.objects.add(crate_prop);

    m.camera.x = 320.0;
    m.fighters[0].pos.x = 220.0;
    m.fighters[1].pos.x = 328.0;
    m.fighters[1].life = 500.0;
    m.fighters[1].power = 0.0;
    m.fighters[1].state_no = 5050;
    m.fighters[1].physics = 'N';
    m.fighters[1].move_type = 'H';
    m.fighters[1].ghv.fall = true;
    m.fighters[1].vel.x = 4.5;

    let mut collected_by_p2 = false;
    let p2_id = m.fighters[1].id;
    for step in 0..(PICKUP_ARM_TICKS + 40) {
        if step > 10 && !m.objects.pickups.is_empty() {
            m.fighters[1].pos.x = m.objects.pickups[0].pos.x;
            m.fighters[1].pos.y = 0.0;
            m.fighters[1].vel.x = 0.0;
            m.fighters[1].vel.y = 0.0;
        }
        m.tick(&[0, 0]);
        for ev in m.objects.events.drain(..) {
            if let ObjectEvent::Collected { fighter, .. } = ev {
                if fighter == p2_id {
                    collected_by_p2 = true;
                }
            }
        }
    }

    assert!(
        collected_by_p2,
        "P2 must collect arena pickups when any_collector is true"
    );
    assert!(m.fighters[1].life > 500.0, "heal pickup restored P2 life");
    assert!(m.fighters[1].power > 0.0, "power pickup restored P2 meter");
}

#[test]
fn explosive_blast_ignites_neighboring_barrel_fuse_for_chain_reaction() {
    let mut m = duel(mugen_char("p1", ""), mugen_char("p2", ""));
    m.stage.bound_left = -680.0;
    m.stage.bound_right = 680.0;
    m.objects.any_collector = true;

    let mut b1 = Breakable::new(1, 300.0, 0.0, 18.0, 50.0, 50.0);
    b1.blast_radius = 96.0;
    b1.blast_damage = 70.0;
    b1.blast_launch = [4.5, -5.2];
    m.objects.add(b1);

    let mut b2 = Breakable::new(1, 385.0, 0.0, 18.0, 50.0, 120.0);
    b2.blast_radius = 96.0;
    b2.blast_damage = 70.0;
    b2.blast_launch = [4.5, -5.2];
    m.objects.add(b2);

    let crate_near = Breakable::new(0, 345.0, 0.0, 20.0, 42.0, 40.0);
    m.objects.add(crate_near);

    m.camera.x = 340.0;
    m.fighters[0].pos.x = 220.0;
    m.fighters[1].pos.x = 290.0;
    m.fighters[1].state_no = 5050;
    m.fighters[1].physics = 'N';
    m.fighters[1].move_type = 'H';
    m.fighters[1].ghv.fall = true;
    m.fighters[1].vel.x = 5.0;

    let mut blasts = Vec::new();
    for t in 0..20 {
        m.tick(&[0, 0]);
        for ev in m.objects.events.drain(..) {
            if let ObjectEvent::Blast { x, .. } = ev {
                blasts.push((t, x));
            }
        }
    }

    assert_eq!(
        blasts.len(),
        2,
        "both barrels must detonate in a chain reaction"
    );
    assert!((blasts[0].1 - 300.0).abs() < 1.0);
    assert!((blasts[1].1 - 385.0).abs() < 1.0);
    assert!(
        blasts[1].0 > blasts[0].0,
        "second barrel detonates after fuse delay"
    );
    assert!(
        m.objects.breakables[2].broken,
        "nearby crate shattered by blast splash"
    );
}
