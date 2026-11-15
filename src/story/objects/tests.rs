use super::*;
use crate::story::STAGES;
use ume_engine::matchs::Match;

#[test]
fn the_bundled_catalog_covers_every_story_stage_with_known_kinds_and_food() {
    let cat = Catalog::parse(DEFAULT_TEXT).expect("objects.toml parses");
    for stage in STAGES.iter() {
        let entry = cat
            .stages
            .get(stage.arena)
            .unwrap_or_else(|| panic!("{} has objects", stage.arena));
        let bonus = entry.bonus.iter().flat_map(|b| b.objects.iter());
        for place in entry.objects.iter().chain(bonus) {
            assert!(
                cat.kinds.contains_key(&place.kind),
                "{} uses unknown kind {}",
                stage.arena,
                place.kind
            );
            assert!(place.fight < crate::story::fights_in(stage));
            if let Some(drop) = &place.drop {
                assert!(cat.pickups.contains_key(drop), "unknown food {drop}");
            }
        }
        assert!(
            (0..crate::story::fights_in(stage)).all(|f| entry.objects.iter().any(|p| p.fight == f))
        );
    }
    assert!(cat
        .kinds
        .values()
        .all(|k| k.frames >= 2 && crate::story::objects_textures::is_bundled(&k.sprite)));
    assert!(cat
        .pickups
        .values()
        .all(|f| crate::story::objects_textures::is_bundled(&f.sprite)));
}

#[test]
fn wave_objects_stay_inside_the_fight_range_and_two_stages_have_a_bonus_round() {
    let cat = Catalog::parse(DEFAULT_TEXT).unwrap();
    for entry in cat.stages.values() {
        assert!(entry
            .objects
            .iter()
            .all(|p| p.x.abs() < crate::story::FIGHT_HALF_RANGE));
    }
    assert_eq!(
        STAGES
            .iter()
            .filter(|s| cat.bonus(s.arena).is_some())
            .count(),
        2
    );
}

#[test]
fn spawning_places_objects_on_the_strip_from_the_current_fight_onwards() {
    let mut m = Match::new(
        ume_engine::common::common_def(),
        None,
        ume_engine::common::flat_stage(),
    );
    let def = &STAGES[0];
    spawn_stage(&mut m, def, 2);
    let cat = catalog();
    let expected = cat.stages["brazil"]
        .objects
        .iter()
        .filter(|p| p.fight >= 2)
        .count();
    assert_eq!(m.objects.breakables.len(), expected);
    let first_center = crate::story::fight_center(def, 2);
    assert!(m
        .objects
        .breakables
        .iter()
        .all(|b| b.x > first_center - crate::story::FIGHT_HALF_RANGE));
    assert!(m
        .objects
        .breakables
        .iter()
        .all(|b| b.lane >= def.lane_far && b.lane <= def.lane_near));
    assert!(spawn_bonus(&mut m, def, 0.0));
    assert!(
        m.objects.breakables.iter().any(|b| b.solid),
        "the bonus car is solid"
    );
    assert!(m.objects.goal_ticks_left().is_some());
    assert!(
        !spawn_bonus(&mut m, &STAGES[1], 0.0),
        "the subway has no bonus round"
    );
}
