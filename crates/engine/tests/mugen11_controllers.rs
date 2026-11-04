mod support;
use support::*;
use std::sync::Arc;
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

#[test]
fn playsnd_carries_pan_loop_and_freqmul() {
    let cns = "
[Statedef 340]
type = S
physics = S

[State 340, snd]
type = PlaySnd
trigger1 = Time = 0
value = 5, 0
abspan = 0
loop = 1
freqmul = 1.5
";
    let mut m = duel(mugen_char("p1", cns), mugen_char("p2", ""));
    m.events.clear();
    enter(&mut m, 0, 340);
    m.tick(&[0, 0]);
    assert!(m.events.iter().any(|e| matches!(e, Event::Sound { looping: true, freq_mul, pan, .. } if *freq_mul == 1.5 && pan.abs() < 1e-3)));
}

const RUNSTATE: &str = "
[Statedef 500]
type = S
movetype = I
physics = S
ctrl = 0

[State 500, run]
type = RunState
trigger1 = 1
value = 510

[State 500, runguarded]
type = RunState
trigger1 = 1
ignorehitpause = 1
value = 520

[Statedef 510]
type = S
movetype = A
physics = S

[State 510, count]
type = VarAdd
trigger1 = 1
v = 1
value = 1

[State 510, once]
type = VarAdd
trigger1 = 1
persistent = 0
v = 2
value = 1

[State 510, set]
type = VarSet
trigger1 = 1
v = 3
value = 9

[Statedef 520]
type = S
movetype = A
physics = S

[State 520, always]
type = VarAdd
trigger1 = 1
ignorehitpause = 1
v = 4
value = 1

[State 520, paused]
type = VarAdd
trigger1 = 1
v = 5
value = 1
";

#[test]
fn runstate_runs_another_state_without_changing_state() {
    let mut m = duel(mugen_char("p1", RUNSTATE), mugen_char("p2", ""));
    enter(&mut m, 0, 500);
    m.tick(&[0, 0]);
    let f = &m.fighters[0];
    assert_eq!(f.state_no, 500, "RunState never changes stateno");
    assert_eq!(f.vars[3], 9);
    assert_eq!(f.vars[1], 1);
    assert_eq!(f.vars[2], 1);
    assert_eq!(f.run_state_depth, 0, "recursion guard is released");
    m.tick(&[0, 0]);
    let f = &m.fighters[0];
    assert_eq!(f.vars[1], 2, "controllers run once per trigger");
    assert_eq!(f.vars[2], 1, "persistent = 0 is respected inside the run state");
}

#[test]
fn runstate_recursion_is_bounded() {
    let cns = "
[Statedef 570]
type = S
physics = S
ctrl = 0

[State 570, count]
type = VarAdd
trigger1 = 1
v = 0
value = 1

[State 570, again]
type = RunState
trigger1 = 1
value = 570
";
    let mut m = duel(mugen_char("p1", cns), mugen_char("p2", ""));
    enter(&mut m, 0, 570);
    m.tick(&[0, 0]);
    assert_eq!(m.fighters[0].vars[0], 9, "one real pass plus eight nested ones");
    assert_eq!(m.fighters[0].run_state_depth, 0);
    assert_eq!(m.fighters[0].state_no, 570);
}

#[test]
fn runstate_honours_ignorehitpause() {
    let mut m = duel(mugen_char("p1", RUNSTATE), mugen_char("p2", ""));
    enter(&mut m, 0, 500);
    m.fighters[0].hit_pause = 4;
    m.tick(&[0, 0]);
    let f = &m.fighters[0];
    assert_eq!(f.vars[1], 0, "the RunState controller itself is skipped during hit pause");
    assert_eq!(f.vars[4], 1, "ignorehitpause controllers inside still run");
    assert_eq!(f.vars[5], 0, "the others stay frozen");
    m.fighters[0].hit_pause = 0;
    m.tick(&[0, 0]);
    assert_eq!(m.fighters[0].vars[5], 1);
}

#[test]
fn height_overrides_size_height_for_one_tick() {
    let cns = "
[Statedef 530]
type = S
physics = S
ctrl = 0

[State 530, h]
type = Height
trigger1 = Time = 0
value = 20, 5
";
    let mut def = Arc::try_unwrap(mugen_char("p1", cns)).ok().unwrap();
    def.anims.insert(400, anim(400, vec![frame(-1, vec![], vec![])]));
    let mut m = duel(Arc::new(def), mugen_char("p2", ""));
    assert_eq!(m.fighters[0].height_top(), 60.0);
    enter(&mut m, 0, 530);
    let own = m.fighters[0].def.clone();
    m.fighters[0].change_anim(400, &own, 1);
    m.tick(&[0, 0]);
    let f = &m.fighters[0];
    assert_eq!((f.height_top(), f.height_bottom()), (20.0, 5.0));
    let hurt = f.boxes(2);
    assert_eq!(hurt.len(), 1, "an animation frame without a hurtbox gets one from Height");
    assert_eq!((hurt[0][1] - f.pos.y, hurt[0][3] - f.pos.y), (-20.0, 5.0));
    m.tick(&[0, 0]);
    assert_eq!(m.fighters[0].height_top(), 60.0, "Height only lasts for the tick it runs");
}

