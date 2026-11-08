//! Built-in common states (original implementation of the standard MUGEN common state set).
use crate::chardef::{CharDef, Source};
use crate::cns::parse_states;
use crate::matchs::StageInfo;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

pub const COMMON_CNS: &str = include_str!("common1.cns");

pub fn common_def() -> Arc<CharDef> {
    static COMMON_DEF: OnceLock<Arc<CharDef>> = OnceLock::new();
    COMMON_DEF
        .get_or_init(|| {
            let mut d = CharDef::empty("common");
            let mut states = HashMap::new();
            parse_states(COMMON_CNS, &mut states, true);
            d.states = states;
            d.source = Source::Native;
            Arc::new(d)
        })
        .clone()
}

pub fn flat_stage() -> StageInfo { StageInfo::default() }
