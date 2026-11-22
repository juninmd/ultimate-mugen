use super::move_fx_spec::FlashSpec;
use super::throw::ThrowCameraSpec;
use super::types::NativeMove;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CinematicCutSpec {
    pub at: i32,
    #[serde(default)]
    pub camera: Option<ThrowCameraSpec>,
    #[serde(default)]
    pub flash: Option<FlashSpec>,
    #[serde(default)]
    pub shake: Option<i32>,
    #[serde(default)]
    pub slowmo: Option<i32>,
    #[serde(default)]
    pub invert: Option<bool>,
    #[serde(default)]
    pub bars: Option<f32>,
    #[serde(default)]
    pub lines: Option<f32>,
    #[serde(default)]
    pub darken: Option<bool>,
    #[serde(default)]
    pub sound: Option<[i32; 2]>,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub anim: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CinematicSpec {
    #[serde(default)]
    pub letterbox: Option<bool>,
    #[serde(default)]
    pub bars: Option<f32>,
    #[serde(default, rename = "hideUi")]
    pub hide_ui: Option<bool>,
    #[serde(default)]
    pub freeze: Option<bool>,
    #[serde(default)]
    pub lines: Option<f32>,
    #[serde(default)]
    pub camera: Option<ThrowCameraSpec>,
    #[serde(default, rename = "finishSlow")]
    pub finish_slow: Option<i32>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub cuts: Vec<CinematicCutSpec>,
}

pub(super) fn zoom_cns(sid: i32, tag: &str, trig: &str, cam: &ThrowCameraSpec) -> String {
    let mut out = String::new();
    if cam.freeze == Some(true) {
        out += &format!(
            "[State {sid}, {tag}_freeze]\ntype = AssertSpecial\ntrigger1 = {trig}\nflag = camerafreeze\n\n"
        );
    }
    if cam.hide_ui == Some(true) {
        out += &format!(
            "[State {sid}, {tag}_hideui]\ntype = AssertSpecial\ntrigger1 = {trig}\nflag = nobardisplay\n\n"
        );
    }
    if let Some(z) = cam.zoom {
        let zt = cam.time.unwrap_or(24).max(1);
        let zlag = cam.lag.unwrap_or(0.5);
        let zdelta = cam.delta.unwrap_or(0.0);
        let zdelay = cam.delay.unwrap_or(0).max(0);
        let zbound = if cam.bound.unwrap_or(true) { 1 } else { 0 };
        let zpos = cam.pos.unwrap_or([0.0, 0.0]);
        out += &format!(
            "[State {sid}, {tag}_zoom]\ntype = Zoom\ntrigger1 = {trig}\nscale = {z}\ndelta = {zdelta}\npos = {}, {}\ntime = {zt}\nlag = {zlag}\nzoomindelay = {zdelay}\nbound = {zbound}\n\n",
            zpos[0], zpos[1]
        );
    }
    if let Some(b) = cam.bars {
        out +=
            &format!("[State {sid}, {tag}_bars]\ntype = Cinema\ntrigger1 = {trig}\nbars = {b}\n\n");
    }
    out
}

pub(super) fn cinema_ctrl_cns(
    sid: i32,
    tag: &str,
    trig: &str,
    bars: f32,
    slow: i32,
    invert: bool,
    lines: f32,
    flash: Option<&FlashSpec>,
    darken: Option<bool>,
) -> String {
    let mut out = String::new();
    let fl_rgb = flash.and_then(|f| f.color).unwrap_or([0, 0, 0]);
    if bars > 0.0 || slow > 0 || invert || lines > 0.0 || flash.is_some() {
        let inv_i = if invert { 1 } else { 0 };
        out += &format!(
            "[State {sid}, {tag}_cinema]\ntype = Cinema\ntrigger1 = {trig}\nignorehitpause = 1\nbars = {bars}\nslow = {slow}\ninvert = {inv_i}\nlines = {lines}\nflash = {}, {}, {}\n\n",
            fl_rgb[0], fl_rgb[1], fl_rgb[2]
        );
    }
    if let Some(fl) = flash {
        let col = fl.color.unwrap_or([255, 255, 255]);
        let under = if fl.under.unwrap_or(false) { 1 } else { 0 };
        let sm = if fl.super_move.unwrap_or(true) {
            "supermove = 1\n"
        } else {
            ""
        };
        out += &format!(
            "[State {sid}, {tag}_flash]\ntype = EnvColor\ntrigger1 = {trig}\nignorehitpause = 1\ntime = {}\nvalue = {}, {}, {}\nunder = {under}\n{sm}\n",
            fl.time.max(1), col[0], col[1], col[2]
        );
    }
    if darken == Some(true) {
        out += &format!(
            "[State {sid}, {tag}_darken]\ntype = BGPalFX\ntrigger1 = {trig}\ntime = 16\nmul = 90, 90, 110\n\n"
        );
    }
    out
}

pub(super) fn move_cns(m: &NativeMove, sid: i32, anim_no: &HashMap<String, i32>) -> String {
    let Some(c) = &m.cinematic else {
        return String::new();
    };
    let mut out = String::new();
    let bars = c.bars.unwrap_or(if c.letterbox.unwrap_or(true) {
        1.0
    } else {
        0.0
    });
    let lines = c.lines.unwrap_or(0.0);
    if bars > 0.0 || lines > 0.0 {
        out += &cinema_ctrl_cns(
            sid,
            "cine_init",
            "Time = 0",
            bars,
            0,
            false,
            lines,
            None,
            None,
        );
    }
    if c.hide_ui == Some(true) {
        out += &format!(
            "[State {sid}, cine_hideui]\ntype = AssertSpecial\ntrigger1 = 1\nflag = nobardisplay\n\n"
        );
    }
    if c.freeze == Some(true) {
        out += &format!(
            "[State {sid}, cine_camfreeze]\ntype = AssertSpecial\ntrigger1 = 1\nflag = camerafreeze\n\n"
        );
    }
    if let Some(title) = &c.title {
        let safe = title.replace('"', "");
        out += &format!(
            "[State {sid}, cine_title]\ntype = Text\ntrigger1 = Time = 0\nignorehitpause = 1\ntext = \"{safe}\"\npos = 160, 44\nremovetime = 48\nid = {sid}\n\n"
        );
    }
    if let Some(cam) = &c.camera {
        out += &zoom_cns(sid, "cine_base", "Time = 0", cam);
    }
    for (idx, cut) in c.cuts.iter().enumerate() {
        let trig = format!("AnimElem = {}", cut.at.max(1));
        if let Some(cam) = &cut.camera {
            out += &zoom_cns(sid, &format!("cut_{idx}"), &trig, cam);
        }
        if let Some(sh) = cut.shake {
            if sh > 0 {
                out += &format!(
                    "[State {sid}, cut_{idx}_shake]\ntype = EnvShake\ntrigger1 = {trig}\nignorehitpause = 1\ntime = {sh}\nfreq = 95\nampl = -6\n\n"
                );
            }
        }
        if let Some([g, i]) = cut.sound {
            out += &format!(
                "[State {sid}, cut_{idx}_snd]\ntype = PlaySnd\ntrigger1 = {trig}\nvalue = {g}, {i}\n\n"
            );
        }
        if let Some(txt) = &cut.text {
            let safe = txt.replace('"', "");
            out += &format!(
                "[State {sid}, cut_{idx}_txt]\ntype = Text\ntrigger1 = {trig}\nignorehitpause = 1\ntext = \"{safe}\"\npos = 160, 44\nremovetime = 42\nid = {sid}\n\n"
            );
        }
        out += &cinema_ctrl_cns(
            sid,
            &format!("cut_{idx}"),
            &trig,
            cut.bars.unwrap_or(bars),
            cut.slowmo.unwrap_or(0),
            cut.invert.unwrap_or(false),
            cut.lines.unwrap_or(0.0),
            cut.flash.as_ref(),
            cut.darken,
        );
        if let Some(ref anim_name) = cut.anim {
            if let Some(&an) = anim_no.get(anim_name) {
                out += &format!(
                    "[State {sid}, cut_{idx}_anim]\ntype = ChangeAnim\ntrigger1 = {trig}\nvalue = {an}\n\n"
                );
            }
        }
    }
    if let Some(fs) = c.finish_slow.filter(|&v| v > 0) {
        out += &format!(
            "[State {sid}, cine_finish_slow]\ntype = Cinema\ntrigger1 = MoveHit = 1 && AnimElemNo(0) >= {}\nignorehitpause = 1\npersistent = 0\nbars = 1.0\nslow = {fs}\ninvert = 1\nlines = 0.85\n\n",
            m.hits.iter().map(|h| h.at).max().unwrap_or(1)
        );
    }
    out
}
