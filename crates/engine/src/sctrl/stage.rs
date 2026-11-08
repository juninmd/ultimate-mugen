use crate::chardef::Controller;
use crate::matchs::Match;
use ume_formats::stage::BgCtrlKind;

fn value(m: &Match, i: usize, sc: &Controller, names: &[&str]) -> Option<f32> {
    let key = names.iter().find(|k| sc.has(k, 0))?;
    Some(m.num(i, sc, key, 0.0, 0) as f32)
}

pub fn modify_stage_var(m: &mut Match, i: usize, sc: &Controller) {
    let l = m.fighters[i].lscl;
    let v = |names: &[&str]| value(m, i, sc, names);
    let bound_left = v(&["camera.boundleft", "boundleft"]);
    let bound_right = v(&["camera.boundright", "boundright"]);
    let bound_high = v(&["camera.boundhigh", "boundhigh"]);
    let bound_low = v(&["camera.boundlow", "boundlow"]);
    let follow = v(&["camera.verticalfollow", "verticalfollow"]);
    let tension = v(&["camera.tension", "tension"]);
    let start_x = v(&["camera.startx", "startx"]);
    let start_y = v(&["camera.starty", "starty"]);
    let zoom_out = v(&["camera.zoomout", "zoomout"]);
    let zoffset = v(&["stageinfo.zoffset", "zoffset"]);
    let p1x = v(&["playerinfo.p1startx", "p1startx"]);
    let p2x = v(&["playerinfo.p2startx", "p2startx"]);
    let s = &mut m.stage;
    if let Some(x) = bound_left {
        s.bound_left = x * l;
    }
    if let Some(x) = bound_right {
        s.bound_right = x * l;
    }
    if let Some(x) = bound_high {
        s.bound_high = x * l;
    }
    if let Some(x) = bound_low {
        s.bound_low = x * l;
    }
    if s.bound_left > s.bound_right {
        std::mem::swap(&mut s.bound_left, &mut s.bound_right);
    }
    if s.bound_high > s.bound_low {
        std::mem::swap(&mut s.bound_high, &mut s.bound_low);
    }
    if let Some(x) = follow {
        s.vertical_follow = x.clamp(0.0, 1.0);
    }
    if let Some(x) = tension {
        s.tension = (x * l).max(0.0);
    }
    if let Some(x) = start_x {
        s.start_x = x * l;
    }
    if let Some(x) = start_y {
        s.start_y = x * l;
    }
    if let Some(x) = zoom_out {
        s.zoom_out = x.max(0.1);
    }
    if let Some(x) = p1x {
        s.p1x = x * l;
    }
    if let Some(x) = p2x {
        s.p2x = x * l;
    }
    if let Some(x) = zoffset {
        s.zoffset = x * l;
        if m.zoffset0 != 0.0 {
            m.zoffset0 = x * l;
        }
    }
    m.refresh_stage_camera();
}

fn index_list(m: &Match, i: usize, sc: &Controller, key: &str) -> Vec<f32> {
    let mut out = vec![];
    let mut k = 0;
    while sc.has(key, k) {
        out.push(m.num(i, sc, key, 0.0, k) as f32);
        k += 1;
    }
    out
}

