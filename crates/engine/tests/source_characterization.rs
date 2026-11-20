use std::sync::Arc;
use ume_engine::chardef::{CharDef, Source};
use ume_engine::fighter::HitDefData;
use ume_engine::matchs::{Match, StageInfo};

fn fighter(source: Source) -> Arc<CharDef> {
    let mut cd = CharDef::empty("dummy");
    cd.local_w = 320.0;
    cd.local_h = 240.0;
    cd.source = source;
    Arc::new(cd)
}

fn life_after_each_hit(source: Source, hits: usize) -> Vec<f32> {
    let cd = fighter(source);
    let mut m = Match::new(cd.clone(), None, StageInfo::default());
    let a = m.add_player(cd.clone(), 0, 0, 1);
    let d = m.add_player(cd, 1, 0, 2);
    m.round_state = 2;
    m.fighters[d].life = 5000.0;
    m.fighters[d].life_max = 5000.0;
    let mut hd = HitDefData::default();
    hd.damage = 100.0;
    hd.p2stateno = 5000;
    hd.num_hits = 1;
    hd.p1stateno = -1;
    (0..hits)
        .map(|_| {
            m.apply_hit(a, d, &hd, (0.0, 0.0), None);
            m.fighters[d].life
        })
        .collect()
}

const MUGEN_LIFE: [f32; 12] = [4900.0, 4800.0, 4700.0, 4600.0, 4500.0, 4400.0, 4300.0, 4200.0, 4100.0, 4000.0, 3900.0, 3800.0];
const NATIVE_LIFE: [f32; 12] = [4900.0, 4800.0, 4710.0, 4630.0, 4560.0, 4500.0, 4450.0, 4410.0, 4380.0, 4355.0, 4330.0, 4305.0];

#[test]
fn mugen_combo_damage_is_never_scaled() {
    assert_eq!(life_after_each_hit(Source::Mugen, 12), MUGEN_LIFE);
}

#[test]
fn native_combo_damage_follows_the_combo_scale() {
    assert_eq!(life_after_each_hit(Source::Native, 12), NATIVE_LIFE);
}

#[test]
fn both_sources_agree_on_the_opening_hits() {
    assert_eq!(life_after_each_hit(Source::Mugen, 2), life_after_each_hit(Source::Native, 2));
}
