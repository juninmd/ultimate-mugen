use super::*;
use crate::replays::{self, Entry, Sort};

const ROWS: usize = 8;

pub struct Browser {
    pub entries: Vec<Entry>,
    pub cur: usize,
    pub top: usize,
    pub sort: Sort,
    pub rename: Option<String>,
    pub confirm_delete: bool,
    pub note: String,
}

impl Default for Browser {
    fn default() -> Self {
        Browser { entries: vec![], cur: 0, top: 0, sort: Sort::Newest, rename: None, confirm_delete: false, note: String::new() }
    }
}

impl Mode {
    pub fn key(self) -> &'static str {
        match self {
            Mode::Arcade => "arcade",
            Mode::Cpu => "cpu",
            Mode::Versus => "versus",
            Mode::Training => "training",
            Mode::Watch => "watch",
            Mode::TeamSeq => "team-sequence",
            Mode::TeamSimul => "team-simul",
        }
    }

    pub fn from_key(key: &str) -> Option<Mode> {
        [Mode::Arcade, Mode::Cpu, Mode::Versus, Mode::Training, Mode::Watch, Mode::TeamSeq, Mode::TeamSimul].into_iter().find(|m| m.key() == key)
    }
}

pub fn key_char(k: KeyCode, shift: bool) -> Option<char> {
    const LETTERS: &str = "abcdefghijklmnopqrstuvwxyz";
    let letter = KeyCode::ALL.iter().take(26).position(|c| *c == k).and_then(|i| LETTERS.chars().nth(i));
    if let Some(c) = letter {
        return Some(if shift { c.to_ascii_uppercase() } else { c });
    }
    let digit = KeyCode::ALL.iter().skip(26).take(10).position(|c| *c == k).and_then(|i| char::from_digit(i as u32, 10));
    if digit.is_some() {
        return digit;
    }
    match k {
        KeyCode::Space => Some('-'),
        KeyCode::Minus => Some(if shift { '_' } else { '-' }),
        _ => None,
    }
}

impl App {
    pub(super) fn snapshot_fight(&self) -> Option<Replay> {
        let s = self.session.as_ref()?;
        let plan = self.load_plan();
        let pals = (0..plan.len()).map(|i| self.palette_of(&plan, i)).collect();
        Some(Replay { sel: self.sel, mode: self.mode, team: self.team.clone(), rec: s.rec.clone(), pals, seed: s.seed, checksum: Some(ume_netplay::checksum(&s.m)) })
    }

    pub(super) fn replay_to_file(&self, r: &Replay) -> ume_netplay::Replay {
        let id = |i: usize| self.reg.chars[i].id.clone();
        let teams = match &r.team {
            Some(t) => [0, 1].map(|side| t.members[side].iter().map(|&i| id(i)).collect()),
            None => [vec![id(r.sel.0)], vec![id(r.sel.1)]],
        };
        let header = ume_netplay::Header {
            build: crate::update::VERSION.to_string(),
            mode: r.mode.key().to_string(),
            stage: self.reg.stages[r.sel.2].id.clone(),
            teams,
            palettes: r.pals.clone(),
            simul: r.team.as_ref().is_some_and(|t| t.simul),
            seed: r.seed,
            created: replays::now(),
            checksum: r.checksum,
        };
        let players = r.rec.iter().map(Vec::len).max().unwrap_or(2);
        let mut file = ume_netplay::Replay::with_header(players, &header);
        for tick in &r.rec {
            file.push(tick);
        }
        file
    }

    pub(super) fn replay_from_file(&self, file: &ume_netplay::Replay) -> Result<Replay, String> {
        file.check_build(crate::update::VERSION)?;
        let h = file.header()?;
        let find = |id: &String| self.reg.chars.iter().position(|c| &c.id == id).ok_or_else(|| format!("Personagem não instalado: {id}"));
        let stage = self.reg.stages.iter().position(|s| s.id == h.stage).ok_or_else(|| format!("Cenário não instalado: {}", h.stage))?;
        let mode = Mode::from_key(&h.mode).ok_or_else(|| format!("Modo desconhecido: {}", h.mode))?;
        let teams = h.teams.iter().map(|t| t.iter().map(find).collect::<Result<Vec<usize>, String>>()).collect::<Result<Vec<_>, _>>()?;
        let (Some(&lead_a), Some(&lead_b)) = (teams[0].first(), teams[1].first()) else { return Err("Replay sem lutadores".into()) };
        let team = mode.is_team().then(|| Team::new(teams[0].clone(), teams[1].clone(), h.simul));
        let rec = (0..file.len()).filter_map(|n| file.frame(n).map(<[u32]>::to_vec)).collect();
        Ok(Replay { sel: (lead_a, lead_b, stage), mode, team, rec, pals: h.palettes, seed: h.seed, checksum: h.checksum })
    }

