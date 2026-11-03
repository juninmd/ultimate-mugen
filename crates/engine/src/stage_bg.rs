use std::sync::Arc;
use ume_formats::air::Animation;
use ume_formats::stage::{BgCtrlKind, MugenStage};

#[derive(Clone, Debug, PartialEq)]
pub struct BgLayerState {
    pub visible: bool, pub enabled: bool, pub pos: [f32; 2], pub vel: [f32; 2],
    pub anim: Option<i32>, pub anim_time: i32, pub sin_x: [f32; 3], pub sin_y: [f32; 3], pub sin_time: i32, pub link: Option<usize>,
}

#[derive(Clone)]
pub struct BgState { pub stage: Arc<MugenStage>, pub layers: Vec<BgLayerState>, pub def_time: Vec<i32> }

fn sin_wave(s: [f32; 3], t: i32) -> f32 {
    if s[1] <= 0.0 { return 0.0; }
    s[0] * (std::f32::consts::TAU * (t as f32 + s[2]) / s[1]).sin()
}

impl BgState {
    pub fn new(stage: Arc<MugenStage>) -> Self {
        let mut last_root = None;
        let layers = stage.layers.iter().enumerate().map(|(k, l)| {
            let link = if l.position_link { last_root } else { last_root = Some(k); None };
            BgLayerState { visible: true, enabled: true, pos: [0.0; 2], vel: l.velocity, anim: None, anim_time: 0, sin_x: l.sin_x, sin_y: l.sin_y, sin_time: 0, link }
        }).collect();
        let def_time = vec![0; stage.ctrl_defs.len()];
        BgState { stage, layers, def_time }
    }

    pub fn reset(&mut self) { *self = BgState::new(self.stage.clone()); }

    pub fn step(&mut self) {
        let stage = self.stage.clone();
        for (d, def) in stage.ctrl_defs.iter().enumerate() {
            let t = self.def_time[d];
            for c in &def.ctrls {
                let tt = if c.loop_time > 0 { t % c.loop_time } else { t };
                if tt < c.start || tt > c.end { continue; }
                for (k, l) in stage.layers.iter().enumerate() {
                    if !c.ids.is_empty() && !c.ids.contains(&l.id) { continue; }
                    let s = &mut self.layers[k];
                    match &c.kind {
                        BgCtrlKind::Null => {}
                        BgCtrlKind::Visible(v) => s.visible = *v,
                        BgCtrlKind::Enable(v) => s.enabled = *v,
                        BgCtrlKind::VelSet(x, y) => { if let Some(x) = x { s.vel[0] = *x; } if let Some(y) = y { s.vel[1] = *y; } }
                        BgCtrlKind::VelAdd(x, y) => { if let Some(x) = x { s.vel[0] += x; } if let Some(y) = y { s.vel[1] += y; } }
                        BgCtrlKind::PosSet(x, y) => { if let Some(x) = x { s.pos[0] = x - l.start[0]; } if let Some(y) = y { s.pos[1] = y - l.start[1]; } }
                        BgCtrlKind::PosAdd(x, y) => { if let Some(x) = x { s.pos[0] += x; } if let Some(y) = y { s.pos[1] += y; } }
                        BgCtrlKind::Anim(a) => { s.anim = Some(*a); s.anim_time = 0; }
                        BgCtrlKind::SinX(v) => s.sin_x = *v,
                        BgCtrlKind::SinY(v) => s.sin_y = *v,
                    }
                }
            }
            let next = t + 1;
            self.def_time[d] = if def.loop_time > 0 && next >= def.loop_time { 0 } else { next };
        }
        for s in self.layers.iter_mut().filter(|s| s.enabled) {
            s.pos[0] += s.vel[0]; s.pos[1] += s.vel[1];
            s.anim_time += 1; s.sin_time += 1;
        }
    }

    pub fn offset(&self, k: usize) -> [f32; 2] {
        let s = &self.layers[k];
        let own = [s.pos[0] + sin_wave(s.sin_x, s.sin_time), s.pos[1] + sin_wave(s.sin_y, s.sin_time)];
        match s.link { Some(r) if r != k => { let p = self.offset(r); [own[0] + p[0], own[1] + p[1]] } _ => own }
    }

    pub fn visible(&self, k: usize) -> bool { let s = &self.layers[k]; s.visible && s.link.is_none_or(|r| self.layers[r].visible) }

    pub fn animation(&self, k: usize) -> Option<&Animation> {
        match self.layers[k].anim { Some(a) => self.stage.anims.get(&a), None => self.stage.layers[k].anim.as_ref() }
    }

    pub fn sprite(&self, k: usize) -> (i32, i32) {
        let l = &self.stage.layers[k];
        let Some(a) = self.animation(k) else { return l.sprite };
        let tot = a.total_time.max(1);
        let ls: i32 = a.frames.iter().take(a.loop_start).map(|f| f.time.max(0)).sum();
        let mut tm = self.layers[k].anim_time;
        if tm >= tot && a.total_time > 0 { tm = ls + (tm - ls) % (tot - ls).max(1); }
        let mut acc = 0;
        for f in &a.frames {
            if f.time < 0 || tm < acc + f.time { return (f.group, f.index); }
            acc += f.time;
        }
        a.frames.last().map(|f| (f.group, f.index)).unwrap_or(l.sprite)
    }
}