#[test]
fn gethitvarset_is_visible_through_gethitvar() {
    let cns = "
[Statedef 540]
type = S
physics = S
ctrl = 0

[State 540, set]
type = GetHitVarSet
trigger1 = Time = 0
hittype = 2
animtype = 3
airtype = 1
groundtype = 2
xvel = -3
yvel = -4
hittime = 12
slidetime = 7
ctrltime = 9
yaccel = 0.5
fall = 1
fall.xvel = -2
fall.yvel = -6
damage = 25

[State 540, read]
type = VarSet
trigger1 = Time = 0
v = 0
value = GetHitVar(hittime) * 100 + GetHitVar(slidetime)

[State 540, readvel]
type = FVarSet
trigger1 = Time = 0
fv = 0
value = GetHitVar(xvel)
";
    let mut m = duel(mugen_char("p1", cns), mugen_char("p2", ""));
    enter(&mut m, 0, 540);
    m.tick(&[0, 0]);
    let g = &m.fighters[0].ghv;
    assert_eq!((g.kind, g.animtype, g.airtype, g.groundtype), (2, 3, 1, 2));
    assert_eq!((g.hittime, g.slidetime, g.ctrltime), (12, 7, 9));
    assert_eq!((g.xvel, g.yvel, g.yaccel), (-3.0, -4.0, 0.5));
    assert!(g.fall && g.fall_xvel == -2.0 && g.fall_yvel == -6.0 && g.damage == 25.0);
    assert_eq!(m.fighters[0].vars[0], 1207);
}

const JAB_WITH_MODIFY: &str = "
[Statedef 200]
type = S
movetype = A
physics = S
anim = 200
ctrl = 0

[State 200, hit]
type = HitDef
trigger1 = Time = 0
attr = S, NA
damage = 30
hitflag = MAFD
guardflag = MA
pausetime = 0, 0
ground.velocity = -4
ground.hittime = 12
priority = 3

[State 200, modify]
type = ModifyHitDef
trigger1 = Time = 0
damage = 55
ground.hittime = 21
ground.velocity = -7, -2

[State 200, end]
type = ChangeState
trigger1 = AnimTime = 0
value = 0
ctrl = 1
";

#[test]
fn modifyhitdef_changes_only_the_listed_fields_of_the_live_hitdef() {
    let mut m = duel(mugen_char("p1", JAB_WITH_MODIFY), mugen_char("p2", ""));
    face_off(&mut m, 40.0);
    enter(&mut m, 0, 200);
    m.tick(&[0, 0]);
    let hd = m.fighters[0].hitdef.clone().expect("hitdef");
    assert_eq!((hd.damage, hd.ground_hit_time, hd.ground_vel), (55.0, 21, [-7.0, -2.0]));
    assert_eq!(hd.priority, 3, "unlisted fields are untouched");
    assert_eq!(hd.guard_pause_p1, 0);
    ticks(&mut m, 4);
    assert_eq!(m.fighters[1].life, 1000.0 - 55.0);
    assert_eq!(m.fighters[1].ghv.hittime, 21);
}

#[test]
fn modifyhitdef_without_a_hitdef_does_nothing() {
    let cns = "
[Statedef 550]
type = S
physics = S

[State 550, m]
type = ModifyHitDef
trigger1 = 1
damage = 99
";
    let mut m = duel(mugen_char("p1", cns), mugen_char("p2", ""));
    enter(&mut m, 0, 550);
    m.tick(&[0, 0]);
    assert!(m.fighters[0].hitdef.is_none());
}

