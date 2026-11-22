use super::types::NativeMove;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SpawnSpec {
    #[serde(default)]
    pub at: i32,
    pub state: i32,
    #[serde(default)]
    pub id: Option<i32>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub pos: Option<[f32; 2]>,
    #[serde(default)]
    pub postype: Option<String>,
    #[serde(default)]
    pub facing: Option<i32>,
    #[serde(default, rename = "ownPal")]
    pub own_pal: Option<i32>,
    #[serde(default, rename = "superMoveTime")]
    pub super_move_time: Option<i32>,
    #[serde(default, rename = "pauseMoveTime")]
    pub pause_move_time: Option<i32>,
}

fn line<T: std::fmt::Display>(key: &str, v: &Option<T>) -> String {
    v.as_ref().map(|v| format!("{key} = {v}\n")).unwrap_or_default()
}

pub(super) fn spawn_cns(m: &NativeMove, sid: i32) -> String {
    m.spawn.iter().map(|s| {
        let trigger = if s.at > 1 { format!("AnimElem = {}", s.at) } else { "Time = 0".to_string() };
        let name = s.name.as_ref().map(|n| format!("name = \"{}\"\n", n.replace('"', ""))).unwrap_or_default();
        let pos = s.pos.map(|p| format!("pos = {}, {}\n", p[0], p[1])).unwrap_or_default();
        format!(
            "[State {sid}, spawn]\ntype = Helper\ntrigger1 = {trigger}\nstateno = {}\n{}{name}{pos}{}{}{}{}{}\n",
            s.state, line("ID", &s.id), line("postype", &s.postype), line("facing", &s.facing), line("ownpal", &s.own_pal),
            line("supermovetime", &s.super_move_time), line("pausemovetime", &s.pause_move_time)
        )
    }).collect()
}

#[cfg(test)]
#[path = "move_spawn_tests.rs"]
mod tests;
