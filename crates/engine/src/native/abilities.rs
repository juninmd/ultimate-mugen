use super::ability_spec::AbilitySpec;
use crate::chardef::MoveInfo;
use crate::commands::{parse_command, CommandDef};
use std::collections::HashMap;

const FLY_STATE: i32 = 3900;
const FLY_FALL_STATE: i32 = 3901;
const TELEPORT_STATE: i32 = 3910;
const SPEED_VAR: i32 = 58;
const AFTER_IMAGE: &str = "type = AfterImage\nframegap = 2\npalbright = 0, 0, 0\npalcontrast = 100, 150, 256\npaladd = 0, 0, 0\npalmul = .85, .85, .85\ntime = 3\nlength = 6\n";

#[derive(Default)]
pub(super) struct AbilityCode {
    pub commands: Vec<CommandDef>,
    pub minus1: String,
    pub minus2: String,
    pub states: String,
    pub movelist: Vec<MoveInfo>,
}

fn entry(name: &str, input: &str, command: &str, cost: f32, drain: bool) -> MoveInfo {
    MoveInfo { name: name.into(), input: input.into(), command: command.into(), cost, drain, forms: u32::MAX }
}

fn anim_of(anim_no: &HashMap<String, i32>, wanted: Option<&String>, fallbacks: &[&str], default: i32) -> i32 {
    wanted.into_iter().map(String::as_str).chain(fallbacks.iter().copied()).find_map(|name| anim_no.get(name).copied()).unwrap_or(default)
}

