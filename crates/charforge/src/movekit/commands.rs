use super::controller::Ctrl;
use super::kinds::is_air;
use super::value::{get_int, get_text};
use toml::Table;

fn kind_of(move_: &Table) -> String {
    get_text(move_, "kind").unwrap_or_else(|| "normal".into())
}

pub fn command_name(move_: &Table) -> String {
    get_text(move_, "command_name").unwrap_or_else(|| format!("spec_{}", get_int(move_, "state", 0)))
}

pub fn command_entries(moves: &[Table]) -> String {
    let mut seen: Vec<String> = Vec::new();
    let mut out = String::new();
    for move_ in moves {
        let Some(command) = get_text(move_, "command") else { continue };
        let name = command_name(move_);
        if seen.contains(&name) {
            continue;
        }
        out += &format!("[Command]\nname = \"{name}\"\ncommand = {command}\ntime = {}\n\n", get_int(move_, "command_time", 15));
        seen.push(name);
    }
    out
}

fn order(kind: &str) -> i32 {
    match kind {
        "super" => 0,
        "air_special" | "special" | "projectile" => 1,
        "throw" => 2,
        _ => 3,
    }
}

pub fn entry_block(moves: &[Table]) -> String {
    let normals: Vec<i64> = moves.iter().filter(|m| matches!(kind_of(m).as_str(), "normal" | "crouch" | "air")).map(|m| get_int(m, "state", 0)).collect();
    let contact = match (normals.iter().min(), normals.iter().max()) {
        (Some(low), Some(high)) => format!("((stateno = [{low}, {high}]) && (movecontact))"),
        _ => "0".to_string(),
    };
    let mut commanded: Vec<&Table> = moves.iter().filter(|m| m.contains_key("command")).collect();
    commanded.sort_by_key(|m| (order(&kind_of(m)), std::cmp::Reverse(get_text(m, "command").map(|c| c.len()).unwrap_or(0))));
    let mut out = "[Statedef -1]\n\n".to_string();
    for move_ in commanded {
        let kind = kind_of(move_);
        let mut conditions = vec!["roundstate = 2".to_string(), (if is_air(&kind) { "statetype = A" } else { "statetype != A" }).to_string()];
        conditions.push(if matches!(kind.as_str(), "normal" | "crouch" | "air" | "throw") { "ctrl".into() } else { format!("(ctrl) || {contact}") });
        if let Some(cost) = move_.get("power").and_then(|v| v.as_integer()) {
            conditions.push(format!("power >= {}", cost.abs()));
        } else if kind == "super" {
            conditions.push("power >= 3000".into());
        }
        if let Some(extra) = get_text(move_, "condition") {
            conditions.push(extra);
        }
        let name = get_text(move_, "name").unwrap_or_else(|| get_int(move_, "state", 0).to_string());
        let triggers: Vec<String> = conditions.iter().map(|c| format!("({c})")).collect();
        out += &Ctrl::new("ChangeState", &format!("spec: {name}")).field("value", get_int(move_, "state", 0))
            .many("triggerall", &triggers).field("trigger1", format!("command = \"{}\"", command_name(move_))).finish();
    }
    out
}