    pub(super) fn persist_replay(&mut self, r: &Replay) {
        let file = self.replay_to_file(r);
        let name = match file.header() {
            Ok(h) => replays::default_name(&h),
            Err(_) => "replay".to_string(),
        };
        if let Err(e) = replays::save_in(&replays::dir(), &name, &file) {
            if !cfg!(target_arch = "wasm32") {
                self.msg = format!("Não foi possível salvar o replay: {e}");
            }
        }
    }

    pub(super) fn open_replays(&mut self) {
        self.replay_ui = Browser::default();
        self.refresh_replays();
        self.go(Screen::Replays);
    }

    fn refresh_replays(&mut self) {
        let b = &mut self.replay_ui;
        b.entries = replays::list_in(&replays::dir(), b.sort);
        b.cur = b.cur.min(b.entries.len().saturating_sub(1));
        b.top = b.top.min(b.cur);
    }

    fn describe_entry(&self, e: &Entry) -> (String, bool) {
        let name_of = |id: &String| self.reg.chars.iter().find(|c| &c.id == id).map_or(id.clone(), |c| c.name.clone());
        match &e.header {
            Err(msg) => (msg.clone(), false),
            Ok(h) => {
                let side = |t: &Vec<String>| t.iter().map(name_of).collect::<Vec<_>>().join(" + ");
                let stage = self.reg.stages.iter().find(|s| s.id == h.stage).map_or(h.stage.clone(), |s| s.name.clone());
                let mode = Mode::from_key(&h.mode).map_or_else(|| h.mode.clone(), |m| m.label().to_string());
                let line = format!("{} vs {} · {} · {} · {} · {}", side(&h.teams[0]), side(&h.teams[1]), mode, stage, replays::format_duration(e.ticks), replays::format_time(if h.created > 0 { h.created } else { e.modified }));
                let compatible = h.build == crate::update::VERSION;
                (if compatible { line } else { format!("{line} · build {}", h.build) }, compatible)
            }
        }
    }

