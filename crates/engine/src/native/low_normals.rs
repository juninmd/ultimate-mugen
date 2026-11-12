use super::types::{NativeAnim, NativeChar, NativeMove, VelChange};

const DOWN_LIGHT_KICK: &str = "D+a";
const AIR_FORWARD_SPEED: f32 = 2.2;
const AIR_DOWN_SPEED: f32 = 2.2;

fn move_by_id(nc: &NativeChar, id: &str) -> Option<NativeMove> {
    nc.moves.iter().find(|m| m.id == id).cloned()
}

fn animation(nc: &NativeChar, id: &str) -> Option<NativeAnim> {
    nc.anims.get(id).cloned()
}

fn air_low_animation(nc: &NativeChar) -> Option<NativeAnim> {
    let kick = animation(nc, "akick")?;
    let mut start = kick.frames.first()?.clone();
    start.ticks = 3;
    start.hit = None;
    let mut prep = kick.frames.get(2)?.clone();
    prep.ticks = 2;
    prep.hit = None;
    let mut strike = prep.clone();
    strike.ticks = 4;
    strike.sprite = Some("airLowKick".into());
    strike.hit = kick.frames.get(1)?.hit.clone();
    let landing = animation(nc, "jumpDown")?;
    let mut recovery = landing.frames.first()?.clone();
    recovery.ticks = 5;
    recovery.hit = None;
    let mut fall = landing.frames.last()?.clone();
    fall.ticks = -1;
    fall.hit = None;
    Some(NativeAnim {
        frames: vec![start, prep, strike, recovery, fall],
        loop_start: None,
    })
}

fn add_animation(nc: &mut NativeChar, id: &str, source: &str) {
    if nc.anims.get(id).is_none() {
        if let Some(anim) = animation(nc, source) {
            nc.anims.0.push((id.into(), anim));
        }
    }
}

fn low_move(base: &NativeMove, id: &str, name: &str, anim: &str, when: &str) -> NativeMove {
    let mut m = base.clone();
    m.id = id.into();
    m.name = name.into();
    m.input = DOWN_LIGHT_KICK.into();
    m.simple = None;
    m.anim = anim.into();
    m.when = Some(when.into());
    m.cancels = None;
    for hit in &mut m.hits {
        hit.guard = Some("low".into());
        hit.level = Some("light".into());
        if id == "airlowkick" {
            hit.at = 3;
        }
    }
    m
}

fn air_low_move(base: &NativeMove) -> NativeMove {
    let mut m = low_move(base, "airlowkick", "Air Low Kick", "airLowKick", "air");
    m.moves = vec![VelChange {
        at: 1,
        vel: [Some(AIR_FORWARD_SPEED), Some(AIR_DOWN_SPEED)],
    }];
    m
}

fn taken(nc: &NativeChar, id: &str, when: &str) -> bool {
    nc.moves
        .iter()
        .any(|m| m.id == id || (m.input == DOWN_LIGHT_KICK && m.when.as_deref() == Some(when)))
}

pub(super) fn complete(mut nc: NativeChar) -> NativeChar {
    if let Some(base) = move_by_id(&nc, "clk").filter(|m| m.when.as_deref() == Some("crouch")) {
        if !taken(&nc, "lowkick", "crouch") {
            add_animation(&mut nc, "lowKick", &base.anim);
            nc.moves
                .push(low_move(&base, "lowkick", "Low Kick", "lowKick", "crouch"));
        }
    }
    if let Some(base) = move_by_id(&nc, "akick").filter(|m| m.when.as_deref() == Some("air")) {
        if !taken(&nc, "airlowkick", "air") {
            if nc.anims.get("airLowKick").is_none() {
                if let Some(anim) = air_low_animation(&nc) {
                    nc.anims.0.push(("airLowKick".into(), anim));
                }
            }
            nc.moves.push(air_low_move(&base));
        }
    }
    nc
}

#[cfg(test)]
mod tests {
    use super::*;
    fn akio() -> NativeChar {
        serde_json::from_str(include_str!("../../../../assets/chars/akio.json")).unwrap()
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
            if nc.npc {
                continue;
            }
            let out = complete(super::super::air_normals::complete(nc));
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
    fn low_kicks_use_down_light_kick_and_low_guard() {
        let out = complete(super::super::air_normals::complete(akio()));
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
        let out = complete(super::super::air_normals::complete(fighter));
        assert!(!out.moves.iter().any(|m| m.id == "lowkick"));
        assert!(out.moves.iter().any(|m| m.id == "authored_lowkick"));
    }
}
