use super::types::NativeFrame;
use serde::{Deserialize, Deserializer};
use serde_json::Value;

#[derive(Deserialize)]
#[serde(untagged)]
enum FrameForm {
    Short(String),
    Full(NativeFrame),
}

fn expand(text: &str) -> Result<NativeFrame, String> {
    let (pose, ticks) = text
        .rsplit_once('*')
        .ok_or_else(|| format!("frame `{text}` must look like `pose*ticks`"))?;
    let ticks = ticks
        .trim()
        .parse::<i32>()
        .map_err(|_| format!("frame `{text}` has no tick count after `*`"))?;
    if pose.trim().is_empty() {
        return Err(format!("frame `{text}` has no pose name"));
    }
    Ok(NativeFrame {
        pose: Some(Value::String(pose.trim().to_string())),
        effect: None,
        sprite: None,
        ticks,
        hit: None,
        hurt: None,
        offset: None,
    })
}

pub fn frames<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<NativeFrame>, D::Error> {
    Vec::<FrameForm>::deserialize(deserializer)?
        .into_iter()
        .map(|form| match form {
            FrameForm::Full(frame) => Ok(frame),
            FrameForm::Short(text) => expand(&text).map_err(serde::de::Error::custom),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::super::types::NativeAnim;

    #[test]
    fn short_and_full_frames_mix() {
        let anim: NativeAnim = serde_json::from_str(
            r#"{"frames":["idle0*10",{"pose":"idle1","ticks":8,"hit":{"part":"fist"}},"a*b*3"]}"#,
        )
        .unwrap();
        assert_eq!(anim.frames[0].ticks, 10);
        assert_eq!(anim.frames[1].hit.as_ref().unwrap().part, "fist");
        assert_eq!(anim.frames[2].pose.as_ref().unwrap(), "a*b");
    }

    #[test]
    fn malformed_short_frames_are_rejected() {
        for bad in [r#""idle""#, r#""idle*x""#, r#""*5""#] {
            let text = format!(r#"{{"frames":[{bad}]}}"#);
            assert!(serde_json::from_str::<NativeAnim>(&text).is_err(), "{bad}");
        }
    }
}
