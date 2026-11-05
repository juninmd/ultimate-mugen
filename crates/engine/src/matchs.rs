//! Match simulation: round flow, state machine driver, collision/hit system, entities, camera.
use crate::chardef::{Box4, CharDef, Controller, Source};
use crate::expr::{truthy, Expr};
use crate::fighter::*;
use crate::flags::gsf;
use crate::triggers::Ctx;
use std::cell::RefCell;
use std::sync::Arc;

mod types;
pub use types::*;

#[derive(Clone)]

pub struct Match {
    pub fighters: Vec<Fighter>,
    pub explods: Vec<Explod>,
    pub projectiles: Vec<Projectile>,
    pub common: Arc<CharDef>,
    pub fx: Option<Arc<CharDef>>,
    pub stage: StageInfo,
    pub camera: V2,
    pub camera_target_y: f32,
    pub zoom_request: Option<crate::camera::ZoomRequest>,
    pub warnings: Vec<String>,
    pub view_w: f32, pub view_h: f32,
    /// Zoom-out for big teams: the play field is `zoom` × the base 426×240 view (so 5v5 has room to fight). Driven by `update_camera`
    /// between `zoom_min` (formation size) and `zoom_max`, depending on how far apart the fighters are.
    pub zoom: f32, pub zoom_min: f32, pub zoom_max: f32,
    /// where the ground sits in the zoomed view: `zoffset0 + (view_h - 240) * ground_k`
    pub zoffset0: f32, pub ground_k: f32,
    pub game_time: i64,
    pub round_state: i32, pub round_no: i32, pub round_time: i32,
    /// side that landed the first hit of the round (None until someone connects)
    pub first_attack: Option<usize>,
    /// Simultaneous 2v2 ("simul"): a team loses the round only when all its members are KO'd.
    pub team_simul: bool,
    pub team_tag: bool,
    /// MUGEN-style pushbox: an attacking fighter does not get pushed back by the defender; off = symmetric split.
    pub classic_pushbox: bool,
    pub tag_cooldown: [i32; 2],
    pub tag_cooldown_max: i32,
    pub tag_cancel_window: i32,
    pub tag_ko_timer: [i32; 2],
    pub team_turns: bool,
    pub team_size: [usize; 2],
    pub team_member: [usize; 2],
    pub team_lead: [String; 2],
    /// how many times an attacker was pushed back because the defender was cornered (diagnostics / tests)
    pub corner_pushes: u32,
    /// extra ticks the first round waits before "FIGHT!" so the fighters can face off / trade lines (0 = none)
    pub intro_ticks: i32,
    pub won_by_ko: bool,
    pub timer: i32, pub round_ticks: i32, pub wins: [i32; 2], pub rounds_to_win: i32, pub winner_side: i32, pub match_over: bool, pub training: bool,
    pub global_asserts: u32,
    pub pause: Option<PauseState>,
    pub shake_time: i32, pub shake_ampl: f32, pub shake_freq: f32, pub shake_phase: f32, pub shake_elapsed: i32, pub shake_dir: u8,
    pub flash: Option<(i32, [f32; 3])>,
    pub bg: Option<crate::stage_bg::BgState>,
    pub all_palfx: Option<PalFx>, pub bg_palfx: Option<PalFx>,
    pub slow_mo: i32,
    pub events: Vec<Event>,
    pub texts: Vec<crate::ikemen::TextItem>,
    pub team_maps: [crate::ikemen::MapStore; 2],
    pub console: crate::sctrl::console::ConsoleLog,
    pub restart_pending: bool,
    pub announce: Option<Announce>,
    pub rng: u32,
    pub next_id: u32,
    pub assigns: RefCell<Vec<(usize, AssignKind, i32, f64)>>,
    pub tick_ids: Vec<u32>,
    pub scratch_c1: Vec<Box4>,
    pub scratch_c2: Vec<Box4>,
}

impl Match {
    pub fn new(common: Arc<CharDef>, fx: Option<Arc<CharDef>>, stage: StageInfo) -> Self {
        Match {
            fighters: vec![], explods: vec![], projectiles: vec![], common, fx, stage, camera: V2::default(), camera_target_y: 0.0, zoom_request: None, warnings: vec![], view_w: BASE_W, view_h: BASE_H, zoom: 1.0, zoom_min: 1.0, zoom_max: 1.0, zoffset0: 0.0, ground_k: 0.5,
            team_simul: false, team_tag: false, classic_pushbox: true, tag_cooldown: [0; 2], tag_cooldown_max: crate::tag::DEFAULT_COOLDOWN, tag_cancel_window: crate::tag::DEFAULT_CANCEL_WINDOW, tag_ko_timer: [0; 2], team_turns: false, team_size: [0, 0], team_member: [0, 0], team_lead: [String::new(), String::new()], won_by_ko: false, corner_pushes: 0, intro_ticks: 0, game_time: 0, round_state: 0, round_no: 1, round_time: 0, first_attack: None, timer: 99 * 60, round_ticks: 99 * 60, wins: [0, 0], rounds_to_win: 2, winner_side: -1,
            match_over: false, training: false, global_asserts: 0, pause: None, shake_time: 0, shake_ampl: 0.0, shake_freq: 60.0, shake_phase: 0.0, shake_elapsed: 0, shake_dir: 0, flash: None, all_palfx: None, bg_palfx: None, bg: None, slow_mo: 0,
            events: vec![], texts: vec![], team_maps: Default::default(), console: Default::default(), restart_pending: false, announce: None, rng: 0x2545_f491, next_id: 56, assigns: RefCell::new(vec![]), tick_ids: vec![],
            scratch_c1: Vec::with_capacity(16), scratch_c2: Vec::with_capacity(16),
        }
    }
    pub fn random(&mut self) -> f32 {
        let mut x = self.rng; x ^= x << 13; x ^= x >> 17; x ^= x << 5; self.rng = x;
        x as f32 / 4294967296.0
    }
    pub fn add_player(&mut self, def: Arc<CharDef>, side: usize, ai_level: i32, pal_no: i32) -> usize {
        let id = self.next_id; self.next_id += 1;
        let n = self.roots().count() + 1;
        let mut f = Fighter::new(id, def, side, n);
        f.ai_level = ai_level; f.pal_no = pal_no;
        self.fighters.push(f);
        self.fighters.len() - 1
    }
    pub fn roots(&self) -> impl Iterator<Item = &Fighter> { self.fighters.iter().filter(|f| !f.is_helper) }
    pub fn team_mode(&self) -> TeamMode { if self.team_tag { TeamMode::Tag } else if self.team_simul { TeamMode::Simul } else if self.team_turns { TeamMode::Turns } else { TeamMode::Single } }
    fn root_of(&self, i: usize) -> usize { if self.fighters[i].is_helper { self.idx(self.fighters[i].root).unwrap_or(i) } else { i } }
    pub fn team_size_of(&self, side: usize) -> usize {
        if self.team_size[side.min(1)] > 0 { self.team_size[side.min(1)] } else { self.roots().filter(|r| r.side == side).count() }
    }
    pub fn num_players(&self) -> usize { self.team_size_of(0) + self.team_size_of(1) }
    pub fn member_no(&self, i: usize) -> usize {
        let r = self.root_of(i);
        let side = self.fighters[r].side;
        if self.team_turns { return self.team_member[side.min(1)] + 1; }
        self.roots().filter(|o| o.side == side).position(|o| o.id == self.fighters[r].id).map_or(1, |p| p + 1)
    }
    pub fn slot_no(&self, i: usize) -> usize {
        let r = self.root_of(i);
        if self.team_turns { return self.team_member[self.fighters[r].side.min(1)] * 2 + self.fighters[r].side.min(1) + 1; }
        self.fighters[r].player_no
    }
    pub fn team_leader_no(&self, side: usize) -> usize {
        if self.team_turns { return side.min(1) + 1; }
        self.fighters.iter().position(|f| !f.is_helper && !f.destroyed && f.side == side).map_or(0, |k| self.slot_no(k))
    }
    pub fn team_leader(&self, side: usize) -> Option<String> {
        if let Some(n) = self.team_lead.get(side).filter(|n| !n.is_empty()) { return Some(n.to_uppercase()); }
        let mut members = self.roots().filter(|f| f.side == side);
        let first = members.next()?;
        members.next().map(|_| first.def.display_name.split_whitespace().next().unwrap_or("").to_uppercase())
    }
    pub fn winner_text(&self, side: usize) -> String {
        if let Some(lead) = self.team_leader(side) { return format!("TEAM {lead} WINS"); }
        self.roots().find(|f| f.side == side).map(|f| format!("{} WINS", f.def.display_name.to_uppercase())).unwrap_or_else(|| "DRAW".into())
    }
    pub fn root_index_buf(&self) -> ([usize; 32], usize) {
        let mut buf = [0usize; 32];
        let mut n = 0;
        for (i, f) in self.fighters.iter().enumerate() { if !f.is_helper && n < buf.len() { buf[n] = i; n += 1; } }
        (buf, n)
    }
    /// Deterministic hash of the whole simulation state, used by replay round-trip tests and desync reports.
    pub fn checksum(&self) -> u64 {
        fn mix(h: &mut u64, v: u64) { *h ^= v; *h = h.wrapping_mul(0x100_0000_01b3); }
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for v in [self.game_time as u64, self.timer as u64, self.round_no as u64, self.round_state as u64, self.winner_side as u64, self.wins[0] as u64, self.wins[1] as u64, self.next_id as u64] { mix(&mut h, v); }
        for f in &self.fighters {
            for v in [f.id as u64, f.life.to_bits() as u64, f.life_max.to_bits() as u64, f.pos.x.to_bits() as u64, f.pos.y.to_bits() as u64, f.vel.x.to_bits() as u64, f.vel.y.to_bits() as u64, f.power.to_bits() as u64, f.state_no as u64, f.state_time as u64, f.facing.to_bits() as u64, f.ctrl as u64, f.move_type as u64, f.state_type as u64, f.combo_count as u64, f.combo_damage.to_bits() as u64] { mix(&mut h, v); }
        }
        h
    }

    pub fn root_indices(&self) -> Vec<usize> { (0..self.fighters.len()).filter(|&i| !self.fighters[i].is_helper).collect() }
    pub fn idx(&self, id: u32) -> Option<usize> { self.fighters.iter().position(|f| f.id == id && !f.destroyed) }
    #[inline]
    pub fn idx_hint(&self, id: u32, hint: usize) -> Option<usize> {
        if let Some(f) = self.fighters.get(hint) {
            if f.id == id && !f.destroyed {
                return Some(hint);
            }
        }
        self.idx(id)
    }
    pub fn enemy(&self, i: usize, n: usize) -> Option<usize> {
        let me = &self.fighters[i];
        if n == 0 {
            return self.fighters.iter().enumerate()
                .filter(|(_, f)| !f.is_helper && f.side != me.side && !f.destroyed)
                .take(16)
                .min_by(|(_, a), (_, b)| (!a.alive).cmp(&!b.alive).then_with(||
                    (a.pos.x - me.pos.x).abs().total_cmp(&(b.pos.x - me.pos.x).abs())))
                .map(|(j, _)| j);
        }
        let mut buf = [0usize; 16];
        let mut len = 0;
        for (j, f) in self.fighters.iter().enumerate() {
            if f.is_helper || f.side == me.side || f.destroyed || f.ikemen.standby { continue; }
            if len == buf.len() { break; }
            buf[len] = j;
            len += 1;
        }
        let list = &mut buf[..len];
        list.sort_by(|&a, &b| (!self.fighters[a].alive).cmp(&!self.fighters[b].alive).then((self.fighters[a].pos.x - me.pos.x).abs().partial_cmp(&(self.fighters[b].pos.x - me.pos.x).abs()).unwrap()));
        list.get(n).copied()
    }
    pub fn partner(&self, i: usize, n: usize) -> Option<usize> {
        let side = self.fighters[i].side;
        (0..self.fighters.len()).filter(|&j| j != i && self.fighters[j].side == side && !self.fighters[j].is_helper && !self.fighters[j].destroyed).nth(n)
    }
    pub fn ground_y(&self) -> f32 { self.stage.zoffset }

    // ---------------------------------------------------------------- evaluation helpers
    pub fn ev(&self, i: usize, e: &Expr) -> f64 { e.eval(&Ctx { m: self, i }) }
    #[inline]
    pub fn num_opt(&self, i: usize, sc: &Controller, k: &str, idx: usize) -> Option<f64> {
        let p = sc.param(k)?;
        let e = p.exprs.get(idx)?;
        let v = self.ev(i, e);
        if v.is_nan() { None } else { Some(v) }
    }
    #[inline]
    pub fn num(&self, i: usize, sc: &Controller, k: &str, d: f64, idx: usize) -> f64 {
        match sc.param(k).and_then(|p| p.exprs.get(idx)) {
            Some(e) => {
                let v = e.eval(&Ctx { m: self, i });
                if !v.is_nan() { v } else { d }
            }
            None => d,
        }
    }
    fn apply_assigns(&mut self) {
        if self.assigns.get_mut().is_empty() { return; }
        for (i, kind, idx, v) in self.assigns.get_mut().drain(..) {
            let Some(f) = self.fighters.get_mut(i) else { continue };
            let u = idx as usize;
            match kind {
                AssignKind::Var if u < 60 => f.vars[u] = v as i32,
                AssignKind::FVar if u < 40 => f.fvars[u] = v,
                AssignKind::SysVar if u < 5 => f.sysvars[u] = v as i32,
                AssignKind::SysFVar if u < 5 => f.sysfvars[u] = v,
                _ => {}
            }
        }
    }

