//! A running fight: fixed 60 Hz simulation, AI/human inputs, events → audio + rumble.
use crate::audio::{Audio, SoundOpts};
use crate::controls::Controls;
use crate::hud::Hud;
use crate::render::{Gfx, MatchRenderer};
use crate::stages::Stage;
use crate::ui::Ui;
use crate::vfx::Vfx;
use std::sync::Arc;
use ume_engine::ai::Ai;
use ume_engine::chardef::{CharDef, Source};
use ume_engine::common::common_def;
use ume_engine::fx::build_fx;
use ume_engine::input::{k, InputSource};
use ume_engine::matchs::{Event, Match};

/// `Replay(inputs, cursor)`: feeds back recorded inputs (the simulation is deterministic, so this reproduces the fight exactly).
pub enum Pilot { Human(usize), Cpu(Ai), Dummy, Replay(Vec<u32>, usize) }

/// Team roster shown by the HUD (sequence / parallel team modes).
#[derive(Clone, Default)]
pub struct TeamInfo { pub names: [Vec<(String, [u8; 3])>; 2], pub idx: [usize; 2], pub ko: [Vec<bool>; 2], pub simul: bool }

pub struct Session {
    pub team: Option<TeamInfo>,
    pub m: Match,
    pub stage: Stage,
    pub r: MatchRenderer,
    pub hud: Hud,
    pub vfx: Vfx,
    pub pilots: Vec<Pilot>,
    pub duo: bool,
    pub paused: bool,
    acc: f32,
    pub over: Option<usize>,
    pub over_timer: f32,
    /// last inputs of the human player as (bits, frames held) — shown in training mode (SF6-style input history)
    pub inputs: Vec<(u32, u16)>,
    /// inputs of every root slot for every simulated tick (the data a replay is rebuilt from)
    pub rec: Vec<Vec<u32>>,
    pub is_replay: bool,
    #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
    pub net: Option<crate::netlink::NetLink>,
}

thread_local! { static FX: Arc<CharDef> = build_fx(true); }

