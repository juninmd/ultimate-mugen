//! A running fight: fixed 60 Hz simulation, AI/human inputs, events → audio + rumble.
use crate::audio::Audio;
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

pub enum Pilot { Human(usize), Cpu(Ai), Dummy }

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
}

thread_local! { static FX: Arc<CharDef> = build_fx(true); }

impl Session {
    /// `teams[side]` = fighters of that side (leader first). Players are added interleaved (P1, P2, P3, P4) so that
    /// `roots()[0]`/`roots()[1]` are always the two team leaders. `life` = leader life ratios (sequence carry-over).
    #[allow(clippy::too_many_arguments)]
    pub fn new(teams: [Vec<Arc<CharDef>>; 2], stage: Stage, pilots: Vec<Pilot>, training: bool, duo: bool, simul: bool, single_round: bool, life: [f32; 2]) -> Self {
        let fx = FX.with(|f| f.clone());
        let mut m = Match::new(common_def(), Some(fx), stage.info.clone());
        m.training = training;
        m.team_simul = simul;
        if single_round { m.rounds_to_win = 1; }
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
        for side in 0..2 { if life[side] < 0.999 { if let Some(f) = m.fighters.iter_mut().find(|f| !f.is_helper && f.side == side) { f.life = (f.life_max * life[side]).max(1.0); } } }
        let r = MatchRenderer::new(&stage);
        let mut vfx = Vfx::new();
        vfx.weather = crate::vfx::Weather::for_stage(&stage.info.name);
        Session { team: None, m, stage, r, hud: Hud::new(), vfx, pilots, duo, paused: false, acc: 0.0, over: None, over_timer: 0.0 }
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
        let roots = self.m.root_indices();
        let mut bits = vec![0u32; roots.len()];
        for (slot, p) in self.pilots.iter_mut().enumerate() {
            let Some(&fi) = roots.get(slot) else { continue };
            if !self.m.fighters[fi].alive { continue; }
            bits[slot] = match p {
                Pilot::Cpu(ai) => ai.poll(&self.m, fi),
                Pilot::Dummy => 0,
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
        self.vfx.pre_tick(&self.m);
        self.m.tick(&bits);
        self.vfx.tick(&self.m);
        for k in std::mem::take(&mut self.vfx.sfx) { audio.sfx(k); }
        for ev in std::mem::take(&mut self.m.events) {
            self.vfx.event(&ev, &self.m);
            for k in std::mem::take(&mut self.vfx.sfx) { audio.sfx(k); }
            match ev {
                Event::Sound { def, group, index, own, volume, builtin, .. } => {
                    if let (true, Some(d)) = (own, def.as_ref()) {
                        if let Some(w) = d.sounds.get(&(group, index)) { audio.sample(&d.id, group, index, Some(w), volume); continue; }
                    }
                    let kind = builtin.unwrap_or(match group { 5 => if index >= 2 { "hitheavy" } else { "hit" }, 6 => "guard", 7 => "land", _ => if index >= 2 { "swingh" } else { "swing" } });
                    if group >= 0 || builtin.is_some() { audio.sfx(kind); }
                }
                Event::Hit { attacker_side, defender_side, damage, guarded, heavy, .. } => {
                    let human = |s: usize, p: &[Pilot]| match p.get(s) { Some(Pilot::Human(slot)) => Some(*slot), _ => None };
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

    /// One simulation tick plus its visual effects update (used by the screenshot tour).
    pub fn step_visual(&mut self, c: &mut Controls, audio: &mut Audio) { self.step(c, audio); self.vfx.update(1.0 / 60.0); }

    /// Advance in real time (fixed step).
    pub fn update(&mut self, dt: f32, c: &mut Controls, audio: &mut Audio) {
        if self.paused { return; }
        self.vfx.update(dt);
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
        g.post.begin();
        let p = self.r.draw(g, &self.m, &self.stage, &mut self.vfx, ui);
        g.post.end(&p, ui.t);
        self.hud.draw(ui, g, &self.m, ui.t, self.team.as_ref());
        self.vfx.draw_banners(ui);
    }
}