    // ---------------------------------------------------------------- state machine
    /// States the engine drives on its own (face-off, awakening) live in a private range and are always resolved from
    /// the built-in common definition: M.U.G.E.N. characters own 190-199, so sharing those numbers made a fighter run
    /// its own intro/move at round start.
    pub fn change_engine_state(&mut self, i: usize, no: i32) {
        let common = self.common.clone();
        self.change_state(i, no, common);
    }
    pub fn change_state(&mut self, i: usize, no: i32, owner: Arc<CharDef>) {
        let owner = if no >= 0 && Arc::ptr_eq(&owner, &self.common) && self.fighters[i].def.states.contains_key(&no) { self.fighters[i].def.clone() } else { owner };
        {
            let f = &mut self.fighters[i];
            f.prev_state_no = f.state_no; f.prev_state_type = f.state_type; f.prev_move_type = f.move_type;
            if f.state_no != no { self.events.push(Event::StateChange { fighter: f.id }); }
            f.state_no = no;
            f.ikemen.dizzy = f.ikemen.dizzy && no == 5300;
            f.ikemen.guard_break = f.ikemen.guard_break && no == 5400;
            if Arc::ptr_eq(&owner, &f.def) { f.state_giver = None; }
            f.st_def = owner.clone();
            f.state_time = 0;
            f.persist.clear();
        }
        let fallback;
        let sd = if let Some(s) = owner.states.get(&no) {
            s
        } else if let Some(_) = self.common.states.get(&no) {
            fallback = self.common.clone();
            fallback.states.get(&no).unwrap()
        } else {
            return;
        };
        let kind = |k: &str| sd.param(k).map(|p| p.raw.trim().chars().next().map(|c| c.to_ascii_uppercase()));
        let ev = |m: &Match, k: &str, idx: usize| sd.param(k).and_then(|p| p.exprs.get(idx)).map(|e| m.ev(i, e));
        {
            let f = &mut self.fighters[i];
            match kind("type") { Some(Some(c)) if c != 'U' => f.state_type = c, None => f.state_type = 'S', _ => {} }
            match kind("movetype") { Some(Some(c)) if c != 'U' => f.move_type = c, None => f.move_type = 'I', _ => {} }
            match kind("physics") { Some(Some(c)) if c != 'U' => f.physics = c, None => f.physics = 'N', _ => {} }
        }
        if let Some(a) = ev(self, "anim", 0) { self.fighters[i].change_anim_own(a as i32, 1); }
        let vx = ev(self, "velset", 0); let vy = ev(self, "velset", 1);
        let ctrl = ev(self, "ctrl", 0); let pa = ev(self, "poweradd", 0); let sp = ev(self, "sprpriority", 0); let fp = ev(self, "facep2", 0);
        let hdp = ev(self, "hitdefpersist", 0).unwrap_or(0.0); let mhp = ev(self, "movehitpersist", 0).unwrap_or(0.0); let hcp = ev(self, "hitcountpersist", 0).unwrap_or(0.0);
        if fp.is_some_and(|v| v != 0.0) { self.face_enemy(i); }
        if let Some(p) = pa { self.add_power(i, p as f32); }
        let f = &mut self.fighters[i];
        if let Some(v) = vx { f.vel.x = v as f32 * f.lscl; }
        if let Some(v) = vy { f.vel.y = v as f32 * f.lscl; }
        if let Some(c) = ctrl { f.ctrl = c != 0.0; }
        if let Some(s) = sp { f.spr_priority = s as i32; }
        if hdp == 0.0 { f.hitdef_active = false; f.reversal = None; f.hitdef_targets.clear(); }
        if mhp == 0.0 { f.move_contact = 0; f.move_hit = 0; f.move_guarded = 0; f.move_reversed = 0; f.move_countered = 0; }
        if hcp == 0.0 { f.hit_count = 0; f.uniq_hit_count = 0; f.uniq_hit_keys.clear(); }
    }
    pub fn face_enemy(&mut self, i: usize) {
        if let Some(e) = self.enemy(i, 0) {
            let ex = self.fighters[e].pos.x; let f = &mut self.fighters[i];
            if ex != f.pos.x { f.facing = if ex > f.pos.x { 1.0 } else { -1.0 }; }
        }
    }
    pub fn add_power(&mut self, i: usize, v: f32) {
        let root = self.fighters[i].root;
        if let Some(r) = self.idx(root) { self.fighters[r].add_power_local(v); }
    }

    fn triggered(&self, i: usize, sc: &Controller) -> bool {
        let ctx = Ctx { m: self, i };
        for e in &sc.trigger_all { if !truthy(e.eval(&ctx)) { return false; } }
        if sc.triggers.is_empty() { return false; }
        sc.triggers.iter().any(|g| g.iter().all(|e| truthy(e.eval(&ctx))))
    }

    /// Run a state's controllers. Returns true if the state changed.
    pub(crate) fn run_state(&mut self, i: usize, sd_owner: &Arc<CharDef>, no: i32, in_pause: bool) -> bool {
        let fallback;
        let sd = if let Some(s) = sd_owner.states.get(&no) {
            s
        } else if no >= 0 {
            fallback = self.common.clone();
            match fallback.states.get(&no) {
                Some(s) => s,
                None => return false,
            }
        } else {
            return false;
        };
        for sc in &sd.controllers {
            if i >= self.fighters.len() || self.fighters[i].destroyed { return true; }
            if in_pause && !sc.ignore_hitpause { continue; }
            if !self.triggered(i, sc) { self.apply_assigns(); continue; }
            self.apply_assigns();
            if sc.persistent != 1 {
                let f = &mut self.fighters[i];
                if let Some(&(_, last)) = f.persist.iter().find(|(k, _)| *k == sc.uid) {
                    if sc.persistent == 0 || f.state_time - last < sc.persistent { continue; }
                }
                if let Some(entry) = f.persist.iter_mut().find(|(k, _)| *k == sc.uid) {
                    entry.1 = f.state_time;
                } else {
                    f.persist.push((sc.uid, f.state_time));
                }
            }
            let (sno, sdef, st) = { let f = &self.fighters[i]; (f.state_no, f.st_def.clone(), f.state_time) };
            let id = self.fighters[i].id;
            crate::controllers::run(self, i, sc);
            self.apply_assigns();
            let Some(i2) = self.idx(id) else { return true };
            if i2 != i { return true; }
            let f = &self.fighters[i];
            if f.state_no != sno || !Arc::ptr_eq(&f.st_def, &sdef) || (f.state_time == 0 && st != 0) { return true; }
        }
        false
    }

    fn run_states(&mut self, i: usize) {
        let in_pause = self.fighters[i].hit_pause > 0 || self.fighters[i].hit_shake > 0;
        {
            let f = &mut self.fighters[i];
            f.asserts = if in_pause { f.asserts & !asf::ANIMATEHITPAUSE } else { 0 };
            if f.ikemen.standby { f.asserts |= asf::INVISIBLE; }
        }
        let own = self.fighters[i].def.clone();
        let custom = !Arc::ptr_eq(&self.fighters[i].st_def, &own) && !Arc::ptr_eq(&self.fighters[i].st_def, &self.common);
        let helper = self.fighters[i].is_helper;
        let id = self.fighters[i].id;
        if !helper && !custom { self.run_state(i, &own, -3, in_pause); }
        let Some(i) = self.idx(id) else { return };
        self.run_state(i, &own, -2, in_pause);
        let Some(i) = self.idx(id) else { return };
        if (!helper || self.fighters[i].key_ctrl) && !custom {
            self.run_state(i, &own, -1, in_pause);
            let Some(i) = self.idx(id) else { return };
            if !in_pause { self.builtin_transitions(i); }
        }
        for _ in 0..16 {
            let Some(i) = self.idx(id) else { return };
            let (same, no) = { let f = &self.fighters[i]; (Arc::ptr_eq(&f.st_def, &own), f.state_no) };
            let owner;
            let st_ref = if same { &own } else { owner = self.fighters[i].st_def.clone(); &owner };
            if !self.run_state(i, st_ref, no, in_pause) { break; }
        }
    }

    pub fn has_command(&self, i: usize, name: &str) -> bool {
        let f = &self.fighters[i];
        let src = if f.is_helper { if f.key_ctrl { self.idx(f.root) } else { None } } else { Some(i) };
        let Some(s) = src else { return false };
        if f.asserted_commands.iter().any(|(n, t)| *t > 0 && n.eq_ignore_ascii_case(name)) { return true; }
        let accepting = if f.is_helper { self.round_state == 2 } else { self.round_state == 2 || self.round_state == 3 };
        if !accepting && !self.training && !self.fighters[s].has_assert(asf::POSTROUNDINPUT) { return false; }
        if !Arc::ptr_eq(&f.st_def, &f.def) && !Arc::ptr_eq(&f.st_def, &self.common) {
            return f.st_def.commands.iter().find(|c| c.name == name).is_some_and(|c| crate::commands::match_command(&self.fighters[s].history, c));
        }
        self.fighters[s].cmd.active.contains(name)
    }

    fn builtin_transitions(&mut self, i: usize) {
        if !self.fighters[i].ctrl || self.fighters[i].ikemen.dizzy || self.fighters[i].ikemen.guard_break || (self.round_state != 2 && !self.training) { return; }
        let cmd = |m: &Match, n: &str| m.fighters[i].cmd.active.contains(n);
        let s = self.fighters[i].state_no;
        let st = self.fighters[i].state_type;
        if let Some(e) = self.enemy(i, 0) {
            let ex = self.fighters[e].pos.x;
            let auto_turn = self.stage.auto_turn;
            let f = &mut self.fighters[i];
            if auto_turn && !f.has_assert(asf::NOAUTOTURN) && matches!(s, 0 | 11 | 20 | 52) && st != 'A' {
                let should = if ex >= f.pos.x { 1.0 } else { -1.0 };
                if should != f.facing && (ex - f.pos.x).abs() > 1.0 {
                    f.facing = should;
                    if s == 0 && f.def.anims.contains_key(&5) { f.change_anim_own(5, 1); }
                    if s == 11 && f.def.anims.contains_key(&6) { f.change_anim_own(6, 1); }
                }
            }
        }
        if self.fighters[i].has_assert(asf::NOHARDCODEDKEYS) { return; }
        if st == 'S' || st == 'C' {
            let guard_blocked = self.fighters[i].has_assert(if st == 'S' { asf::NOSTANDGUARD } else { asf::NOCROUCHGUARD });
            let guard_input = cmd(self, "holdback") || self.fighters[i].has_assert(asf::AUTOGUARD);
            if guard_input && !guard_blocked && self.in_guard_dist(i) && !(120..=155).contains(&s) { let own = self.fighters[i].def.clone(); self.change_state(i, 120, own); return; }
            if st == 'S' {
                if cmd(self, "holdup") && !self.fighters[i].has_assert(asf::NOJUMP) { let own = self.fighters[i].def.clone(); self.change_state(i, 40, own); return; }
                if cmd(self, "holddown") && s != 10 && s != 11 && !self.fighters[i].has_assert(asf::NOCROUCH) { let own = self.fighters[i].def.clone(); self.change_state(i, 10, own); return; }
                if (cmd(self, "holdfwd") || cmd(self, "holdback")) && s != 20 && !self.fighters[i].has_assert(asf::NOWALK) && (s == 0 || s == 52) { let own = self.fighters[i].def.clone(); self.change_state(i, 20, own); }
            } else if !cmd(self, "holddown") && s == 11 && !self.fighters[i].has_assert(asf::NOSTAND) { let own = self.fighters[i].def.clone(); self.change_state(i, 12, own); }
        } else if st == 'A' {
            let f = &self.fighters[i];
            let maxj = f.c("movement.airjump.num", 1.0) as i32;
            let h = &f.history;
            let pressed_up = h.at(h.tick) & 1 != 0 && h.at(h.tick - 1) & 1 == 0;
            if s == 50 && f.air_jumps < maxj && !f.has_assert(asf::NOAIRJUMP) && pressed_up && f.pos.y < -f.c("movement.airjump.height", 35.0) * f.lscl {
                self.fighters[i].air_jumps += 1;
                let own = self.fighters[i].def.clone(); self.change_state(i, 45, own);
            }
        }
    }

    pub fn in_guard_dist(&self, i: usize) -> bool {
        let me = &self.fighters[i];
        for e in &self.fighters {
            if e.side == me.side || e.destroyed { continue; }
            if e.move_type == 'A' && e.hitdef_active { if let Some(h) = &e.hitdef { if (e.pos.x - me.pos.x).abs() <= h.guard_dist * e.lscl { return true; } } }
        }
        self.projectiles.iter().any(|p| p.side != me.side && !p.dying && (p.pos.x - me.pos.x).abs() <= p.guard_dist)
    }

    fn physics_step(&mut self, i: usize) {
        if self.fighters[i].pos_freeze { self.fighters[i].pos_freeze = false; self.update_bind(i); return; }
        if self.fighters[i].bind.is_some() { self.update_bind(i); return; }
        let mut land = false;
        {
            let f = &mut self.fighters[i];
            f.pos.x += f.vel.x * f.facing;
            f.pos.y += f.vel.y;
            match f.physics {
                'S' | 'C' => {
                    let sliding = f.move_type == 'H' && f.slide_time > 0;
                    if !sliding {
                        let fr = if f.physics == 'S' { f.c("movement.stand.friction", 0.85) } else { f.c("movement.crouch.friction", 0.82) };
                        f.vel.x *= fr;
                        if f.vel.x.abs() < 0.05 { f.vel.x = 0.0; }
                    }
                    if f.pos.y > f.ground_level { f.pos.y = f.ground_level; }
                    // idle/hit states with ground physics cannot hold a fighter in mid-air: let it drop back to the floor
                    // (attack states keep their scripted height, e.g. hover or teleport moves)
                    if f.pos.y < f.ground_level && f.state_type != 'A' && f.move_type != 'A' {
                        f.vel.y = f.vel.y.max(0.0) + f.c("movement.yaccel", 0.44) * f.lscl;
                    } else if f.state_type != 'A' { f.vel.y = 0.0; }
                }
                'A' => {
                    f.vel.y += f.c("movement.yaccel", 0.44) * f.lscl;
                    if f.pos.y >= f.ground_level && f.vel.y > 0.0 { f.pos.y = f.ground_level; f.vel.y = 0.0; f.air_jumps = 0; if f.move_type != 'H' { land = true; } }
                }
                _ => {}
            }
            if f.slide_time > 0 { f.slide_time -= 1; }
            if f.move_type == 'H' && f.ghv.xaccel != 0.0 { f.vel.x += f.ghv.xaccel; }
        }
        if land { let d = self.fighters[i].def.clone(); self.change_state(i, 52, d); }
    }
    fn rebind(&mut self) {
        for i in 0..self.fighters.len() {
            let Some(b) = self.fighters[i].bind else { continue };
            let Some(t) = self.idx(b.to) else { continue };
            let (tx, ty, tf) = (self.fighters[t].pos.x, self.fighters[t].pos.y, self.fighters[t].facing);
            let f = &mut self.fighters[i];
            f.pos.x = tx + b.x * tf;
            f.pos.y = ty + b.y;
        }
    }
    fn orphan_custom_states(&mut self) {
        for i in 0..self.fighters.len() {
            let Some(giver) = self.fighters[i].state_giver else { continue };
            if self.idx(giver).is_some() || Arc::ptr_eq(&self.fighters[i].st_def, &self.fighters[i].def) { continue; }
            let d = self.fighters[i].def.clone();
            self.fighters[i].state_giver = None;
            self.change_state(i, 5050, d);
            let f = &mut self.fighters[i];
            f.state_type = 'A'; f.move_type = 'H'; f.physics = 'A';
        }
    }
    pub fn palfx_of(&self, i: usize) -> Option<&PalFx> {
        let f = &self.fighters[i];
        if f.own_pal || !f.is_helper { return f.palfx.as_ref(); }
        f.palfx.as_ref().or_else(|| f.parent.and_then(|p| self.idx(p)).and_then(|p| self.palfx_of(p)))
    }
    pub fn update_bind(&mut self, i: usize) {
        let Some(b) = self.fighters[i].bind else { return };
        let Some(t) = self.idx(b.to) else { self.fighters[i].bind = None; return };
        if b.time == 0 { self.fighters[i].bind = None; return; }
        let (tx, ty, tf) = (self.fighters[t].pos.x, self.fighters[t].pos.y, self.fighters[t].facing);
        let f = &mut self.fighters[i];
        f.pos.x = tx + b.x * tf;
        f.pos.y = ty + b.y;
        if b.time > 0 { f.bind.as_mut().unwrap().time -= 1; }
    }