    fn rename_input(&mut self) {
        let Some(mut text) = self.replay_ui.rename.take() else { return };
        let shift = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);
        let mut done = false;
        let mut cancel = false;
        while let Some(k) = get_last_key_pressed() {
            match k {
                KeyCode::Enter | KeyCode::KpEnter => done = true,
                KeyCode::Escape => cancel = true,
                KeyCode::Backspace => { text.pop(); }
                other => { if let Some(c) = key_char(other, shift) { if text.chars().count() < 40 { text.push(c); } } }
            }
        }
        if cancel {
            return;
        }
        if done {
            let Some(path) = self.replay_ui.entries.get(self.replay_ui.cur).map(|e| e.path.clone()) else { return };
            match replays::rename(&path, &text) {
                Ok(_) => self.replay_ui.note = "Replay renomeado.".into(),
                Err(e) => { self.replay_ui.note = format!("Não foi possível renomear: {e}"); self.replay_ui.rename = Some(text); return; }
            }
            self.refresh_replays();
            return;
        }
        self.replay_ui.rename = Some(text);
    }

    pub(super) async fn replays_screen(&mut self) {
        self.ui.background();
        self.ui.text("REPLAYS", 40.0, 64.0, 34.0, hex("#ffcc33"), Align::Left, true);
        let dir = replays::dir();
        let sort_label = format!("ordem: {}", replays::sort_label(self.replay_ui.sort));
        self.ui.text(&sort_label, VW - 40.0, 64.0, 20.0, hex("#a49cc0"), Align::Right, false);
        let total = self.replay_ui.entries.len();
        if total == 0 {
            let msg = if cfg!(target_arch = "wasm32") { "Replays em disco não estão disponíveis na versão web." } else { "Nenhum replay salvo ainda. Cada luta terminada é gravada automaticamente." };
            self.ui.text(msg, 40.0, 140.0, 22.0, WHITE, Align::Left, false);
            if !cfg!(target_arch = "wasm32") { self.ui.text(&format!("Pasta: {}", dir.display()), 40.0, 176.0, 16.0, hex("#a49cc0"), Align::Left, false); }
        }
        if self.replay_ui.rename.is_some() {
            self.rename_input();
        } else if self.replay_ui.confirm_delete {
            if is_key_pressed(KeyCode::Enter) || self.c.pressed(nav::OK) {
                if let Some(path) = self.replay_ui.entries.get(self.replay_ui.cur).map(|e| e.path.clone()) {
                    self.replay_ui.note = match replays::delete(&path) { Ok(()) => "Replay apagado.".into(), Err(e) => format!("Não foi possível apagar: {e}") };
                }
                self.replay_ui.confirm_delete = false;
                self.refresh_replays();
            } else if self.c.pressed(nav::BACK) || is_key_pressed(KeyCode::N) {
                self.replay_ui.confirm_delete = false;
            }
        } else {
            if total > 0 {
                if self.c.pressed(nav::DOWN) { self.replay_ui.cur = (self.replay_ui.cur + 1) % total; self.audio.sfx("select"); }
                if self.c.pressed(nav::UP) { self.replay_ui.cur = (self.replay_ui.cur + total - 1) % total; self.audio.sfx("select"); }
            }
            if is_key_pressed(KeyCode::Tab) || self.clicked_rect(880.0, 640.0, 160.0, 48.0) { self.replay_ui.sort = self.replay_ui.sort.next(); self.refresh_replays(); }
            let have = !self.replay_ui.entries.is_empty();
            if have && (is_key_pressed(KeyCode::R) || self.clicked_rect(520.0, 640.0, 160.0, 48.0)) {
                let name = self.replay_ui.entries[self.replay_ui.cur].name.clone();
                while get_last_key_pressed().is_some() {}
                self.replay_ui.rename = Some(name);
                self.replay_ui.note.clear();
            }
            if have && (is_key_pressed(KeyCode::Delete) || self.clicked_rect(700.0, 640.0, 160.0, 48.0)) {
                self.replay_ui.confirm_delete = true;
                self.replay_ui.note.clear();
            }
            let play = have && (self.c.pressed(nav::OK | nav::START) || self.clicked_rect(340.0, 640.0, 160.0, 48.0));
            if self.c.pressed(nav::BACK) || self.clicked_rect(40.0, 640.0, 160.0, 48.0) { self.go(Screen::Menu { cur: 10 }); return; }
            if play { self.play_selected_replay().await; return; }
        }
        let b = &mut self.replay_ui;
        if b.cur < b.top { b.top = b.cur; }
        if b.cur >= b.top + ROWS { b.top = b.cur + 1 - ROWS; }
        let (cur, top, renaming, confirming) = (b.cur, b.top, b.rename.clone(), b.confirm_delete);
        for (slot, i) in (top..total.min(top + ROWS)).enumerate() {
            let y = 100.0 + slot as f32 * 64.0;
            let e = self.replay_ui.entries[i].clone();
            if !confirming && renaming.is_none() && self.clicked_rect(40.0, y, 1200.0, 58.0) {
                if i == cur { self.play_selected_replay().await; return; }
                self.replay_ui.cur = i;
            }
            let (line, ok) = self.describe_entry(&e);
            let on = i == self.replay_ui.cur;
            self.ui.rect(40.0, y, 1200.0, 58.0, if on { Color::new(1.0, 0.3, 0.43, 0.35) } else { Color::new(1.0, 1.0, 1.0, if slot % 2 == 0 { 0.07 } else { 0.03 }) });
            let title = match (&renaming, on) { (Some(t), true) => format!("{t}_"), _ => e.name.clone() };
            self.ui.text(&title, 56.0, y + 26.0, 22.0, if on { WHITE } else { hex("#ffcc33") }, Align::Left, true);
            self.ui.text(&line.chars().take(110).collect::<String>(), 56.0, y + 48.0, 16.0, if ok { hex("#c9f7ff") } else { hex("#ff8a9a") }, Align::Left, false);
        }
        if total > ROWS { self.ui.text(&format!("{}/{}", cur + 1, total), VW - 40.0, 96.0, 16.0, hex("#a49cc0"), Align::Right, false); }
        let buttons: [(&str, f32, bool); 6] = [("Voltar", 40.0, true), ("Jogar", 340.0, total > 0), ("Renomear (R)", 520.0, total > 0), ("Apagar (Del)", 700.0, total > 0), ("Ordem (Tab)", 880.0, true), ("", 0.0, false)];
        for (label, x, enabled) in buttons.iter().filter(|b| !b.0.is_empty()) {
            self.ui.rect(*x, 640.0, 160.0, 48.0, if *enabled { Color::new(1.0, 1.0, 1.0, 0.12) } else { Color::new(1.0, 1.0, 1.0, 0.04) });
            self.ui.text(label, x + 80.0, 672.0, 18.0, if *enabled { WHITE } else { hex("#6b6485") }, Align::Center, true);
        }
        let status = if renaming.is_some() { "Digite o novo nome · Enter confirma · Esc cancela".to_string() } else if confirming { "Apagar este replay? Enter confirma · Esc cancela".to_string() } else { self.replay_ui.note.clone() };
        self.ui.text(&status, 40.0, 622.0, 18.0, hex("#ffcc33"), Align::Left, true);
    }

    async fn play_selected_replay(&mut self) {
        let Some(path) = self.replay_ui.entries.get(self.replay_ui.cur).map(|e| e.path.clone()) else { return };
        let outcome = replays::load(&path).and_then(|file| self.replay_from_file(&file));
        match outcome {
            Ok(r) => {
                self.last_fight = Some(r);
                self.audio.stop_music();
                self.start_replay().await;
                self.replay_from_browser = true;
            }
            Err(e) => self.replay_ui.note = e,
        }
    }

    pub(super) fn draw_replay_controls(&mut self) {
        let Some(s) = self.session.as_mut() else { return };
        if !s.is_replay || s.paused { return; }
        let (held, speed, ticks) = (s.hold, s.speed, s.rec.len());
        let total = self.replaying.as_ref().map_or(0, |r| r.rec.len());
        let verdict = self.replay_verdict;
        let label = format!("{}  {}x  {} / {}", if held { "PAUSADO" } else { "REPLAY" }, speed, ticks, total);
        self.ui.rect(VW / 2.0 - 330.0, VH - 74.0, 660.0, 58.0, Color::new(0.02, 0.02, 0.08, 0.72));
        self.ui.text(&label, VW / 2.0 - 316.0, VH - 38.0, 20.0, WHITE, Align::Left, true);
        if let Some(ok) = verdict {
            self.ui.text(if ok { "VERIFICADO" } else { "DIVERGIU" }, VW / 2.0 - 316.0, VH - 20.0, 14.0, hex(if ok { "#6dff9e" } else { "#ff4d6d" }), Align::Left, true);
        }
        let controls: [(&str, f32); 5] = [("1x", 40.0), ("2x", 92.0), ("4x", 144.0), (if held { ">" } else { "II" }, 196.0), ("|>", 248.0)];
        let mut action: Option<usize> = None;
        for (i, (label, dx)) in controls.iter().enumerate() {
            let (x, y) = (VW / 2.0 - 330.0 + 160.0 + dx + 40.0, VH - 68.0);
            let on = (i < 3 && speed == [1, 2, 4][i]) || (i == 3 && held);
            self.ui.rect(x, y, 44.0, 46.0, if on { hex("#ff4d6d") } else { Color::new(1.0, 1.0, 1.0, 0.14) });
            self.ui.text(label, x + 22.0, y + 31.0, 20.0, WHITE, Align::Center, true);
            if self.clicked_rect(x, y, 44.0, 46.0) { action = Some(i); }
        }
        let Some(s) = self.session.as_mut() else { return };
        let keys = [(KeyCode::Key1, 0), (KeyCode::Key2, 1), (KeyCode::Key4, 2), (KeyCode::Space, 3), (KeyCode::Period, 4), (KeyCode::Right, 4)];
        for (k, a) in keys { if is_key_pressed(k) { action = Some(a); } }
        if is_key_pressed(KeyCode::Up) { s.cycle_speed(true); }
        if is_key_pressed(KeyCode::Down) { s.cycle_speed(false); }
        match action {
            Some(0) => s.speed = 1,
            Some(1) => s.speed = 2,
            Some(2) => s.speed = 4,
            Some(3) => s.hold = !s.hold,
            Some(4) => { s.hold = true; s.step_once = true; }
            _ => {}
        }
    }

    pub(super) fn check_replay_verdict(&mut self) {
        let (Some(s), Some(r)) = (self.session.as_ref(), self.replaying.as_ref()) else { return };
        if self.replay_verdict.is_none() && !r.rec.is_empty() && s.rec.len() == r.rec.len() {
            self.replay_verdict = r.checksum.map(|sum| sum == ume_netplay::checksum(&s.m));
        }
    }
}
