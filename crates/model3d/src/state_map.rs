use crate::error::ModelError;
use serde_json::Value;

#[derive(Clone, Debug, PartialEq)]
pub struct ClipBinding {
    pub clip: String,
    pub looping: bool,
    pub speed: f32,
}

#[derive(Clone, Debug)]
struct Range {
    from: i32,
    to: i32,
    binding: ClipBinding,
}

#[derive(Clone, Debug)]
pub struct StateMap {
    ranges: Vec<Range>,
    fallback: ClipBinding,
}

fn binding(v: &Value) -> Result<ClipBinding, ModelError> {
    Ok(ClipBinding {
        clip: v["clip"]
            .as_str()
            .ok_or(ModelError::Missing("clip name"))?
            .to_string(),
        looping: v["loop"].as_bool().unwrap_or(true),
        speed: v["speed"].as_f64().unwrap_or(1.0) as f32,
    })
}

impl StateMap {
    pub fn from_json(text: &str) -> Result<StateMap, ModelError> {
        let v: Value = serde_json::from_str(text).map_err(|e| ModelError::Json(e.to_string()))?;
        let mut ranges = Vec::new();
        for s in v["states"].as_array().into_iter().flatten() {
            let from = s["from"]
                .as_i64()
                .ok_or(ModelError::Missing("state from"))? as i32;
            let to = s["to"].as_i64().map(|t| t as i32).unwrap_or(from);
            ranges.push(Range {
                from,
                to,
                binding: binding(s)?,
            });
        }
        Ok(StateMap {
            ranges,
            fallback: binding(&v["fallback"])?,
        })
    }

    pub fn resolve(&self, stateno: i32) -> &ClipBinding {
        self.ranges
            .iter()
            .find(|r| (r.from..=r.to).contains(&stateno))
            .map(|r| &r.binding)
            .unwrap_or(&self.fallback)
    }
}

pub fn clip_time(binding: &ClipBinding, state_ticks: i32, duration: f32) -> f32 {
    let t = state_ticks.max(0) as f32 / 60.0 * binding.speed;
    if duration <= 0.0 {
        0.0
    } else if binding.looping {
        t % duration
    } else {
        t.min(duration)
    }
}