    // ---------------------------------------------------------------- round flow
    /// Zoom limits from the team sizes (call before `start_round`): 1v1 = no zoom-out, 5v5 = up to 1.8× the field.
    pub fn set_zoom_for_teams(&mut self, per_side: usize, ground_k: f32) {
        let n = per_side.max(1) as f32 - 1.0;
        let stage_out = if self.stage.zoom_out > 0.0 { 1.0 / self.stage.zoom_out } else { 1.0 };
        self.zoom_min = 1.0 + 0.1 * n; self.zoom_max = (1.0 + 0.2 * n).max(stage_out); self.zoom = self.zoom_min; self.zoffset0 = self.stage.zoffset; self.ground_k = ground_k;
        self.apply_zoom();
    }
    pub(crate) fn refresh_stage_camera(&mut self) {
        self.apply_zoom();
        self.camera.x = self.camera.x.clamp(self.stage.bound_left, self.stage.bound_right);
        self.camera.y = self.camera.y.clamp(self.stage.bound_high, self.stage.bound_low);
    }
    pub(crate) fn apply_zoom(&mut self) {
        self.view_w = BASE_W * self.zoom; self.view_h = BASE_H * self.zoom;
        if self.zoffset0 != 0.0 { self.stage.zoffset = self.zoffset0 + (self.view_h - BASE_H) * self.ground_k; }
    }
    pub fn start_round(&mut self) {
        self.round_state = 0; self.round_time = 0; self.timer = if self.round_ticks > 0 { self.round_ticks } else { 99 * 60 }; self.winner_side = -1; self.slow_mo = 0; self.won_by_ko = false;
        self.first_attack = None;
        for f in self.fighters.iter_mut() { f.received_damage = 0.0; f.received_hits = 0; f.combo_count = 0; f.combo_damage = 0.0; f.combo_target = None; f.combo_timer = 0; }
        self.explods.clear(); self.projectiles.clear(); self.pause = None;
        self.fighters.retain(|f| !f.is_helper || f.preserve);
        self.camera = V2 { x: self.stage.start_x, y: self.stage.start_y };
        self.camera_target_y = self.camera.y;
        self.zoom_request = None;
        if let Some(bg) = self.bg.as_mut() { bg.reset(); }
        let mut slot = [0usize; 2];
        let team_n = [self.fighters.iter().filter(|f| !f.is_helper && f.side == 0).count(), self.fighters.iter().filter(|f| !f.is_helper && f.side != 0).count()];
        for i in 0..self.fighters.len() {
            if self.fighters[i].is_helper { continue; }
            {
                let p1x = self.stage.p1x; let p2x = self.stage.p2x; let first = self.round_no == 1;
                let f = &mut self.fighters[i];
                let k = { let s = f.side.min(1); slot[s] += 1; (slot[s] - 1) as f32 };
                f.destroyed = false;
                // teammates line up behind the leader; big teams (up to 5) are packed tighter so everyone starts on screen
                let gap = if team_n[f.side.min(1)] <= 2 { 52.0 } else { 130.0 / (team_n[f.side.min(1)] - 1) as f32 };
                f.pos = V2 { x: if f.side == 0 { p1x - gap * k } else { p2x + gap * k }, y: 0.0 };
                f.vel = V2::default();
                f.ground_level = 0.0;
                f.facing = if f.side == 0 { 1.0 } else { -1.0 };
                f.life = f.life_max; f.alive = true;
                if f.form > 0 { f.attack_mul /= 1.3; f.defence_mul /= 1.25; f.form = 0; f.form_time = 0; }
                if first { f.power = 0.0; }
                f.hit_pause = 0; f.hit_shake = 0; f.targets.clear(); f.bind = None; f.hitdef_active = false; f.ctrl = false;
                f.st_def = f.def.clone(); f.ghv = GetHitVars::default(); f.sysvars = [0; 5];
            }
            crate::ikemen::life::reset_for_round(self, i);
            let d = self.fighters[i].def.clone();
            self.change_state(i, 5900, d);
            self.fighters[i].ctrl = false;
        }
        if self.team_tag { self.reset_tag_round(); }
        // round-count-aware announcer (Guilty Gear: "Final" for the deciding duel)
        let decider = self.rounds_to_win > 1 && self.wins[0] == self.rounds_to_win - 1 && self.wins[1] == self.rounds_to_win - 1;
        self.announce = Some(Announce { text: if decider { "FINAL ROUND".into() } else { format!("ROUND {}", self.round_no) }, time: 70, total: 70, kind: AnnounceKind::Round });
        self.events.push(Event::RoundStart(self.round_no));
    }

    fn round_logic(&mut self) {
        match self.round_state {
            0 if self.round_time > 40 => {
                self.round_state = 1; self.round_time = 0;
                if self.intro_ticks > 0 && self.round_no == 1 {
                    // face-off: the first fighter of each side poses, the rest of the team stands ready
                    let mut seen = [false; 2];
                    for i in self.root_indices() {
                        let s = self.fighters[i].side.min(1);
                        if seen[s] || !self.fighters[i].def.anims.contains_key(&190) { continue; }
                        seen[s] = true;
                        let taunt = s == 1 && self.fighters[i].def.anims.contains_key(&195);
                        self.change_engine_state(i, if taunt { ENGINE_STATE_FACE_OFF_TAUNT } else { ENGINE_STATE_FACE_OFF });
                    }
                }
            }
            // MUGEN: "FIGHT!" waits while any player asserts the Intro flag (capped for safety)
            1 if self.round_time > 60 + if self.round_no == 1 { self.intro_ticks } else { 0 } && (self.global_asserts & gsf::INTRO == 0 || self.round_time > 600) => {
                self.round_state = 2; self.round_time = 0;
                self.announce = Some(Announce { text: "FIGHT!".into(), time: 50, total: 50, kind: AnnounceKind::Fight });
                self.events.push(Event::Fight);
                for f in self.fighters.iter_mut() {
                    if !f.is_helper && (f.state_no == 0 || f.state_no == 5900 || f.state_no == ENGINE_STATE_FACE_OFF || f.state_no == ENGINE_STATE_FACE_OFF_TAUNT) { f.ctrl = true; }
                }
            }
            2 => {
                if !self.training && self.round_ticks > 0 && self.pause.is_none() && self.global_asserts & (gsf::TIMERFREEZE | gsf::ROUNDFREEZE) == 0 { self.timer = (self.timer - 1).max(0); }
                let (root_buf, root_len) = self.root_index_buf();
                let roots = &root_buf[..root_len];
                if self.training {
                    for &i in roots { if !self.fighters[i].alive || self.fighters[i].state_no == 5150 { self.fighters[i].alive = true; self.fighters[i].life = self.fighters[i].life_max; self.fighters[i].move_type = 'I'; self.fighters[i].ctrl = true; let d = self.fighters[i].def.clone(); self.change_state(i, 0, d); } if self.fighters[i].life <= 0.0 { self.fighters[i].life = 1.0; } if self.fighters[i].move_type != 'H' && self.fighters[i].state_time > 90 && self.fighters[i].life < self.fighters[i].life_max { self.fighters[i].life = self.fighters[i].life_max; } }
                    for &i in roots { self.fighters[i].power = self.fighters[i].power_max; }
                    return;
                }
                let ko_disabled = self.global_asserts & gsf::GLOBALNOKO != 0;
                let mut dead: Vec<usize> = roots.iter().copied().filter(|&i| self.fighters[i].life <= 0.0 && !ko_disabled && !self.fighters[i].has_assert(asf::NOKO)).collect();
                let team_life = |m: &Match, s: usize| -> (f32, f32) { roots.iter().filter(|&&i| m.fighters[i].side == s).fold((0.0, 0.0), |a, &i| (a.0 + m.fighters[i].life.max(0.0), a.1 + m.fighters[i].life_max)) };
                if self.team_simul || self.team_tag {
                    // a fallen teammate stays down; the round ends only when a whole team is KO'd
                    for &i in &dead { self.fighters[i].alive = false; }
                    let out = |m: &Match, s: usize| roots.iter().filter(|&&i| m.fighters[i].side == s).all(|&i| m.fighters[i].life <= 0.0 && !m.fighters[i].has_assert(asf::NOKO));
                    let (o0, o1) = (out(self, 0), out(self, 1));
                    dead = roots.iter().copied().filter(|&i| (self.fighters[i].side == 0 && o0) || (self.fighters[i].side == 1 && o1)).collect();
                }
                if !dead.is_empty() || self.timer <= 0 {
                    self.round_state = 3; self.round_time = 0;
                    self.won_by_ko = !dead.is_empty();
                    if !dead.is_empty() {
                        for &i in &dead { self.fighters[i].alive = false; }
                        let sides_dead = [dead.iter().any(|&i| self.fighters[i].side == 0), dead.iter().any(|&i| self.fighters[i].side == 1)];
                        self.winner_side = if sides_dead[0] && sides_dead[1] { -2 } else if sides_dead[0] { 1 } else { 0 };
                        let slow = self.global_asserts & gsf::NOKOSLOW == 0;
                        self.slow_mo = if slow { 75 } else { 0 };
                        self.announce = Some(Announce { text: "K.O.".into(), time: 110, total: 110, kind: AnnounceKind::Ko });
                        self.events.push(Event::Ko { loser_side: self.fighters[dead[0]].side, silent: self.global_asserts & gsf::NOKOSND != 0, slow });
                    } else {
                        let (l0, l1) = (team_life(self, 0), team_life(self, 1));
                        let (r0, r1) = (l0.0 / l0.1.max(1.0), l1.0 / l1.1.max(1.0));
                        self.winner_side = if (r0 - r1).abs() < 1e-6 { -2 } else if r0 > r1 { 0 } else { 1 };
                        self.announce = Some(Announce { text: "TIME OVER".into(), time: 110, total: 110, kind: AnnounceKind::Other });
                    }
                }
            }
            3 => {
                let settled = self.roots().all(|f| f.ikemen.standby || if f.alive { f.move_type != 'H' && (f.ctrl || f.state_no == 0) } else { f.state_no == 5150 || f.state_no == 5110 });
                if ((settled && self.round_time > 90) || self.round_time > 300) && (self.global_asserts & crate::flags::gsf::ROUNDNOTOVER == 0 || self.round_time > 900) {
                    self.round_state = 4; self.round_time = 0;
                    let decides = self.winner_side >= 0 && self.wins[self.winner_side as usize] + 1 >= self.rounds_to_win;
                    for (k, i) in self.root_indices().into_iter().enumerate() {
                        if !self.fighters[i].alive {
                            // every KO plays its own defeat sequence: a different one each round, the longest on the match-deciding one
                            let v = if decides { 3 } else { (self.round_no as usize - 1 + k) % 3 };
                            if self.fighters[i].def.anims.contains_key(&(197 + v as i32)) { let d = self.fighters[i].def.clone(); self.change_state(i, 5190 + v as i32, d); }
                            continue;
                        }
                        self.fighters[i].ctrl = false;
                        let d = self.fighters[i].def.clone();
                        let s = self.fighters[i].side as i32;
                        let no = if self.winner_side == s { 180 } else if self.winner_side == -2 { 175 } else { 170 };
                        self.change_state(i, no, d);
                    }
                    if self.winner_side >= 0 { self.wins[self.winner_side as usize] += 1; }
                    let text = if self.winner_side >= 0 { self.winner_text(self.winner_side as usize) } else { "DRAW".into() };
                    self.announce = Some(Announce { text, time: 150, total: 150, kind: AnnounceKind::Win });
                    self.events.push(Event::RoundEnd(self.winner_side));
                }
            }
            4 if self.round_time > if self.wins[0] >= self.rounds_to_win || self.wins[1] >= self.rounds_to_win { 250 } else { 170 } => {
                if self.wins[0] >= self.rounds_to_win || self.wins[1] >= self.rounds_to_win {
                    if !self.match_over {
                        self.match_over = true;
                        self.events.push(Event::MatchOver(if self.wins[0] > self.wins[1] { 0 } else { 1 }));
                    }
                } else { self.round_no += 1; self.start_round(); }
            }
            _ => {}
        }
    }

