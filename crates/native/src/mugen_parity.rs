use super::pack::read_sprite;
use super::pixel::sprite_roots;
use super::types::NativeChar;
use serde_json::Value;

pub(crate) fn exact_boxes(nc: &NativeChar) -> bool {
    nc.art.style.get("boxes").and_then(Value::as_str) == Some("exact")
}

pub(crate) fn pack_sounds(nc: &NativeChar) -> Vec<((i32, i32), Vec<u8>)> {
    let roots = sprite_roots(nc.sprite_dir());
    let Some((root, meta)) = roots.iter().find_map(|root| Some((root, serde_json::from_slice::<Value>(&read_sprite(root, "meta.json")?).ok()?))) else { return vec![] };
    let keys = meta.get("sounds").and_then(Value::as_array).cloned().unwrap_or_default();
    keys.iter()
        .filter_map(|key| Some((key.get(0)?.as_u64()? as i32, key.get(1)?.as_u64()? as i32)))
        .filter_map(|(g, i)| Some(((g, i), read_sprite(root, &format!("snd_{g}_{i}.wav"))?.to_vec())))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fighter(extra: &str) -> NativeChar {
        let mut v: Value = serde_json::from_str(include_str!("../../../assets/chars/kaito.json")).unwrap();
        v["id"] = Value::from(format!("parity_{extra}"));
        v["art"]["style"]["boxes"] = Value::from(extra);
        serde_json::from_value(v).unwrap()
    }

    #[test]
    fn exact_boxes_only_when_asked() {
        assert!(exact_boxes(&fighter("exact")));
        assert!(!exact_boxes(&fighter("fit")));
    }

    #[test]
    fn sounds_listed_in_meta_are_loaded_from_the_pack() {
        let wav = b"RIFF0000WAVE".to_vec();
        let pack = crate::pack_build::encode_pack("parity_exact", &[("meta.json".into(), br#"{"sounds":[[1,2],[9,9]]}"#.to_vec()), ("snd_1_2.wav".into(), wav.clone())]).unwrap();
        crate::pack::insert("parity_exact", &pack).unwrap();
        assert_eq!(pack_sounds(&fighter("exact")), vec![((1, 2), wav)]);
    }

    #[test]
    fn move_sounds_play_at_their_frame() {
        let mut m = fighter("exact").moves[0].clone();
        m.voice = None;
        m.sounds = serde_json::from_str(r#"[{"at":3,"snd":[2,1]},{"at":0,"snd":[5,4],"fight":true}]"#).unwrap();
        let cns = crate::compile::extras::voice_state(&m, 200);
        assert!(cns.contains("trigger1 = AnimElem = 3\nvalue = 2, 1"));
        assert!(cns.contains("trigger1 = AnimElem = 1\nvalue = F5, 4"));
    }
}
