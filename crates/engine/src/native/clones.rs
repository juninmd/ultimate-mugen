use super::abilities::AbilityCode;
use super::compile::extras::{hitdef_from_spec, hitdef_projectile};
use super::multihit::{glue, projectile_knockback};
use super::types::NativeMove;
use crate::chardef::MoveInfo;
use crate::commands::parse_command;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const CLONE_BRAIN: i32 = 7100;
const CLONE_POOF: i32 = 7109;
const CLONE_ATTACK_BASE: i32 = 7120;
const CLONE_SUMMON_STATE: i32 = 7150;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CloneSpec {
    #[serde(default)]
    pub id: Option<i32>,
    pub input: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub count: Option<u32>,
    #[serde(default, rename = "maxActive")]
    pub max_active: Option<u32>,
    #[serde(default)]
    pub cost: Option<f32>,
    #[serde(default)]
    pub duration: Option<i32>,
    #[serde(default, rename = "spawnOffset")]
    pub spawn_offset: Option<[f32; 2]>,
    #[serde(default, rename = "walkSpeed")]
    pub walk_speed: Option<f32>,
    #[serde(default, rename = "attackRange")]
    pub attack_range: Option<f32>,
    #[serde(default, rename = "oneHitPoof")]
    pub one_hit_poof: Option<bool>,
    #[serde(default, rename = "idleAnim")]
    pub idle_anim: Option<String>,
    #[serde(default, rename = "walkAnim")]
    pub walk_anim: Option<String>,
    #[serde(default, rename = "poofAnim")]
    pub poof_anim: Option<String>,
    #[serde(default)]
    pub attacks: Vec<NativeMove>,
}

fn clone_attacks<'a>(spec: &'a CloneSpec, parent_moves: &'a [NativeMove]) -> Vec<NativeMove> {
    if !spec.attacks.is_empty() {
        return spec.attacks.clone();
    }
    parent_moves
        .iter()
        .filter(|m| {
            m.throw.is_none()
                && m.when.as_deref().unwrap_or("stand") == "stand"
                && m.kind != "super"
        })
        .take(3)
        .cloned()
        .collect()
}