    // ---------------------------------------------------------------- tick
    /// Advance one 1/60s tick. `inputs[k]` = bits for the k-th root player.
    pub fn tick(&mut self, inputs: &[u32]) {
        if self.slow_mo > 0 {
            self.slow_mo -= 1;
            if self.slow_mo % 3 != 0 { self.visual_timers(); return; }
        }
        if std::mem::take(&mut self.restart_pending) {
            self.wins = [0, 0];
            self.round_no = 1;
            self.start_round();
        }
        self.game_time += 1;
        if self.global_asserts & gsf::ROUNDFREEZE == 0 { self.round_time += 1; }
        // any attack button skips the first-round face-off (inputs are recorded, so replays skip identically)
        if self.round_state == 1 && self.round_no == 1 && self.intro_ticks > 0 && self.global_asserts & gsf::ROUNDNOTSKIP == 0 && self.round_time > 8 && self.round_time < 60 + self.intro_ticks
            && inputs.iter().any(|b| b & (crate::input::k::A | crate::input::k::B | crate::input::k::C | crate::input::k::X | crate::input::k::Y | crate::input::k::Z) != 0) { self.round_time = 60 + self.intro_ticks; }
        self.round_logic();
        self.global_asserts = 0;
        for p in [&mut self.all_palfx, &mut self.bg_palfx] { if p.as_ref().is_some_and(|fx| fx.expired()) { *p = None; } }
        let paused_by = self.pause.as_ref().map(|p| (p.by, p.move_time, p.is_super));
        let victory_screen = self.round_state >= 4;
        if victory_screen { for f in self.fighters.iter_mut().filter(|f| f.is_helper) { f.asserts |= crate::fighter::asf::INVISIBLE; } }
        let frozen_by_pause = |f: &Fighter| (victory_screen && f.is_helper) || match paused_by {
            None => false,
            Some((by, mt, sup)) => !(f.id == by && mt > 0) && !f.pause_bool && (if sup { f.super_move_time } else { f.pause_move_time }) <= 0,
        };
        let mut k = 0;
        let mut awaken = [0usize; 8];
        let mut awaken_len = 0;
        let mut tag_requests = [0usize; 8];
        let mut tag_len = 0;
        let tagged = self.team_tag;
        for (fi, f) in self.fighters.iter_mut().enumerate() {
            if f.is_helper { continue; }
            let mut bits = if f.has_assert(asf::NOINPUT) { 0 } else { inputs.get(k).copied().unwrap_or(0) };
            if tagged {
                if bits & crate::input::k::TAG != 0 && tag_len < tag_requests.len() { tag_requests[tag_len] = fi; tag_len += 1; }
                bits &= !crate::input::k::TAG;
                if f.ikemen.standby { bits = 0; }
            }
            if bits & crate::input::k::T != 0 && awaken_len < awaken.len() { awaken[awaken_len] = fi; awaken_len += 1; }
            k += 1;
            if f.assert_input.1 > 0 { bits |= f.assert_input.0; f.assert_input.1 -= 1; }
            if f.input_shift > 0 { let (b, fa) = (bits, f.facing as i32); for _ in 0..f.input_shift { f.history.push(b, fa); } }
            else if f.input_shift < 0 { f.history.rewind(f.input_shift); }
            f.input_shift = 0;
            f.history.push(bits, f.facing as i32);
            let paused = f.hit_pause > 0 || f.hit_shake > 0 || frozen_by_pause(f);
            f.cmd.update(&f.history, &f.def.commands, paused);
            f.asserted_commands.retain_mut(|(_, t)| { *t -= 1; *t > 0 });
        }
        for &fi in &awaken[..awaken_len] { self.try_transform(fi); }
        if tagged { self.tag_step(&tag_requests[..tag_len]); }
        let mut ids = std::mem::take(&mut self.tick_ids);
        ids.clear();
        self.collect_run_order(&mut ids);
        for (order_i, &id) in ids.iter().enumerate() {
            let Some(i) = self.idx_hint(id, order_i) else { continue };
            if frozen_by_pause(&self.fighters[i]) { continue; }
            let f = &mut self.fighters[i];
            let frozen = f.hit_pause > 0 || f.hit_shake > 0;
            f.width_override = None; f.height_override = None; f.clsn_mods.retain_mut(|c| { if c.time > 0 { c.time -= 1; } c.time != 0 }); f.player_push = !f.ikemen.standby; f.trans = None; f.draw_offset = V2::default();
            if f.palfx.as_ref().is_some_and(|p| p.expired()) { f.palfx = None; }
            if (!frozen || f.has_assert(asf::ANIMATEHITPAUSE)) && !f.has_assert(asf::ANIMFREEZE) { f.advance_anim(); }
            self.run_states(i);
        }
        if paused_by.is_some() { if let Some(p) = self.pause.as_mut() { if p.move_time > 0 { p.move_time -= 1; } } }
        if let Some((_, _, sup)) = paused_by {
            for f in self.fighters.iter_mut() { let t = if sup { &mut f.super_move_time } else { &mut f.pause_move_time }; if *t > 0 { *t -= 1; } }
        }
        crate::ikemen::tick(self);
        for (order_i, &id) in ids.iter().enumerate() {
            let Some(i) = self.idx_hint(id, order_i) else { continue };
            if frozen_by_pause(&self.fighters[i]) { continue; }
            {
                let f = &mut self.fighters[i];
                if f.hit_pause > 0 || f.hit_shake > 0 {
                    if f.hit_pause > 0 { f.hit_pause -= 1; }
                    if f.hit_shake > 0 { f.hit_shake -= 1; }
                    continue;
                }
            }
            self.physics_step(i);
            let f = &mut self.fighters[i];
            f.state_time += 1;
            if f.form > 0 { f.form_time += 1; f.power = (f.power + 1.2).min(f.power_max); }
            // the final boss keeps charging on its own (a super every ~10 s even when not hitting)
            if f.def.boss && self.round_state == 2 && f.alive { f.power = (f.power + 0.35).min(f.power_max); }
            if f.move_type == 'H' { if f.hit_over_time > 0 { f.hit_over_time -= 1; } f.fall_time += 1; } else { f.juggle_used = 0; f.super_defence_mul = 1.0; }
            if f.move_contact > 0 { f.move_contact += 1; }
            if f.move_hit > 0 { f.move_hit += 1; }
            if f.move_guarded > 0 { f.move_guarded += 1; }
            if f.move_countered > 0 { f.move_countered += 1; }
            if !f.not_hit_by.is_empty() { f.not_hit_by.retain_mut(|n| { n.1 -= 1; n.1 > 0 }); }
            if !f.hit_by.is_empty() { f.hit_by.retain_mut(|n| { n.1 -= 1; n.1 > 0 }); }
            if let Some(p) = f.palfx.as_mut() { p.elapsed += 1; }
            let tp = f.frame().map(|fr| TrailPoint { x: f.pos.x, y: f.pos.y, group: fr.group, index: fr.index, fx: fr.x, fy: fr.y, flip_h: fr.flip_h, facing: f.facing });
            if let Some(ai) = f.after_image.as_mut() {
                ai.ticks += 1;
                if let Some(tp) = tp.filter(|_| (ai.ticks - 1) % ai.params.time_gap.max(1) == 0) { ai.trail.insert(0, tp); }
                ai.trail.truncate(ai.length);
                if ai.time > 0 { ai.time -= 1; if ai.time == 0 { f.after_image = None; } }
            }
        }
        self.tick_ids = ids;
        self.rescue_stuck();
        self.push_and_bound();
        self.rebind();
        self.orphan_custom_states();
        self.detect_hits();
        crate::ikemen::life::maintain(self);
        for f in self.fighters.iter_mut() {
            for o in f.hit_overrides.iter_mut() { if let Some(h) = o { if h.time > 0 { h.time -= 1; if h.time == 0 { *o = None; } } } }
        }
        let paused = paused_by.is_some();
        self.update_projectiles(paused);
        self.update_explods(paused);
        if self.global_asserts & gsf::CAMERAFREEZE == 0 { self.update_camera(); }
        if !paused { if let Some(bg) = self.bg.as_mut() { bg.step(); } }
        self.visual_timers();
        self.skip_asserted_announcements();
        if let Some(p) = self.pause.as_mut() { p.time -= 1; if p.time <= 0 { self.pause = None; } }
        for f in self.fighters.iter_mut() {
            if f.hit_pause == 0 && f.hit_shake == 0 && f.unhittable > 0 { f.unhittable -= 1; }
            if f.combo_timer > 0 {
                f.combo_timer -= 1;
                if f.combo_timer == 0 { f.combo_count = 0; f.combo_damage = 0.0; f.combo_target = None; }
            }
            f.sprite_remap.retain_mut(|(_, _, t)| { if *t > 0 { *t -= 1; } *t != 0 });
            let mut clear_transform = false;
            if let Some((_, _, _, t)) = f.sprite_transform.as_mut() { if *t > 0 { *t -= 1; clear_transform = *t == 0; } }
            if clear_transform { f.sprite_transform = None; }
        }
        // drop stale targets
        for k in 0..self.fighters.len() {
            if self.fighters[k].targets.is_empty() { continue; }
            let (st, mc) = (self.fighters[k].state_time, self.fighters[k].move_contact);
            let mut targets = std::mem::take(&mut self.fighters[k].targets);
            targets.retain(|(t, _)| self.fighters.iter().any(|f| f.id == *t && (f.move_type == 'H' || st < 2 || mc > 0)));
            self.fighters[k].targets = targets;
        }
    }

    fn collect_run_order(&self, ids: &mut Vec<u32>) {
        ids.extend(self.fighters.iter().map(|f| f.id));
        if self.fighters.iter().all(|f| f.asserts & (asf::RUNFIRST | asf::RUNLAST) == 0) { return; }
        let rank = |id: &u32| match self.fighters.iter().find(|f| f.id == *id) {
            Some(f) if f.has_assert(asf::RUNFIRST) && !f.has_assert(asf::RUNLAST) => 0,
            Some(f) if f.has_assert(asf::RUNLAST) && !f.has_assert(asf::RUNFIRST) => 2,
            _ => 1,
        };
        ids.sort_by_key(rank);
    }

    pub fn run_order_ids(&self) -> Vec<u32> {
        let mut ids = vec![];
        self.collect_run_order(&mut ids);
        ids
    }

    fn skip_asserted_announcements(&mut self) {
        let Some(kind) = self.announce.as_ref().map(|a| a.kind) else { return };
        let skipped = match kind {
            AnnounceKind::Round => gsf::SKIPROUNDDISPLAY,
            AnnounceKind::Fight => gsf::SKIPFIGHTDISPLAY,
            AnnounceKind::Ko => gsf::SKIPKODISPLAY,
            AnnounceKind::Win => gsf::SKIPWINDISPLAY,
            AnnounceKind::Other => 0,
        };
        if self.global_asserts & skipped != 0 { self.announce = None; }
    }

    /// Transformation ("awakening"): costs one super bar, once per round, lasts until the round ends — stronger hits, tougher, regenerating
    /// meter and a small life refill, with a cinematic pause. Native characters only (they carry the power-up animation).
    pub fn try_transform(&mut self, i: usize) -> bool {
        let f = &self.fighters[i];
        if f.is_helper || f.form > 0 || !f.alive || !f.ctrl || f.state_type == 'A' || f.power < 1000.0 || self.round_state != 2 || self.pause.is_some() || !f.def.anims.contains_key(&196) { return false; }
        let id = f.id;
        {
            let f = &mut self.fighters[i];
            f.power -= 1000.0; f.form = 1; f.form_time = 0; f.attack_mul *= 1.3; f.defence_mul *= 1.25;
            f.life = (f.life + f.life_max * 0.1).min(f.life_max);
        }
        self.change_engine_state(i, ENGINE_STATE_AWAKENING);
        self.fighters[i].ctrl = false;
        self.pause = Some(PauseState { by: id, time: 80, move_time: 80, is_super: true, darken: true, unhittable: true, p2_def_mul: 1.0, pause_bg: true, ..Default::default() });
        self.do_shake(40, 5.0);
        self.events.push(Event::Transform { fighter: id });
        true
    }

    pub fn fighter_pal(&self, i: usize) -> crate::palette::PalParams {
        let own = self.palfx_of(i).map(|p| p.params()).unwrap_or_default();
        match &self.all_palfx { Some(a) => own.combine(a.params()), None => own }
    }
    pub fn stage_pal(&self) -> crate::palette::PalParams {
        let bg = self.bg_palfx.as_ref().map(|p| p.params()).unwrap_or_default();
        match &self.all_palfx { Some(a) => bg.combine(a.params()), None => bg }
    }
    fn visual_timers(&mut self) {
        for p in [&mut self.all_palfx, &mut self.bg_palfx] { if let Some(fx) = p.as_mut() { fx.elapsed += 1; } }
        if self.shake_time > 0 { self.shake_time -= 1; self.shake_elapsed += 1; }
        if let Some((t, _)) = self.flash.as_mut() { *t -= 1; if *t <= 0 { self.flash = None; } }
        if let Some(a) = self.announce.as_mut() { a.time -= 1; if a.time <= 0 { self.announce = None; } }
    }

    /// Safety net for scripts that never hand control back: a fighter lying down past `data.liedown.time` (the M.U.G.E.N. engine
    /// stands it up itself, many characters' own common1.cns has no exit from 5110) or without control for 15 s goes back to a
    /// controllable state instead of freezing the character for the rest of the match.
    fn rescue_stuck(&mut self) {
        const NO_CTRL_LIMIT: i32 = 400;
        for i in 0..self.fighters.len() {
            let free = { let f = &self.fighters[i]; f.alive && !f.is_helper && f.round_free(self.round_state == 2, self.pause.is_some()) };
            let f = &mut self.fighters[i];
            f.no_ctrl_ticks = if free && !f.ctrl { f.no_ctrl_ticks + 1 } else { 0 };
            let lying = free && f.state_no == 5110 && f.state_time > f.def.c("data.liedown.time", 60.0) as i32 + 30;
            if !lying && f.no_ctrl_ticks <= NO_CTRL_LIMIT { continue; }
            f.no_ctrl_ticks = 0;
            let def = f.def.clone();
            let getup = lying && (def.states.contains_key(&5120) || self.common.states.contains_key(&5120));
            if !getup { let f = &mut self.fighters[i]; f.vel = V2::default(); f.ctrl = true; }
            self.change_state(i, if getup { 5120 } else { 0 }, def);
            if !getup { self.fighters[i].ctrl = true; }
        }
    }

