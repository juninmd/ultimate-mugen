use crate::error::ModelError;
use serde_json::Value;

const MAGIC: u32 = 0x4654_6C67;
const JSON_CHUNK: u32 = 0x4E4F_534A;
const BIN_CHUNK: u32 = 0x004E_4942;

pub struct Glb {
    pub json: Value,
    pub bin: Vec<u8>,
}

fn word(bytes: &[u8], at: usize) -> Result<u32, ModelError> {
    let slice = bytes
        .get(at..at + 4)
        .ok_or(ModelError::Container("truncated"))?;
    Ok(u32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]]))
}

pub fn parse(bytes: &[u8]) -> Result<Glb, ModelError> {
    if word(bytes, 0)? != MAGIC {
        return Err(ModelError::Container("bad magic"));
    }
    if word(bytes, 4)? != 2 {
        return Err(ModelError::Unsupported("glb version other than 2"));
    }
    let total = (word(bytes, 8)? as usize).min(bytes.len());
    let mut at = 12;
    let mut json = None;
    let mut bin = Vec::new();
    while at + 8 <= total {
        let len = word(bytes, at)? as usize;
        let kind = word(bytes, at + 4)?;
        let body = bytes
            .get(at + 8..at + 8 + len)
            .ok_or(ModelError::Container("chunk past end"))?;
        match kind {
            JSON_CHUNK => {
                json = Some(
                    serde_json::from_slice(body).map_err(|e| ModelError::Json(e.to_string()))?,
                )
            }
            BIN_CHUNK if bin.is_empty() => bin = body.to_vec(),
            _ => {}
        }
        at += 8 + len.div_ceil(4) * 4;
    }
    Ok(Glb {
        json: json.ok_or(ModelError::Missing("json chunk"))?,
        bin,
    })
}
