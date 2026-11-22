use crate::chardef::Animation;
use std::collections::HashMap;
use ume_formats::sff::DecodedSff;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MugenText {
    pub name: String,
    pub text: String,
}

impl MugenText {
    pub fn new(name: impl Into<String>, text: impl Into<String>) -> Self {
        MugenText { name: name.into(), text: text.into() }
    }
}

#[derive(Clone, Debug, Default)]
pub struct MugenSources {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub author: String,
    pub localcoord: Option<String>,
    pub mugen_version: Option<String>,
    pub stcommon: Option<MugenText>,
    pub cns: MugenText,
    pub st: Vec<MugenText>,
    pub cmd: MugenText,
    pub sff: Option<DecodedSff>,
    pub anims: HashMap<i32, Animation>,
    pub sounds: HashMap<(i32, i32), Vec<u8>>,
    pub pal_files: Vec<Option<Vec<u8>>>,
}