    fn push_and_bound(&mut self) {
        let n = self.fighters.len();
        for a in 0..n {
            for b in a + 1..n {
                let (fa, fb) = (&self.fighters[a], &self.fighters[b]);
                let pushable = |f: &Fighter| !f.destroyed && (!f.is_helper || f.helper_player) && f.player_push && f.state_type != 'L';
                if fa.side == fb.side || !pushable(fa) || !pushable(fb) { continue; }
                if fa.pos.y - fa.height_top() > fb.pos.y + fb.height_bottom() || fb.pos.y - fb.height_top() > fa.pos.y + fa.height_bottom() { continue; }
                let (l, r) = if fa.pos.x <= fb.pos.x { (a, b) } else { (b, a) };
                let (fl, fr) = (&self.fighters[l], &self.fighters[r]);
                let lw = if fl.facing > 0.0 { fl.front_width() } else { fl.back_width() };
                let rw = if fr.facing < 0.0 { fr.front_width() } else { fr.back_width() };
                let overlap = fl.pos.x + lw - (fr.pos.x - rw);
                if overlap > 0.0 {
                    let (mut lp, mut rp) = (overlap / 2.0, overlap / 2.0);
                    if self.classic_pushbox {
                        if fl.state_type == 'A' && fr.state_type != 'A' { lp = overlap; rp = 0.0; } else if fr.state_type == 'A' && fl.state_type != 'A' { rp = overlap; lp = 0.0; }
                    }
                    match (fl.has_assert(asf::IMMOVABLE), fr.has_assert(asf::IMMOVABLE)) {
                        (true, false) => { lp = 0.0; rp = overlap; }
                        (false, true) => { lp = overlap; rp = 0.0; }
                        (true, true) => { lp = 0.0; rp = 0.0; }
                        (false, false) => {}
                    }
                    self.fighters[l].pos.x -= lp; self.fighters[r].pos.x += rp;
                }
            }
        }
        let left = self.camera.x - self.view_w / 2.0; let right = self.camera.x + self.view_w / 2.0;
        let (bl, br, vw) = (self.stage.bound_left, self.stage.bound_right, self.view_w);
        for f in self.fighters.iter_mut() {
            if f.destroyed || (f.is_helper && !f.helper_player) { continue; }
            if !f.screen_bound { f.screen_bound = true; continue; }
            let lw = if f.facing > 0.0 { f.back_width() } else { f.front_width() };
            let rw = if f.facing > 0.0 { f.front_width() } else { f.back_width() };
            let minx = left.max(bl - vw / 2.0) + lw; let maxx = right.min(br + vw / 2.0) - rw;
            f.pos.x = f.pos.x.clamp(minx, maxx.max(minx));
        }
    }

    pub fn update_camera(&mut self) {
        let (mut minx, mut maxx, mut highest, mut any) = (f32::MAX, f32::MIN, f32::MAX, false);
        let (mut lo, mut hi, mut live) = (f32::MAX, f32::MIN, false);
        for f in self.roots() {
            if f.destroyed { continue; }
            if !f.ikemen.standby {
                minx = minx.min(f.pos.x);
                maxx = maxx.max(f.pos.x);
                highest = highest.min(f.pos.y);
                any = true;
            }
            if f.alive {
                lo = lo.min(f.pos.x);
                hi = hi.max(f.pos.x);
                live = true;
            }
        }
        if !any { return; }
        let natural = if self.zoom_max > 1.0 {
            let need = if !live { self.zoom_min } else { (hi - lo + 150.0) / BASE_W };
            need.clamp(self.zoom_min, self.zoom_max)
        } else { self.zoom_min };
        let (offset, bound) = self.step_zoom(natural);
        let half = self.view_w / 2.0; let t = self.stage.tension.clamp(0.0, half);
        let (lo, hi) = ((maxx - half + t).max(self.stage.bound_left), (minx + half - t).min(self.stage.bound_right));
        let held = if lo <= hi { self.camera.x.clamp(lo, hi) } else { self.camera.x };
        let target = if bound { (held + offset.x).clamp(self.stage.bound_left, self.stage.bound_right) } else { held.clamp(self.stage.bound_left, self.stage.bound_right) + offset.x };
        self.camera.x += (target - self.camera.x) * 0.5;
        let cam = &self.stage.camera;
        let top = self.effective_bound_high();
        let lowest = if cam.lowest_cap { self.stage.start_y.min(0.0) } else { 0.0 };
        let follow = (highest * self.stage.vertical_follow).min(lowest);
        let ty = if bound { (follow + offset.y).clamp(top, self.stage.bound_low.max(top)) } else { follow.clamp(top, self.stage.bound_low.max(top)) + offset.y };
        if (ty - self.camera_target_y).abs() >= cam.vertical_follow_delta { self.camera_target_y = ty; }
        self.camera.y += (self.camera_target_y - self.camera.y) * cam.y_scroll_speed.clamp(0.0, 1.0);
    }

    // ---------------------------------------------------------------- hits
    pub fn overlap(a: &[Box4], b: &[Box4]) -> Option<(f32, f32)> {
        if a.is_empty() || b.is_empty() {
            return None;
        }
        if a.len() > 1 || b.len() > 1 {
            let (mut ax0, mut ay0, mut ax2, mut ay2) = (a[0][0], a[0][1], a[0][2], a[0][3]);
            for x in &a[1..] {
                ax0 = ax0.min(x[0]);
                ay0 = ay0.min(x[1]);
                ax2 = ax2.max(x[2]);
                ay2 = ay2.max(x[3]);
            }
            let (mut bx0, mut by0, mut bx2, mut by2) = (b[0][0], b[0][1], b[0][2], b[0][3]);
            for y in &b[1..] {
                bx0 = bx0.min(y[0]);
                by0 = by0.min(y[1]);
                bx2 = bx2.max(y[2]);
                by2 = by2.max(y[3]);
            }
            if ax0 > bx2 || ax2 < bx0 || ay0 > by2 || ay2 < by0 {
                return None;
            }
        }
        for x in a {
            let (x0, y0, x2, y2) = (x[0], x[1], x[2], x[3]);
            for y in b {
                if x0 <= y[2] && x2 >= y[0] && y0 <= y[3] && y2 >= y[1] {
                    return Some(((x0.max(y[0]) + x2.min(y[2])) * 0.5, (y0.max(y[1]) + y2.min(y[3])) * 0.5));
                }
            }
        }
        None
    }
    fn attr_match(hd: &HitDefData, spec: &str) -> bool {
        let mut it = spec.split(',');
        let s = it.next().unwrap_or("");
        let my = hd.attr_state.chars().find(|c| "SCA".contains(*c)).unwrap_or('S');
        if !s.chars().any(|c| c.to_ascii_uppercase() == my) { return false; }
        let mut has_kinds = false;
        for raw in it {
            let k = raw.trim();
            if k.is_empty() { continue; }
            has_kinds = true;
            let kb = k.as_bytes();
            if hd.attr_kind.iter().any(|ak| ak.eq_ignore_ascii_case(k) || (kb.len() == 2 && kb[0].eq_ignore_ascii_case(&b'A') && ak.as_bytes().last().is_some_and(|l| l.eq_ignore_ascii_case(&kb[1])))) { return true; }
        }
        !has_kinds
    }
    pub fn can_be_hit(&self, d: usize, hd: &HitDefData, side: usize) -> bool {
        let f = &self.fighters[d];
        let team_ok = match hd.affect_team { 'B' => true, 'F' => f.side == side, _ => f.side != side };
        if f.destroyed || !team_ok || !f.alive || f.ikemen.standby || f.helper_projectile { return false; }
        if f.move_type == 'H' {
            if hd.chain_id.is_some_and(|c| f.ghv.hit_id != c) { return false; }
            if f.ghv.hit_id >= 0 && hd.no_chain_id.contains(&Some(f.ghv.hit_id)) { return false; }
        }
        if self.round_state != 2 && !(self.round_state == 3 && f.alive) { return false; }
        if f.state_no == 5150 || f.unhittable > 0 { return false; }
        let hf = &hd.hitflag; let st = f.state_type;
        if hf.contains('+') && f.move_type != 'H' { return false; }
        if hf.contains('-') && f.move_type == 'H' { return false; }
        let falling = f.move_type == 'H' && f.ghv.fall && st == 'A';
        let hf_falling = hf.contains('F') && !f.has_assert(asf::NOFALLHITFLAG);
        // air juggle budget: a falling, airborne target can only absorb `airjuggle` points of moves in a row
        if falling && f.juggle_used + hd.juggle > hd.airjuggle { return false; }
        if st == 'S' && !(hf.contains('H') || hf.contains('M')) { return false; }
        if st == 'C' && !(hf.contains('L') || hf.contains('M')) { return false; }
        if st == 'A' && !hf.contains('A') && !(falling && hf_falling) { return false; }
        if falling && !hf_falling { return false; }
        if st == 'L' && !hf.contains('D') { return false; }
        if f.not_hit_by.iter().any(|(a, _)| Self::attr_match(hd, a)) { return false; }
        if f.hit_by.iter().any(|(a, _)| !Self::attr_match(hd, a)) { return false; }
        true
    }
    fn is_guarding(&self, d: usize, hd: &HitDefData) -> bool {
        let f = &self.fighters[d];
        if hd.guardflag.is_empty() { return false; }
        let gf = &hd.guardflag; let st = f.state_type;
        let blocked = match st { 'S' => asf::NOSTANDGUARD, 'C' => asf::NOCROUCHGUARD, 'A' => asf::NOAIRGUARD, _ => 0 };
        if f.has_assert(blocked) { return false; }
        let ok = (st == 'S' && (gf.contains('H') || gf.contains('M'))) || (st == 'C' && (gf.contains('L') || gf.contains('M'))) || (st == 'A' && gf.contains('A'));
        if !ok { return false; }
        if (120..=155).contains(&f.state_no) { return true; }
        f.ctrl && f.move_type != 'H' && st != 'A' && !f.is_helper && (self.has_command(d, "holdback") || f.has_assert(asf::AUTOGUARD))
    }

    fn detect_reversals(&mut self) {
        if !self.fighters.iter().any(|f| f.reversal.is_some()) { return; }
        let mut found = [(0usize, 0usize, (0.0f32, 0.0f32)); 4];
        let mut found_len = 0;
        for r in 0..self.fighters.len() {
            let fr = &self.fighters[r];
            let Some((attr, _)) = fr.reversal.as_ref() else { continue };
            if fr.destroyed { continue; }
            fr.fill_boxes(1, &mut self.scratch_c1);
            if self.scratch_c1.is_empty() { continue; }
            for a in 0..self.fighters.len() {
                let fa = &self.fighters[a];
                if a == r || fa.side == fr.side || fa.destroyed || !fa.hitdef_active || fa.move_type != 'A' { continue; }
                let Some(hd) = fa.hitdef.as_ref() else { continue };
                if !Self::attr_match(hd, attr) { continue; }
                fa.fill_boxes(1, &mut self.scratch_c2);
                if let Some(at) = Self::overlap(&self.scratch_c1, &self.scratch_c2) {
                    if found_len < found.len() {
                        found[found_len] = (r, a, at);
                        found_len += 1;
                    }
                    break;
                }
            }
        }
        for &(r, a, at) in &found[..found_len] {
            let Some((_, rd)) = self.fighters[r].reversal.take() else { continue };
            let aid = self.fighters[a].id;
            {
                let fr = &mut self.fighters[r];
                fr.hitdef_active = false;
                if !fr.targets.iter().any(|t| t.0 == aid) { fr.targets.push((aid, rd.id)); }
            }
            self.spawn_spark(r, rd.spark_no, rd.spark_own, at.0 + rd.spark_x * self.fighters[r].facing, at.1 + rd.spark_y, false, false, 1.0);
            if let Some(snd) = rd.hit_sound { self.sound(r, snd.0, snd.1, snd.2, Some("hit")); }
            if rd.p2stateno >= 0 { let owner = if rd.p2getp1state { self.fighters[r].def.clone() } else { self.fighters[a].def.clone() }; self.change_state(a, rd.p2stateno, owner); }
            if rd.p1stateno >= 0 { let d = self.fighters[r].def.clone(); self.change_state(r, rd.p1stateno, d); }
            { let fr = &mut self.fighters[r]; fr.move_contact = 1; fr.move_hit = 1; fr.hit_pause = rd.pause_p1; fr.spr_priority = rd.p1sprpriority; }
            let fa = &mut self.fighters[a];
            fa.hitdef_active = false; fa.move_reversed = 1; fa.hit_pause = rd.pause_p2; fa.spr_priority = rd.p2sprpriority;
        }
    }

    fn detect_hits(&mut self) {
        self.detect_reversals();
        let mut pending = [(0usize, 0usize, (0.0f32, 0.0f32)); 8];
        let mut pending_len = 0;
        for a in 0..self.fighters.len() {
            let fa = &self.fighters[a];
            if fa.destroyed || fa.ikemen.standby || !fa.hitdef_active || (fa.move_type != 'A' && !fa.is_helper) { continue; }
            let Some(hd) = fa.hitdef.as_ref() else { continue };
            let Some(fr) = fa.frame() else { continue };
            if fr.clsn1.is_empty() { continue; }
            fa.fill_boxes(1, &mut self.scratch_c1);
            if self.scratch_c1.is_empty() { continue; }
            for d in 0..self.fighters.len() {
                let fd = &self.fighters[d];
                if d == a || fd.root == fa.root || fa.hitdef_targets.contains(&fd.id) { continue; }
                if hd.hit_once && !fa.hitdef_targets.is_empty() { continue; }
                if hd.p2clsncheck && fd.frame().is_none_or(|f| f.clsn2.is_empty()) { continue; }
                if !self.can_be_hit(d, hd, fa.side) { continue; }
                fd.fill_boxes(2, &mut self.scratch_c2);
                let targets: &[Box4] = if hd.p2clsnrequire >= 0 {
                    let req = hd.p2clsnrequire as usize;
                    if req < self.scratch_c2.len() { &self.scratch_c2[req..req + 1] } else { &[] }
                } else {
                    &self.scratch_c2
                };
                if targets.is_empty() { continue; }
                if let Some(at) = Self::overlap(&self.scratch_c1, targets) {
                    if pending_len < pending.len() {
                        pending[pending_len] = (a, d, at);
                        pending_len += 1;
                    }
                }
            }
        }
        let mut struck = [0usize; 8];
        let mut struck_len = 0;
        for &(a, d, at) in &pending[..pending_len] {
            let Some(hd) = self.fighters[a].hitdef.clone() else { continue };
            if hd.hit_once && struck[..struck_len].contains(&a) { continue; }
            if !self.fighters[a].hitdef_active { continue; }
            let did = self.fighters[d].id;
            self.fighters[a].hitdef_targets.push(did);
            if hd.hit_once { self.fighters[a].targets.retain(|t| t.0 == did); }
            let projectile_helper = self.fighters[a].helper_projectile;
            let contacts_before = self.fighters[a].proj_contacts.len();
            let as_projectile = projectile_helper.then(|| (self.fighters[a].helper_id, self.fighters[a].facing));
            self.apply_hit(a, d, &hd, at, as_projectile);
            if projectile_helper { self.share_projectile_helper_contacts(a, contacts_before); }
            if struck_len < struck.len() {
                struck[struck_len] = a;
                struck_len += 1;
            }
        }
        for &a in &struck[..struck_len] { if self.fighters.get(a).is_some_and(|f| f.def.source == Source::Mugen) { self.fighters[a].hitdef_active = false; } }
    }

