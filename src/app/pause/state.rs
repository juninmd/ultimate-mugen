use super::layout::Layout;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tab {
    Game,
    Moves,
    System,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Action {
    Continue,
    Restart,
    Change,
    Hitboxes,
    Settings,
    MainMenu,
}

impl Tab {
    pub const ALL: [Tab; 3] = [Tab::Game, Tab::Moves, Tab::System];

    pub fn index(self) -> usize {
        Tab::ALL.iter().position(|t| *t == self).unwrap_or(0)
    }

    pub fn step(self, delta: i32) -> Tab {
        let n = Tab::ALL.len() as i32;
        Tab::ALL[(self.index() as i32 + delta).rem_euclid(n) as usize]
    }

    pub fn actions(self) -> &'static [Action] {
        match self {
            Tab::Game => &[Action::Continue, Action::Restart, Action::Change],
            Tab::Moves => &[],
            Tab::System => &[Action::Hitboxes, Action::Settings, Action::MainMenu],
        }
    }
}

pub struct PauseState {
    pub tab: Tab,
    pub cur: usize,
    pub move_cur: usize,
    pub scroll: f32,
    pub open: f32,
    pub flash: [f32; 4],
    pub toast: Option<(String, f32)>,
    pub drag: Option<f32>,
    pub follow: bool,
}

pub fn follow_target(scroll: f32, cur: usize, lay: &Layout) -> f32 {
    let top = cur as f32 * lay.pitch;
    let bottom = top + lay.row_h;
    if top < scroll {
        top
    } else if bottom > scroll + lay.list.h {
        bottom - lay.list.h
    } else {
        scroll
    }
}

impl PauseState {
    pub fn new() -> Self {
        PauseState {
            tab: Tab::Game,
            cur: 0,
            move_cur: 0,
            scroll: 0.0,
            open: 0.0,
            flash: [0.0; 4],
            toast: None,
            drag: None,
            follow: true,
        }
    }

    pub fn reset(&mut self) {
        *self = PauseState::new();
    }

    pub fn switch_tab(&mut self, tab: Tab) {
        if tab != self.tab {
            self.tab = tab;
            self.cur = 0;
            self.drag = None;
        }
    }

    pub fn tick(&mut self, dt: f32) {
        self.open = (self.open + dt * 6.0).min(1.0);
        for f in &mut self.flash {
            *f = (*f - dt * 2.2).max(0.0);
        }
        if let Some((_, t)) = &mut self.toast {
            *t -= dt;
        }
        if self.toast.as_ref().is_some_and(|(_, t)| *t <= 0.0) {
            self.toast = None;
        }
    }

    pub fn settle_scroll(&mut self, lay: &Layout, total: usize) {
        if self.follow {
            let target = follow_target(self.scroll, self.move_cur, lay);
            self.scroll += (target - self.scroll) * 0.4;
            if (target - self.scroll).abs() < 0.5 {
                self.scroll = target;
            }
        }
        self.scroll = self.scroll.clamp(0.0, lay.max_scroll(total));
    }

    pub fn ease(&self) -> f32 {
        1.0 - (1.0 - self.open).powi(3)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tabs_wrap_in_both_directions() {
        assert_eq!(Tab::Game.step(-1), Tab::System);
        assert_eq!(Tab::System.step(1), Tab::Game);
        assert_eq!(Tab::Game.step(1), Tab::Moves);
    }

    #[test]
    fn every_action_is_reachable_once() {
        let all: Vec<Action> = Tab::ALL.iter().flat_map(|t| t.actions().iter().copied()).collect();
        assert_eq!(all.len(), 6);
        for a in &all {
            assert_eq!(all.iter().filter(|b| *b == a).count(), 1);
        }
    }

    #[test]
    fn switching_tab_resets_the_cursor_only_when_it_changes() {
        let mut s = PauseState::new();
        s.cur = 2;
        s.switch_tab(Tab::Game);
        assert_eq!(s.cur, 2);
        s.switch_tab(Tab::System);
        assert_eq!(s.cur, 0);
    }

    #[test]
    fn cursor_scrolling_keeps_the_row_visible() {
        let lay = Layout::new(1.0);
        let mut s = PauseState::new();
        s.move_cur = 30;
        for _ in 0..60 {
            s.settle_scroll(&lay, 40);
        }
        let top = 30.0 * lay.pitch;
        assert!(top >= s.scroll && top + lay.row_h <= s.scroll + lay.list.h + 0.5);
        s.move_cur = 0;
        for _ in 0..60 {
            s.settle_scroll(&lay, 40);
        }
        assert_eq!(s.scroll, 0.0);
    }

    #[test]
    fn manual_scroll_is_clamped_and_not_dragged_back() {
        let lay = Layout::new(1.0);
        let mut s = PauseState::new();
        s.follow = false;
        s.scroll = 9999.0;
        s.settle_scroll(&lay, 40);
        assert_eq!(s.scroll, lay.max_scroll(40));
        s.scroll = -50.0;
        s.settle_scroll(&lay, 40);
        assert_eq!(s.scroll, 0.0);
    }

    #[test]
    fn open_animation_settles_and_toast_expires() {
        let mut s = PauseState::new();
        s.toast = Some(("x".into(), 0.5));
        for _ in 0..60 {
            s.tick(1.0 / 60.0);
        }
        assert_eq!(s.ease(), 1.0);
        assert!(s.toast.is_none());
    }
}
