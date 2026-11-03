mod support;
use support::*;
use ume_engine::matchs::Event;
use ume_engine::sctrl::format_clipboard;

const CNS: &str = r#"
[Statedef 300]
type = S
movetype = I
physics = S
anim = 0
ctrl = 0

[State 300, null]
type = Null
trigger1 = 1

[State 300, offset]
type = Offset
trigger1 = Time = 0
x = 10
y = -5

[State 300, quote]
type = VictoryQuote
trigger1 = Time = 0
value = 3

[State 300, rumble]
type = ForceFeedback
trigger1 = Time = 0
time = 30
ampl = 255
waveform = square

[State 300, stop]
type = StopSnd
trigger1 = Time = 0
channel = 2

[State 300, pan]
type = SndPan
trigger1 = Time = 0
channel = 2
abspan = 0

[State 300, clip]
type = DisplayToClipboard
trigger1 = Time = 0
text = "life=%d ratio=%f"
params = 1000, 0.5

[State 300, append]
type = AppendToClipboard
trigger1 = Time = 0
text = "\nnext"

[State 300, shake]
type = EnvShake
trigger1 = Time = 0
time = 20
freq = 90
ampl = -8

[Statedef 310]
type = S
movetype = A
physics = S
anim = 0
ctrl = 0

[State 310, super]
type = SuperPause
trigger1 = Time = 0
time = 40
anim = -1
p2defmul = 2

[Statedef 320]
type = S
movetype = I
physics = S
anim = 0

[State 320, pause]
type = Pause
trigger1 = Time = 0
time = 15

[Statedef 330]
type = S
movetype = I
physics = S

[State 330, clear]
type = ClearClipboard
trigger1 = Time = 0
"#;

#[test]
fn misc_mugen11_controllers_take_effect() {
    let mut m = duel(mugen_char("p1", CNS), mugen_char("p2", ""));
    m.events.clear();
    enter(&mut m, 0, 300);
    m.tick(&[0, 0]);
    let f = &m.fighters[0];
    assert_eq!(f.state_no, 300);
    assert_eq!((f.draw_offset.x, f.draw_offset.y), (10.0, -5.0));
    assert_eq!(f.victory_quote, 3);
    assert_eq!(f.clipboard, vec!["life=1000 ratio=0.500000".to_string(), "next".to_string()]);
    let id = f.id;
    assert!(m.events.iter().any(|e| matches!(e, Event::StopSound { owner, channel: 2 } if *owner == id)));
    assert!(m.events.iter().any(|e| matches!(e, Event::SoundPan { owner, channel: 2, pan } if *owner == id && pan.abs() < 1e-3)));
    assert!(m.events.iter().any(|e| matches!(e, Event::Rumble { fighter, ms: 500, .. } if *fighter == id)));
    assert_eq!(m.shake_freq, 90.0);
    assert_eq!(m.shake_phase, 90.0, "phase defaults to 90 when freq >= 90");
    m.tick(&[0, 0]);
    assert_eq!(m.fighters[0].draw_offset.x, 0.0, "Offset only lasts for the tick it runs");
    enter(&mut m, 0, 330);
    m.tick(&[0, 0]);
    assert!(m.fighters[0].clipboard.is_empty());
}

#[test]
fn superpause_and_pause_are_distinct() {
    let mut m = duel(mugen_char("p1", CNS), mugen_char("p2", ""));
    enter(&mut m, 0, 320);
    m.tick(&[0, 0]);
    let p = m.pause.as_ref().expect("Pause started");
    assert!(!p.is_super && !p.darken && p.time <= 15);
    ticks(&mut m, 20);
    assert!(m.pause.is_none());

    let p2 = m.fighters[1].id;
    m.fighters[0].targets.push((p2, 0));
    enter(&mut m, 1, 5000);
    m.fighters[1].hit_over_time = 100;
    enter(&mut m, 0, 310);
    m.tick(&[0, 0]);
    let p = m.pause.as_ref().expect("SuperPause started");
    assert!(p.is_super && p.darken && p.unhittable && p.time > 30);
    assert!(m.fighters[0].unhittable > 0, "the super user cannot be hit while the screen is frozen");
    let hd = ume_engine::fighter::HitDefData { hitflag: "MAF".into(), attr_state: "S".into(), ..Default::default() };
    assert!(!m.can_be_hit(0, &hd, 1));
    assert_eq!(m.fighters[1].super_defence_mul, 2.0, "p2defmul reaches current targets");
    ticks(&mut m, 60);
    assert!(m.pause.is_none() && m.fighters[0].unhittable == 0);
}

#[test]
fn clipboard_formatting() {
    assert_eq!(format_clipboard("%d%% of %f\\n", &[50.7, 2.0]), "50% of 2.000000\n");
    assert_eq!(format_clipboard("no args %d", &[]), "no args ");
}
