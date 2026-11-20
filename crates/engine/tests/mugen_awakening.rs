use std::sync::Arc;
use ume_engine::common::{common_def, flat_stage};
use ume_engine::input::k;
use ume_engine::loader::load_mugen_char;
use ume_engine::matchs::Match;
use ume_formats::vfs::{add_zip, find_char_defs, Vfs};

#[test]
fn mugen_can_awaken_without_a_native_transformation_action() {
    let mut vfs = Vfs::new();
    add_zip(
        &mut vfs,
        include_bytes!("../../../assets/mugen/elecbyte.zip"),
        "",
    )
    .unwrap();
    let path = find_char_defs(&vfs).into_iter().next().unwrap();
    let mut fighter = load_mugen_char(&vfs, &path, 1, false).unwrap();
    fighter.anims.remove(&196);
    let fighter = Arc::new(fighter);
    let mut battle = Match::new(common_def(), None, flat_stage());
    battle.add_player(fighter.clone(), 0, 0, 1);
    battle.add_player(fighter, 1, 0, 2);
    battle.start_round();
    for _ in 0..700 {
        if battle.round_state == 2 {
            break;
        }
        battle.tick(&[0, 0]);
    }

    let base = &battle.fighters[0];
    assert_eq!(base.form, 0);
    let attack = base.effective_attack();
    let life = base.life - 100.0;
    battle.fighters[0].life = life;
    battle.fighters[0].power = 1000.0;
    battle.tick(&[k::T, 0]);

    let awakened = &battle.fighters[0];
    assert_eq!(awakened.form, 1);
    assert!(awakened.power < 2.0);
    assert_eq!(awakened.life, life);
    assert!(awakened.effective_attack() > attack);
    assert!((awakened.form_speed_multiplier() - 1.08).abs() < 0.001);
    assert!((awakened.exposed_damage_multiplier() - 1.15).abs() < 0.001);
    for _ in 0..75 {
        battle.tick(&[0, 0]);
    }
    assert!(battle.fighters[0].ctrl);
    battle.start_round();
    assert_eq!(battle.fighters[0].form, 0);
    assert!((battle.fighters[0].effective_attack() - attack).abs() < 0.001);
}
