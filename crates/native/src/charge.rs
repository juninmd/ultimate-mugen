use super::abilities::AbilityCode;
use super::ability_spec::ChargeSpec;
use super::charge_ramp::gain_expression;
use ume_engine::chardef::MoveInfo;
use ume_engine::commands::parse_command;
use std::collections::HashMap;

pub(crate) use ume_engine::rules::{CHARGE_COMMAND, CHARGE_STATE};
const MINIMUM_CHARGE_TICKS: i32 = 6;
const POWER_UP_ANIM: i32 = 196;

pub(super) fn charge_code(spec: Option<&ChargeSpec>, anim_no: &HashMap<String, i32>) -> AbilityCode {
    let custom_anim = spec.and_then(|s| s.anim.as_deref());
    let anim = custom_anim.into_iter().chain(["charge", "powerUp"]).find_map(|name| anim_no.get(name).copied()).unwrap_or(POWER_UP_ANIM);
    let input = spec.and_then(|s| s.input.as_deref()).unwrap_or("/s");
    let display = if input == "/s" { "hold Start" } else { input };
    let color = spec.and_then(|s| s.aura_color).unwrap_or([30, 30, 80]);
    let shake = if spec.and_then(|s| s.shake).unwrap_or(false) {
        format!("[State {CHARGE_STATE}, shake]\ntype = EnvShake\ntrigger1 = Time % 8 = 0\ntime = 8\nampl = 3\nfreq = 80\n\n")
    } else { String::new() };
    let sound = spec.and_then(|s| s.sound).map(|s| {
        format!("[State {CHARGE_STATE}, snd]\ntype = PlaySnd\ntrigger1 = Time = 0\nvalue = {}, {}\n\n", s[0], s[1])
    }).unwrap_or_default();
    let explod = spec.and_then(|s| s.explod.as_deref()).and_then(|name| anim_no.get(name).copied()).map(|ea| {
        format!("[State {CHARGE_STATE}, aura]\ntype = Explod\ntrigger1 = Time = 0\nanim = {ea}\npos = 0, 0\nbindtime = -1\nremovetime = -1\nsprpriority = 3\nownpal = 1\n\n")
    }).unwrap_or_default();
    let mut code = AbilityCode::default();
    code.commands.push(parse_command(CHARGE_COMMAND, input, 1, 1));
    code.movelist.push(MoveInfo { name: "Charge Energy".into(), input: display.into(), command: CHARGE_COMMAND.into(), cost: 0.0, drain: false, forms: u32::MAX, state: -1 });
    code.minus1 += &format!("[State -1, charge]\ntype = ChangeState\nvalue = {CHARGE_STATE}\ntriggerall = !IsHelper && command = \"{CHARGE_COMMAND}\"\ntriggerall = power < powermax\ntrigger1 = ctrl && statetype = S\n\n");
    code.states += &format!(
        "[Statedef {CHARGE_STATE}]\ntype = S\nmovetype = I\nphysics = S\nanim = {anim}\nctrl = 0\nvelset = 0, 0\nsprpriority = 1\n\n\
         [State {CHARGE_STATE}, loop]\ntype = ChangeAnim\ntrigger1 = AnimTime = 0\nvalue = {anim}\n\n\
         [State {CHARGE_STATE}, power]\ntype = PowerAdd\ntrigger1 = 1\nvalue = {}\n\n\
         [State {CHARGE_STATE}, glow]\ntype = PalFX\ntrigger1 = 1\ntime = 2\nadd = {}, {}, {}\nsinadd = 40, 40, 70, 14\n\n\
         {shake}{sound}{explod}\
         [State {CHARGE_STATE}, done]\ntype = ChangeState\ntrigger1 = Time >= {MINIMUM_CHARGE_TICKS} && command != \"{CHARGE_COMMAND}\"\ntrigger2 = power >= powermax\nvalue = 0\nctrl = 1\n\n",
        gain_expression(), color[0], color[1], color[2]
    );
    code
}