pub fn modify_stage_bg(m: &mut Match, i: usize, sc: &Controller) {
    let id = if sc.has("id", 0) {
        Some(m.num(i, sc, "id", -1.0, 0) as i32)
    } else {
        None
    };
    let visible = sc
        .has("visible", 0)
        .then(|| m.num(i, sc, "visible", 1.0, 0) != 0.0);
    let enabled = sc
        .has("enabled", 0)
        .then(|| m.num(i, sc, "enabled", 1.0, 0) != 0.0);
    let vel = index_list(m, i, sc, "velocity");
    let pos = index_list(m, i, sc, "pos");
    let offset = index_list(m, i, sc, "offset");
    let sin_x = index_list(m, i, sc, "sinx");
    let sin_y = index_list(m, i, sc, "siny");
    let anim = sc
        .has("anim", 0)
        .then(|| m.num(i, sc, "anim", 0.0, 0) as i32);
    let Some(bg) = m.bg.as_mut() else { return };
    let stage = bg.stage.clone();
    for (k, layer) in stage.layers.iter().enumerate() {
        if id.is_some_and(|id| id >= 0 && layer.id != id) {
            continue;
        }
        let s = &mut bg.layers[k];
        if let Some(v) = visible {
            s.visible = v;
        }
        if let Some(v) = enabled {
            s.enabled = v;
        }
        if let Some(x) = vel.first() {
            s.vel[0] = *x;
        }
        if let Some(y) = vel.get(1) {
            s.vel[1] = *y;
        }
        if let Some(x) = pos.first() {
            s.pos[0] = *x - layer.start[0];
        }
        if let Some(y) = pos.get(1) {
            s.pos[1] = *y - layer.start[1];
        }
        if let Some(x) = offset.first() {
            s.pos[0] += *x;
        }
        if let Some(y) = offset.get(1) {
            s.pos[1] += *y;
        }
        if sin_x.len() >= 2 {
            s.sin_x = [sin_x[0], sin_x[1], sin_x.get(2).copied().unwrap_or(0.0)];
        }
        if sin_y.len() >= 2 {
            s.sin_y = [sin_y[0], sin_y[1], sin_y.get(2).copied().unwrap_or(0.0)];
        }
        if let Some(a) = anim {
            s.anim = Some(a);
            s.anim_time = 0;
        }
    }
}

pub fn modify_bg_ctrl(m: &mut Match, i: usize, sc: &Controller) {
    let id = sc.has("id", 0).then(|| m.num(i, sc, "id", -1.0, 0) as i32);
    let time = index_list(m, i, sc, "time");
    let looptime = sc
        .has("looptime", 0)
        .then(|| m.num(i, sc, "looptime", -1.0, 0) as i32);
    let value = index_list(m, i, sc, "value");
    let x = sc.has("x", 0).then(|| m.num(i, sc, "x", 0.0, 0) as f32);
    let y = sc.has("y", 0).then(|| m.num(i, sc, "y", 0.0, 0) as f32);
    let Some(bg) = m.bg.as_mut() else { return };
    for def in bg.ctrl_defs.iter_mut() {
        if let Some(l) = looptime {
            def.loop_time = l;
        }
        for c in def.ctrls.iter_mut() {
            if id.is_some_and(|id| id >= 0 && !c.ids.contains(&id)) {
                continue;
            }
            if let Some(t) = time.first() {
                c.start = *t as i32;
                c.end = *t as i32;
            }
            if let Some(t) = time.get(1) {
                c.end = *t as i32;
            }
            if let Some(t) = time.get(2) {
                c.loop_time = *t as i32;
            }
            let v = value.first().copied();
            c.kind = match &c.kind {
                BgCtrlKind::Visible(_) => v
                    .map(|v| BgCtrlKind::Visible(v != 0.0))
                    .unwrap_or_else(|| c.kind.clone()),
                BgCtrlKind::Enable(_) => v
                    .map(|v| BgCtrlKind::Enable(v != 0.0))
                    .unwrap_or_else(|| c.kind.clone()),
                BgCtrlKind::Anim(_) => v
                    .map(|v| BgCtrlKind::Anim(v as i32))
                    .unwrap_or_else(|| c.kind.clone()),
                BgCtrlKind::VelSet(ox, oy) => BgCtrlKind::VelSet(x.or(*ox), y.or(*oy)),
                BgCtrlKind::VelAdd(ox, oy) => BgCtrlKind::VelAdd(x.or(*ox), y.or(*oy)),
                BgCtrlKind::PosSet(ox, oy) => BgCtrlKind::PosSet(x.or(*ox), y.or(*oy)),
                BgCtrlKind::PosAdd(ox, oy) => BgCtrlKind::PosAdd(x.or(*ox), y.or(*oy)),
                BgCtrlKind::SinX(o) => BgCtrlKind::SinX(if value.len() >= 2 {
                    [value[0], value[1], value.get(2).copied().unwrap_or(o[2])]
                } else {
                    *o
                }),
                BgCtrlKind::SinY(o) => BgCtrlKind::SinY(if value.len() >= 2 {
                    [value[0], value[1], value.get(2).copied().unwrap_or(o[2])]
                } else {
                    *o
                }),
                BgCtrlKind::Null => BgCtrlKind::Null,
            };
        }
    }
}
