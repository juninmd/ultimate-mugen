use ume_native::{parse_char, NativeChar};

fn roster() -> Vec<NativeChar> {
    let dir = format!("{}/../../assets/chars", env!("CARGO_MANIFEST_DIR"));
    let mut fighters: Vec<NativeChar> = std::fs::read_dir(dir)
        .unwrap()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json") && path.file_name().unwrap() != "index.json")
        .map(|path| parse_char(&std::fs::read_to_string(&path).unwrap()).unwrap_or_else(|error| panic!("{}: {error}", path.display())))
        .collect();
    fighters.sort_by(|a, b| a.id.cmp(&b.id));
    fighters
}

fn count(fighter: &NativeChar, kind: &str) -> usize {
    fighter.moves.iter().filter(|m| m.kind == kind).count()
}

#[test]
fn every_native_fighter_is_umf2() {
    for fighter in roster() {
        assert_eq!(fighter.format, "umf/2", "{} still declares {}", fighter.id, fighter.format);
    }
}

#[test]
fn roster_keeps_the_move_depth_floor() {
    for fighter in roster() {
        let floor = (17, 9, 3);
        assert!(count(&fighter, "normal") >= floor.0, "{} normals", fighter.id);
        assert!(count(&fighter, "special") >= floor.1, "{} specials", fighter.id);
        assert!(count(&fighter, "super") >= floor.2, "{} supers", fighter.id);
        let projectiles = fighter.moves.iter().filter(|m| m.projectile.is_some()).count();
        assert!(projectiles >= 3, "{} needs a projectile per button, has {projectiles}", fighter.id);
    }
}

#[test]
fn frame_data_stays_sane() {
    for fighter in roster() {
        for (name, anim) in &fighter.anims.0 {
            assert!(!anim.frames.is_empty(), "{}:{name} has no frames", fighter.id);
            assert!(anim.frames.iter().all(|frame| frame.ticks != 0), "{}:{name} has a zero tick frame", fighter.id);
        }
        for m in &fighter.moves {
            for hit in &m.hits {
                assert!(hit.damage > 0.0 && hit.at >= 0, "{}:{} has an invalid hit", fighter.id, m.id);
            }
        }
    }
}
