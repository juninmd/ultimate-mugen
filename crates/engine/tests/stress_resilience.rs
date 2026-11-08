use std::sync::Arc;
use ume_engine::ai::Ai;
use ume_engine::chardef::safe_expr;
use ume_engine::cns::parse_states;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::input::InputSource;
use ume_engine::loader::load_mugen_char;
use ume_engine::matchs::Match;
use ume_engine::triggers::Ctx;
use ume_formats::vfs::{add_zip, find_char_defs, Vfs};

fn load_kfm() -> ume_engine::chardef::CharDef {
    let zip_path = format!(
        "{}/../../assets/mugen/elecbyte.zip",
        env!("CARGO_MANIFEST_DIR")
    );
    let mut vfs = Vfs::new();
    add_zip(&mut vfs, &std::fs::read(zip_path).unwrap(), "").unwrap();
    let def = find_char_defs(&vfs).into_iter().next().unwrap();
    load_mugen_char(&vfs, &def, 1, true).unwrap()
}

#[test]
fn hostile_expressions_and_bytecode_limits() {
    let kfm = Arc::new(load_kfm());
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(kfm, 0, 0, 1);
    let ctx = Ctx { m: &m, i: 0 };

    let deep = format!("{}{}{}", "(".repeat(64), "1 + 2", ")".repeat(64));
    let deep_add = (0..48).map(|_| "1").collect::<Vec<_>>().join(" + (") + &")".repeat(47);
    let cases = [
        "",
        "(((",
        ")))",
        "1 / 0",
        "1.0 / 0.0",
        "10 % 0",
        "var(-999999) + var(999999) + fvar(-50) + fvar(5000) + sysvar(-1) + sysvar(99)",
        "var(999999) := 42",
        "fvar(-999999) := 1.0 / 0.0",
        "parent, var(0) + root, var(0) + helper(9999), time + target(9999), life + partner, life",
        "ifelse(parent, time, 1, 2)",
        "cond(enemynear(99), life > 0, 10, 20)",
        "clamp(10, 20, 5)",
        "min(parent, time, 5)",
        "max(5, parent, time)",
        "sqrt(-1) + ln(-1) + asin(99)",
        "timemod = 0, 0",
        "animelem = -99, >= 0",
        "projhit9999 = 1, < 5",
        "command = \"nonexistent_cmd\"",
        "hitdefattr = SCA, NA, SA, HA",
        &deep,
        &deep_add,
    ];
    for src in cases {
        let e = safe_expr(src);
        let _ = e.eval(&ctx);
    }
}

#[test]
fn entity_flood_and_recursive_helper_bounds() {
    let mut def = load_kfm();
    let flood_cns = r#"
[Statedef 0]
type = S
physics = S
sprpriority = 0

[State 0, RecursiveHelper]
type = Helper
trigger1 = 1
id = 777
stateno = 0
ownpal = 1

[State 0, FloodExplod]
type = Explod
trigger1 = 1
anim = 0
id = 888
removetime = -1

[State 0, FloodProj]
type = Projectile
trigger1 = 1
projid = 999
projanim = 0
projremovetime = -1
velocity = 0, 0

[State 0, FloodText]
type = Text
trigger1 = 1
text = "stress %d"
params = gametime
removetime = -1

[State 0, InfiniteLoopGuard]
type = ChangeState
trigger1 = time = 0
value = 0
"#;
    parse_states(flood_cns, &mut def.states, true);
    let def = Arc::new(def);
    let mut m = Match::new(common_def(), None, flat_stage());
    m.add_player(def.clone(), 0, 0, 1);
    m.add_player(def, 1, 0, 1);

    for _ in 0..300 {
        m.tick(&[0; 8]);
        assert!(m.fighters.len() <= 61);
        assert!(m.explods.len() <= 512);
        assert!(m.projectiles.len() <= 256);
        assert!(m.texts.len() <= 128);
    }
}

#[test]
fn simul_4v4_stress_match() {
    let kfm = Arc::new(load_kfm());
    let mut m = Match::new(common_def(), None, flat_stage());
    let mut ais = Vec::new();
    for side in 0..2 {
        for slot in 0..4 {
            m.add_player(kfm.clone(), side, 8, 1 + slot);
            ais.push(Ai::new(8, 0xCAFE_0000 + (side as u32) * 16 + slot as u32));
        }
    }
    let mut inputs = [0u32; 8];
    for _ in 0..1200 {
        for (idx, ai) in ais.iter_mut().enumerate() {
            inputs[idx] = ai.poll(&m, idx);
        }
        m.tick(&inputs);
        for f in &m.fighters {
            assert!(f.pos.x.is_finite() && f.pos.y.is_finite());
            assert!(f.vel.x.is_finite() && f.vel.y.is_finite());
            assert!(f.life.is_finite());
        }
    }
}