pub(super) fn compile(
    spec: Option<&CloneSpec>,
    anim_no: &HashMap<String, i32>,
    parent_moves: &[NativeMove],
) -> AbilityCode {
    let mut code = AbilityCode::default();
    let Some(spec) = spec else { return code };
    let hid = spec.id.unwrap_or(CLONE_BRAIN);
    let count = spec.count.unwrap_or(1).clamp(1, 4);
    let max_active = spec.max_active.unwrap_or(3).max(1);
    let cost = spec.cost.unwrap_or(500.0).max(0.0);
    let duration = spec.duration.unwrap_or(480).max(30);
    let off = spec.spawn_offset.unwrap_or([42.0, 0.0]);
    let walk_speed = spec.walk_speed.unwrap_or(3.2).max(0.5);
    let range = spec.attack_range.unwrap_or(38.0).max(18.0);
    let one_hit = spec.one_hit_poof.unwrap_or(true);
    let idle_anim = spec
        .idle_anim
        .as_deref()
        .and_then(|a| anim_no.get(a).copied())
        .unwrap_or(0);
    let walk_anim = spec
        .walk_anim
        .as_deref()
        .and_then(|a| anim_no.get(a).copied())
        .unwrap_or(20);
    let poof_anim = spec
        .poof_anim
        .as_deref()
        .and_then(|a| anim_no.get(a).copied())
        .unwrap_or(idle_anim);
    let label = spec.name.clone().unwrap_or_else(|| "Shadow Clone".into());
    let mut cmd = parse_command("clone_summon", &spec.input, 20, 4);
    cmd.pause_buffer = 6;
    code.commands.push(cmd);
    code.movelist.push(MoveInfo {
        name: label,
        input: spec.input.clone(),
        command: "clone_summon".into(),
        cost,
        drain: false,
        forms: u32::MAX,
        state: CLONE_SUMMON_STATE,
    });
    let paid = if cost > 0.0 {
        format!("triggerall = power >= {cost}\n")
    } else {
        String::new()
    };
    code.minus1 += &format!(
        "[State -1, clone_cast]\ntype = ChangeState\nvalue = {CLONE_SUMMON_STATE}\ntriggerall = !IsHelper && command = \"clone_summon\"\ntriggerall = statetype = S && ctrl && NumHelper({hid}) < {max_active}\n{paid}trigger1 = RoundState = 2\n\n"
    );
    let mut spawns = String::new();
    for k in 0..count {
        let sign = if k % 2 == 0 { 1.0 } else { -1.0 };
        let mul = (k / 2 + 1) as f32;
        let sx = off[0] * mul * sign;
        spawns += &format!(
            "[State {CLONE_SUMMON_STATE}, spawn_{k}]\ntype = Helper\ntrigger1 = Time = 0 && NumHelper({hid}) < {max_active}\nhelpertype = normal\nid = {hid}\nstateno = {CLONE_BRAIN}\nkeyctrl = 0\nownpal = 1\npos = {sx}, {}\n\n",
            off[1]
        );
    }
    code.states += &format!(
        "[Statedef {CLONE_SUMMON_STATE}]\ntype = S\nmovetype = I\nphysics = S\nanim = {idle_anim}\nctrl = 0\nvelset = 0, 0\npoweradd = -{cost}\n\n{spawns}[State {CLONE_SUMMON_STATE}, end]\ntype = ChangeState\ntrigger1 = Time >= 8\nvalue = 0\nctrl = 1\n\n"
    );
    let hit_ovr = if one_hit {
        format!("type = HitOverride\nignorehitpause = 1\ntrigger1 = 1\nattr = SCA, AA, AT, AP\nstateno = {CLONE_POOF}\ntime = 1\n\n")
    } else {
        String::new()
    };
    let attacks = clone_attacks(spec, parent_moves);
    let mut ai_atks = String::new();
    let mut atk_states = String::new();
    let total = attacks.len().max(1);
    for (idx, mv) in attacks.iter().enumerate() {
        let sid = CLONE_ATTACK_BASE + idx as i32;
        let anim = anim_no.get(&mv.anim).copied().unwrap_or(idle_anim);
        let thresh = ((idx + 1) * 1000) / total;
        ai_atks += &format!(
            "[State {CLONE_BRAIN}, ai_atk_{idx}]\ntype = ChangeState\nvalue = {sid}\ntrigger1 = RoundState = 2 && Time >= 6 && P2BodyDist X <= {range} && P2Dist Y > -90 && P2StateType != L && Random < {thresh}\n\n"
        );
        atk_states += &format!(
            "[Statedef {sid}]\ntype = S\nmovetype = A\nphysics = N\nanim = {anim}\nctrl = 0\nvelset = 0, 0\n\n[State {sid}, sb]\ntype = ScreenBound\ntrigger1 = 1\nvalue = 0\nmovecamera = 0, 0\n\n[State {sid}, age]\ntype = MapAdd\ntrigger1 = 1\nmap = \"__ume_clone_age\"\nvalue = 1\n\n"
        );
        if one_hit {
            atk_states += &format!("[State {sid}, ho]\n{hit_ovr}");
        }
        for v in &mv.moves {
            atk_states += &format!(
                "[State {sid}, vel]\ntype = VelSet\ntrigger1 = AnimElem = {}\n{}{}\n",
                v.at,
                v.vel[0].map(|x| format!("x = {x}\n")).unwrap_or_default(),
                v.vel[1].map(|y| format!("y = {y}\n")).unwrap_or_default()
            );
        }
        for h in glue(&mv.hits).iter() {
            atk_states += &format!(
                "[State {sid}, hit]\ntype = HitDef\ntrigger1 = AnimElem = {}\n{}\n",
                h.at,
                hitdef_from_spec(h, 0.75, "S, SA", false)
            );
        }
        for (pi, p) in mv
            .projectile
            .as_ref()
            .map(|p| p.to_vec())
            .unwrap_or_default()
            .iter()
            .enumerate()
        {
            let pa = anim_no.get(&p.anim).copied().unwrap_or(0);
            let pid = sid * 10 + pi as i32;
            atk_states += &format!(
                "[State {sid}, proj_{pi}]\ntype = Projectile\ntrigger1 = AnimElem = {}\nprojid = {pid}\nprojanim = {pa}\noffset = {}, {}\nvelocity = {}, {}\nprojhits = {}\nprojmisstime = 6\nprojpriority = {}\nprojremovetime = {}\n{}\n",
                p.at, p.offset[0], p.offset[1], p.speed[0], p.speed[1],
                p.hits.unwrap_or(1), p.priority.unwrap_or(1), p.remove_time.unwrap_or(-1),
                hitdef_projectile(p.damage * 0.75, p.level.as_deref(), projectile_knockback(p), p.launch.unwrap_or(false), "S, SP", p.element.as_deref(), p.stun)
            );
        }
        atk_states += &format!(
            "[State {sid}, expire]\ntype = ChangeState\ntrigger1 = map(__ume_clone_age) >= {duration} || !ParentExist || Parent, Alive = 0 || RoundState != 2\nvalue = {CLONE_POOF}\n\n[State {sid}, back]\ntype = ChangeState\ntrigger1 = AnimTime = 0 || Time >= 90\nvalue = {CLONE_BRAIN}\n\n"
        );
    }
    let brain_ho = if one_hit {
        format!("[State {CLONE_BRAIN}, ho]\n{hit_ovr}")
    } else {
        String::new()
    };
    let poof_cond = if spec.poof_anim.is_some() {
        "AnimTime = 0 || Time >= 12"
    } else {
        "Time >= 0"
    };
    code.states += &format!(
        "[Statedef {CLONE_BRAIN}]\ntype = S\nmovetype = I\nphysics = N\nanim = {idle_anim}\nctrl = 0\nvelset = 0, 0\n\n[State {CLONE_BRAIN}, sb]\ntype = ScreenBound\ntrigger1 = 1\nvalue = 0\nmovecamera = 0, 0\n\n[State {CLONE_BRAIN}, age]\ntype = MapAdd\ntrigger1 = 1\nmap = \"__ume_clone_age\"\nvalue = 1\n\n{brain_ho}[State {CLONE_BRAIN}, expire]\ntype = ChangeState\ntrigger1 = map(__ume_clone_age) >= {duration} || !ParentExist || Parent, Alive = 0 || RoundState != 2\nvalue = {CLONE_POOF}\n\n[State {CLONE_BRAIN}, turn]\ntype = Turn\ntrigger1 = P2Dist X < -6\n\n[State {CLONE_BRAIN}, chase_vel]\ntype = VelSet\ntrigger1 = 1\nx = ifelse(P2BodyDist X > {range}, {walk_speed}, 0)\n\n[State {CLONE_BRAIN}, chase_anim]\ntype = ChangeAnim\ntrigger1 = P2BodyDist X > {range} && Anim != {walk_anim}\nvalue = {walk_anim}\n\n[State {CLONE_BRAIN}, idle_anim]\ntype = ChangeAnim\ntrigger1 = P2BodyDist X <= {range} && Anim != {idle_anim}\nvalue = {idle_anim}\n\n{ai_atks}[Statedef {CLONE_POOF}]\ntype = S\nmovetype = I\nphysics = N\nanim = {poof_anim}\nctrl = 0\nvelset = 0, 0\n\n[State {CLONE_POOF}, inv]\ntype = NotHitBy\nignorehitpause = 1\ntrigger1 = 1\nvalue = SCA\ntime = 1\n\n[State {CLONE_POOF}, kill]\ntype = DestroySelf\nignorehitpause = 1\ntrigger1 = {poof_cond}\n\n{atk_states}"
    );
    code
}
