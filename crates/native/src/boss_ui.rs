use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct UiTextRule {
    pub text: String,
    #[serde(default)]
    pub params: Vec<String>,
    #[serde(default)]
    pub id: Option<i32>,
    #[serde(default)]
    pub pos: Option<[f32; 2]>,
    #[serde(default)]
    pub vel: Option<[f32; 2]>,
    #[serde(default)]
    pub scale: Option<[f32; 2]>,
    #[serde(default)]
    pub color: Option<[u8; 3]>,
    #[serde(default)]
    pub align: Option<i32>,
    #[serde(default)]
    pub time: Option<i32>,
    #[serde(default)]
    pub when: Option<String>,
    #[serde(default)]
    pub persistent: Option<i32>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ModifyPlayerRule {
    #[serde(default)]
    pub target: Option<String>,
    #[serde(default, rename = "lifeMax")]
    pub life_max: Option<String>,
    #[serde(default)]
    pub life: Option<String>,
    #[serde(default, rename = "powerMax")]
    pub power_max: Option<String>,
    #[serde(default)]
    pub power: Option<String>,
    #[serde(default, rename = "aiLevel")]
    pub ai_level: Option<String>,
    #[serde(default, rename = "redLife")]
    pub red_life: Option<String>,
    #[serde(default, rename = "dizzyPoints")]
    pub dizzy_points: Option<String>,
    #[serde(default, rename = "guardPoints")]
    pub guard_points: Option<String>,
    #[serde(default)]
    pub ctrl: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BossPhaseRule {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default, rename = "lifeBelowRatio")]
    pub life_below_ratio: Option<f32>,
    #[serde(default)]
    pub when: Option<String>,
    #[serde(default)]
    pub once: Option<bool>,
    #[serde(default)]
    pub asserts: Vec<String>,
    #[serde(default)]
    pub texts: Vec<UiTextRule>,
    #[serde(default, rename = "modifyPlayer")]
    pub modify_player: Vec<ModifyPlayerRule>,
    #[serde(default, rename = "attackMul")]
    pub attack_mul: Option<f32>,
    #[serde(default, rename = "defenceMul")]
    pub defence_mul: Option<f32>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BossRulesSpec {
    #[serde(default, rename = "hideHud")]
    pub hide_hud: bool,
    #[serde(default, rename = "hideLifeBar")]
    pub hide_life_bar: bool,
    #[serde(default, rename = "hidePowerBar")]
    pub hide_power_bar: bool,
    #[serde(default, rename = "hideTimer")]
    pub hide_timer: bool,
    #[serde(default, rename = "freezeTimer")]
    pub freeze_timer: bool,
    #[serde(default)]
    pub asserts: Vec<String>,
    #[serde(default)]
    pub texts: Vec<UiTextRule>,
    #[serde(default, rename = "modifyPlayer")]
    pub modify_player: Vec<ModifyPlayerRule>,
    #[serde(default)]
    pub phases: Vec<BossPhaseRule>,
}

fn emit_asserts(flags: &[String], cond: &str, out: &mut String) {
    for chunk in flags.chunks(5) {
        *out += "[State -4, boss_assert]\ntype = AssertSpecial\nignorehitpause = 1\n";
        for (idx, flag) in chunk.iter().enumerate() {
            let key = if idx == 0 {
                "flag".to_string()
            } else {
                format!("flag{}", idx + 1)
            };
            *out += &format!("{key} = {flag}\n");
        }
        *out += &format!("trigger1 = !IsHelper && {cond}\n\n");
    }
}

fn emit_text(t: &UiTextRule, extra_cond: Option<&str>, out: &mut String) {
    let pos = t.pos.unwrap_or([160.0, 28.0]);
    let vel = t.vel.unwrap_or([0.0, 0.0]);
    let scale = t.scale.unwrap_or([1.0, 1.0]);
    let color = t.color.unwrap_or([255, 255, 255]);
    let align = t.align.unwrap_or(0);
    let time = t.time.unwrap_or(1);
    let escaped = t.text.replace('"', "'");
    *out += "[State -4, boss_text]\ntype = Text\nignorehitpause = 1\n";
    if let Some(id) = t.id {
        *out += &format!("id = {id}\n");
    }
    if let Some(p) = t.persistent {
        *out += &format!("persistent = {p}\n");
    }
    *out += &format!("text = \"{escaped}\"\n");
    if !t.params.is_empty() {
        *out += &format!("params = {}\n", t.params.join(", "));
    }
    *out += &format!(
        "pos = {}, {}\nvelocity = {}, {}\nscale = {}, {}\ncolor = {}, {}, {}\nalign = {align}\nremovetime = {time}\n",
        pos[0], pos[1], vel[0], vel[1], scale[0], scale[1], color[0], color[1], color[2]
    );
    if let Some(c) = extra_cond {
        *out += &format!("triggerall = {c}\n");
    }
    let when = t.when.as_deref().unwrap_or("RoundState = 2");
    *out += &format!("trigger1 = !IsHelper && ({when})\n\n");
}

fn emit_modify(m: &ModifyPlayerRule, cond: &str, out: &mut String) {
    *out += "[State -4, boss_mod]\ntype = ModifyPlayer\nignorehitpause = 1\n";
    if m.target
        .as_deref()
        .is_some_and(|t| t.eq_ignore_ascii_case("enemy"))
    {
        *out += "id = EnemyNear, ID\n";
    }
    for (k, v) in [
        ("lifemax", &m.life_max),
        ("life", &m.life),
        ("powermax", &m.power_max),
        ("power", &m.power),
        ("ailevel", &m.ai_level),
        ("redlife", &m.red_life),
        ("dizzypoints", &m.dizzy_points),
        ("guardpoints", &m.guard_points),
        ("ctrl", &m.ctrl),
    ] {
        if let Some(val) = v {
            *out += &format!("{k} = {val}\n");
        }
    }
    *out += &format!("trigger1 = !IsHelper && ({cond})\n\n");
}

pub(super) fn boss_ui_cns(spec: Option<&BossRulesSpec>) -> String {
    let Some(spec) = spec else {
        return String::new();
    };
    let mut out = String::from("[Statedef -4]\n\n");
    let mut flags = spec.asserts.clone();
    if spec.hide_hud {
        flags.push("nobardisplay".into());
    }
    if spec.hide_life_bar {
        flags.push("nolifebardisplay".into());
    }
    if spec.hide_power_bar {
        flags.push("nopowerbardisplay".into());
    }
    if spec.hide_timer {
        flags.push("notimedisplay".into());
    }
    if spec.freeze_timer {
        flags.push("timerfreeze".into());
    }
    if !flags.is_empty() {
        emit_asserts(&flags, "RoundState = 2", &mut out);
    }
    for t in &spec.texts {
        emit_text(t, None, &mut out);
    }
    for m in &spec.modify_player {
        emit_modify(m, "RoundState = 2", &mut out);
    }
    for (idx, p) in spec.phases.iter().enumerate() {
        let map_key = format!(
            "__ume_phase_{}",
            p.id.as_deref().unwrap_or(&idx.to_string())
        );
        let mut conds = vec!["RoundState = 2".to_string()];
        if let Some(r) = p.life_below_ratio {
            conds.push(format!("Life <= LifeMax * {r}"));
        }
        if let Some(w) = &p.when {
            conds.push(format!("({w})"));
        }
        let gate = conds.join(" && ");
        let once = p.once.unwrap_or(true);
        let active = if once {
            out += &format!("[State -4, phase_reset_{idx}]\ntype = MapReset\nignorehitpause = 1\nmap = \"{map_key}\"\ntrigger1 = !IsHelper && RoundState = 0 && map({map_key}) != 0\n\n");
            for m in &p.modify_player {
                emit_modify(m, &format!("map({map_key}) = 0 && {gate}"), &mut out);
            }
            for t in p.texts.iter().filter(|t| t.time.unwrap_or(1) > 1) {
                emit_text(t, Some(&format!("map({map_key}) = 0 && {gate}")), &mut out);
            }
            out += &format!("[State -4, phase_latch_{idx}]\ntype = MapSet\nignorehitpause = 1\nmap = \"{map_key}\"\nvalue = 1\ntrigger1 = !IsHelper && map({map_key}) = 0 && {gate}\n\n");
            for t in p.texts.iter().filter(|t| t.time.unwrap_or(1) <= 1) {
                emit_text(t, Some(&format!("map({map_key}) != 0")), &mut out);
            }
            format!("map({map_key}) != 0")
        } else {
            for m in &p.modify_player {
                emit_modify(m, &gate, &mut out);
            }
            for t in &p.texts {
                emit_text(t, Some(&gate), &mut out);
            }
            gate.clone()
        };
        if !p.asserts.is_empty() {
            emit_asserts(&p.asserts, &active, &mut out);
        }
        if let Some(atk) = p.attack_mul {
            out += &format!("[State -4, phase_atk_{idx}]\ntype = AttackMulSet\nignorehitpause = 1\nvalue = {atk}\ntrigger1 = !IsHelper && {active}\n\n");
        }
        if let Some(def) = p.defence_mul {
            out += &format!("[State -4, phase_def_{idx}]\ntype = DefenceMulSet\nignorehitpause = 1\nvalue = {def}\ntrigger1 = !IsHelper && {active}\n\n");
        }
    }
    out
}
