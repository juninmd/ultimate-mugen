use super::abilities::AbilityCode;
use super::charge_ramp::gain_expression;
use crate::chardef::MoveInfo;
use crate::commands::parse_command;
use std::collections::HashMap;

pub(crate) const CHARGE_STATE: i32 = 3920;
pub(crate) const CHARGE_COMMAND: &str = "charge_cmd";
const MINIMUM_CHARGE_TICKS: i32 = 6;
const POWER_UP_ANIM: i32 = 196;

pub(super) fn charge_code(anim_no: &HashMap<String, i32>) -> AbilityCode {
    let anim = ["charge", "powerUp"].iter().find_map(|name| anim_no.get(*name).copied()).unwrap_or(POWER_UP_ANIM);
    let mut code = AbilityCode::default();
    code.commands.push(parse_command(CHARGE_COMMAND, "/s", 1, 1));
    code.movelist.push(MoveInfo { name: "Charge Energy".into(), input: "hold Start".into(), command: CHARGE_COMMAND.into(), cost: 0.0, drain: false, forms: u32::MAX });
    code.minus1 += &format!("[State -1, charge]\ntype = ChangeState\nvalue = {CHARGE_STATE}\ntriggerall = command = \"{CHARGE_COMMAND}\"\ntriggerall = power < powermax\ntrigger1 = ctrl && statetype = S\n\n");
    code.states += &format!(
        "[Statedef {CHARGE_STATE}]\ntype = S\nmovetype = I\nphysics = S\nanim = {anim}\nctrl = 0\nvelset = 0, 0\nsprpriority = 1\n\n\
         [State {CHARGE_STATE}, loop]\ntype = ChangeAnim\ntrigger1 = AnimTime = 0\nvalue = {anim}\n\n\
         [State {CHARGE_STATE}, power]\ntype = PowerAdd\ntrigger1 = 1\nvalue = {}\n\n\
         [State {CHARGE_STATE}, glow]\ntype = PalFX\ntrigger1 = 1\ntime = 2\nadd = 30, 30, 80\nsinadd = 40, 40, 70, 14\n\n\
         [State {CHARGE_STATE}, done]\ntype = ChangeState\ntrigger1 = Time >= {MINIMUM_CHARGE_TICKS} && command != \"{CHARGE_COMMAND}\"\ntrigger2 = power >= powermax\nvalue = 0\nctrl = 1\n\n",
        gain_expression()
    );
    code
}
