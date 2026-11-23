use crate::accessor::Source;
use crate::error::ModelError;
use serde_json::Value;

pub struct Primitive {
    pub node: usize,
    pub positions: Vec<[f32; 3]>,
    pub joints: Vec<[u16; 4]>,
    pub weights: Vec<[f32; 4]>,
    pub indices: Vec<u32>,
    pub color: [f32; 3],
    pub uvs: Vec<[f32; 2]>,
    pub texture: Option<usize>,
    pub emissive: bool,
}

fn triples(data: Vec<f32>) -> Vec<[f32; 3]> {
    data.chunks_exact(3).map(|c| [c[0], c[1], c[2]]).collect()
}

fn quads(data: Vec<f32>) -> Vec<[f32; 4]> {
    data.chunks_exact(4)
        .map(|c| [c[0], c[1], c[2], c[3]])
        .collect()
}

fn texture_of(json: &Value, material: Option<u64>) -> Option<usize> {
    let texture = json["materials"].get(material? as usize)?["pbrMetallicRoughness"]
        ["baseColorTexture"]["index"]
        .as_u64()?;
    json["textures"].get(texture as usize)?["source"]
        .as_u64()
        .map(|s| s as usize)
}

fn emissive_of(json: &Value, material: Option<u64>) -> bool {
    material
        .and_then(|m| json["materials"].get(m as usize))
        .and_then(|m| m["emissiveFactor"].as_array())
        .is_some_and(|f| f.iter().any(|v| v.as_f64().unwrap_or(0.0) > 0.0))
}

fn base_color(json: &Value, material: Option<u64>) -> [f32; 3] {
    if texture_of(json, material).is_some() {
        return [1.0; 3];
    }
    let factor = material
        .and_then(|m| json["materials"].get(m as usize))
        .and_then(|m| m["pbrMetallicRoughness"]["baseColorFactor"].as_array());
    match factor {
        Some(f) => [0, 1, 2].map(|i| f.get(i).and_then(Value::as_f64).unwrap_or(1.0) as f32),
        None => [0.86, 0.52, 0.24],
    }
}

pub fn read_primitives(src: &Source) -> Result<Vec<Primitive>, ModelError> {
    let mut out = Vec::new();
    for (node, n) in src.json["nodes"]
        .as_array()
        .into_iter()
        .flatten()
        .enumerate()
    {
        let Some(mesh) = n["mesh"]
            .as_u64()
            .and_then(|m| src.json["meshes"].get(m as usize))
        else {
            continue;
        };
        for p in mesh["primitives"].as_array().into_iter().flatten() {
            if p["mode"].as_u64().unwrap_or(4) != 4 {
                return Err(ModelError::Unsupported(
                    "primitive mode other than triangles",
                ));
            }
            let attrs = &p["attributes"];
            let pos = attrs["POSITION"]
                .as_u64()
                .ok_or(ModelError::Missing("POSITION"))? as usize;
            let positions = triples(src.floats(pos)?.0);
            let joints = match attrs["JOINTS_0"].as_u64() {
                Some(a) => src
                    .floats(a as usize)?
                    .0
                    .chunks_exact(4)
                    .map(|c| [c[0] as u16, c[1] as u16, c[2] as u16, c[3] as u16])
                    .collect(),
                None => Vec::new(),
            };
            let weights = match attrs["WEIGHTS_0"].as_u64() {
                Some(a) => quads(src.floats(a as usize)?.0),
                None => Vec::new(),
            };
            let uvs = match attrs["TEXCOORD_0"].as_u64() {
                Some(a) => src
                    .floats(a as usize)?
                    .0
                    .chunks_exact(2)
                    .map(|c| [c[0], c[1]])
                    .collect(),
                None => Vec::new(),
            };
            let indices = match p["indices"].as_u64() {
                Some(a) => src.indices(a as usize)?,
                None => (0..positions.len() as u32).collect(),
            };
            if indices.iter().any(|&i| i as usize >= positions.len()) {
                return Err(ModelError::OutOfRange("vertex index"));
            }
            out.push(Primitive {
                node,
                positions,
                joints,
                weights,
                indices,
                color: base_color(src.json, p["material"].as_u64()),
                uvs,
                texture: texture_of(src.json, p["material"].as_u64()),
                emissive: emissive_of(src.json, p["material"].as_u64()),
            });
        }
    }
    Ok(out)
}
