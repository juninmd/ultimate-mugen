pub use super::move_fx_spec::*;
use super::types::NativeMove;
use std::collections::HashMap;

fn trigger_at(at: Option<i32>) -> String {
    match at {
        Some(frame) if frame > 0 => format!("AnimElem = {frame}"),
        _ => "Time = 0".to_string(),
    }
}

pub(super) fn super_pause_cns(m: &NativeMove, sid: i32, default_freeze: i32) -> String {
    let mut out = String::new();
    if let Some(sp) = &m.super_pause {
        let trig = trigger_at(sp.at);
        let time = sp.time.unwrap_or(default_freeze);
        let pos = sp.pos.unwrap_or([0.0, -60.0]);
        let anim = sp.anim.unwrap_or(30);
        let darken = if sp.darken.unwrap_or(true) { 1 } else { 0 };
        out += &format!(
            "[State {sid}, super]\ntype = SuperPause\ntrigger1 = {trig}\npersistent = 0\ntime = {time}\nanim = {anim}\npos = {}, {}\ndarken = {darken}\nsound = -1\n\n",
            pos[0], pos[1]
        );
    } else if m.kind == "super" {
        out += &format!(
            "[State {sid}, super]\ntype = SuperPause\ntrigger1 = Time = 0\ntime = {default_freeze}\nanim = 30\npos = 0, -60\nsound = -1\n\n"
        );
    }
    if let Some(ai) = &m.afterimage {
        out += &afterimage_cns(ai, sid);
    } else if m.kind == "super" {
        out += &format!(
            "[State {sid}, ai]\ntype = AfterImage\ntrigger1 = Time = 0\nframegap = 2\npalbright = 0, 0, 0\npalcontrast = 100, 150, 256\npaladd = 0, 0, 0\npalmul = .85, .85, .85\ntime = 60\nlength = 10\n\n"
        );
    } else if m.fx.as_deref() == Some("aura") {
        out += &format!(
            "[State {sid}, ai]\ntype = AfterImage\ntrigger1 = Time = 0\nframegap = 2\npalbright = 0, 0, 0\npalcontrast = 100, 150, 256\npaladd = 0, 0, 0\npalmul = .85, .85, .85\ntime = 20\nlength = 6\n\n"
        );
    }
    out
}

fn afterimage_cns(ai: &AfterImageSpec, sid: i32) -> String {
    let trig = trigger_at(ai.at);
    let time = ai.time.unwrap_or(25);
    let length = ai.length.unwrap_or(8);
    let gap = ai.framegap.unwrap_or(2);
    let br = ai.pal_bright.unwrap_or([0, 0, 0]);
    let ct = ai.pal_contrast.unwrap_or([100, 150, 256]);
    let add = ai.pal_add.unwrap_or([0, 0, 0]);
    let mul = ai.pal_mul.unwrap_or([0.85, 0.85, 0.85]);
    let trans = ai
        .trans
        .as_deref()
        .map(|t| format!("trans = {t}\n"))
        .unwrap_or_default();
    format!(
        "[State {sid}, ai]\ntype = AfterImage\ntrigger1 = {trig}\ntime = {time}\nlength = {length}\nframegap = {gap}\n{trans}palbright = {}, {}, {}\npalcontrast = {}, {}, {}\npaladd = {}, {}, {}\npalmul = {}, {}, {}\n\n",
        br[0], br[1], br[2], ct[0], ct[1], ct[2], add[0], add[1], add[2], mul[0], mul[1], mul[2]
    )
}

pub(super) fn teleport_cns(tp: &TeleportStep, sid: i32) -> String {
    match tp {
        TeleportStep::At(frame) => format!(
            "[State {sid}, tp]\ntype = PosAdd\ntrigger1 = AnimElem = {frame}\nx = p2dist x + 40\n\n[State {sid}, tpt]\ntype = Turn\ntrigger1 = AnimElem = {frame}\n\n"
        ),
        TeleportStep::Spec(spec) => {
            let frame = spec.at.max(1);
            let off = spec.offset.unwrap_or([0.0, 0.0]);
            let mode = spec.mode.as_deref().unwrap_or("behind");
            let (x_expr, y_line, default_turn) = match mode {
                "front" => (format!("p2dist x - {}", if off[0] != 0.0 { off[0].abs() } else { 40.0 }), if off[1] != 0.0 { format!("y = {}\n", off[1]) } else { String::new() }, false),
                "above" => (format!("p2dist x + {}", off[0]), format!("y = {}\n", if off[1] != 0.0 { off[1] } else { -80.0 }), false),
                "offset" => (format!("{}", off[0]), if off[1] != 0.0 { format!("y = {}\n", off[1]) } else { String::new() }, false),
                _ => (format!("p2dist x + {}", if off[0] != 0.0 { off[0] } else { 40.0 }), if off[1] != 0.0 { format!("y = {}\n", off[1]) } else { String::new() }, true),
            };
            let mut out = format!("[State {sid}, tp]\ntype = PosAdd\ntrigger1 = AnimElem = {frame}\nx = {x_expr}\n{y_line}\n");
            if spec.turn.unwrap_or(default_turn) {
                out += &format!("[State {sid}, tpt]\ntype = Turn\ntrigger1 = AnimElem = {frame}\n\n");
            }
            out
        }
    }
}

