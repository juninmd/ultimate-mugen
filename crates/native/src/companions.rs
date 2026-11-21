use super::abilities::AbilityCode;
use super::clones::CloneSpec;
use super::types::{HitSpec, NativeMove, OneOrMany, ProjectileSpec, VelChange};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct StandMoveSpec {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub input: Option<String>,
    #[serde(default, rename = "triggerMove")]
    pub trigger_move: Option<String>,
    pub anim: String,
    #[serde(default)]
    pub cost: Option<f32>,
    #[serde(default)]
    pub detach: bool,
    #[serde(default)]
    pub hits: Vec<HitSpec>,
    #[serde(default)]
    pub moves: Vec<VelChange>,
    #[serde(default)]
    pub projectile: Option<OneOrMany<ProjectileSpec>>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct StandSpec {
    #[serde(default)]
    pub id: Option<i32>,
    #[serde(default)]
    pub input: Option<String>,
    #[serde(default, rename = "autoSummon")]
    pub auto_summon: bool,
    #[serde(default)]
    pub cost: Option<f32>,
    #[serde(default)]
    pub drain: Option<f32>,
    #[serde(default, rename = "idleAnim")]
    pub idle_anim: String,
    #[serde(default, rename = "despawnAnim")]
    pub despawn_anim: Option<String>,
    #[serde(default)]
    pub offset: Option<[f32; 2]>,
    #[serde(default)]
    pub bind: Option<bool>,
    #[serde(default)]
    pub hittable: Option<bool>,
    #[serde(default, rename = "lifeTransfer")]
    pub life_transfer: Option<f32>,
    #[serde(default, rename = "sprPriority")]
    pub spr_priority: Option<i32>,
    #[serde(default)]
    pub moves: Vec<StandMoveSpec>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CompanionsSpec {
    #[serde(default)]
    pub stand: Option<StandSpec>,
    #[serde(default)]
    pub clones: Option<CloneSpec>,
}

pub(super) fn compile(
    spec: Option<&CompanionsSpec>,
    anim_no: &HashMap<String, i32>,
    move_states: &HashMap<String, i32>,
    parent_moves: &[NativeMove],
) -> AbilityCode {
    let mut out = AbilityCode::default();
    let Some(spec) = spec else { return out };
    let st = super::stand::compile(spec.stand.as_ref(), anim_no, move_states, parent_moves);
    out.commands.extend(st.commands);
    out.minus1 += &st.minus1;
    out.minus2 += &st.minus2;
    out.states += &st.states;
    out.movelist.extend(st.movelist);
    let cl = super::clones::compile(spec.clones.as_ref(), anim_no, parent_moves);
    out.commands.extend(cl.commands);
    out.minus1 += &cl.minus1;
    out.minus2 += &cl.minus2;
    out.states += &cl.states;
    out.movelist.extend(cl.movelist);
    out
}
