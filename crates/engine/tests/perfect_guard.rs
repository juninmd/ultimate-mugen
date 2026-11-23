use crate::support;
use support::*;
use ume_engine::fighter::HitDefData;
use ume_engine::matchs::Event;

fn strike_and_hurt_char(name: &str, native: bool) -> std::sync::Arc<ume_engine::chardef::CharDef> {
    let mut c = mugen_char(name, "");
    let def = std::sync::Arc::get_mut(&mut c).expect("unique");
    if native {
        def.source = ume_engine::chardef::Source::Native;
    }
    if let Some(anim) = def.anims.get_mut(&0) {
        for fr in &mut anim.frames {
            fr.clsn1 = vec![[-10.0, -70.0, 55.0, -20.0]];
            fr.clsn2 = vec![[-18.0, -80.0, 18.0, 0.0]];
        }
    }
    c
}

#[test]
fn perfect_guard_heals_awards_meter_and_emits_event() {
    let mut m = duel(
        strike_and_hurt_char("p1", true),
        strike_and_hurt_char("p2", true),
    );
    m.fighters[0].pos.x = -20.0;
    m.fighters[1].pos.x = 20.0;

    let mut hd = HitDefData::default();
    hd.attr_state = "S".to_string();
    hd.attr_kind = vec!["NA".to_string()];
    hd.hitflag = "MAF".to_string();
    hd.guardflag = "MA".to_string();
    hd.damage = 80.0;
    hd.guard_damage = 15.0;
    hd.guard_hit_time = 12;
    hd.guard_vel = -6.0;

    m.fighters[0].move_type = 'A';
    m.fighters[0].hitdef_active = true;
    m.fighters[0].hitdef = Some(hd);

    m.fighters[1].life = 800.0;
    m.fighters[1].power = 100.0;
    m.fighters[1].state_no = 130;
    m.fighters[1]
        .asserted_commands
        .push(("holdback".to_string(), 2));

    m.tick(&[0, ume_engine::input::k::R]);

    let mut saw_pg = false;
    for ev in &m.events {
        if let Event::PerfectGuard { healed, meter, .. } = ev {
            assert!(*healed >= 14.0, "expected >= 14 HP healed, got {healed}");
            assert!(*meter >= 119.0, "expected >= 119 meter reward, got {meter}");
            saw_pg = true;
        }
    }
    assert!(saw_pg, "Just-Defend block must emit Event::PerfectGuard");
    assert!(
        m.fighters[1].life >= 814.0,
        "Perfect Guard must heal defender (life={})",
        m.fighters[1].life
    );
    assert!(
        m.fighters[1].power >= 219.0,
        "Perfect Guard must grant meter reward (power={})",
        m.fighters[1].power
    );
}
