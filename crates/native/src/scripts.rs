use super::types::OneOrMany;
use ume_engine::chardef::{MoveInfo, StateDef};
use ume_engine::cns::parse_states;
use ume_engine::commands::{parse_command, CommandDef};
use ume_engine::zss::parse_state_file;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ScriptCommandSpec {
    pub name: String,
    pub input: String,
    #[serde(default)]
    pub display: Option<String>,
    #[serde(default)]
    pub time: Option<i32>,
    #[serde(default)]
    pub buffer: Option<i32>,
    #[serde(default, rename = "pauseBuffer")]
    pub pause_buffer: Option<i32>,
    #[serde(default)]
    pub cost: Option<f32>,
    #[serde(default)]
    pub state: Option<i32>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ScriptSpec {
    #[serde(default)]
    pub zss: Option<OneOrMany<String>>,
    #[serde(default)]
    pub cns: Option<OneOrMany<String>>,
    #[serde(default)]
    pub commands: Vec<ScriptCommandSpec>,
    #[serde(default, rename = "override")]
    pub override_existing: bool,
}

pub(super) fn script_commands(
    spec: Option<&ScriptSpec>,
    commands: &mut Vec<CommandDef>,
    movelist: &mut Vec<MoveInfo>,
) {
    let Some(spec) = spec else { return };
    for c in &spec.commands {
        let mut def = parse_command(
            &c.name,
            &c.input,
            c.time.unwrap_or(20),
            c.buffer.unwrap_or(4),
        );
        if let Some(pb) = c.pause_buffer {
            def.pause_buffer = pb.max(0);
        }
        commands.push(def);
        if let Some(display) = &c.display {
            movelist.push(MoveInfo {
                name: display.clone(),
                input: c.input.clone(),
                command: c.name.clone(),
                cost: c.cost.unwrap_or(0.0),
                drain: false,
                forms: u32::MAX,
                state: c.state.unwrap_or(-1),
            });
        }
    }
}

fn replace_tag(src: &str, tag: &str, map: &HashMap<String, i32>) -> String {
    let mut out = String::with_capacity(src.len());
    let mut rest = src;
    while let Some(pos) = rest.find(tag) {
        out.push_str(&rest[..pos]);
        let after = &rest[pos + tag.len()..];
        if let Some(end) = after.find(')') {
            let key = after[..end].trim().trim_matches('"');
            if let Some(id) = map.get(key) {
                out.push_str(&id.to_string());
            } else {
                out.push_str(tag);
                out.push_str(&after[..=end]);
            }
            rest = &after[end + 1..];
        } else {
            out.push_str(tag);
            rest = after;
        }
    }
    out.push_str(rest);
    out
}

fn expand_refs(
    src: &str,
    anim_no: &HashMap<String, i32>,
    move_states: &HashMap<String, i32>,
) -> String {
    let a = replace_tag(src, "@anim(", anim_no);
    replace_tag(&a, "@move(", move_states)
}

pub(super) fn compile_scripts(
    spec: Option<&ScriptSpec>,
    anim_no: &HashMap<String, i32>,
    move_states: &HashMap<String, i32>,
    states: &mut HashMap<i32, StateDef>,
) {
    let Some(spec) = spec else { return };
    if let Some(cns) = &spec.cns {
        let joined = cns.to_vec().join("\n\n");
        let expanded = expand_refs(&joined, anim_no, move_states);
        parse_states(&expanded, states, spec.override_existing);
    }
    if let Some(zss) = &spec.zss {
        let joined = zss.to_vec().join("\n\n");
        let expanded = expand_refs(&joined, anim_no, move_states);
        parse_state_file("umf_script.zss", &expanded, states, spec.override_existing);
    }
}
