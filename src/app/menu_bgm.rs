//! Menu ambience: the score theme plays while a menu screen is open and stops when leaving the menus.
use super::screen::Screen;
use super::App;
use std::cell::Cell;

const MENU_THEME: u32 = 0;

thread_local! {
    static MENU_MUSIC: Cell<bool> = const { Cell::new(false) };
}

fn is_menu(screen: &Screen) -> bool {
    match screen {
        Screen::Title
        | Screen::Menu { .. }
        | Screen::Select { .. }
        | Screen::Stages { .. }
        | Screen::Controls
        | Screen::CharTuning { .. }
        | Screen::Mugen { .. }
        | Screen::Market
        | Screen::Replays { .. } => true,
        Screen::Settings { from_fight, .. } => !from_fight,
        _ => false,
    }
}

impl App {
    pub(crate) async fn menu_bgm_tick(&mut self) {
        let want = is_menu(&self.screen);
        if want == MENU_MUSIC.with(|m| m.get()) {
            return;
        }
        MENU_MUSIC.with(|m| m.set(want));
        if want {
            self.audio.reset_bgm();
            self.audio.play_music(MENU_THEME).await;
        } else {
            self.audio.stop_music();
        }
    }
}
