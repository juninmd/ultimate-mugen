//! Player card screen: tab bar, navigation and the record hook after each bout.
use super::player_card_edit::CardEdit;
use super::*;
use crate::controls::nav;
use crate::i18n::tr3;
use crate::player_card::Record;

pub(super) const TABS: usize = 5;
const TAB_X: f32 = 40.0;
const TAB_Y: f32 = 130.0;
const TAB_W: f32 = 220.0;
const TAB_H: f32 = 44.0;
const TAB_STEP: f32 = 236.0;
pub(super) const EDIT_X: f32 = 60.0;
pub(super) const EDIT_Y: f32 = 652.0;
pub(super) const EDIT_W: f32 = 380.0;
pub(super) const EDIT_H: f32 = 36.0;
const MENU_INDEX: usize = 9;

pub(super) fn tab_label(i: usize) -> &'static str {
    match i {
        0 => tr3("Ficha", "Card", "Ficha"),
        1 => tr3("Lutadores", "Fighters", "Luchadores"),
        2 => tr3("Partidas", "Matches", "Partidas"),
        3 => tr3("Amigos", "Friends", "Amigos"),
        _ => tr3("Conquistas", "Achievements", "Logros"),
    }
}

impl App {
    pub(super) fn open_player_card(&mut self) {
        self.player = Record::load();
        self.card_edit = None;
        self.go(Screen::PlayerCard { tab: 0 });
    }

    pub(super) fn record_player_bout(&mut self, winner: usize) {
        if self.replaying.is_some() || self.mode == Mode::Training {
            return;
        }
        let (Some(me), Some(rival)) = (self.reg.chars.get(self.sel.0), self.reg.chars.get(self.sel.1)) else {
            return;
        };
        let (me, rival) = (me.id.clone(), rival.id.clone());
        self.player.record_set(winner == 0, &me, &rival);
    }

    pub(super) fn fighter_index(&self, id: &str) -> Option<usize> {
        self.reg.chars.iter().position(|c| c.id.eq_ignore_ascii_case(id))
    }

    pub(super) fn fighter_name(&self, id: &str) -> String {
        self.fighter_index(id)
            .map(|i| self.reg.chars[i].name.clone())
            .unwrap_or_else(|| id.to_string())
    }

    pub(super) fn main_fighter(&self) -> Option<usize> {
        self.player
            .most_played()
            .and_then(|id| self.fighter_index(id))
            .or(Some(self.sel.0))
            .filter(|&i| i < self.reg.chars.len())
    }

    pub(super) fn player_name(&self) -> String {
        let name = self.player.card.name.trim();
        if name.is_empty() {
            tr3("Jogador", "Player", "Jugador").to_string()
        } else {
            name.to_string()
        }
    }

    pub(super) fn player_card_screen(&mut self) {
        let Screen::PlayerCard { mut tab } = self.screen else {
            return;
        };
        if self.card_edit.is_some() {
            self.card_edit_step();
        } else {
            if self.c.pressed(nav::RIGHT) {
                tab = (tab + 1) % TABS;
                self.audio.sfx("select");
            }
            if self.c.pressed(nav::LEFT) {
                tab = (tab + TABS - 1) % TABS;
                self.audio.sfx("select");
            }
            for i in 0..TABS {
                if self.clicked_rect(TAB_X + i as f32 * TAB_STEP, TAB_Y, TAB_W, TAB_H) {
                    tab = i;
                }
            }
            if tab == 0 && (self.c.pressed(nav::OK) || self.clicked_rect(EDIT_X, EDIT_Y, EDIT_W, EDIT_H)) {
                self.card_edit = Some(CardEdit::open(&self.player.card));
            }
            if self.c.pressed(nav::BACK) {
                self.screen = Screen::Menu { cur: MENU_INDEX };
                return;
            }
        }
        self.draw_player_card(tab);
        self.screen = Screen::PlayerCard { tab };
    }

    fn draw_player_card(&mut self, tab: usize) {
        crate::screen_art::ScreenArt::Records.background(&self.ui);
        self.ui.motes(30, 0.8);
        self.ui.adapted_header(
            tr3("Ficha do Jogador", "Player card", "Ficha del jugador"),
            tr3("Salva neste aparelho", "Saved on this device", "Guardada en este dispositivo"),
            VW,
        );
        for i in 0..TABS {
            let (x, active) = (TAB_X + i as f32 * TAB_STEP, i == tab);
            self.ui.adapted_tab(x, TAB_Y, TAB_W, TAB_H, active);
            let color = if active { WHITE } else { hex("#a49cc0") };
            self.ui.text(tab_label(i), x + TAB_W / 2.0, TAB_Y + TAB_H * 0.68, 20.0, color, Align::Center, active);
        }
        match tab {
            0 => self.card_tab(),
            1 => self.roster_tab(),
            2 => self.matches_tab(),
            3 => self.friends_tab(),
            _ => self.achievements_tab(),
        }
        self.card_edit_draw();
        self.ui.text(
            tr3("Esq/Dir abas · OK editar card · Esc voltar", "Left/Right tabs · OK edit card · Esc back", "Izq/Der pestañas · OK editar ficha · Esc volver"),
            VW / 2.0,
            VH - 16.0,
            16.0,
            hex("#a49cc0"),
            Align::Center,
            false,
        );
    }
}
