use crate::error::ModelError;
use serde_json::Value;

pub struct Source<'a> {
    pub json: &'a Value,
    pub bin: &'a [u8],
}

fn component_size(kind: u64) -> Result<usize, ModelError> {
    match kind {
        5120 | 5121 => Ok(1),
        5122 | 5123 => Ok(2),
        5125 | 5126 => Ok(4),
        _ => Err(ModelError::Unsupported("accessor component type")),
    }
}

fn width(kind: &str) -> Result<usize, ModelError> {
    match kind {
        "SCALAR" => Ok(1),
        "VEC2" => Ok(2),
        "VEC3" => Ok(3),
        "VEC4" => Ok(4),
        "MAT4" => Ok(16),
        _ => Err(ModelError::Unsupported("accessor type")),
    }
}

fn decode(raw: &[u8], kind: u64, normalized: bool) -> f32 {
    match (kind, normalized) {
        (5126, _) => f32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]),
        (5121, n) => {
            if n {
                raw[0] as f32 / 255.0
            } else {
                raw[0] as f32
            }
        }
        (5120, n) => {
            if n {
                (raw[0] as i8 as f32 / 127.0).max(-1.0)
            } else {
                raw[0] as i8 as f32
            }
        }
        (5123, n) => {
            let v = u16::from_le_bytes([raw[0], raw[1]]) as f32;
            if n {
                v / 65535.0
            } else {
                v
            }
        }
        (5122, n) => {
            let v = i16::from_le_bytes([raw[0], raw[1]]) as f32;
            if n {
                (v / 32767.0).max(-1.0)
            } else {
                v
            }
        }
        _ => u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]) as f32,
    }
}

impl Source<'_> {
    pub fn floats(&self, index: usize) -> Result<(Vec<f32>, usize), ModelError> {
        let acc = self.json["accessors"]
            .get(index)
            .ok_or(ModelError::OutOfRange("accessor"))?;
        if acc.get("sparse").is_some() {
            return Err(ModelError::Unsupported("sparse accessor"));
        }
        let kind = acc["componentType"]
            .as_u64()
            .ok_or(ModelError::Missing("componentType"))?;
        let lanes = width(
            acc["type"]
                .as_str()
                .ok_or(ModelError::Missing("accessor type"))?,
        )?;
        let count = acc["count"].as_u64().ok_or(ModelError::Missing("count"))? as usize;
        let normalized = acc["normalized"].as_bool().unwrap_or(false);
        let size = component_size(kind)?;
        let view = self.json["bufferViews"]
            .get(
                acc["bufferView"]
                    .as_u64()
                    .ok_or(ModelError::Missing("bufferView"))? as usize,
            )
            .ok_or(ModelError::OutOfRange("bufferView"))?;
        let base = (view["byteOffset"].as_u64().unwrap_or(0) as usize)
            .checked_add(acc["byteOffset"].as_u64().unwrap_or(0) as usize)
            .ok_or(ModelError::OutOfRange("accessor bounds"))?;
        let stride = view["byteStride"]
            .as_u64()
            .map(|s| s as usize)
            .unwrap_or(size * lanes);
        let span = count
            .checked_sub(1)
            .map_or(Some(0), |last| last.checked_mul(stride))
            .and_then(|v| v.checked_add(size * lanes))
            .and_then(|v| v.checked_add(base))
            .ok_or(ModelError::OutOfRange("accessor bounds"))?;
        if count > 0 && span > self.bin.len() {
            return Err(ModelError::OutOfRange("accessor bounds"));
        }
        let mut out = Vec::with_capacity(count * lanes);
        for item in 0..count {
            for lane in 0..lanes {
                let at = base + item * stride + lane * size;
                let raw = self
                    .bin
                    .get(at..at + size)
                    .ok_or(ModelError::OutOfRange("buffer read"))?;
                out.push(decode(raw, kind, normalized));
            }
        }
        Ok((out, lanes))
    }

    pub fn indices(&self, index: usize) -> Result<Vec<u32>, ModelError> {
        Ok(self
            .floats(index)?
            .0
            .into_iter()
            .map(|v| v as u32)
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read(count: u64, stride: u64) -> Result<(Vec<f32>, usize), ModelError> {
        let json = serde_json::json!({
            "accessors": [{"bufferView": 0, "componentType": 5126, "type": "VEC3", "count": count}],
            "bufferViews": [{"buffer": 0, "byteOffset": 0, "byteLength": 12, "byteStride": stride}]
        });
        let bin = [0u8; 12];
        Source { json: &json, bin: &bin }.floats(0)
    }

    #[test]
    fn forged_counts_and_strides_are_rejected_before_allocating() {
        assert!(read(1, 12).is_ok());
        assert!(read(1_000_000_000, 12).is_err());
        assert!(read(u64::MAX, 12).is_err());
        assert!(read(2, u64::MAX).is_err());
    }
}