pub(super) fn ability_code(spec: Option<&AbilitySpec>, anim_no: &HashMap<String, i32>) -> AbilityCode {
    let mut code = super::charge::charge_code(anim_no);
    code.minus2 += &super::air_variant::reset_rule();
    let Some(spec) = spec else { return code };
    if let Some(fly) = &spec.fly {
        let input = fly.input.as_deref().unwrap_or("a+b");
        let cost = fly.cost.unwrap_or(0.0).max(0.0);
        let speed = fly.speed.unwrap_or(3.5);
        let anim = anim_of(anim_no, fly.anim.as_ref(), &["fly", "jumpUp"], 41);
        let fall_anim = anim_of(anim_no, None, &["flyEnd", "jumpDown"], 44);
        let fuel = if cost > 0.0 { "triggerall = power >= 1\n" } else { "" };
        let drain = if cost > 0.0 { format!("[State {FLY_STATE}, fuel]\ntype = PowerAdd\ntrigger1 = 1\nvalue = {}\n\n[State {FLY_STATE}, empty]\ntype = ChangeState\ntrigger1 = power < 1\nvalue = {FLY_FALL_STATE}\nctrl = 1\n\n", -cost / 60.0) } else { String::new() };
        let mut fly_cmd = parse_command("fly_toggle", input, 1, 4); fly_cmd.pause_buffer = 4;
        code.commands.push(fly_cmd);
        code.movelist.push(entry("Fly (toggle)", input, "fly_toggle", cost, true));
        code.minus1 += &format!("[State -1, fly]\ntype = ChangeState\nvalue = {FLY_STATE}\ntriggerall = command = \"fly_toggle\"\ntriggerall = stateno != {FLY_STATE}\n{fuel}trigger1 = ctrl && (statetype = S || statetype = A)\n\n");
        code.states += &format!("[Statedef {FLY_STATE}]\ntype = A\nmovetype = I\nphysics = N\nanim = {anim}\nctrl = 1\nsprpriority = 1\n\n[State {FLY_STATE}, lift]\ntype = VelSet\ntrigger1 = Time = 0\ny = -3\n\n[State {FLY_STATE}, steer]\ntype = VelSet\ntrigger1 = Time > 0\nx = ifelse(command = \"holdfwd\", {speed}, ifelse(command = \"holdback\", -{speed}, 0))\ny = ifelse(command = \"holdup\", -{speed}, ifelse(command = \"holddown\", {speed}, 0))\n\n[State {FLY_STATE}, ceiling]\ntype = PosSet\ntrigger1 = Pos Y < -190\ny = -190\n\n[State {FLY_STATE}, land]\ntype = ChangeState\ntrigger1 = Time > 3 && Pos Y >= 0 && Vel Y >= 0\nvalue = 52\n\n[State {FLY_STATE}, off]\ntype = ChangeState\ntrigger1 = command = \"fly_toggle\" && Time > 8\nvalue = {FLY_FALL_STATE}\nctrl = 1\n\n{drain}[Statedef {FLY_FALL_STATE}]\ntype = A\nmovetype = I\nphysics = N\nanim = {fall_anim}\nctrl = 1\n\n[State {FLY_FALL_STATE}, grav]\ntype = VelAdd\ntrigger1 = 1\ny = const(movement.yaccel)\n\n[State {FLY_FALL_STATE}, land]\ntype = ChangeState\ntrigger1 = Pos Y >= 0 && Vel Y > 0\nvalue = 52\n\n");
    }
    if let Some(tp) = &spec.teleport {
        let input = tp.input.as_deref().unwrap_or("b+c");
        let cost = tp.cost.unwrap_or(0.0).max(0.0);
        let anim = anim_of(anim_no, tp.anim.as_ref(), &["teleport", "idle"], 0);
        let paid = if cost > 0.0 { format!("triggerall = power >= {cost}\n") } else { String::new() };
        let mut tp_cmd = parse_command("teleport_cmd", input, 1, 4); tp_cmd.pause_buffer = 4;
        code.commands.push(tp_cmd);
        code.movelist.push(entry("Teleport", input, "teleport_cmd", cost, false));
        code.minus1 += &format!("[State -1, teleport]\ntype = ChangeState\nvalue = {TELEPORT_STATE}\ntriggerall = command = \"teleport_cmd\"\n{paid}trigger1 = ctrl && statetype != L\n\n");
        code.states += &format!("[Statedef {TELEPORT_STATE}]\ntype = U\nmovetype = I\nphysics = N\nanim = {anim}\nctrl = 0\nvelset = 0, 0\npoweradd = {}\n\n[State {TELEPORT_STATE}, inv]\ntype = NotHitBy\ntrigger1 = Time = 0\nvalue = SCA\ntime = 14\n\n[State {TELEPORT_STATE}, blink]\ntype = PosAdd\ntrigger1 = Time = 4\nx = P2Dist X + 40\n\n[State {TELEPORT_STATE}, turn]\ntype = Turn\ntrigger1 = Time = 4\n\n[State {TELEPORT_STATE}, trail]\n{AFTER_IMAGE}trigger1 = Time = 0\n\n[State {TELEPORT_STATE}, ground]\ntype = ChangeState\ntrigger1 = Time >= 12 && Pos Y >= 0\nvalue = 0\nctrl = 1\n\n[State {TELEPORT_STATE}, air]\ntype = ChangeState\ntrigger1 = Time >= 12 && Pos Y < 0\nvalue = {FLY_FALL_STATE}\nctrl = 1\n\n", -cost);
    }
    if let Some(ss) = &spec.super_speed {
        let input = ss.input.as_deref().unwrap_or("a+c");
        let cost = ss.cost.unwrap_or(0.0).max(0.0);
        let mul = ss.mul.unwrap_or(1.8).max(1.0);
        let fuel = if cost > 0.0 { "triggerall = var(58) = 1 || power >= 1\n" } else { "" };
        let empty = if cost > 0.0 { "\ntrigger3 = power < 1" } else { "" };
        let mut spd_cmd = parse_command("speed_toggle", input, 1, 4); spd_cmd.pause_buffer = 4;
        code.commands.push(spd_cmd);
        code.movelist.push(entry("Super Speed (toggle)", input, "speed_toggle", cost, true));
        code.minus1 += &format!("[State -1, super_speed]\ntype = VarSet\nv = {SPEED_VAR}\nvalue = ifelse(var({SPEED_VAR}) = 0, 1, 0)\ntriggerall = command = \"speed_toggle\"\n{fuel}trigger1 = ctrl\n\n");
        code.minus2 += &format!("[State -2, speed_move]\ntype = PosAdd\ntrigger1 = var({SPEED_VAR}) = 1 && movetype != H && statetype != L\nx = Vel X * {}\n\n[State -2, speed_trail]\n{AFTER_IMAGE}trigger1 = var({SPEED_VAR}) = 1 && statetype != L\n\n[State -2, speed_off]\ntype = VarSet\nv = {SPEED_VAR}\nvalue = 0\ntrigger1 = var({SPEED_VAR}) = 1 && movetype = H\ntrigger2 = RoundState != 2{empty}\n\n", mul - 1.0);
        if cost > 0.0 { code.minus2 += &format!("[State -2, speed_fuel]\ntype = PowerAdd\ntrigger1 = var({SPEED_VAR}) = 1\nvalue = {}\n\n", -cost / 60.0); }
    }
    code
}
