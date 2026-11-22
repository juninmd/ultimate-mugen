mod support;
use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::sync::Arc;
use support::gogeta::{load, synth, Duel};
use support::render::{frame, sheet};
use tiny_skia::Pixmap;
use ume_engine::chardef::CharDef;
use ume_engine::input::k;
use ume_engine::matchs::{Event, Match};

struct Run {
    shots: Vec<Pixmap>,
    sounds: BTreeSet<(i32, i32)>,
    missing: BTreeSet<(i32, i32)>,
    states: BTreeSet<i32>,
    min: (f32, f32),
    power: (f32, f32),
    end: (i32, bool, f32),
    damage: f32,
}

fn umf() -> Option<Arc<CharDef>> {
    let path = std::env::var("UME_TEST_UMF_JSON").ok()?;
    let nc = ume_native::parse_char(&std::fs::read_to_string(path).ok()?).ok()?;
    Some(Arc::new(ume_native::compile_native(&nc, 1, true)))
}

fn sounds(m: &mut Match, run: &mut Run) {
    for ev in m.events.drain(..) {
        if let Event::Sound { def: Some(def), group, index, own: true, .. } = ev {
            run.sounds.insert((group, index));
            if !def.sounds.contains_key(&(group, index)) {
                run.missing.insert((group, index));
            }
        }
    }
}

fn observe(r: &mut Duel, ticks: usize, every: usize, run: &mut Run) {
    let life = r.m.fighters[1].life;
    for t in 0..ticks {
        r.tick(1);
        sounds(&mut r.m, run);
        let f = &r.m.fighters[0];
        run.states.insert(f.state_no);
        run.min = (run.min.0.min(f.pos.x), run.min.1.min(f.pos.y));
        if t % every == every - 1 && run.shots.len() < 6 {
            run.shots.push(frame(&r.m, false));
        }
    }
    let f = &r.m.fighters[0];
    run.end = (f.state_no, f.ctrl, f.pos.y);
    run.damage += life - r.m.fighters[1].life;
}

fn new_run(r: &Duel) -> Run {
    let f = &r.m.fighters[0];
    Run { shots: vec![], sounds: BTreeSet::new(), missing: BTreeSet::new(), states: BTreeSet::new(), min: (f.pos.x, f.pos.y), power: (f.power, f.power), end: (0, false, 0.0), damage: 0.0 }
}

fn charge(d: &Arc<CharDef>, held: u32) -> Run {
    let mut r = Duel::new(d.clone(), 140.0);
    r.m.fighters[0].power = 0.0;
    let mut run = new_run(&r);
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
    let mut run = new_run(&r);
    r.push(0, &synth(c));
    observe(&mut r, ticks, ticks / 6, &mut run);
    run
}

fn forced(d: &Arc<CharDef>, state: i32, ticks: usize) -> Run {
    let mut r = Duel::new(d.clone(), 50.0);
    let mut run = new_run(&r);
    r.m.change_state(0, state, d.clone());
    observe(&mut r, ticks, ticks / 6, &mut run);
    run
}

fn move_state(d: &Arc<CharDef>, command: &str) -> i32 {
    d.movelist.iter().find(|mv| mv.command == command).map(|mv| mv.state).unwrap_or_else(|| panic!("move {command} missing"))
}

fn intro(d: &Arc<CharDef>) -> Run {
    let mut m = Match::new(ume_engine::common::common_def(), None, ume_engine::common::flat_stage());
    m.add_player(d.clone(), 0, 0, 1);
    m.add_player(d.clone(), 1, 0, 2);
    m.intro_ticks = 170;
    m.start_round();
    let mut run = Run { shots: vec![], sounds: BTreeSet::new(), missing: BTreeSet::new(), states: BTreeSet::new(), min: (0.0, 0.0), power: (0.0, 0.0), end: (0, false, 0.0), damage: 0.0 };
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

fn row(report: &mut String, name: &str, side: &str, r: &Run) {
    let _ = writeln!(report, "| {name} | {side} | {:?} | ({:.0}, {:.0}) | {:.0} -> {:.0} | {:.0} | {:?} | {:?} | {:?} |",
        r.states, r.min.0, r.min.1, r.power.0, r.power.1, r.damage, r.sounds, r.missing, r.end);
}

fn save(out: &std::path::Path, name: &str, a: &Run, b: &Run) {
    let mut all: Vec<Pixmap> = a.shots.clone();
    all.resize_with(6, || frame(&Match::new(ume_engine::common::common_def(), None, ume_engine::common::flat_stage()), false));
    all.extend(b.shots.iter().cloned());
    sheet(&all, 6).save_png(out.join(format!("{name}.png"))).unwrap();
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
        ("meteor", forced(&orig, 4010, 900), forced(&conv, move_state(&conv, "m_m_atack"), 900)),
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
    assert!(charge_s.1.power.1 > 0.0, "original charge reference did not run");
}
