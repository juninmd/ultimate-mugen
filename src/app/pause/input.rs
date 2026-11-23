use super::binding::{fresh_slot, held_slots, ok_slot, toggle, Bound};
use super::layout::Layout;
use super::state::{Action, Tab};
use crate::app::*;
use ume_engine::chardef::MoveInfo;

pub(super) enum Outcome {
    None,
    Resume,
    Run(Action),
}

impl App {
    pub(super) fn pause_input(&mut self, lay: &Layout, name: &str, moves: &[MoveInfo], accept_nav: bool) -> Outcome {
        let held = held_slots(self.c.pad_info.iter().map(|p| p.bits), is_key_down);
        let key_slot = fresh_slot(held, self.c.shortcuts.prev[0]);
        self.c.shortcuts.prev[0] = held;
        for (i, t) in Tab::ALL.iter().enumerate() {
            let r = lay.tabs[i];
            if self.clicked_rect(r.x, r.y, r.w, r.h) {
                self.pause_set_tab(*t);
            }
        }
        if self.c.pressed(nav::LEFT) {
            self.pause_set_tab(self.pause.tab.step(-1));
        }
        if self.c.pressed(nav::RIGHT) {
            self.pause_set_tab(self.pause.tab.step(1));
        }
        let mut slot = key_slot;
        for (i, r) in lay.slots.iter().enumerate() {
            if self.clicked_rect(r.x, r.y, r.w, r.h) {
                if self.pause.tab == Tab::Moves {
                    slot = Some(i);
                } else {
                    self.pause_set_tab(Tab::Moves);
                }
            }
        }
        let touch_ok = accept_nav && self.c.touch.enabled && self.c.pressed(nav::OK);
        if slot.is_none() && touch_ok && self.pause.tab == Tab::Moves {
            slot = moves.get(self.pause.move_cur).map(|m| ok_slot(&self.c.shortcuts, name, &m.command));
        }
        if self.pause.tab == Tab::Moves {
            self.pause_moves_input(lay, name, moves, slot);
        } else if let Some(out) = self.pause_actions_input(lay, accept_nav) {
            return out;
        }
        if accept_nav && self.c.pressed(nav::BACK) {
            return Outcome::Resume;
        }
        Outcome::None
    }

    fn pause_set_tab(&mut self, tab: Tab) {
        if tab != self.pause.tab {
            self.audio.sfx("select");
        }
        self.pause.switch_tab(tab);
    }

    fn pause_actions_input(&mut self, lay: &Layout, accept_nav: bool) -> Option<Outcome> {
        let actions = self.pause.tab.actions(self.mode == super::super::Mode::Training);
        let n = actions.len();
        if self.c.pressed(nav::DOWN) {
            self.pause.cur = (self.pause.cur + 1) % n;
        }
        if self.c.pressed(nav::UP) {
            self.pause.cur = (self.pause.cur + n - 1) % n;
        }
        let mut ok = accept_nav && self.c.pressed(nav::OK);
        for i in 0..n {
            let r = lay.action_row(i, actions.len());
            if self.clicked_rect(r.x, r.y, r.w, r.h) {
                self.pause.cur = i;
                ok = true;
            }
        }
        ok.then(|| {
            let act = actions[self.pause.cur.min(n - 1)];
            self.audio.sfx("confirm");
            if act == Action::Continue {
                Outcome::Resume
            } else {
                Outcome::Run(act)
            }
        })
    }

    fn pause_moves_input(&mut self, lay: &Layout, name: &str, moves: &[MoveInfo], slot: Option<usize>) {
        let total = moves.len();
        if total == 0 {
            return;
        }
        let before = self.pause.move_cur;
        if self.c.pressed(nav::DOWN) {
            self.pause.move_cur = (self.pause.move_cur + 1) % total;
        }
        if self.c.pressed(nav::UP) {
            self.pause.move_cur = (self.pause.move_cur + total - 1) % total;
        }
        for i in 0..total {
            let r = lay.move_row(i, self.pause.scroll);
            if lay.list.contains(r.x + 4.0, r.y + r.h / 2.0) && self.clicked_rect(r.x, r.y, r.w, r.h) {
                self.pause.move_cur = i;
            }
        }
        self.pause.move_cur = self.pause.move_cur.min(total - 1);
        let wheel = mouse_wheel().1;
        if wheel != 0.0 {
            self.pause.scroll -= wheel.signum() * lay.pitch * 1.5;
            self.pause.follow = false;
        }
        self.pause_drag(lay);
        if self.pause.move_cur != before {
            self.pause.follow = true;
        }
        self.pause.settle_scroll(lay, total);
        if let Some(slot) = slot {
            let mv = &moves[self.pause.move_cur];
            let bound = toggle(&mut self.c.shortcuts, name, slot, &mv.command);
            self.c.shortcuts.save();
            self.audio.sfx("select");
            let (index, text) = match bound {
                Bound::Set(i) => (i, format!("{} -> {}", crate::i18n::fighter_lines::shown(&mv.name), i + 1)),
                Bound::Cleared(i) => (i, format!("{} x {}", crate::i18n::fighter_lines::shown(&mv.name), i + 1)),
            };
            self.pause.flash[index] = 1.0;
            self.pause.toast = Some((text, 2.2));
        }
    }

    fn pause_drag(&mut self, lay: &Layout) {
        let Some(pos) = self.drag_pointer() else {
            self.pause.drag = None;
            return;
        };
        match self.pause.drag {
            Some(last) => {
                self.pause.scroll -= pos.y - last;
                if (pos.y - last).abs() > 0.5 {
                    self.pause.follow = false;
                }
                self.pause.drag = Some(pos.y);
            }
            None if lay.list.contains(pos.x, pos.y) => self.pause.drag = Some(pos.y),
            None => {}
        }
    }

    fn drag_pointer(&self) -> Option<Vec2> {
        let dpi = dpi_scale().max(0.01);
        let finger = touches()
            .into_iter()
            .find(|t| !matches!(t.phase, TouchPhase::Ended | TouchPhase::Cancelled));
        match finger {
            Some(t) => Some(self.ui.to_virtual(t.position / dpi)),
            None if is_mouse_button_down(MouseButton::Left) => Some(self.ui.to_virtual(mouse_position().into())),
            None => None,
        }
    }
}
