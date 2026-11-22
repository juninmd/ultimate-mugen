use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct MugenFile {
    pub name: String,
    pub text: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MugenSpec {
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub info: BTreeMap<String, String>,
    #[serde(default, rename = "mugenVersion", skip_serializing_if = "Option::is_none")]
    pub mugen_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub localcoord: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stcommon: Option<MugenFile>,
    pub cns: MugenFile,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub st: Vec<MugenFile>,
    pub cmd: MugenFile,
    pub anims: BTreeMap<i32, ExactAnim>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub names: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ExactAnim {
    #[serde(default, rename = "loop", skip_serializing_if = "is_zero")]
    pub loop_start: usize,
    pub frames: Vec<ExactFrame>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ExactFrame {
    pub g: i32,
    pub i: i32,
    #[serde(default, skip_serializing_if = "is_zero_f")]
    pub x: f32,
    #[serde(default, skip_serializing_if = "is_zero_f")]
    pub y: f32,
    pub t: i32,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub flip: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub blend: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alpha: Option<[i32; 2]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<[f32; 2]>,
    #[serde(default, skip_serializing_if = "is_zero_f")]
    pub angle: f32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub clsn1: Vec<[f32; 4]>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub clsn2: Vec<[f32; 4]>,
}

fn is_zero(v: &usize) -> bool {
    *v == 0
}

fn is_zero_f(v: &f32) -> bool {
    *v == 0.0 && v.is_sign_positive()
}
