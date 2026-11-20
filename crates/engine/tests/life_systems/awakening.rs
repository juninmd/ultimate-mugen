use super::*;

#[test]
fn awakening_increases_open_hit_damage_but_not_guard_chip() {
    let mut awakened = pair(&[]);
    let mut base = awakened.clone();
    awakened.fighters[1].power = 1000.0;
    assert!(awakened.try_transform(1));
    ticks(&mut awakened, 75);
    ticks(&mut base, 75);

    let mut hd = hit(100.0);
    hd.guard_damage = 10.0;
    let mut base_guarding = base.clone();
    let mut awakened_guarding = awakened.clone();
    for battle in [&mut base_guarding, &mut awakened_guarding] {
        enter(battle, 1, 120);
        ticks(battle, 1);
        enter(battle, 1, 120);
        battle.apply_hit(0, 1, &hd, (0.0, 0.0), None);
        assert!(battle.fighters[1].ghv.guarded);
        assert_eq!(battle.fighters[1].ghv.damage, 10.0);
    }
    base.apply_hit(0, 1, &hd, (0.0, 0.0), None);
    awakened.apply_hit(0, 1, &hd, (0.0, 0.0), None);
    assert_eq!(base.fighters[1].ghv.damage, 100.0);
    assert_eq!(awakened.fighters[1].ghv.damage, 115.0);
}
