use crate::accessor::Source;
use crate::error::ModelError;
use crate::math::{compose, Mat4, IDENTITY};
use serde_json::Value;

#[derive(Clone, Debug)]
pub struct Node {
    pub parent: Option<usize>,
    pub translation: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
    pub matrix: Option<Mat4>,
    pub skin: Option<usize>,
}

impl Node {
    pub fn local(&self) -> Mat4 {
        self.matrix
            .unwrap_or_else(|| compose(self.translation, self.rotation, self.scale))
    }
}

pub struct Skin {
    pub joints: Vec<usize>,
    pub inverse_bind: Vec<Mat4>,
}

fn array<const N: usize>(v: &Value, default: [f32; N]) -> [f32; N] {
    let Some(items) = v.as_array() else {
        return default;
    };
    let mut out = default;
    for (slot, item) in out.iter_mut().zip(items) {
        *slot = item.as_f64().unwrap_or(0.0) as f32;
    }
    out
}

pub fn read_nodes(json: &Value) -> Vec<Node> {
    let list = json["nodes"].as_array().map(Vec::as_slice).unwrap_or(&[]);
    let mut nodes: Vec<Node> = list
        .iter()
        .map(|n| Node {
            parent: None,
            translation: array(&n["translation"], [0.0; 3]),
            rotation: array(&n["rotation"], [0.0, 0.0, 0.0, 1.0]),
            scale: array(&n["scale"], [1.0; 3]),
            matrix: n.get("matrix").map(|m| array(m, IDENTITY)),
            skin: n["skin"].as_u64().map(|s| s as usize),
        })
        .collect();
    for (index, n) in list.iter().enumerate() {
        for child in n["children"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_u64)
        {
            if let Some(slot) = nodes.get_mut(child as usize) {
                slot.parent = Some(index);
            }
        }
    }
    nodes
}

pub fn read_skins(src: &Source) -> Result<Vec<Skin>, ModelError> {
    let mut skins = Vec::new();
    for s in src.json["skins"].as_array().into_iter().flatten() {
        let joints: Vec<usize> = s["joints"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_u64)
            .map(|j| j as usize)
            .collect();
        let inverse_bind = match s["inverseBindMatrices"].as_u64() {
            Some(acc) => src
                .floats(acc as usize)?
                .0
                .chunks_exact(16)
                .map(|c| <Mat4>::try_from(c).unwrap_or(IDENTITY))
                .collect(),
            None => vec![IDENTITY; joints.len()],
        };
        if inverse_bind.len() < joints.len() {
            return Err(ModelError::OutOfRange("inverse bind matrices"));
        }
        skins.push(Skin {
            joints,
            inverse_bind,
        });
    }
    Ok(skins)
}