    fn share_projectile_helper_contacts(&mut self, a: usize, from: usize) {
        let gt = self.game_time;
        let (id, root) = (self.fighters[a].helper_id, self.fighters[a].root);
        self.fighters[a].proj_contacts.push(('c', id, gt));
        let fresh: Vec<_> = self.fighters[a].proj_contacts[from..].to_vec();
        if let Some(r) = self.idx(root) { if r != a { self.fighters[r].proj_contacts.extend(fresh); } }
    }

    /// Is fighter `i` standing against the edge of the playable area (screen / stage bound)?
    pub fn at_corner(&self, i: usize) -> bool {
        let f = &self.fighters[i];
        let left = self.camera.x - self.view_w / 2.0; let right = self.camera.x + self.view_w / 2.0;
        let (bl, br, vw) = (self.stage.bound_left, self.stage.bound_right, self.view_w);
        let lw = if f.facing > 0.0 { f.back_width() } else { f.front_width() };
        let rw = if f.facing > 0.0 { f.front_width() } else { f.back_width() };
        let (minx, maxx) = (left.max(bl - vw / 2.0) + lw, right.min(br + vw / 2.0) - rw);
        f.pos.x <= minx + 2.0 || f.pos.x >= maxx - 2.0
    }

    fn hit_override(&self, d: usize, hd: &HitDefData) -> Option<(i32, bool)> {
        self.fighters[d].hit_overrides.iter().flatten().find(|o| o.time != 0 && Self::attr_match(hd, &o.attr)).map(|o| (o.state_no, o.force_air))
    }

    pub fn apply_hit(&mut self, a: usize, d: usize, hd: &HitDefData, at: (f32, f32), proj: Option<(i32, f32)>) {
        let guarded = !self.fighters[a].has_assert(asf::UNGUARDABLE) && self.is_guarding(d, hd);
        let overridden = if guarded { None } else { self.hit_override(d, hd) };
        let from_proj = proj.is_some();
        let a_facing = proj.map(|p| p.1).unwrap_or(self.fighters[a].facing);
        let (aid, aside, apos, aroot) = { let f = &self.fighters[a]; (f.id, f.side, f.pos, f.root) };
        let did = self.fighters[d].id;
        let ano = self.fighters[a].player_no;
        let d_attacking = self.fighters[d].move_type == 'A';
        let mut corner_pushed = 0.0f32;
        if !from_proj {
            let fa = &mut self.fighters[a];
            fa.move_contact = 1;
            if !fa.targets.iter().any(|t| t.0 == did) { fa.targets.push((did, hd.id)); }
        }
        let attack_mul = self.idx(aroot).map(|r| self.fighters[r].attack_mul).unwrap_or(1.0);
        {
            let fd = &mut self.fighters[d];
            fd.last_hit_by = Some(aid);
            if fd.bind.as_ref().is_some_and(|b| b.kind == BindKind::Target) { fd.bind = None; }
            if !from_proj && hd.p2facing == 0 && fd.state_type != 'A' { fd.facing = if apos.x >= fd.pos.x { 1.0 } else { -1.0 }; }
            else if hd.p2facing == 1 { fd.facing = -a_facing; } else if hd.p2facing == -1 { fd.facing = a_facing; }
            fd.hitdef_active = false;
        }
        let dir = if self.fighters[d].facing == -a_facing { 1.0 } else { -1.0 };
        let gt = self.game_time;
        let cornered = !from_proj && self.at_corner(d);
        let d_air = self.fighters[d].state_type == 'A' || self.fighters[d].air_height() < -1.0;
        if let Some((state_no, force_air)) = overridden {
            {
                let fd = &mut self.fighters[d];
                fd.ghv = GetHitVars { attacker_id: aid, hit_id: hd.id, chainid: hd.chain_id.unwrap_or(-1), hittime: hd.ground_hit_time, ..GetHitVars::default() };
                fd.ghv.record_hitdef(hd, false, (aid, aside, ano), a_facing, proj.map(|p| p.0));
                fd.hit_shake = hd.pause_p2; fd.move_type = 'H';
                if force_air { fd.state_type = 'A'; fd.ghv.fall = true; }
            }
            if let Some((pid, _)) = proj { self.fighters[a].proj_contacts.push(('h', pid, gt)); } else { let fa = &mut self.fighters[a]; fa.move_hit = 1; fa.hit_pause = hd.pause_p1; }
            self.fighters[d].hit_overridden_at = gt;
            if !from_proj { self.record_move_hit(a, d, at, true, 0.0); }
            let dd = self.fighters[d].def.clone();
            self.change_state(d, state_no, dd);
            if hd.miss_on_override {
                let f = &self.fighters[a];
                let pos = V2 { x: f.pos.x, y: f.pos.y - 92.0 * f.lscl };
                self.texts.push(crate::ikemen::TextItem { owner: f.id, id: -1, text: "MISS".into(), pos, vel: V2 { x: 0.0, y: -1.0 }, scale: [1.0, 1.0], color: [255, 220, 80], align: 0, time: 30 });
            }
            self.events.push(Event::Hit { attacker_side: aside, defender_side: self.fighters[d].side, damage: 0.0, guarded: false, heavy: false, attacker: self.idx(aroot).unwrap_or(a), defender: d });
            return;
        }
        if guarded {
            let training = self.training;
            let dmg = if self.fighters[d].has_assert(asf::NOGUARDDAMAGE) { 0.0 } else { (hd.guard_damage * attack_mul).round() };
            let pause_p1 = hd.guard_pause_p1;
            {
                let fd = &mut self.fighters[d];
                fd.life = if hd.guard_kill && !training && !fd.has_assert(asf::NOGUARDKO) { (fd.life - dmg).max(0.0) } else { (fd.life - dmg).max(fd.life.min(1.0)) };
                fd.received_damage += dmg; fd.received_hits += 1;
                let gc = fd.ghv.guardcount;
                fd.ghv = GetHitVars::default();
                let g = &mut fd.ghv;
                g.guarded = true; g.hittime = hd.guard_hit_time; g.slidetime = hd.guard_slide_time; g.ctrltime = hd.guard_ctrl_time;
                g.xvel = (if fd.state_type == 'A' { hd.air_guard_vel[0] } else { hd.guard_vel }) * dir;
                g.yvel = if fd.state_type == 'A' { hd.air_guard_vel[1] } else { 0.0 };
                g.yaccel = hd.yaccel; g.damage = dmg; g.guardcount = gc + 1;
                g.record_hitdef(hd, true, (aid, aside, ano), a_facing, proj.map(|p| p.0));
                fd.hit_shake = hd.guard_pause_p2; fd.hit_over_time = hd.guard_hit_time; fd.slide_time = hd.guard_slide_time;
                fd.ctrl = false; fd.move_type = 'H';
            }
            if let Some((pid, _)) = proj { self.fighters[a].proj_contacts.push(('g', pid, gt)); } else { self.fighters[a].move_guarded = 1; self.fighters[a].hit_pause = pause_p1; }
            let st = if self.fighters[d].air_height() < -1.0 { 'A' } else { self.fighters[d].state_type };
            let dd = self.fighters[d].def.clone();
            self.change_state(d, if st == 'A' { 154 } else if st == 'C' { 152 } else { 150 }, dd);
            let point_mul = attack_mul / (self.fighters[d].defence_mul * self.fighters[d].super_defence_mul).max(0.01);
            crate::ikemen::life::apply_hit_points(self, d, hd, true, point_mul);
            crate::ikemen::life::enter_stun_states(self, d, true);
            self.add_power(a, hd.guard_get_power); self.add_power(d, hd.guard_give_power);
            self.spawn_spark(a, hd.guard_spark_no, hd.guard_spark_own, at.0, at.1, true, false, hd.spark_scale);
            if let Some(s) = hd.guard_sound { self.sound(a, s.0, s.1, s.2, Some("guard")); }
            if cornered && self.fighters[a].def.source == Source::Mugen && !self.fighters[a].has_assert(asf::NOCORNERPUSH) { corner_pushed = if d_air { hd.air_guard_corner_push } else { hd.guard_corner_push }; self.fighters[a].vel.x = corner_pushed; }
        } else {
            let training = self.training;
            let was_hit = self.fighters[d].move_type == 'H';
            let prev_hc = if was_hit { self.fighters[d].ghv.hitcount } else { 0 };
            let scale = if self.fighters[a].def.source == crate::chardef::Source::Native { combo_scale(prev_hc) } else { 1.0 };
            let mut dmg = (hd.damage * attack_mul * scale / (self.fighters[d].defence_mul * self.fighters[d].super_defence_mul).max(0.01)).round();
            if hd.damage > 0.0 { dmg = dmg.max(1.0); }
            if self.fighters[d].has_assert(asf::NOHITDAMAGE) { dmg = 0.0; }
            let prev_fall = self.fighters[d].ghv.fall;
            let ko_global_off = self.global_asserts & gsf::GLOBALNOKO != 0;
            if was_hit && prev_fall && (self.fighters[d].state_type == 'A' || self.fighters[d].air_height() < -1.0) { self.fighters[d].juggle_used += hd.juggle; }
            {
                let fd = &mut self.fighters[d];
                let mut nl = fd.life - dmg;
                if (!hd.kill || training) && nl < 1.0 { nl = fd.life.min(1.0); }
                fd.life = nl.max(0.0);
                fd.received_damage += dmg; fd.received_hits += 1;
                if hd.unhittable_time > 0 { fd.unhittable = fd.unhittable.max(hd.unhittable_time); }
                let air = fd.state_type == 'A' || fd.air_height() < -1.0;
                let down = fd.state_type == 'L';
                fd.ghv = GetHitVars::default();
                let g = &mut fd.ghv;
                g.attacker_id = aid; g.damage = dmg; g.hitcount = prev_hc + 1;
                g.animtype = if air { hd.air_animtype } else { hd.animtype };
                g.groundtype = hd.ground_type; g.airtype = hd.air_type; g.kind = if air { 3 } else { hd.ground_type };
                g.fall = (if air { hd.air_fall } else { hd.fall }) || (was_hit && prev_fall);
                g.hittime = if down { hd.down_hit_time } else if air { hd.air_hit_time } else { hd.ground_hit_time };
                g.slidetime = hd.ground_slide_time; g.ctrltime = hd.ground_hit_time;
                let v = if down { hd.down_vel } else if air { hd.air_vel } else { hd.ground_vel };
                g.hit_id = hd.id; g.fall_env_shake = hd.fall_env_shake; g.down_bounce = hd.down_bounce;
                g.xvel = v[0] * dir; g.yvel = v[1]; g.yaccel = hd.yaccel; g.xaccel = hd.xaccel * dir;
                g.fall_xvel = hd.fall_xvel.map(|x| x * dir).unwrap_or(g.xvel); g.fall_yvel = hd.fall_yvel;
                g.fall_recover = hd.fall_recover; g.fall_recover_time = hd.fall_recover_time; g.fall_damage = hd.fall_damage; g.fall_kill = hd.fall_kill; g.kill = hd.kill;
                g.chainid = hd.chain_id.unwrap_or(-1);
                g.record_hitdef(hd, false, (aid, aside, ano), a_facing, proj.map(|p| p.0));
                let ko_disabled = ko_global_off || fd.asserts & asf::NOKO != 0;
                if fd.life <= 0.0 && !ko_disabled {
                    if fd.asserts & asf::NOKOFALL == 0 { g.fall = true; }
                    if g.yvel == 0.0 && fd.asserts & asf::NOKOVELOCITY == 0 { g.yvel = -5.0 * fd.lscl; g.xvel = g.xvel.min(-2.5 * fd.lscl); }
                    fd.alive = false;
                }
                fd.hit_shake = hd.pause_p2; fd.hit_over_time = fd.ghv.hittime; fd.slide_time = fd.ghv.slidetime; fd.fall_time = 0;
                fd.ctrl = false; fd.move_type = 'H'; fd.spr_priority = hd.p2sprpriority;
            }
            if let Some((pid, _)) = proj { self.fighters[a].proj_contacts.push(('h', pid, gt)); }
            else { let fa = &mut self.fighters[a]; fa.move_hit = 1; fa.move_countered = d_attacking as i32; fa.hit_count += hd.num_hits; if !fa.uniq_hit_keys.contains(&(did, hd.id)) { fa.uniq_hit_keys.push((did, hd.id)); fa.uniq_hit_count += 1; } fa.hit_pause = hd.pause_p1; fa.spr_priority = hd.p1sprpriority; }
            let st = if self.fighters[d].air_height() < -1.0 && self.fighters[d].state_type != 'L' { 'A' } else { self.fighters[d].state_type };
            if hd.p2stateno >= 0 {
                let owner = if hd.p2getp1state { self.fighters[a].def.clone() } else { self.fighters[d].def.clone() };
                let giver = if hd.p2getp1state { Some(self.fighters[a].id) } else { None };
                self.change_state(d, hd.p2stateno, owner);
                if giver.is_some() { self.fighters[d].state_giver = giver; }
            } else {
                let dd = self.fighters[d].def.clone();
                let no = if st == 'A' || st == 'L' { 5020 } else if st == 'C' && !hd.forcestand { 5010 } else { 5000 };
                self.change_state(d, no, dd);
            }
            let point_mul = attack_mul * scale / (self.fighters[d].defence_mul * self.fighters[d].super_defence_mul).max(0.01);
            crate::ikemen::life::apply_hit_points(self, d, hd, false, point_mul);
            if hd.p2stateno < 0 { crate::ikemen::life::enter_stun_states(self, d, false); }
            if hd.p1stateno >= 0 && !from_proj { let ad = self.fighters[a].def.clone(); self.change_state(a, hd.p1stateno, ad); }
            self.add_power(a, hd.get_power); self.add_power(d, hd.give_power);
            if self.first_attack.is_none() { self.first_attack = Some(aside); }
            if let Some(r) = self.idx(aroot) {
                let did = self.fighters[d].id;
                let fr = &mut self.fighters[r];
                let chained = was_hit && fr.combo_target == Some(did) && fr.combo_timer > 0;
                fr.combo_count = prev_hc + 1;
                fr.combo_damage = if chained { fr.combo_damage + dmg } else { dmg };
                fr.combo_target = Some(did);
                fr.combo_timer = COMBO_WINDOW;
                if fr.asserts & asf::NOSCORE == 0 {
                    fr.score += hd.score[0] as f64;
                    fr.score_total += hd.score[0] as f64;
                }
            }
            if self.fighters[d].asserts & asf::NOSCORE == 0 {
                self.fighters[d].score += hd.score[1] as f64;
                self.fighters[d].score_total += hd.score[1] as f64;
            }
            let heavy = hd.animtype >= 2 || dmg >= 70.0;
            self.spawn_spark(a, hd.spark_no, hd.spark_own, at.0, at.1, false, heavy, hd.spark_scale);
            if let Some(s) = hd.hit_sound { self.sound(a, s.0, s.1, s.2, Some(if heavy { "hitheavy" } else { "hit" })); }
            if hd.env_shake.time > 0 { self.shake_with_dir(hd.env_shake.time, hd.env_shake.freq, hd.env_shake.ampl, hd.env_shake.phase, hd.env_shake.dir); }
            if let Some(p) = hd.target_palfx.clone() { self.fighters[d].palfx = Some(p); }
            if cornered && self.fighters[a].def.source == Source::Mugen && !self.fighters[a].has_assert(asf::NOCORNERPUSH) { corner_pushed = if d_air { hd.air_corner_push } else { hd.ground_corner_push }; self.fighters[a].vel.x = corner_pushed; }
            self.place_target(a, d, hd);
            let droot = self.fighters[d].root;
            self.explods.retain(|e| !(e.remove_on_get_hit && self.fighters.iter().any(|f| f.id == e.owner && f.root == droot)));
        }
        // cornered defenders push the attacker back instead of absorbing all the knock-back (SF/Tekken/GG behaviour)
        if !from_proj && self.fighters[a].def.source == Source::Native && self.fighters[a].state_type != 'A' && !self.fighters[a].has_assert(asf::NOCORNERPUSH) && self.at_corner(d) {
            let push = corner_push(self.fighters[d].ghv.xvel);
            let away = if apos.x <= self.fighters[d].pos.x { -1.0 } else { 1.0 };
            self.fighters[a].pos.x += away * push;
            self.corner_pushes += 1;
        }
        if !from_proj { self.record_move_hit(a, d, at, false, corner_pushed); }
        let dmg = self.fighters[d].ghv.damage;
        self.events.push(Event::Hit { attacker_side: aside, defender_side: self.fighters[d].side, damage: dmg, guarded, heavy: hd.animtype >= 2, attacker: self.idx(aroot).unwrap_or(a), defender: d });
    }

