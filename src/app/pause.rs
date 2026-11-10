use super::*;

mod binding;
mod draw_chrome;
mod draw_list;
mod draw_slots;
mod input;
mod input_fit;
mod labels;
mod layout;
mod skin;
mod state;

use draw_slots::SlotCard;
use input::Outcome;
use layout::Layout;
use state::{Action, Tab};
pub(super) use state::PauseState;

impl App {
    pub(super) async fn pause_menu(&mut self, accept_back: bool) {
        self.pause.tick(get_frame_time());
        let ease = self.pause.ease();
        let def = self.session.as_ref().unwrap().m.fighters[0].def.clone();
        let (name, moves) = (def.display_name.clone(), &def.movelist[..]);
        let author = def.author.clone();
        let base = Layout::new(self.ui.s);
        let outcome = self.pause_input(&base, &name, moves, accept_back);
        let lay = base.shifted((1.0 - ease) * 22.0);
        draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::new(0.02, 0.01, 0.06, 0.55 * ease));
        self.pause_chrome(&lay, self.pause.tab, &name, &author, ease);
        let selected = moves.get(self.pause.move_cur);
        let cards: [SlotCard; 4] = std::array::from_fn(|i| {
            let command = self.c.shortcuts.command(&name, i);
            let mv = command.and_then(|c| moves.iter().find(|m| m.command == c));
            SlotCard {
                mv,
                selected: self.pause.tab == Tab::Moves
                    && selected.is_some_and(|s| self.c.shortcuts.slot_of(&name, &s.command) == Some(i)),
                bindable: self.pause.tab == Tab::Moves,
            }
        });
        self.pause_slots(&lay, &cards, ease);
        if self.pause.tab == Tab::Moves {
            let slot_of = |command: &str| self.c.shortcuts.slot_of(&name, command);
            self.pause_moves_list(&lay, moves, &slot_of, ease);
        } else {
            self.pause_actions_list(&lay, self.pause.tab.actions(self.mode == Mode::Training), ease);
        }
        self.pause_footer(lay.footer, &lay, self.pause.tab, ease);
        match outcome {
            Outcome::None => {}
            Outcome::Resume => self.session.as_mut().unwrap().paused = false,
            Outcome::Run(action) => self.pause_run(action).await,
        }
    }

    pub(crate) fn pause_preview(&mut self, tab: usize, cur: usize, binds: &[(usize, usize)]) {
        let def = self.session.as_ref().unwrap().m.fighters[0].def.clone();
        for (slot, mv) in binds {
            if let Some(m) = def.movelist.get(*mv) {
                binding::toggle(&mut self.c.shortcuts, &def.display_name, *slot, &m.command);
            }
        }
        self.pause.switch_tab(Tab::ALL[tab % 3]);
        self.pause.cur = cur;
        self.pause.move_cur = cur;
    }

    async fn pause_run(&mut self, action: Action) {
        match action {
            Action::Continue => self.session.as_mut().unwrap().paused = false,
            Action::Dummy => {
                self.dummy_plan = self.dummy_plan.next();
                if let Some(s) = self.session.as_mut() { s.set_dummy_plan(self.dummy_plan); }
            }
            Action::Hitboxes => {
                let s = self.session.as_mut().unwrap();
                s.r.debug = !s.r.debug;
                self.debug_boxes = s.r.debug;
            }
            Action::Restart => {
                self.reset_team();
                self.start_fight().await;
            }
            Action::Change => {
                self.audio.stop_music();
                self.session = None;
                self.team = None;
                self.team_pick.clear();
                self.screen = Screen::Select {
                    cur: [self.sel.0, self.sel.1],
                    picking: 0,
                    picked: [None, None],
                };
            }
            Action::Settings => {
                self.screen = Screen::Settings {
                    tab: 0,
                    cur: 0,
                    from_fight: true,
                }
            }
            Action::MainMenu => {
                self.audio.stop_music();
                self.session = None;
                self.team = None;
                self.screen = Screen::Menu { cur: 0 };
            }
        }
    }
}
