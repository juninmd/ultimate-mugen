#[cfg(test)]
mod tests {
    use super::super::{complete, AIR_DOWN_SPEED, AIR_FORWARD_SPEED, DOWN_LIGHT_KICK};
    use crate::native::{air_normals, audit_native_sprite_frames, NativeChar};

    fn akio() -> NativeChar {
        serde_json::from_str(include_str!("../../../../../assets/chars/akio.json")).unwrap()
    }

    #[test]
    fn every_roster_fighter_gets_ground_and_air_low_kicks() {
        for entry in std::fs::read_dir("../../assets/chars").unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_none_or(|e| e != "json")
                || path.file_stem().is_some_and(|s| s == "index")
            {
                continue;
            }
            let nc: NativeChar =
                serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
            let out = complete(air_normals::complete(nc));
            for id in ["lowkick", "airlowkick"] {
                assert!(
                    out.moves.iter().any(|m| m.id == id),
                    "{} lacks {id}",
                    out.id
                );
            }
        }
    }

    #[test]
    fn every_ground_and_air_low_kick_resolves_three_distinct_images() {
        for path in std::fs::read_dir("../../assets/chars").unwrap() {
            let path = path.unwrap().path();
            if path.extension().is_none_or(|extension| extension != "json")
                || path.file_stem().is_some_and(|stem| stem == "index")
            {
                continue;
            }
            let character: NativeChar =
                serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
            let character = complete(air_normals::complete(character));
            let animation = character.anims.get("lowKick").unwrap();
            assert_eq!(
                animation.frames.iter().map(|frame| frame.sprite.as_deref()).collect::<Vec<_>>(),
                vec![Some("crouch0"), Some("lowKick"), Some("crouch1"), Some("crouch")],
                "{} low kick frames",
                character.id
            );
            let audit = audit_native_sprite_frames(&character);
            for name in ["lowKick", "airLowKick"] {
                let report = audit["animations"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|animation| animation["animation"] == name)
                    .unwrap();
                assert!(
                    report["uniqueBodyImages"].as_u64().unwrap_or_default() >= 3,
                    "{}/{} should resolve at least three distinct images",
                    character.id,
                    name
                );
            }
        }
    }

    #[test]
    fn low_kicks_use_down_light_kick_and_low_guard() {
        let out = complete(air_normals::complete(akio()));
        for (id, when) in [("lowkick", "crouch"), ("airlowkick", "air")] {
            let attack = out.moves.iter().find(|m| m.id == id).unwrap();
            assert_eq!(
                (attack.input.as_str(), attack.when.as_deref()),
                (DOWN_LIGHT_KICK, Some(when))
            );
            assert!(attack
                .hits
                .iter()
                .all(|hit| hit.guard.as_deref() == Some("low")));
            assert!(out.anims.get(&attack.anim).is_some());
        }
        let animation = out.anims.get("airLowKick").unwrap();
        assert_eq!(animation.frames.len(), 5);
        assert_eq!(animation.frames[2].sprite.as_deref(), Some("airLowKick"));
        assert!(animation.frames[2].hit.is_some());
        let air = out.moves.iter().find(|m| m.id == "airlowkick").unwrap();
        assert_eq!(
            air.moves[0].vel,
            [Some(AIR_FORWARD_SPEED), Some(AIR_DOWN_SPEED)]
        );
        assert_eq!(air.hits[0].at, 3);
    }

    #[test]
    fn existing_authored_down_light_kicks_are_preserved() {
        let mut fighter = akio();
        let mut authored = fighter
            .moves
            .iter()
            .find(|m| m.id == "clk")
            .unwrap()
            .clone();
        authored.id = "authored_lowkick".into();
        authored.input = DOWN_LIGHT_KICK.into();
        fighter.moves.push(authored);
        let out = complete(air_normals::complete(fighter));
        assert!(!out.moves.iter().any(|m| m.id == "lowkick"));
        assert!(out.moves.iter().any(|m| m.id == "authored_lowkick"));
    }
}
