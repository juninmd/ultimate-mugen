mod support;
use std::sync::Arc;
use support::*;
use ume_engine::chardef::CharDef;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::input::k;
use ume_engine::matchs::Match;
use ume_engine::native::{compile_native, NativeChar};

const CHARGE_STATE: i32 = 3920;
const HOLD_TICKS: usize = 100;
const MINIMUM_GAIN_PER_TICK: f32 = 3.0;
const RELEASE_RECOVERY_TICKS: usize = 14;
const JAB_STATE: i32 = 200;

fn roster() -> Vec<(NativeChar, Arc<CharDef>)> {
    let mut chars: Vec<NativeChar> = std::fs::read_dir("../../assets/chars").unwrap().filter_map(|entry| {
        let path = entry.unwrap().path();
        let stem = path.file_stem()?.to_string_lossy().to_string();
        (path.extension()? == "json" && stem != "index").then(|| serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap())
    }).collect();
    chars.sort_by(|a, b| a.id.cmp(&b.id));
    chars.into_iter().map(|nc| { let def = Arc::new(compile_native(&nc, 1, true)); (nc, def) }).collect()
}

fn arena(def: &Arc<CharDef>) -> Match {
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(def.clone(), 0, 0, 1);
    m.add_player(def.clone(), 1, 0, 2);
    m.start_round();
    while m.round_state != 2 { m.tick(&[0, 0]); }
    face_off(&mut m, 200.0);
    m.fighters[0].power = 0.0;
    m
}

#[test]
fn every_fighter_charges_energy_while_start_is_held() {
    for (nc, def) in roster() {
        let mut m = arena(&def);
        let mut entered = false;
        for _ in 0..HOLD_TICKS {
            m.tick(&[k::S, 0]);
            entered |= m.fighters[0].state_no == CHARGE_STATE;
        }
        let gained = m.fighters[0].power;
        assert!(entered, "{} never entered the charge state", nc.id);
        assert!(gained >= MINIMUM_GAIN_PER_TICK * HOLD_TICKS as f32, "{} gained only {gained} power in {HOLD_TICKS} ticks", nc.id);
        assert!(gained <= m.fighters[0].power_max, "{} overflowed the meter", nc.id);
        assert!(!m.fighters[0].ctrl && m.fighters[0].move_type == 'I', "{} should be busy but vulnerable while charging", nc.id);
        for _ in 0..RELEASE_RECOVERY_TICKS { m.tick(&[0, 0]); }
        assert!(m.fighters[0].ctrl && m.fighters[0].state_no != CHARGE_STATE, "{} stayed in the charge state after release (state {})", nc.id, m.fighters[0].state_no);
        assert!(def.movelist.iter().any(|mv| mv.command == "charge_cmd"), "{} does not list the charge in its move list", nc.id);
    }
}

#[test]
fn charging_stops_at_a_full_meter_and_a_hit_breaks_it() {
    for (nc, def) in roster() {
        let mut m = arena(&def);
        m.fighters[0].power = m.fighters[0].power_max - 10.0;
        for _ in 0..20 { m.tick(&[k::S, 0]); }
        assert!(m.fighters[0].power <= m.fighters[0].power_max, "{} overflowed the meter", nc.id);
        assert!(m.fighters[0].state_no != CHARGE_STATE, "{} kept charging with a full meter", nc.id);

        let mut m = arena(&def);
        for _ in 0..10 { m.tick(&[k::S, 0]); }
        face_off(&mut m, 30.0);
        let life = m.fighters[0].life;
        enter(&mut m, 1, JAB_STATE);
        for _ in 0..40 { m.tick(&[k::S, 0]); }
        assert!(m.fighters[0].life < life, "{} was not hit while charging", nc.id);
        assert!(m.fighters[0].state_no != CHARGE_STATE || m.fighters[0].move_type == 'I', "{} state {}", nc.id, m.fighters[0].state_no);
    }
}

#[test]
fn a_fighter_with_forms_charges_first_and_transforms_on_the_next_press() {
    let mut transformed = 0;
    for (nc, def) in roster().into_iter().filter(|(_, def)| !def.forms.is_empty()) {
        let cost = def.forms[0].cost.max(1.0);
        let mut m = arena(&def);
        let mut guard = 0;
        while m.fighters[0].power < cost && guard < 2000 { m.tick(&[k::S, 0]); guard += 1; }
        assert_eq!(m.fighters[0].form, 0, "{} transformed while still holding the charge", nc.id);
        for _ in 0..RELEASE_RECOVERY_TICKS { m.tick(&[0, 0]); }
        for _ in 0..4 { m.tick(&[k::T, 0]); }
        assert_eq!(m.fighters[0].form, 1, "{} did not transform after charging {cost} power and pressing Awaken", nc.id);
        transformed += 1;
    }
    assert!(transformed >= 2, "expected the form fighters to be covered");
}

#[test]
fn charging_accelerates_with_hold_time_and_restarts_after_release() {
    use ume_engine::native::charge_ramp::gain_at;
    let (_, def) = roster().into_iter().next().unwrap();
    let mut m = arena(&def);
    let mut gains = vec![];
    let mut last = 0.0;
    for _ in 0..120 {
        m.tick(&[k::S, 0]);
        gains.push(m.fighters[0].power - last);
        last = m.fighters[0].power;
    }
    let early: f32 = gains[10..20].iter().sum();
    let late: f32 = gains[100..110].iter().sum();
    assert!(late > early * 2.0, "early {early} late {late}");
    assert!(gains[110] <= gain_at(1000) + 0.01);
    for _ in 0..RELEASE_RECOVERY_TICKS { m.tick(&[0, 0]); }
    let before = m.fighters[0].power;
    for _ in 0..14 { m.tick(&[k::S, 0]); }
    let restarted = m.fighters[0].power - before;
    assert!(restarted < early * 1.5 + 60.0, "ramp did not reset: {restarted}");
    println!("early10={early} late10={late} total120={last}");
}
