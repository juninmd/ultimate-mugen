use super::types::NativeMove;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const VICTIM_STATE_BASE: i32 = 3800;
const STEP_STATE_BASE: i32 = 18000;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ThrowCameraSpec {
    #[serde(default)]
    pub zoom: Option<f32>,
    #[serde(default)]
    pub delta: Option<f32>,
    #[serde(default)]
    pub pos: Option<[f32; 2]>,
    #[serde(default)]
    pub time: Option<i32>,
    #[serde(default)]
    pub delay: Option<i32>,
    #[serde(default)]
    pub lag: Option<f32>,
    #[serde(default)]
    pub bound: Option<bool>,
    #[serde(default)]
    pub freeze: Option<bool>,
    #[serde(default, rename = "hideUi")]
    pub hide_ui: Option<bool>,
    #[serde(default)]
    pub bars: Option<f32>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ThrowStepSpec {
    pub at: i32,
    #[serde(default)]
    pub bind: Option<[f32; 2]>,
    #[serde(default)]
    pub anim: Option<String>,
    #[serde(default, rename = "p2Anim")]
    pub p2_anim: Option<String>,
    #[serde(default, rename = "p2State")]
    pub p2_state: Option<i32>,
    #[serde(default)]
    pub damage: Option<f32>,
    #[serde(default)]
    pub shake: Option<i32>,
    #[serde(default)]
    pub facing: Option<i32>,
    #[serde(default)]
    pub launch: Option<[f32; 2]>,
    #[serde(default)]
    pub camera: Option<ThrowCameraSpec>,
    #[serde(default)]
    pub flash: Option<super::move_fx_spec::FlashSpec>,
    #[serde(default)]
    pub sound: Option<[i32; 2]>,
    #[serde(default)]
    pub slowmo: Option<i32>,
    #[serde(default)]
    pub invert: Option<bool>,
    #[serde(default)]
    pub bars: Option<f32>,
    #[serde(default)]
    pub lines: Option<f32>,
}

pub fn is_victim_state(state_no: i32) -> bool {
    (VICTIM_STATE_BASE..VICTIM_STATE_BASE + 150).contains(&state_no)
}

pub(super) fn victim_state(sid: i32) -> i32 {
    VICTIM_STATE_BASE + (sid % 100) + if sid >= 3000 { 50 } else { 0 }
}

fn step_state_id(sid: i32, idx: usize) -> i32 {
    STEP_STATE_BASE + (sid % 1000) * 10 + idx as i32
}

fn victim_statedef(
    state_no: i32,
    p2_anim: Option<i32>,
    camera_free: bool,
    failsafe_after: i32,
) -> String {
    let mut s = format!(
        "[Statedef {state_no}]\ntype = A\nmovetype = H\nphysics = N\n{}ctrl = 0\nvelset = 0, 0\nsprpriority = 0\n\n",
        if p2_anim.is_some() { "" } else { "anim = 5030\n" }
    );
    if let Some(a) = p2_anim {
        s += &format!(
            "[State {state_no}, p2anim]\ntype = ChangeAnim2\ntrigger1 = Time = 0\nvalue = {a}\n\n"
        );
    }
    if camera_free {
        s += &format!("[State {state_no}, camfree]\ntype = ScreenBound\ntrigger1 = 1\nvalue = 0\nmovecamera = 0, 0\n\n");
    }
    s += &format!(
        "[State {state_no}, safety]\ntype = SelfState\ntrigger1 = Time >= 60\ntrigger2 = Time >= {failsafe_after} && !GetHitVar(IsBound)\nvalue = 5050\nctrl = 0\n\n"
    );
    s
}

pub(super) fn throw_states(
    m: &NativeMove,
    sid: i32,
    anim_no: &HashMap<String, i32>,
) -> (String, String) {
    let Some(throw) = &m.throw else {
        return (String::new(), String::new());
    };
    let grab = throw.at.unwrap_or(1).max(1);
    let victim = throw.p2_state.unwrap_or_else(|| victim_state(sid));
    let bind = throw.bind.unwrap_or([34.0, 0.0]);
    let launch = throw.launch.unwrap_or([-5.0, -6.0]);
    let release = throw.release_at.unwrap_or_else(|| {
        throw
            .steps
            .iter()
            .map(|s| s.at)
            .max()
            .map(|mx| mx + 1)
            .unwrap_or(grab + 2)
    });
    let scale = if super::ultimate::is(&m.id) {
        super::ultimate::DAMAGE
    } else {
        1.0
    };
    let mut out = format!(
        "[State {sid}, throw_grab]\ntype = HitDef\ntrigger1 = AnimElem = {grab}\nattr = S, NT\nhitflag = M-\nguardflag =\npriority = 7, Miss\npausetime = 0, 0\nsparkno = -1\nsparkxy = 0, -60\nhitsound = 5, 3\np2stateno = {victim}\np2facing = 1\nground.hittime = 60\n\n"
    );
    if throw.camera_free {
        out += &format!("[State {sid}, camfree]\ntype = ScreenBound\ntrigger1 = 1\nvalue = 0\nmovecamera = 0, 0\n\n");
    }
    if let Some(cam) = &throw.camera {
        out += &super::cinematic::zoom_cns(
            sid,
            "cam",
            &format!("AnimElem = {grab} && NumTarget > 0"),
            cam,
        );
    }
    let mut victim_def = String::new();
    let base_anim = throw
        .p2_anim
        .as_deref()
        .and_then(|a| anim_no.get(a).copied());
    if throw.p2_state.is_none() {
        victim_def += &victim_statedef(victim, base_anim, throw.camera_free, 10);
    }
    if throw.steps.is_empty() {
        out += &format!(
            "[State {sid}, throw_bind]\ntype = TargetBind\ntrigger1 = NumTarget > 0 && AnimElemNo(0) >= {grab} && AnimElemNo(0) < {release}\ntime = 2\npos = {}, {}\n\n\
             [State {sid}, throw_damage]\ntype = TargetLifeAdd\ntrigger1 = AnimElem = {release} && NumTarget > 0\nvalue = -{}\nkill = 1\n\n\
             [State {sid}, throw_launch]\ntype = TargetState\ntrigger1 = AnimElem = {release} && NumTarget > 0\nvalue = 5050\n\n\
             [State {sid}, throw_velocity]\ntype = TargetVelSet\ntrigger1 = AnimElem = {release} && NumTarget > 0\nx = {}\ny = {}\n\n\
             [State {sid}, throw_shake]\ntype = EnvShake\ntrigger1 = AnimElem = {release} && NumTarget > 0\ntime = 12\nfreq = 90\nampl = -5\n\n",
            bind[0], bind[1], (throw.damage * scale) as i32, launch[0], launch[1]
        );
        return (out, victim_def);
    }
    out += &format!(
        "[State {sid}, throw_bind_init]\ntype = TargetBind\ntrigger1 = NumTarget > 0 && AnimElemNo(0) >= {grab} && AnimElemNo(0) < {}\ntime = 2\npos = {}, {}\n\n",
        throw.steps[0].at.max(grab + 1), bind[0], bind[1]
    );
    let mut launched_by_step = false;
    for (idx, step) in throw.steps.iter().enumerate() {
        let sat = step.at;
        let trig = format!("AnimElem = {sat} && NumTarget > 0");
        let sb = step.bind.unwrap_or(bind);
        if step.launch.is_none() {
            out += &format!("[State {sid}, step_bind_{idx}]\ntype = TargetBind\ntrigger1 = {trig}\ntime = 12\npos = {}, {}\n\n", sb[0], sb[1]);
        }
        if let Some(cam) = &step.camera {
            out += &super::cinematic::zoom_cns(sid, &format!("scam_{idx}"), &trig, cam);
        }
        if let Some([g, i]) = step.sound {
            out += &format!("[State {sid}, step_snd_{idx}]\ntype = PlaySnd\ntrigger1 = {trig}\nvalue = {g}, {i}\n\n");
        }
        out += &super::cinematic::cinema_ctrl_cns(
            sid,
            &format!("scine_{idx}"),
            &trig,
            step.bars
                .unwrap_or(if step.camera.is_some() || step.slowmo.is_some() {
                    1.0
                } else {
                    0.0
                }),
            step.slowmo.unwrap_or(0),
            step.invert.unwrap_or(false),
            step.lines.unwrap_or(0.0),
            step.flash.as_ref(),
            None,
        );
        if let Some(face) = step.facing {
            out += &format!("[State {sid}, step_face_{idx}]\ntype = TargetFacing\ntrigger1 = {trig}\nvalue = {face}\n\n");
        }
        if let Some(p2s) = step.p2_state {
            out += &format!("[State {sid}, step_p2s_{idx}]\ntype = TargetState\ntrigger1 = {trig}\nvalue = {p2s}\n\n");
        } else if let Some(ref pa_name) = step.p2_anim {
            if let Some(pa) = anim_no.get(pa_name).copied() {
                let sub = step_state_id(sid, idx);
                out += &format!("[State {sid}, step_p2a_{idx}]\ntype = TargetState\ntrigger1 = {trig}\nvalue = {sub}\n\n");
                victim_def += &victim_statedef(sub, Some(pa), throw.camera_free, 15);
            }
        }
        if let Some(dmg) = step.damage {
            out += &format!(
                "[State {sid}, step_dmg_{idx}]\ntype = TargetLifeAdd\ntrigger1 = {trig}\nvalue = -{}\nkill = {}\n\n",
                (dmg * scale) as i32,
                if step.launch.is_some() { 1 } else { 0 }
            );
        }
        if let Some(sh) = step.shake.filter(|s| *s > 0) {
            out += &format!("[State {sid}, step_shake_{idx}]\ntype = EnvShake\ntrigger1 = {trig}\ntime = {sh}\nfreq = 90\nampl = -5\n\n");
        }
        if let Some(lv) = step.launch {
            launched_by_step = true;
            out += &format!(
                "[State {sid}, step_rel_{idx}]\ntype = TargetState\ntrigger1 = {trig}\nvalue = 5050\n\n\
                 [State {sid}, step_vel_{idx}]\ntype = TargetVelSet\ntrigger1 = {trig}\nx = {}\ny = {}\n\n",
                lv[0], lv[1]
            );
        }
        if let Some(ref anim_name) = step.anim {
            if let Some(&an) = anim_no.get(anim_name) {
                out += &format!("[State {sid}, step_anim_{idx}]\ntype = ChangeAnim\ntrigger1 = {trig}\nvalue = {an}\n\n");
            }
        }
    }
    if !launched_by_step {
        if throw.damage > 0.0 {
            out += &format!(
                "[State {sid}, throw_damage]\ntype = TargetLifeAdd\ntrigger1 = AnimElem = {release} && NumTarget > 0\nvalue = -{}\nkill = 1\n\n",
                (throw.damage * scale) as i32
            );
        }
        out += &format!(
            "[State {sid}, throw_launch]\ntype = TargetState\ntrigger1 = AnimElem = {release} && NumTarget > 0\nvalue = 5050\n\n\
             [State {sid}, throw_velocity]\ntype = TargetVelSet\ntrigger1 = AnimElem = {release} && NumTarget > 0\nx = {}\ny = {}\n\n\
             [State {sid}, throw_shake]\ntype = EnvShake\ntrigger1 = AnimElem = {release} && NumTarget > 0\ntime = 12\nfreq = 90\nampl = -5\n\n",
            launch[0], launch[1]
        );
    }
    (out, victim_def)
}