impl Session {
    /// `teams[side]` = fighters of that side (leader first). Players are added interleaved (P1, P2, P3, P4) so that
    /// `roots()[0]`/`roots()[1]` are always the two team leaders. `life` = leader life ratios (sequence carry-over).
    #[allow(clippy::too_many_arguments)]
    pub fn new(teams: [Vec<Arc<CharDef>>; 2], stage: Stage, pilots: Vec<Pilot>, training: bool, duo: bool, simul: bool, single_round: bool, life: [f32; 2]) -> Self {
        let fx = FX.with(|f| f.clone());
        let mut m = Match::new(common_def(), Some(fx), stage.info.clone());
        if let crate::stages::StageArt::Mugen(bg) = &stage.art { m.bg = Some(ume_engine::stage_bg::BgState::new(bg.stage.clone())); }
        m.training = training;
        m.team_simul = simul;
        if single_round { m.rounds_to_win = 1; }
        // big teams get a zoomed-out field (room to fight); the ground keeps its place in the picture
        let ground_k = if matches!(stage.art, crate::stages::StageArt::Procedural { .. }) { stage.info.zoffset / ume_engine::matchs::BASE_H } else { 0.5 };
        m.set_zoom_for_teams(if simul { teams[0].len().max(teams[1].len()) } else { 1 }, ground_k);
        // face-off at the start of the first round (fighters trade lines before "FIGHT!")
        if !training { m.intro_ticks = 170; }
        let n = teams[0].len().max(teams[1].len());
        let mut slot = 0;
        for k in 0..n {
            for side in 0..2 {
                let Some(d) = teams[side].get(k) else { continue };
                let lv = if let Some(Pilot::Cpu(a)) = pilots.get(slot) { a.level } else { 0 };
                // same character on the field twice → alternate palette
                let dup = teams[1 - side].iter().any(|o| o.id == d.id) && side == 1 || teams[side][..k].iter().any(|o| o.id == d.id);
                m.add_player(d.clone(), side, lv, if dup { 2 } else { 1 });
                slot += 1;
            }
        }
        m.start_round();
        for f in m.fighters.iter_mut().filter(|f| !f.is_helper && f.def.boss) { f.power = 2000.0; }
        for side in 0..2 { if life[side] < 0.999 { if let Some(f) = m.fighters.iter_mut().find(|f| !f.is_helper && f.side == side) { f.life = (f.life_max * life[side]).max(1.0); } } }
        let r = MatchRenderer::new(&stage);
        let mut vfx = Vfx::new();
        vfx.weather = crate::vfx::Weather::for_stage(&stage.info.name);
        Session { team: None, m, stage, r, hud: Hud::new(), vfx, pilots, duo, paused: false, acc: 0.0, over: None, over_timer: 0.0, inputs: vec![], rec: vec![], is_replay: false, #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))] net: None }
    }

    fn mugen_macro(facing: f32, which: u32) -> Vec<u32> {
        let (f, b, d) = (if facing >= 0.0 { k::R } else { k::L }, if facing >= 0.0 { k::L } else { k::R }, k::D);
        let qcf = [d, d, d | f, d | f, f, f];
        match which {
            1 => { let mut v = qcf.to_vec(); v.extend([f | k::X, 0]); v }
            2 => vec![f, f, d, d, d | f, d | f | k::X, 0],
            3 => vec![d, d, d | b, d | b, b, b | k::A, 0],
            _ => { let mut v = qcf.to_vec(); v.push(0); v.extend(qcf); v.extend([f | k::X, 0]); v }
        }
    }

    pub fn step(&mut self, c: &mut Controls, audio: &mut Audio) {
        #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
        if self.net.is_some() { self.net_step(c, audio); return; }
        let roots = self.m.root_indices();
        if self.m.team_simul { if let Some(cur) = self.pilots.iter().position(|p| matches!(p, Pilot::Human(0))) { if roots.get(cur).is_some_and(|&fi| !self.m.fighters[fi].alive) { self.switch_control(); } } }
        let mut bits = vec![0u32; roots.len()];
        for (slot, p) in self.pilots.iter_mut().enumerate() {
            let Some(&fi) = roots.get(slot) else { continue };
            if !self.m.fighters[fi].alive { continue; }
            bits[slot] = match p {
                Pilot::Cpu(ai) => ai.poll(&self.m, fi),
                Pilot::Dummy => 0,
                Pilot::Replay(v, i) => { let b = v.get(*i).copied().unwrap_or(0); *i += 1; b }
                Pilot::Human(s) => {
                    let mut b = c.player_bits(*s, self.duo);
                    // simple-mode buttons for MUGEN characters become motion macros
                    if self.m.fighters[fi].def.source == Source::Mugen && b & (k::D2 | k::W | k::E) != 0 && c.touch.macro_q.is_empty() {
                        let which = if b & k::D2 != 0 && b & k::W != 0 { 4 } else if b & k::D2 != 0 { 1 } else if b & k::W != 0 { 2 } else { 3 };
                        c.touch.macro_q.extend(Self::mugen_macro(self.m.fighters[fi].facing, which));
                        b &= !(k::D2 | k::W | k::E);
                    }
                    b
                }
            };
        }
        if let Some(slot) = self.pilots.iter().position(|p| matches!(p, Pilot::Human(0))) {
            let b = bits.get(slot).copied().unwrap_or(0);
            match self.inputs.last_mut() { Some(l) if l.0 == b => l.1 = (l.1 + 1).min(99), _ => { self.inputs.push((b, 1)); if self.inputs.len() > 14 { self.inputs.remove(0); } } }
        }
        self.rec.push(bits.clone());
        self.vfx.pre_tick(&self.m);
        self.m.tick(&bits);
        self.vfx.tick(&self.m);
        self.drain_events(c, audio);
    }

    #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
    fn net_step(&mut self, c: &mut Controls, audio: &mut Audio) {
        let Some(mut link) = self.net.take() else { return };
        let bits = c.player_bits(0, false);
        let (vfx, rec) = (&mut self.vfx, &mut self.rec);
        let advanced = link.step(&mut self.m, bits, |m, inputs, newest| {
            if newest {
                rec.push(inputs.to_vec());
                vfx.pre_tick(m);
                m.tick(inputs);
                vfx.tick(m);
            } else {
                m.tick(inputs);
            }
        });
        if link.lost.is_some() && self.over.is_none() { self.over = Some(link.local); }
        self.net = Some(link);
        if advanced { self.drain_events(c, audio); }
    }

    fn drain_events(&mut self, c: &mut Controls, audio: &mut Audio) {
        for k in std::mem::take(&mut self.vfx.sfx) { audio.sfx(k); }
        for ev in std::mem::take(&mut self.m.events) {
            self.vfx.event(&ev, &self.m);
            for k in std::mem::take(&mut self.vfx.sfx) { audio.sfx(k); }
            match ev {
                Event::Sound { def, group, index, own, volume, builtin, channel, owner, .. } => {
                    let pan = self.m.idx(owner).map_or(0.0, |i| ((self.m.fighters[i].pos.x - self.m.camera.x) / (self.m.view_w / 2.0)).clamp(-1.0, 1.0));
                    if let (true, Some(d)) = (own, def.as_ref()) {
                        if let Some(w) = d.sounds.get(&(group, index)) { audio.sample(&d.id, group, index, Some(w), SoundOpts { volume, pan, owner, channel, ..SoundOpts::default() }); continue; }
                    }
                    let kind = builtin.unwrap_or(match group { 5 => if index >= 2 { "hitheavy" } else { "hit" }, 6 => "guard", 7 => "land", _ => if index >= 2 { "swingh" } else { "swing" } });
                    if group >= 0 || builtin.is_some() { audio.sfx_at(kind, pan); }
                }
                Event::Hit { damage, guarded, heavy, attacker, defender, .. } => {
                    let roots = self.m.root_indices();
                    let (attacker_side, defender_side) = (attacker, defender);
                    let human = |fi: usize, p: &[Pilot]| roots.iter().position(|&r| r == fi).and_then(|sl| match p.get(sl) { Some(Pilot::Human(slot)) => Some(*slot), _ => None });
                    let mag = (damage / 120.0).clamp(0.15, 1.0);
                    if let Some(sl) = human(defender_side, &self.pilots) { if guarded { c.rumble(sl, 0.0, 0.35, 70); } else { c.rumble(sl, mag, (mag + 0.2).min(1.0), if heavy { 220 } else { 120 }); } }
                    if let Some(sl) = human(attacker_side, &self.pilots) { c.rumble(sl, 0.0, if guarded { 0.15 } else { mag * 0.6 }, 60); }
                }
                Event::Super { side, .. } => { for (s, p) in self.pilots.iter().enumerate() { if let Pilot::Human(sl) = p { c.rumble(*sl, if s == side { 0.6 } else { 0.3 }, 0.6, 400); } } }
                Event::Ko { .. } => { audio.sfx("ko"); for p in &self.pilots { if let Pilot::Human(sl) = p { c.rumble(*sl, 1.0, 1.0, 700); } } }
                Event::MatchOver(w) => { self.over = Some(w); }
                Event::Fight => audio.sfx("fight"),
                Event::RoundStart(_) => audio.sfx("round"),
                Event::RoundEnd(w) if w >= 0 => audio.sfx("win"),
                _ => {}
            }
        }
    }

    /// Ids of the root fighters currently driven by a human (the P1 fighter you control, P2 in versus).
    pub fn human_ids(&self) -> Vec<u32> {
        let roots = self.m.root_indices();
        self.pilots.iter().enumerate().filter(|(_, p)| matches!(p, Pilot::Human(_))).filter_map(|(slot, _)| roots.get(slot).map(|&fi| self.m.fighters[fi].id)).collect()
    }
    /// Parallel teams: hand your controller to the next living teammate (Tab / automatically when yours falls).
    pub fn switch_control(&mut self) {
        let roots = self.m.root_indices();
        let Some(cur) = self.pilots.iter().position(|p| matches!(p, Pilot::Human(0))) else { return };
        let side = self.m.fighters[roots[cur]].side;
        let mates: Vec<usize> = (0..roots.len()).filter(|&sl| self.m.fighters[roots[sl]].side == side).collect();
        let at = mates.iter().position(|&sl| sl == cur).unwrap_or(0);
        for k in 1..mates.len() {
            let cand = mates[(at + k) % mates.len()];
            if self.m.fighters[roots[cand]].alive && cand < self.pilots.len() {
                self.pilots.swap(cur, cand);
                // the AI that used to drive the new fighter now drives the old one: drop its cached per-character data
                if let Pilot::Cpu(a) = &mut self.pilots[cur] { a.reset(); }
                return;
            }
        }
    }

    /// One simulation tick plus its visual effects update (used by the screenshot tour).
    pub fn step_visual(&mut self, c: &mut Controls, audio: &mut Audio) { self.step(c, audio); self.vfx.update(1.0 / 60.0); }

    /// Advance in real time (fixed step).
    pub fn update(&mut self, dt: f32, c: &mut Controls, audio: &mut Audio) {
        if self.paused { return; }
        self.vfx.update(dt);
        // face-off camera push-in
        self.vfx.intro_target = if self.m.round_state == 1 && self.m.round_no == 1 && self.m.intro_ticks > 0 && self.m.round_time > 10 { 0.2 } else { 0.0 };
        self.acc += dt.min(0.1) * self.vfx.time_scale();
        let step = 1.0 / 60.0;
        let mut n = 0;
        while self.acc >= step && n < 5 { self.step(c, audio); self.acc -= step; n += 1; }
        if n == 5 { self.acc = 0.0; }
        // pull the music down under supers and the KO slow-motion so the effects punch through
        audio.duck = if self.m.pause.as_ref().is_some_and(|p| p.is_super) || self.vfx.slow_t > 0.0 { 0.28 } else { 1.0 };
        if self.over.is_some() { self.over_timer += dt; }
    }
    pub fn draw(&mut self, g: &mut Gfx, ui: &Ui) {
        self.r.human = self.human_ids();
        g.post.begin();
        let p = self.r.draw(g, &self.m, &self.stage, &mut self.vfx, ui);
        g.post.end(&p, ui.t);
        let human = self.human_ids();
        self.hud.replay = self.is_replay;
        self.hud.inputs = if self.m.training { self.inputs.clone() } else { vec![] };
        self.hud.draw(ui, g, &self.m, ui.t, self.team.as_ref(), &human);
        self.vfx.draw_banners(ui);
    }
}
