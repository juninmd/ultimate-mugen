mod support;
use support::*;
use ume_engine::fighter::HitDefData;
use ume_engine::matchs::Event;

fn attacking_char(name: &str) -> std::sync::Arc<ume_engine::chardef::CharDef> {
    let mut c = mugen_char(name, "");
    let def = std::sync::Arc::get_mut(&mut c).expect("unique");
    def.source = ume_engine::chardef::Source::Native;
    if let Some(anim) = def.anims.get_mut(&0) {
        for fr in &mut anim.frames {
            fr.clsn1 = vec![[-10.0, -70.0, 55.0, -20.0]];
            fr.clsn2 = vec![[-18.0, -80.0, 18.0, 0.0]];
        }
    }
    c
}

#[test]
fn simultaneous_overlapping_strikes_trigger_symmetric_clash() {
    let mut m = duel(attacking_char("p1"), attacking_char("p2"));
    m.fighters[0].pos.x = -25.0;
    m.fighters[1].pos.x = 25.0;

    let mut hd = HitDefData::default();
    hd.attr_kind = vec!["NA".to_string()];
    hd.damage = 60.0;
    for i in 0..2 {
        m.fighters[i].move_type = 'A';
        m.fighters[i].hitdef_active = true;
        m.fighters[i].hitdef = Some(hd.clone());
        m.fighters[i].power = 100.0;
        m.fighters[i].life = 1000.0;
    }

    m.tick(&[0, 0]);

    let saw_clash = m.events.iter().any(|e| matches!(e, Event::Clash { .. }));
    assert!(
        saw_clash,
        "overlapping active Clsn1 hitboxes must emit Event::Clash"
    );
    assert!(
        !m.fighters[0].hitdef_active && !m.fighters[1].hitdef_active,
        "clashing strikes must cancel both active hitdefs"
    );
    assert!(
        m.fighters[0].hit_pause >= 11 && m.fighters[1].hit_pause >= 11,
        "both fighters must enter symmetric clash hitstop"
    );
    assert!(
        (m.fighters[0].life - 1000.0).abs() < 0.01 && (m.fighters[1].life - 1000.0).abs() < 0.01,
        "clashed strikes must deal zero damage"
    );
    assert!(
        m.fighters[0].power >= 159.0 && m.fighters[1].power >= 159.0,
        "both fighters must gain clash meter bonus"
    );
}

#[test]
fn throws_do_not_trigger_strike_clash() {
    let mut m = duel(attacking_char("p1"), attacking_char("p2"));
    m.fighters[0].pos.x = -25.0;
    m.fighters[1].pos.x = 25.0;

    let mut hd = HitDefData::default();
    hd.attr_kind = vec!["NT".to_string()];
    for i in 0..2 {
        m.fighters[i].move_type = 'A';
        m.fighters[i].hitdef_active = true;
        m.fighters[i].hitdef = Some(hd.clone());
    }

    m.tick(&[0, 0]);
    let saw_clash = m.events.iter().any(|e| matches!(e, Event::Clash { .. }));
    assert!(
        !saw_clash,
        "throw HitDefs (attr containing 'T') must not clash"
    );
}