pub(super) fn move_fx_cns(m: &NativeMove, sid: i32, anim_no: &HashMap<String, i32>) -> String {
    let mut out = String::new();
    for ex in &m.explods {
        let Some(anim) = anim_no
            .get(&ex.anim)
            .copied()
            .or_else(|| ex.anim.parse::<i32>().ok())
        else {
            continue;
        };
        let pos = ex.pos.unwrap_or([0.0, 0.0]);
        let postype = ex.postype.as_deref().unwrap_or("p1");
        let bind = ex.bind.unwrap_or(1);
        let rem = ex.remove_time.unwrap_or(-2);
        let prio = ex.priority.unwrap_or(3);
        let scale = ex.scale.unwrap_or([1.0, 1.0]);
        let sm = if ex.super_move.unwrap_or(false) {
            "supermove = 1\n"
        } else {
            ""
        };
        let vel = ex
            .vel
            .map(|v| format!("vel = {}, {}\n", v[0], v[1]))
            .unwrap_or_default();
        let accel = ex
            .accel
            .map(|a| format!("accel = {}, {}\n", a[0], a[1]))
            .unwrap_or_default();
        out += &format!(
            "[State {sid}, explod]\ntype = Explod\ntrigger1 = AnimElem = {}\nanim = {anim}\npos = {}, {}\npostype = {postype}\nbindtime = {bind}\nremovetime = {rem}\nsprpriority = {prio}\nscale = {}, {}\nownpal = 1\n{sm}{vel}{accel}\n",
            ex.at.max(1), pos[0], pos[1], scale[0], scale[1]
        );
    }
    if let Some(shakes) = &m.shake {
        for sh in shakes.to_vec() {
            out += &format!(
                "[State {sid}, shake]\ntype = EnvShake\ntrigger1 = {}\ntime = {}\nampl = {}\nfreq = {}\n\n",
                trigger_at(sh.at), sh.time.max(1), sh.ampl.unwrap_or(4.0), sh.freq.unwrap_or(60.0)
            );
        }
    }
    if let Some(flashes) = &m.flash {
        for fl in flashes.to_vec() {
            let col = fl.color.unwrap_or([255, 255, 255]);
            let under = if fl.under.unwrap_or(false) { 1 } else { 0 };
            let sm = if fl.super_move.unwrap_or(false) {
                "supermove = 1\n"
            } else {
                ""
            };
            out += &format!(
                "[State {sid}, flash]\ntype = EnvColor\ntrigger1 = {}\ntime = {}\nvalue = {}, {}, {}\nunder = {under}\n{sm}\n",
                trigger_at(fl.at), fl.time.max(1), col[0], col[1], col[2]
            );
        }
    }
    out
}

pub(super) fn follow_up_cns(m: &NativeMove, sid: i32, target_sid: Option<i32>) -> String {
    let (Some(fu), Some(next_sid)) = (&m.follow_up, target_sid) else {
        return String::new();
    };
    let on = fu.on.as_deref().unwrap_or("hit");
    let cond = match on {
        "contact" => "MoveContact > 0",
        "end" => "1",
        "hitNow" => "MoveHit = 1",
        _ => "MoveHit > 0",
    };
    let timing = if on == "hitNow" {
        "1".to_string()
    } else {
        fu.at.filter(|a| *a > 0).map(|a| format!("AnimElem = {a}")).unwrap_or_else(|| "AnimTime = 0".to_string())
    };
    format!("[State {sid}, followup]\ntype = ChangeState\ntrigger1 = {cond} && {timing}\nvalue = {next_sid}\nctrl = 0\n\n")
}

pub(super) fn land_cns(land_at: Option<i32>, sid: i32, anim: i32) -> String {
    let landed = "Pos Y >= 0 && Vel Y > 0";
    match land_at {
        Some(elem) => format!("[State {sid}, land]\ntype = ChangeAnim\ntrigger1 = {landed} && AnimElemNo(0) < {elem}\nvalue = {anim}\nelem = {elem}\n\n[State {sid}, land_stop]\ntype = VelSet\ntrigger1 = {landed}\nx = 0\ny = 0\n\n"),
        None => format!("[State {sid}, land]\ntype = ChangeState\ntrigger1 = {landed}\nvalue = 52\n\n"),
    }
}
