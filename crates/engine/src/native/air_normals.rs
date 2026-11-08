use super::types::{HitSpec, NativeChar, NativeMove};

const HEAVY_PUNCH_DAMAGE_SHARE: f32 = 1.5;
const LIGHT_KICK_DAMAGE_SHARE: f32 = 0.7;
const DIVE_KICK_INPUT: &str = "D+b";

fn air_move<'a>(nc: &'a NativeChar, id: &str) -> Option<&'a NativeMove> {
    nc.moves.iter().find(|m| m.id == id && m.when.as_deref() == Some("air"))
}

fn taken(nc: &NativeChar, input: &str) -> bool {
    nc.moves.iter().any(|m| m.when.as_deref() == Some("air") && m.input == input)
}

fn variant(base: &NativeMove, id: &str, name: &str, input: &str, share: f32, level: &str) -> NativeMove {
    let mut m = base.clone();
    m.id = id.to_string();
    m.name = name.to_string();
    m.input = input.to_string();
    m.simple = None;
    m.cancels = None;
    m.hits = base.hits.iter().map(|h: &HitSpec| HitSpec { damage: (h.damage * share).round(), level: Some(level.to_string()), ..h.clone() }).collect();
    m
}

pub(super) fn complete(mut nc: NativeChar) -> NativeChar {
    if let Some(dive) = nc.moves.iter_mut().find(|m| m.id == "divekick" && m.when.as_deref() == Some("air") && m.input == "y") {
        dive.input = DIVE_KICK_INPUT.to_string();
    }
    let punch = air_move(&nc, "ajab").cloned();
    let kick = air_move(&nc, "akick").cloned();
    if let Some(punch) = punch.filter(|_| !taken(&nc, "y")) {
        nc.moves.push(variant(&punch, "astrong", "Air Heavy Punch", "y", HEAVY_PUNCH_DAMAGE_SHARE, "medium"));
    }
    if let Some(kick) = kick.filter(|_| !taken(&nc, "a")) {
        nc.moves.push(variant(&kick, "alkick", "Air Light Kick", "a", LIGHT_KICK_DAMAGE_SHARE, "light"));
    }
    nc
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fighter(extra: &str) -> NativeChar {
        let json = format!(r##"{{"format":"f","id":"t","name":"T","series":"s","author":"a","color":"#fff","art":{{"type":"rig"}},"anims":{{}},"moves":[
            {{"id":"ajab","name":"J","input":"x","kind":"normal","anim":"ajab","when":"air","hits":[{{"at":2,"damage":40}}]}},
            {{"id":"akick","name":"K","input":"b","kind":"normal","anim":"akick","when":"air","hits":[{{"at":2,"damage":60}}]}}{extra}]}}"##);
        serde_json::from_str(&json).unwrap()
    }

    #[test]
    fn the_air_gets_every_button_and_the_dive_kick_moves_to_down_kick() {
        let dive = r#",{"id":"divekick","name":"D","input":"y","kind":"normal","anim":"akick","when":"air","hits":[{"at":2,"damage":60}]}"#;
        let nc = complete(fighter(dive));
        let inputs: Vec<&str> = nc.moves.iter().filter(|m| m.when.as_deref() == Some("air")).map(|m| m.input.as_str()).collect();
        for expected in ["x", "b", "y", "a", "D+b"] { assert!(inputs.contains(&expected), "{inputs:?}"); }
        let heavy = nc.moves.iter().find(|m| m.id == "astrong").unwrap();
        assert_eq!((heavy.anim.as_str(), heavy.hits[0].damage), ("ajab", 60.0));
    }

    #[test]
    fn existing_air_inputs_are_kept() {
        let special = r#",{"id":"fallstar","name":"F","input":"y","kind":"special","anim":"akick","when":"air","hits":[{"at":2,"damage":60}]}"#;
        let nc = complete(fighter(special));
        assert!(nc.moves.iter().all(|m| m.id != "astrong") && nc.moves.iter().any(|m| m.id == "alkick"));
    }
}
