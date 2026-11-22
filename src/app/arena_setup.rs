use super::*;
use crate::i18n::tr3;
use ume_engine::matchs::{arena_tiles, ARENA_HALF_WIDTH, SWARM_MAX_TOTAL};

pub(crate) const SQUADS: [usize; 5] = [1, 2, 3, 4, 5];
pub(crate) const FOE_TOTALS: [usize; 6] = [1, 5, 10, 25, 50, 100];

pub(crate) const fn active_cap() -> usize {
    if cfg!(any(target_os = "android", target_os = "ios")) {
        8
    } else if cfg!(target_arch = "wasm32") {
        12
    } else {
        16
    }
}

pub(crate) const fn pool_cap() -> usize {
    if cfg!(any(target_os = "android", target_os = "ios")) {
        3
    } else if cfg!(target_arch = "wasm32") {
        6
    } else {
        8
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ArenaCfg {
    pub squad: usize,
    pub total: usize,
    pub random: bool,
    pub foe: usize,
}

impl Default for ArenaCfg {
    fn default() -> Self {
        ArenaCfg { squad: 1, total: 1, random: true, foe: 0 }
    }
}

impl ArenaCfg {
    pub fn slots(&self) -> usize {
        self.total.min(active_cap()).min(SWARM_MAX_TOTAL as usize)
    }
}

const ROW_SQUAD: usize = 0;
const ROW_FOES: usize = 1;
const ROW_KIND: usize = 2;
const ROW_GO: usize = 3;
const CHIP_W: f32 = 112.0;
const CHIP_H: f32 = 56.0;
const CHIP_GAP: f32 = 14.0;
const CHIP_X: f32 = 430.0;
const ROW_Y: [f32; 3] = [170.0, 262.0, 354.0];

impl App {
    pub(crate) fn arena_opponents(&self) -> usize {
        if self.mode == Mode::PowerArena { self.arena.total } else { self.team_n }
    }

    pub(crate) fn open_arena_setup(&mut self) {
        self.set_mode(Mode::PowerArena);
        self.audio.sfx("confirm");
        self.screen = Screen::ArenaSetup { row: ROW_GO };
    }

    fn arena_chip(&self, x: f32, y: f32, w: f32, label: &str, on: bool) -> bool {
        let hot = self.hover_rect(x, y, w, CHIP_H);
        let tint = if on { Some(crate::tweaks::accent(0.9)) } else if hot { Some(Color::new(1.0, 1.0, 1.0, 0.22)) } else { None };
        self.ui.adapted_plate(x, y, w, CHIP_H, tint);
        self.ui.text(label, x + w / 2.0, y + 37.0, 24.0, if on { BLACK } else { WHITE }, Align::Center, true);
        self.clicked_rect(x, y, w, CHIP_H)
    }

    fn arena_row<T: PartialEq + Copy>(&mut self, row: usize, cur_row: usize, title: &str, values: &[(T, String)], now: T) -> Option<T> {
        let y = ROW_Y[row];
        let focus = cur_row == row;
        let ink = if focus { hex("#ffcc33") } else { hex("#c9c2e2") };
        self.ui.text(title, 120.0, y + 38.0, 26.0, ink, Align::Left, true);
        let mut picked = None;
        let mut x = CHIP_X;
        for (value, label) in values {
            let w = CHIP_W.max(label.chars().count() as f32 * 17.0 + 36.0);
            if self.arena_chip(x, y, w, label, *value == now) {
                picked = Some(*value);
            }
            x += w + CHIP_GAP;
        }
        picked
    }

    pub(crate) fn arena_setup_screen(&mut self) {
        let Screen::ArenaSetup { mut row } = self.screen else {
            return;
        };
        crate::screen_art::ScreenArt::Select.background(&self.ui);
        self.ui.motes(24, 0.8);
        let squad = self.arena.squad;
        let slots = self.arena.slots();
        let tiles = arena_tiles(squad + slots);
        let sub = format!("{}v{}", squad, self.arena.total);
        self.ui.adapted_header(tr3("ARENA DO PODER", "POWER ARENA", "ARENA DEL PODER"), &sub, VW);

        let squads: Vec<(usize, String)> = SQUADS.iter().map(|&n| (n, n.to_string())).collect();
        let foes: Vec<(usize, String)> = FOE_TOTALS.iter().map(|&n| (n, n.to_string())).collect();
        let kinds = [
            (true, tr3("ALEATÓRIOS", "RANDOM", "AL AZAR").to_string()),
            (false, tr3("ESCOLHER", "CHOOSE", "ELEGIR").to_string()),
        ];
        let mut changed = false;
        if let Some(n) = self.arena_row(ROW_SQUAD, row, tr3("SEU TIME", "YOUR SQUAD", "TU EQUIPO"), &squads, squad) {
            self.arena.squad = n;
            row = ROW_SQUAD;
            changed = true;
        }
        if let Some(n) = self.arena_row(ROW_FOES, row, tr3("INIMIGOS", "ENEMIES", "ENEMIGOS"), &foes, self.arena.total) {
            self.arena.total = n;
            row = ROW_FOES;
            changed = true;
        }
        if let Some(random) = self.arena_row(ROW_KIND, row, tr3("TIPO", "TYPE", "TIPO"), &kinds, self.arena.random) {
            self.arena.random = random;
            row = ROW_KIND;
            changed = true;
        }
        let nav_dir = i32::from(self.c.pressed(nav::RIGHT)) - i32::from(self.c.pressed(nav::LEFT));
        if self.c.pressed(nav::DOWN) {
            row = (row + 1).min(ROW_GO);
            changed = true;
        }
        if self.c.pressed(nav::UP) {
            row = row.saturating_sub(1);
            changed = true;
        }
        if nav_dir != 0 {
            changed = true;
            match row {
                ROW_SQUAD => self.arena.squad = step_in(&SQUADS, self.arena.squad, nav_dir),
                ROW_FOES => self.arena.total = step_in(&FOE_TOTALS, self.arena.total, nav_dir),
                ROW_KIND => self.arena.random = !self.arena.random,
                _ => {}
            }
        }
        if changed {
            self.audio.sfx("select");
        }

        let info_y = 470.0;
        crate::menu_art::glass(&self.ui, 120.0, info_y, VW - 240.0, 110.0, 1.0);
        let line = |a: &str, b: &str, c: &str| tr3(a, b, c).to_string();
        let (foe_pt, foe_en, foe_es) = if slots == 1 { ("inimigo", "enemy", "enemigo") } else { ("inimigos", "enemies", "enemigos") };
        let lines = [
            line(
                &format!("No máximo {slots} {foe_pt} ao mesmo tempo: quando um cai, outro entra em um ponto aleatório do cenário."),
                &format!("Up to {slots} {foe_en} at once: when one falls another enters at a random spot of the stage."),
                &format!("Hasta {slots} {foe_es} a la vez: cuando uno cae, otro entra en un punto al azar del escenario."),
            ),
            line(
                &format!("Mapa {}x mais largo ({} px). O minimapa mostra quem está fora da tela.", tiles, (ARENA_HALF_WIDTH * 2.0) as u32 * tiles),
                &format!("Map {}x wider ({} px). The minimap shows who is off screen.", tiles, (ARENA_HALF_WIDTH * 2.0) as u32 * tiles),
                &format!("Mapa {}x más ancho ({} px). El minimapa muestra quién está fuera de pantalla.", tiles, (ARENA_HALF_WIDTH * 2.0) as u32 * tiles),
            ),
        ];
        for (n, text) in lines.iter().enumerate() {
            let shown = self.fit_label(text, 18.0, VW - 280.0);
            self.ui.text(&shown, 144.0, info_y + 42.0 + n as f32 * 36.0, 18.0, hex("#c9c2e2"), Align::Left, false);
        }

        let go = (VW / 2.0 - 170.0, 610.0, 340.0, 70.0);
        let hot = row == ROW_GO || self.hover_rect(go.0, go.1, go.2, go.3);
        self.ui.adapted_button(go.0, go.1, go.2, go.3, tr3("ESCOLHER LUTADORES", "PICK FIGHTERS", "ELEGIR LUCHADORES"), hot);
        let start = self.clicked_rect(go.0, go.1, go.2, go.3) || (self.c.pressed(nav::OK | nav::START) && row == ROW_GO);
        if self.c.pressed(nav::BACK) || is_key_pressed(KeyCode::Escape) {
            self.audio.sfx("cancel");
            self.screen = Screen::Menu { cur: 10 };
            return;
        }
        if start {
            self.arena_begin_pick();
            return;
        }
        if self.c.pressed(nav::OK) && row != ROW_GO {
            row = (row + 1).min(ROW_GO);
        }
        self.screen = Screen::ArenaSetup { row };
    }

    fn arena_begin_pick(&mut self) {
        self.team_n = self.arena.squad;
        self.team_pick.clear();
        self.team_pal.clear();
        self.team = None;
        self.audio.sfx("confirm");
        self.screen = Screen::Select { cur: [self.sel.0, self.sel.1], picking: 0, picked: [None, None] };
    }
}

fn step_in(values: &[usize], now: usize, dir: i32) -> usize {
    let at = values.iter().position(|&v| v == now).unwrap_or(0) as i32;
    values[(at + dir).clamp(0, values.len() as i32 - 1) as usize]
}
