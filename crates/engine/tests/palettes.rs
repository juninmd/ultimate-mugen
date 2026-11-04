mod support;
use std::sync::Arc;
use support::*;
use ume_engine::chardef::IndexedSprite;
use ume_engine::palette::{AfterImageParams, BlendMode, PalFx, PalParams, Trans};

const CNS: &str = "
[Statedef 300]
type = S
physics = S
anim = 0

[State 300, p]
type = PalFX
trigger1 = Time = 0
time = 1
add = 40, 0, 0
invertall = 1

[Statedef 310]
type = S
physics = S
anim = 0

[State 310, all]
type = AllPalFX
trigger1 = Time = 0
time = 5
mul = 128, 256, 256

[State 310, bg]
type = BGPalFX
trigger1 = Time = 0
time = 5
add = 0, 0, 90

[Statedef 320]
type = S
physics = S
anim = 0

[State 320, r]
type = RemapPal
trigger1 = Time = 0
source = 1, 1
dest = 1, 2

[State 320, t]
type = Trans
trigger1 = Time = 0
trans = addalpha
alpha = 128, 128

[Statedef 330]
type = S
physics = S

[State 330, r]
type = RemapPal
trigger1 = Time = 0
source = 1, 1
dest = -1, -1

[Statedef 340]
type = S
physics = S
anim = 0

[State 340, ai]
type = AfterImage
trigger1 = Time = 0
time = 30
length = 10
timegap = 2
framegap = 2
trans = add1
";

#[test]
fn palfx_with_time_one_lasts_exactly_one_tick() {
    let mut m = duel(mugen_char("a", CNS), mugen_char("b", ""));
    enter(&mut m, 0, 300);
    m.tick(&[0, 0]);
    let p = m.fighter_pal(0);
    assert!(p.invert && p.add[0] == 40.0, "visible on the tick it was set");
    m.tick(&[0, 0]);
    assert!(m.fighters[0].palfx.is_none(), "gone on the next tick");
}

#[test]
fn allpalfx_and_bgpalfx_are_separate() {
    let mut m = duel(mugen_char("a", CNS), mugen_char("b", ""));
    enter(&mut m, 0, 310);
    m.tick(&[0, 0]);
    assert_eq!(m.fighter_pal(1).mul[0], 128.0, "AllPalFX reaches every fighter");
    assert_eq!(m.fighter_pal(1).add[2], 0.0, "BGPalFX leaves fighters alone");
    let st = m.stage_pal();
    assert_eq!((st.add[2], st.mul[0]), (90.0, 128.0), "the stage gets both");
    ticks(&mut m, 6);
    assert!(m.all_palfx.is_none() && m.bg_palfx.is_none());
}

#[test]
fn palparams_apply_matches_mugen_formula() {
    let p = PalParams { add: [10.0, 0.0, 0.0], mul: [128.0, 256.0, 256.0], color: 256.0, invert: false };
    assert_eq!(p.apply([100, 50, 255]), [55, 50, 255]);
    let inv = PalParams { invert: true, ..Default::default() };
    assert_eq!(inv.apply([0, 100, 255]), [255, 155, 0]);
    let gray = PalParams { color: 0.0, ..Default::default() };
    assert_eq!(gray.apply([30, 60, 90]), [60, 60, 60]);
    let mut s = PalFx { sin_add: [64.0, 0.0, 0.0], sin_period: 20, ..Default::default() };
    s.elapsed = 5;
    assert!((s.params().add[0] - 64.0).abs() < 1e-3);
}

fn remap_char() -> Arc<ume_engine::chardef::CharDef> {
    let mut d = Arc::try_unwrap(mugen_char("pal", CNS)).ok().unwrap();
    d.indexed.insert((0, 0), IndexedSprite::new(2, 1, Arc::new(vec![0, 1]), (1, 1), None));
    let pal = |r: u8| { let mut p = vec![0u8; 1024]; p[4] = r; p[7] = 255; Arc::new(p) };
    d.palettes.insert((1, 1), pal(10));
    d.palettes.insert((1, 2), pal(200));
    Arc::new(d)
}

#[test]
fn remappal_recolors_through_the_palette_table() {
    let d = remap_char();
    assert!(d.remapped((0, 0), &[]).is_none());
    let img = d.remapped((0, 0), &[((1, 1), (1, 2))]).expect("remapped image");
    assert_eq!(&img.rgba[..], &[0, 0, 0, 0, 200, 0, 0, 255]);
    assert_eq!(d.remap_palette((0, 0), &[((1, 1), (1, 2))]).map(|p| p[4]), Some(200));
    assert!(d.remap_palette((0, 0), &[((1, 1), (1, 1))]).is_none());
    let again = d.remapped((0, 0), &[((1, 1), (1, 2))]).unwrap();
    assert_eq!(img.id, again.id, "cached");

    let mut m = duel(d, mugen_char("b", ""));
    enter(&mut m, 0, 320);
    m.tick(&[0, 0]);
    assert_eq!(m.fighters[0].remap, vec![((1, 1), (1, 2))]);
    assert_eq!(m.fighters[0].trans, Some(Trans { mode: BlendMode::AddAlpha, src: 128.0, dst: 128.0 }));
    enter(&mut m, 0, 330);
    m.tick(&[0, 0]);
    assert!(m.fighters[0].remap.is_empty(), "dest -1 restores the palette");
    assert!(m.fighters[0].trans.is_none(), "Trans only lasts for the tick it runs");
}

#[test]
fn afterimage_timegap_framegap_and_palette_steps() {
    let mut m = duel(mugen_char("a", CNS), mugen_char("b", ""));
    enter(&mut m, 0, 340);
    ticks(&mut m, 9);
    let ai = m.fighters[0].after_image.as_ref().expect("afterimage running");
    assert_eq!(ai.trail.len(), 5, "one sample every 2 ticks");
    assert_eq!(ai.params.trans.mode, BlendMode::Add1);
    assert_eq!(ai.params.shown(ai.trail.len(), ai.length), vec![2, 4]);
    let d = AfterImageParams::default();
    let (a, b) = (d.image_params(0), d.image_params(1));
    assert!(b.mul[0] < a.mul[0] && b.add[2] > a.add[2], "each later image is dimmer and bluer");
}

#[test]
fn trans_parsing() {
    assert_eq!(Trans::parse("add1", None), Trans { mode: BlendMode::Add1, src: 256.0, dst: 128.0 });
    assert_eq!(Trans::parse("SUB", Some((10.0, 20.0))).mode, BlendMode::Sub);
    assert_eq!(Trans::parse("addalpha", Some((300.0, -5.0))), Trans { mode: BlendMode::AddAlpha, src: 256.0, dst: 0.0 });
}