    fn record_move_hit(&mut self, a: usize, d: usize, at: (f32, f32), overridden: bool, corner_push: f32) {
        let (did, dno) = (self.fighters[d].id as i32, self.fighters[d].player_no as i32);
        let hit_time = self.game_time;
        let fa = &mut self.fighters[a];
        let l = fa.lscl;
        fa.move_hit_var = crate::fighter::MoveHitVars { target_id: did, target_no: dno, spark_x: (at.0 - fa.pos.x) * fa.facing / l, spark_y: (at.1 - fa.pos.y) / l, corner_push: corner_push / l, hit_time, overridden };
    }

    fn place_target(&mut self, a: usize, d: usize, hd: &HitDefData) {
        let (ax, ay, af) = { let f = &self.fighters[a]; (f.pos.x, f.pos.y, f.facing) };
        let fd = &mut self.fighters[d];
        if let Some(s) = hd.snap { fd.pos.x = ax + s.x * af; fd.pos.y = ay + s.y; return; }
        let mut rel = [(fd.pos.x - ax) * af, fd.pos.y - ay];
        for k in 0..2 {
            if let Some(lo) = hd.min_dist[k] { rel[k] = rel[k].max(lo); }
            if let Some(hi) = hd.max_dist[k] { rel[k] = rel[k].min(hi); }
        }
        fd.pos.x = ax + rel[0] * af;
        if hd.min_dist[1].is_some() || hd.max_dist[1].is_some() { fd.pos.y = ay + rel[1]; }
    }

    // ---------------------------------------------------------------- effects
    fn sound(&mut self, i: usize, g: i32, n: i32, own: bool, builtin: Option<&'static str>) {
        let pan = crate::sctrl::sound_pan(self, i, 0.0, false);
        let f = &self.fighters[i];
        self.events.push(Event::Sound { def: if own { Some(f.def.clone()) } else { None }, group: g, index: n, own, volume: 1.0, channel: -1, owner: f.id, builtin, pan, looping: false, freq_mul: 1.0, low_priority: false, stop_on_get_hit: false, stop_on_change_state: false, loop_start: 0, loop_end: 0, loop_count: 0 });
    }
    pub fn do_shake(&mut self, t: i32, ampl: f32) { self.shake(t, 60.0, -ampl.abs(), 0.0); }
    pub fn shake(&mut self, t: i32, freq: f32, ampl: f32, phase: f32) {
        if t <= 0 { return; }
        self.shake_time = t; self.shake_elapsed = 0; self.shake_freq = freq; self.shake_phase = phase;
        self.shake_ampl = if ampl != 0.0 { ampl } else { -4.0 };
    }
    pub fn shake_with_dir(&mut self, t: i32, freq: f32, ampl: f32, phase: f32, dir: u8) {
        self.shake_dir = dir.min(2);
        self.shake(t, freq, ampl, phase);
    }
    /// `dir`: 0 = both axes (default), 1 = horizontal only, 2 = vertical only.
    pub fn shake_horizontal(&self) -> bool { self.shake_dir != 2 }
    pub fn shake_vertical(&self) -> bool { self.shake_dir != 1 }
    pub fn shake_offset(&self) -> f32 {
        if self.shake_time <= 0 { return 0.0; }
        let decay = 1.0 - self.shake_elapsed as f32 / (self.shake_elapsed + self.shake_time) as f32;
        self.shake_ampl * decay * (self.shake_phase + self.shake_freq * self.shake_elapsed as f32).to_radians().sin()
    }

    fn push_explod(&mut self, e: Explod) {
        if self.explods.len() < MAX_EXPLODS {
            self.explods.push(e);
        }
    }

    fn make_explod(&self, owner: &Fighter, def: Arc<CharDef>, anim: i32, x: f32, y: f32, lscl: f32) -> Explod {
        Explod { owner: owner.id, id: -1, cur: AnimCursor::new(def, anim), pos: V2 { x, y }, vel: V2::default(), accel: V2::default(), facing: owner.facing, vfacing: 1.0,
            bind_time: 0, bind_to: None, bind_off: V2::default(), remove_time: -2, time: 0, spr_priority: 5, on_top: true, under: false, scale: [1.0, 1.0],
            add: false, remove_on_get_hit: false, super_move: 0, pause_move: 0, lscl, ignore_hit_pause: true, screen: false }
    }
    pub fn spawn_spark(&mut self, a: usize, no: i32, own: bool, x: f32, y: f32, guard: bool, heavy: bool, scale: f32) {
        let f = &self.fighters[a];
        if own && f.def.anims.contains_key(&no) {
            let mut e = self.make_explod(f, f.def.clone(), no, x, y, f.lscl);
            e.scale = [scale, scale];
            self.push_explod(e);
            return;
        }
        let Some(fx) = self.fx.clone() else { return };
        let mut n = if guard { 40 } else if heavy { 2 } else { no };
        if !fx.anims.contains_key(&n) { n = if guard { 40 } else if heavy { 2 } else { 0 }; }
        if !fx.anims.contains_key(&n) { return; }
        let mut e = self.make_explod(f, fx, n, x, y, 1.0);
        e.scale = [scale, scale];
        self.push_explod(e);
    }
    pub fn spawn_fx(&mut self, i: usize, no: i32, x: f32, y: f32, under: bool) {
        let Some(fx) = self.fx.clone() else { return };
        let n = if fx.anims.contains_key(&no) { no } else { 0 };
        let mut e = self.make_explod(&self.fighters[i], fx, n, x, y, 1.0);
        e.under = under; e.on_top = !under;
        self.push_explod(e);
    }
    pub fn spawn_super_fx(&mut self, i: usize, ox: f32, oy: f32, own_anim: i32) {
        let f = &self.fighters[i];
        let (def, no, l) = if own_anim >= 0 { (f.def.clone(), own_anim, f.lscl) } else { match self.fx.clone() { Some(fx) => (fx, 30, 1.0), None => return } };
        if !def.anims.contains_key(&no) { return; }
        let mut e = self.make_explod(f, def, no, f.pos.x + ox * f.facing, f.pos.y + oy, l);
        e.super_move = 9999; e.add = true;
        self.push_explod(e);
    }
    pub fn spawn_dust(&mut self, i: usize, x: f32, y: f32) {
        let Some(fx) = self.fx.clone() else { return };
        if !fx.anims.contains_key(&120) || self.fighters[i].has_assert(asf::NOMAKEDUST) { return; }
        let mut e = self.make_explod(&self.fighters[i], fx, 120, x, y, 1.0);
        e.on_top = false; e.spr_priority = -3;
        self.push_explod(e);
    }

    pub fn spawn_explod(&mut self, i: usize, sc: &Controller) {
        if self.explods.len() >= 512 { return; }
        let num = |m: &Match, k: &str, d: f64, idx: usize| m.num(i, sc, k, d, idx) as f32;
        let anim_raw = sc.raw("anim").unwrap_or("0").trim().to_string();
        let from_fx = anim_raw.starts_with(['f', 'F']);
        let anim_no = anim_raw.trim_start_matches(['f', 'F', 's', 'S']).trim().parse::<i32>().unwrap_or_else(|_| num(self, "anim", 0.0, 0) as i32);
        let f = &self.fighters[i];
        let def = if from_fx { match self.fx.clone() { Some(x) => x, None => return } } else { f.def.clone() };
        if !def.anims.contains_key(&anim_no) { return; }
        let postype = sc.raw("postype").unwrap_or("p1").to_ascii_lowercase();
        let screen_space = matches!(postype.as_str(), "front" | "back" | "left" | "right");
        let (px, py) = (num(self, "pos", 0.0, 0) * f.lscl, num(self, "pos", 0.0, 1) * f.lscl);
        let fac = if num(self, "facing", 1.0, 0) < 0.0 { -1.0 } else { 1.0 };
        let (x, y, facing, bind_to);
        let half = self.view_w / 2.0;
        match postype.as_str() {
            "p2" => {
                let e = self.enemy(i, 0).unwrap_or(i);
                let fe = &self.fighters[e];
                x = fe.pos.x + px * fe.facing;
                y = fe.pos.y + py;
                bind_to = Some(fe.id);
                facing = fe.facing * fac;
            }
            "front" => {
                x = half * f.facing + px * f.facing;
                y = py;
                bind_to = None;
                facing = f.facing * fac;
            }
            "back" => {
                x = -half * f.facing + px * f.facing;
                y = py;
                bind_to = None;
                facing = f.facing * fac;
            }
            "left" => {
                x = -half + px;
                y = py;
                bind_to = None;
                facing = fac;
            }
            "right" => {
                x = half + px;
                y = py;
                bind_to = None;
                facing = fac;
            }
            _ => {
                x = f.pos.x + px * f.facing;
                y = f.pos.y + py;
                bind_to = Some(f.id);
                facing = f.facing * fac;
            }
        }
        let lscl = if from_fx { 1.0 } else { f.lscl };
        let mut e = self.make_explod(f, def, anim_no, x, y, lscl);
        e.id = num(self, "id", -1.0, 0) as i32;
        e.facing = facing;
        e.vfacing = if num(self, "vfacing", 1.0, 0) < 0.0 { -1.0 } else { 1.0 };
        e.vel = V2 { x: num(self, "vel", 0.0, 0) * f.lscl, y: num(self, "vel", 0.0, 1) * f.lscl };
        e.accel = V2 { x: num(self, "accel", 0.0, 0) * f.lscl, y: num(self, "accel", 0.0, 1) * f.lscl };
        e.bind_time = num(self, "bindtime", 0.0, 0) as i32;
        if e.bind_time != 0 { e.bind_to = bind_to; e.bind_off = V2 { x: px, y: py }; }
        e.remove_time = num(self, "removetime", -2.0, 0) as i32;
        e.spr_priority = num(self, "sprpriority", 0.0, 0) as i32;
        e.on_top = num(self, "ontop", 0.0, 0) != 0.0;
        e.under = num(self, "under", 0.0, 0) != 0.0;
        e.scale = [num(self, "scale", 1.0, 0), num(self, "scale", 1.0, 1)];
        e.remove_on_get_hit = num(self, "removeongethit", 0.0, 0) != 0.0;
        e.super_move = num(self, "supermove", num(self, "supermovetime", 0.0, 0) as f64, 0) as i32;
        e.pause_move = num(self, "pausemovetime", 0.0, 0) as i32;
        e.add = sc.raw("trans").is_some_and(|t| t.to_ascii_lowercase().starts_with("add"));
        e.ignore_hit_pause = num(self, "ignorehitpause", 1.0, 0) != 0.0;
        e.screen = screen_space || sc.raw("space").is_some_and(|s| s.trim().eq_ignore_ascii_case("screen"));
        if e.screen && !screen_space { e.pos.x -= self.camera.x; e.pos.y -= self.camera.y; e.bind_to = None; }
        self.push_explod(e);
    }

