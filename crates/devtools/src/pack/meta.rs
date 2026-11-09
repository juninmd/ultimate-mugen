use serde_json::Value;
use std::collections::BTreeMap;

const ANCHOR_KEYS: [&str; 3] = ["axisX", "axisY", "refHeight"];

fn scaled(value: &Value, scale: f64) -> Option<Value> {
    value.as_f64().map(|v| Value::from(v * scale))
}

fn rescale_hair(values: &[Value], scale: f64) -> Value {
    Value::Array(values.iter().enumerate().map(|(index, v)| {
        let scaled = v.as_f64().unwrap_or(0.0) * scale;
        Value::from(if index < 2 { scaled.floor() } else { scaled.ceil() } as i64)
    }).collect())
}

pub fn rescale_meta(meta: &[u8], scales: &BTreeMap<String, f64>) -> Result<Vec<u8>, String> {
    let mut meta: Value = serde_json::from_slice(meta).map_err(|e| format!("meta.json: {e}"))?;
    let reference = meta.get("refHeight").and_then(Value::as_f64);
    let mut frames = meta.get("frames").and_then(Value::as_object).cloned().unwrap_or_default();
    for (name, scale) in scales.iter().filter(|(_, s)| **s != 1.0) {
        let stem = name.trim_end_matches(".png").to_string();
        match frames.get_mut(&stem).and_then(Value::as_object_mut) {
            None => {
                if let Some(reference) = reference {
                    frames.insert(stem, serde_json::json!({ "refHeight": reference * scale }));
                }
            }
            Some(frame) => {
                for key in ANCHOR_KEYS {
                    if let Some(value) = frame.get(key).and_then(|v| scaled(v, *scale)) {
                        frame.insert(key.to_string(), value);
                    }
                }
                if let Some(hair) = frame.get("awakeningHair").and_then(Value::as_array).filter(|a| a.len() == 4).cloned() {
                    frame.insert("awakeningHair".to_string(), rescale_hair(&hair, *scale));
                }
            }
        }
    }
    meta["frames"] = Value::Object(frames);
    serde_json::to_vec(&meta).map_err(|e| e.to_string())
}
