mod support;
use std::sync::Arc;
use support::gogeta::{load, synth, Duel};
use support::parity::{forced, move_state, observe, row, save, sounds, Run};
use support::render::frame;
use ume_engine::chardef::CharDef;
use ume_engine::input::k;
use ume_engine::matchs::Match;

fn umf() -> Option<Arc<CharDef>> {
    let path = std::env::var("UME_TEST_UMF_JSON").ok()?;
    let nc = ume_native::parse_char(&std::fs::read_to_string(path).ok()?).ok()?;
    Some(Arc::new(ume_native::compile_native(&nc, 1, true)))
}

fn charge(d: &Arc<CharDef>, held: u32) -> Run {
    let mut r = Duel::new(d.clone(), 140.0);
    r.m.fighters[0].power = 0.0;
    let mut run = Run::start(&r);
    run.power.0 = 0.0;
    r.push(0, &vec![held; 150]);
    observe(&mut r, 150, 25, &mut run);
    run.power.1 = r.m.fighters[0].power;
    observe(&mut r, 90, 1000, &mut run);
    run
}

fn command(d: &Arc<CharDef>, name: &str, ticks: usize) -> Run {
    let c = d.commands.iter().find(|c| c.name == name).unwrap_or_else(|| panic!("command {name} missing"));
    let mut r = Duel::new(d.clone(), 160.0);
    r.m.fighters[0].pos.x = 0.0;
    r.m.fighters[1].pos.x = 160.0;
    let mut run = Run::start(&r);
    r.push(0, &synth(c));
    observe(&mut r, ticks, ticks / 6, &mut run);
    run
}

fn intro(d: &Arc<CharDef>) -> Run {
    let mut m = Match::new(ume_engine::common::common_def(), None, ume_engine::common::flat_stage());
    m.add_player(d.clone(), 0, 0, 1);
    m.add_player(d.clone(), 1, 0, 2);
    m.intro_ticks = 170;
    m.start_round();
    let mut run = Run::empty();
    for t in 0..460 {
        m.tick(&[0, 0]);
        sounds(&mut m, &mut run);
        run.states.insert(m.fighters[0].state_no);
        if t % 60 == 59 && run.shots.len() < 6 {
            run.shots.push(frame(&m, false));
        }
        if m.round_state == 2 {
            break;
        }
    }
    run
}

#[test]
fn original_vs_umf_parity() {
    let (Some(orig), Some(conv)) = (load(1), umf()) else { return };
    let out = std::path::PathBuf::from(std::env::var("UME_EVIDENCE_DIR").unwrap_or_else(|_| "target/umf-parity".into()));
    std::fs::create_dir_all(&out).unwrap();
    let mut report = String::from("| scenario | side | states | min (x, y) | power | damage | sounds | sounds without data | end (state, ctrl, y) |\n|---|---|---|---|---|---|---|---|---|\n");
    let scenarios: [(&str, Run, Run); 6] = [
        ("intro", intro(&orig), intro(&conv)),
        ("charge", charge(&orig, k::A | k::B), charge(&conv, k::S)),
        ("genki_dama", command(&orig, "supergenkidama", 420), command(&conv, "m_supergenkidama", 420)),
        ("teleport", command(&orig, "teleport_a", 90), command(&conv, "m_teleport_a", 90)),
        ("meteor", forced(&orig, 4010, 900, false), forced(&conv, move_state(&conv, "m_m_atack").expect("move m_m_atack"), 900, false)),
        ("kamehameha", command(&orig, "airkamehameha", 240), command(&conv, "m_airkamehameha", 240)),
    ];
    for (name, a, b) in &scenarios {
        row(&mut report, name, "original", a);
        row(&mut report, name, "umf", b);
        save(&out, name, a, b);
    }
    std::fs::write(out.join("parity.md"), &report).unwrap();
    println!("{report}");
    let [intro_s, charge_s, genki, tele, meteor, kame] = &scenarios;
    assert_eq!(meteor.1.sounds.difference(&meteor.1.missing).count(), meteor.2.sounds.len(), "Meteor must play every original sound");
    assert_eq!(kame.1.sounds.difference(&kame.1.missing).count(), kame.2.sounds.len(), "Kamehameha must play every original sound");
    assert!(intro_s.2.states.contains(&190), "UMF must run its own intro state");
    assert_eq!(intro_s.1.sounds, intro_s.2.sounds, "intro sounds differ");
    assert!(charge_s.2.power.1 > 300.0, "UMF charge must fill the meter");
    assert!(charge_s.2.missing.is_empty() && genki.2.missing.is_empty(), "UMF played sounds missing from the pack");
    assert!(genki.2.min.1 <= -90.0 && genki.2.min.0 <= genki.1.min.0 + 20.0, "Genki Dama must teleport up and back like the original");
    assert!(genki.2.end.1 && genki.2.end.2.abs() < 1.0, "Genki Dama must land and return control");
    assert!((tele.2.min.0 - tele.1.min.0).abs() < 20.0, "teleport must travel like the original");
    assert!((meteor.2.min.1 - meteor.1.min.1).abs() < 5.0, "Meteor must jump as high as the original (VelAdd/VelMul)");
    assert!(charge_s.1.power.1 > 0.0, "original charge reference did not run");
}