#[test]
fn modifyreversaldef_edits_the_pending_reversal() {
    let cns = "
[Statedef 400]
type = S
movetype = I
physics = S
anim = 300
ctrl = 0

[State 400, r]
type = ReversalDef
trigger1 = Time = 0
reversal.attr = SCA, NA
pausetime = 5, 7
p1stateno = 410
p2stateno = 420

[State 400, m]
type = ModifyReversalDef
trigger1 = Time = 0
p1stateno = 411
pausetime = 2, 3

[Statedef 410]
type = S
physics = S

[Statedef 411]
type = S
movetype = A
physics = S

[Statedef 420]
type = S
movetype = H
physics = S
";
    let jab = "
[Statedef 200]
type = S
movetype = A
physics = S
anim = 200
ctrl = 0

[State 200, hit]
type = HitDef
trigger1 = Time = 0
attr = S, NA
damage = 30
hitflag = MAFD
";
    let mut m = duel(mugen_char("rev", cns), mugen_char("atk", jab));
    face_off(&mut m, 40.0);
    enter(&mut m, 0, 400);
    enter(&mut m, 1, 200);
    m.tick(&[0, 0]);
    let (_, rd) = m.fighters[0].reversal.clone().expect("reversal");
    assert_eq!((rd.p1stateno, rd.p2stateno, rd.pause_p1, rd.pause_p2), (411, 420, 2, 3));
    ticks(&mut m, 5);
    assert_eq!(m.fighters[0].state_no, 411);
    assert_eq!(m.fighters[1].state_no, 420);
}

fn clsn_jab(modifier: &str) -> String {
    format!("
[Statedef 200]
type = S
movetype = A
physics = S
anim = 200
ctrl = 0

[State 200, hit]
type = HitDef
trigger1 = Time = 0
attr = S, NA
damage = 30
hitflag = MAFD
pausetime = 0, 0

{modifier}

[State 200, end]
type = ChangeState
trigger1 = AnimTime = 0
value = 0
ctrl = 1
")
}

fn jab_result(modifier: &str, gap: f32) -> (f32, bool) {
    let mut m = duel(mugen_char("p1", &clsn_jab(modifier)), mugen_char("p2", ""));
    face_off(&mut m, gap);
    enter(&mut m, 0, 200);
    ticks(&mut m, 5);
    let reach = m.fighters[0].boxes(1).first().map(|b| b[2] - m.fighters[0].pos.x).unwrap_or(0.0);
    (reach, m.fighters[1].life < 1000.0)
}

#[test]
fn clsn_controllers_change_collision() {
    assert_eq!(jab_result("", 100.0), (60.0, false), "baseline: out of reach");
    let (reach, hit) = jab_result("[State 200, o]\ntype = OverrideClsn\ntrigger1 = 1\nclsn = 1\nvalue = 0, -60, 120, -30", 100.0);
    assert!(hit && reach == 120.0, "OverrideClsn replaces the attack box");
    let (reach, hit) = jab_result("[State 200, r]\ntype = ResizeClsn\ntrigger1 = 1\nclsn = 1\nvalue = 0, 0, 50, 0", 100.0);
    assert!(hit && reach == 110.0, "ResizeClsn grows the right edge");
    let (reach, hit) = jab_result("[State 200, t]\ntype = TransformClsn\ntrigger1 = 1\nclsn = 1\nscale = 2, 1", 100.0);
    assert!(hit && reach == 120.0, "TransformClsn scales the box");
    let (reach, hit) = jab_result("[State 200, t]\ntype = TransformClsn\ntrigger1 = 1\nclsn = 1\noffset = 45, 0", 100.0);
    assert!(hit && reach == 105.0, "TransformClsn offsets the box");
}

#[test]
fn clsn_mods_expire_and_replace_by_slot() {
    let cns = "
[Statedef 560]
type = S
physics = S

[State 560, a]
type = ResizeClsn
trigger1 = Time = 0
clsn = 2
time = 2
value = 1, 1, 1, 1

[State 560, b]
type = ResizeClsn
trigger1 = Time = 0
clsn = 2
time = 2
value = 5, 5, 5, 5

[State 560, c]
type = OverrideClsn
trigger1 = Time = 0
clsn = 2
index = 0
time = 5
value = 0, 0, 10, 10
";
    let mut m = duel(mugen_char("p1", cns), mugen_char("p2", ""));
    enter(&mut m, 0, 560);
    m.tick(&[0, 0]);
    assert_eq!(m.fighters[0].clsn_mods.len(), 2, "the second ResizeClsn replaces the first");
    let base = m.fighters[0].pos;
    let b = m.fighters[0].boxes(2);
    assert_eq!(b, vec![[base.x, base.y, base.x + 10.0, base.y + 10.0]], "mods apply in the order they were added, so the later override replaces the resized box");
    m.tick(&[0, 0]);
    assert_eq!(m.fighters[0].clsn_mods.len(), 2);
    m.tick(&[0, 0]);
    assert_eq!(m.fighters[0].clsn_mods.len(), 1, "time = 2 expires");
    ticks(&mut m, 4);
    assert!(m.fighters[0].clsn_mods.is_empty());
}
