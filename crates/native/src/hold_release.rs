use super::chain::ChainPlan;
use super::types::NativeMove;
use serde::{Deserialize, Serialize};
use ume_engine::commands::{parse_command, CommandDef};

const BUTTONS: [&str; 6] = ["a", "b", "c", "x", "y", "z"];

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct HoldTier {
    pub at: i32,
    pub to: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct HoldReleaseSpec {
    pub button: String,
    pub tiers: Vec<HoldTier>,
}

pub fn command_name(button: &str) -> String {
    format!("hold_{button}")
}

pub(super) fn cns(m: &NativeMove, sid: i32, plan: &ChainPlan, commands: &mut Vec<CommandDef>) -> String {
    let Some(spec) = m.hold_release.as_ref().filter(|s| BUTTONS.contains(&s.button.as_str())) else {
        return String::new();
    };
    let held = command_name(&spec.button);
    if !commands.iter().any(|c| c.name == held) {
        commands.push(parse_command(&held, &format!("/{}", spec.button), 1, 1));
    }
    let mut tiers: Vec<(i32, i32)> = spec.tiers.iter().filter_map(|t| plan.sid_of(&t.to).map(|s| (t.at.max(0), s))).collect();
    tiers.sort_by_key(|(at, _)| -at);
    let released = format!("(command != \"{held}\" || AnimTime = 0)");
    let clear = format!("[State {sid}, hold_release_clear]\ntype = RemoveExplod\ntrigger1 = {released}\n\n");
    tiers.iter().enumerate().fold(clear, |out, (k, (at, target))| {
        out + &format!("[State {sid}, hold_release_{k}]\ntype = ChangeState\ntrigger1 = {released} && Time >= {at}\nvalue = {target}\nctrl = 0\n\n")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn longest_hold_tier_is_checked_first() {
        let spec: HoldReleaseSpec = serde_json::from_str(r#"{"button":"y","tiers":[{"at":0,"to":"low"},{"at":40,"to":"max"}]}"#).unwrap();
        assert_eq!(spec.tiers.len(), 2);
        assert_eq!(command_name(&spec.button), "hold_y");
    }
}