    fn update_explods(&mut self, paused: bool) {
        if self.explods.is_empty() { return; }
        let ps = self.pause.as_ref().map(|p| p.is_super);
        let mut f_info = [(0u32, false, V2::default(), 0.0f32); 8];
        let f_len = self.fighters.len().min(8);
        for i in 0..f_len {
            let f = &self.fighters[i];
            f_info[i] = (f.id, f.hit_pause > 0 || f.hit_shake > 0, f.pos, f.facing);
        }
        let info = &f_info[..f_len];
        let all_in_info = self.fighters.len() <= 8;
        let fighters = &self.fighters;
        self.explods.retain_mut(|e| {
            if paused {
                let allowed = if ps == Some(true) { e.super_move > 0 } else { e.pause_move > 0 };
                if !allowed { return true; }
                if ps == Some(true) { e.super_move -= 1; } else { e.pause_move -= 1; }
            }
            let owner_hit_pause = if all_in_info {
                info.iter().find(|i| i.0 == e.owner).map(|i| i.1).unwrap_or(false)
            } else {
                fighters.iter().find(|f| f.id == e.owner).is_some_and(|f| f.hit_pause > 0 || f.hit_shake > 0)
            };
            if owner_hit_pause && !e.ignore_hit_pause { return true; }
            let bound = if e.screen {
                None
            } else if all_in_info {
                e.bind_to.and_then(|b| info.iter().find(|i| i.0 == b)).map(|i| (i.2, i.3))
            } else {
                e.bind_to.and_then(|b| fighters.iter().find(|f| f.id == b)).map(|f| (f.pos, f.facing))
            };
            e.time += 1;
            e.cur.advance();
            match bound {
                Some(b) if e.bind_time > 0 || e.bind_time == -1 => {
                    e.pos.x = b.0.x + e.bind_off.x * b.1; e.pos.y = b.0.y + e.bind_off.y;
                    if e.bind_time > 0 { e.bind_time -= 1; }
                }
                _ => { e.pos.x += e.vel.x * e.facing; e.pos.y += e.vel.y; e.vel.x += e.accel.x; e.vel.y += e.accel.y; }
            }
            match e.remove_time { -2 => !e.cur.done(), -1 => true, t => e.time < t }
        });
    }

    pub fn spawn_helper(&mut self, i: usize, sc: &Controller) {
        if self.fighters.len() > 60 { return; }
        let num = |m: &Match, k: &str, d: f64, idx: usize| m.num(i, sc, k, d, idx) as f32;
        let root_i = self.idx(self.fighters[i].root).unwrap_or(i);
        let root = self.fighters[root_i].clone();
        let f = &self.fighters[i];
        let id = self.next_id;
        let mut h = Fighter::new(id, root.def.clone(), f.side, f.player_no);
        h.is_helper = true;
        h.helper_id = num(self, "id", 0.0, 0) as i32;
        h.parent = Some(f.id); h.root = root.id;
        h.key_ctrl = num(self, "keyctrl", 0.0, 0) != 0.0;
        h.pause_move_time = num(self, "pausemovetime", 0.0, 0) as i32;
        h.pause_bool = num(self, "pausebool", 0.0, 0) != 0.0;
        h.super_move_time = num(self, "supermovetime", 0.0, 0) as i32;
        h.preserve = num(self, "preserve", 0.0, 0) != 0.0;
        h.own_pal = num(self, "ownpal", 0.0, 0) != 0.0;
        let helper_type = sc.raw("helpertype").map(|t| t.trim().to_ascii_lowercase()).unwrap_or_default();
        h.helper_player = helper_type == "player";
        h.helper_projectile = helper_type == "projectile";
        h.own_projectile = num(self, "ownprojectile", 0.0, 0) != 0.0;
        h.own_clsn_scale = num(self, "ownclsnscale", 0.0, 0) != 0.0;
        if !h.own_clsn_scale { h.size_overrides = f.size_overrides.clone(); }
        for (name, param) in &sc.params {
            if !name.starts_with("size.") { continue; }
            let Some(e) = param.exprs.first() else { continue };
            let v = self.ev(i, e) as f32;
            if v.is_nan() { continue; }
            match h.size_overrides.iter_mut().find(|(n, _)| n == name) { Some(slot) => slot.1 = v, None => h.size_overrides.push((name.clone(), v)) }
        }
        h.pal_no = f.pal_no; h.ai_level = root.ai_level; h.life = root.life_max; h.life_max = root.life_max;
        let postype = sc.raw("postype").unwrap_or("p1").to_ascii_lowercase();
        let (px, py) = (num(self, "pos", 0.0, 0) * f.lscl, num(self, "pos", 0.0, 1) * f.lscl);
        let fac = if num(self, "facing", 1.0, 0) < 0.0 { -1.0 } else { 1.0 };
        h.facing = f.facing * fac;
        h.pos = match postype.as_str() {
            "p2" => { let e = self.enemy(i, 0).unwrap_or(i); let fe = &self.fighters[e]; V2 { x: fe.pos.x + px * fe.facing, y: fe.pos.y + py } }
            "front" | "back" | "left" | "right" => {
                let edge = match postype.as_str() { "left" => -1.0, "right" => 1.0, "front" => f.facing, _ => -f.facing };
                V2 { x: self.camera.x + edge * self.view_w / 2.0 + px, y: py }
            }
            _ => V2 { x: f.pos.x + px * f.facing, y: f.pos.y + py },
        };
        h.state_no = -1; h.ctrl = false;
        let owner = if Arc::ptr_eq(&f.st_def, &f.def) { h.def.clone() } else { f.st_def.clone() };
        let st = num(self, "stateno", 0.0, 0) as i32;
        self.next_id += 1;
        self.fighters.push(h);
        let hi = self.fighters.len() - 1;
        self.change_state(hi, st, owner);
    }
    pub fn destroy_helper(&mut self, i: usize) {
        if !self.fighters[i].is_helper { return; }
        let id = self.fighters[i].id;
        self.fighters[i].destroyed = true;
        self.fighters.remove(i);
        for e in self.explods.iter_mut() { if e.bind_to == Some(id) { e.bind_to = None; } }
    }

    pub fn spawn_projectile(&mut self, i: usize, sc: &Controller) {
        if self.projectiles.len() >= 256 { return; }
        let num = |m: &Match, k: &str, d: f64, idx: usize| m.num(i, sc, k, d, idx) as f32;
        let f = &self.fighters[i];
        let anim = num(self, "projanim", 0.0, 0) as i32;
        if !f.def.anims.contains_key(&anim) { return; }
        let hd = crate::hitdef::build(self, i, sc);
        let f = &self.fighters[i];
        let (ox, oy) = (num(self, "offset", 0.0, 0) * f.lscl, num(self, "offset", 0.0, 1) * f.lscl);
        let (mut x, mut y) = (f.pos.x + ox * f.facing, f.pos.y + oy);
        if sc.raw("postype").is_some_and(|p| p.trim().eq_ignore_ascii_case("p2")) {
            if let Some(e) = self.enemy(i, 0) { x = self.fighters[e].pos.x + ox * f.facing; y = self.fighters[e].pos.y + oy; }
        }
        let has = |k: &str| sc.has(k, 0);
        let hit_anim = if has("projhitanim") { num(self, "projhitanim", -1.0, 0) as i32 } else { -1 };
        let p = Projectile {
            owner: f.id, side: f.side, teamside: f.side, root: f.root, tally: f.projectile_tally_id(), id: num(self, "projid", 0.0, 0) as i32, cur: AnimCursor::new(f.def.clone(), anim),
            hit_anim, rem_anim: if has("projremanim") { num(self, "projremanim", -1.0, 0) as i32 } else { hit_anim },
            cancel_anim: if has("projcancelanim") { num(self, "projcancelanim", -1.0, 0) as i32 } else { -1 },
            pos: V2 { x, y }, vel: V2 { x: num(self, "velocity", 0.0, 0) * f.lscl, y: num(self, "velocity", 0.0, 1) * f.lscl },
            accel: V2 { x: num(self, "accel", 0.0, 0) * f.lscl, y: num(self, "accel", 0.0, 1) * f.lscl },
            vel_mul: V2 { x: num(self, "velmul", 1.0, 0), y: num(self, "velmul", 1.0, 1) },
            rem_vel: V2 { x: num(self, "remvelocity", 0.0, 0) * f.lscl, y: num(self, "remvelocity", 0.0, 1) * f.lscl },
            facing: f.facing, hits: num(self, "projhits", 1.0, 0) as i32, miss_time: num(self, "projmisstime", 0.0, 0) as i32, miss_cd: 0,
            priority: num(self, "projpriority", 1.0, 0) as i32, spr_priority: num(self, "projsprpriority", 3.0, 0) as i32,
            edge_bound: num(self, "projedgebound", 40.0, 0) * f.lscl,
            height_bound: [num(self, "projheightbound", -240.0, 0) * f.lscl, num(self, "projheightbound", 1.0, 1) * f.lscl],
            remove_time: num(self, "projremovetime", -1.0, 0) as i32, time: 0, guard_dist: hd.guard_dist * f.lscl, hitdef: hd, dying: false,
            scale: [num(self, "projscale", 1.0, 0), num(self, "projscale", 1.0, 1)], remove: false, lscl: f.lscl,
            remove_on_hit: num(self, "projremove", 1.0, 0) != 0.0, stage_bound: num(self, "projstagebound", 40.0, 0) * f.lscl,
            super_move: num(self, "supermovetime", 0.0, 0) as i32, pause_move: num(self, "pausemovetime", 0.0, 0) as i32,
        };
        if self.projectiles.len() < MAX_PROJECTILES {
            self.projectiles.push(p);
        }
    }
    pub fn proj_boxes(p: &Projectile, kind: u8) -> Vec<Box4> {
        let Some(fr) = p.cur.frame() else { return vec![] };
        let list = if kind == 1 { &fr.clsn1 } else { &fr.clsn2 };
        list.iter().map(|b| {
            let (mut x1, mut x2) = (b[0] * p.lscl * p.scale[0], b[2] * p.lscl * p.scale[0]);
            if p.facing < 0.0 { let t = -x2; x2 = -x1; x1 = t; }
            [p.pos.x + x1, p.pos.y + b[1] * p.lscl * p.scale[1], p.pos.x + x2, p.pos.y + b[3] * p.lscl * p.scale[1]]
        }).collect()
    }
    fn kill_proj(p: &mut Projectile, anim: i32) {
        if anim >= 0 && p.cur.def.anims.contains_key(&anim) { p.cur = AnimCursor::new(p.cur.def.clone(), anim); p.dying = true; } else { p.remove = true; }
    }
    fn update_projectiles(&mut self, paused: bool) {
        if self.projectiles.is_empty() { return; }
        let super_pause = self.pause.as_ref().is_some_and(|p| p.is_super);
        let (left, right) = (self.camera.x - self.view_w / 2.0, self.camera.x + self.view_w / 2.0);
        let (stage_l, stage_r) = (self.stage.bound_left - self.view_w / 2.0, self.stage.bound_right + self.view_w / 2.0);
        let mut frozen_buf = [false; 64];
        let n = self.projectiles.len();
        let use_stack = n <= frozen_buf.len();
        let mut frozen_heap = if !use_stack { vec![false; n] } else { vec![] };
        for (k, p) in self.projectiles.iter_mut().enumerate() {
            if paused {
                let budget = if super_pause { &mut p.super_move } else { &mut p.pause_move };
                if *budget <= 0 {
                    if use_stack { frozen_buf[k] = true; } else { frozen_heap[k] = true; }
                    continue;
                }
                *budget -= 1;
            }
            p.cur.advance(); p.time += 1;
            if p.dying {
                p.pos.x += p.rem_vel.x * p.facing; p.pos.y += p.rem_vel.y;
                if p.cur.done() || p.cur.a().is_none_or(|a| a.total_time < 0) { p.remove = true; }
                continue;
            }
            p.pos.x += p.vel.x * p.facing; p.pos.y += p.vel.y;
            p.vel.x = p.vel.x * p.vel_mul.x + p.accel.x; p.vel.y = p.vel.y * p.vel_mul.y + p.accel.y;
            if p.miss_cd > 0 { p.miss_cd -= 1; }
            if p.pos.x < left - p.edge_bound || p.pos.x > right + p.edge_bound { p.remove = true; }
            if p.pos.x < stage_l - p.stage_bound || p.pos.x > stage_r + p.stage_bound { p.remove = true; }
            if p.pos.y < p.height_bound[0] || p.pos.y > p.height_bound[1] { p.remove = true; }
            if p.remove_time >= 0 && p.time >= p.remove_time { let a = p.rem_anim; Self::kill_proj(p, a); }
        }
        let is_frozen = |idx: usize| if use_stack { frozen_buf[idx] } else { frozen_heap[idx] };
        for a in 0..n { for b in (a + 1)..n {
            if is_frozen(a) || is_frozen(b) { continue; }
            let (pa, pb) = (&self.projectiles[a], &self.projectiles[b]);
            if pa.dying || pb.dying || pa.remove || pb.remove || pa.side == pb.side { continue; }
            let ba: Vec<Box4> = Self::proj_boxes(pa, 1).into_iter().chain(Self::proj_boxes(pa, 2)).collect();
            let bb: Vec<Box4> = Self::proj_boxes(pb, 1).into_iter().chain(Self::proj_boxes(pb, 2)).collect();
            if Self::overlap(&ba, &bb).is_some() {
                let (pra, prb) = (pa.priority, pb.priority);
                let gt = self.game_time;
                for (k, lost) in [(a, pra <= prb), (b, prb <= pra)] {
                    if !lost { self.projectiles[k].priority -= 1; continue; }
                    let p = &mut self.projectiles[k]; let an = if p.cancel_anim >= 0 { p.cancel_anim } else { p.rem_anim }; Self::kill_proj(p, an);
                    let (owner, pid) = (p.owner, p.id);
                    if let Some(o) = self.idx(owner) { self.fighters[o].proj_contacts.push(('x', pid, gt)); }
                }
            }
        } }
        for pi in 0..self.projectiles.len() {
            if is_frozen(pi) { continue; }
            for d in 0..self.fighters.len() {
                let p = &self.projectiles[pi];
                if p.dying || p.remove || p.miss_cd > 0 { break; }
                let Some(oi) = self.idx(p.owner).or_else(|| self.idx(p.root)) else { break };
                if self.fighters[d].root == p.root { continue; }
                if !self.can_be_hit(d, &p.hitdef, p.side) { continue; }
                let c1 = Self::proj_boxes(p, 1);
                if c1.is_empty() { break; }
                let Some(at) = Self::overlap(&c1, &self.fighters[d].boxes(2)) else { continue };
                let (hd, pid, pf) = (p.hitdef.clone(), p.id, p.facing);
                self.apply_hit(oi, d, &hd, at, Some((pid, pf)));
                let gt = self.game_time;
                self.fighters[oi].proj_contacts.push(('c', pid, gt));
                let p = &mut self.projectiles[pi];
                p.hits -= 1;
                if p.hits <= 0 && p.remove_on_hit { let a = p.hit_anim; Self::kill_proj(p, a); } else { p.miss_cd = p.miss_time.max(1); }
            }
        }
        self.projectiles.retain(|p| !p.remove);
    }

    pub fn is_mugen(&self, i: usize) -> bool { self.fighters[i].def.source == Source::Mugen }
}
