//! Ambience per screen: the menus share one score theme, story cutscenes get another, and music stops elsewhere.
use super::screen::Screen;
use super::App;
use std::cell::Cell;

const MENU_THEME: u32 = 0;
const CUTSCENE_THEME: u32 = 3;

thread_local! {
    static CURRENT_THEME: Cell<Option<u32>> = const { Cell::new(None) };
}

fn theme_for(screen: &Screen) -> Option<u32> {
    match screen {
        Screen::Title
        | Screen::Menu { .. }
        | Screen::StoryCheckpoint { .. }
        | Screen::Select { .. }
        | Screen::Stages { .. }
        | Screen::Controls
        | Screen::CharTuning { .. }
        | Screen::StoryStudio { .. }
        | Screen::EpilogueStudio { .. }
        | Screen::Mugen { .. }
        | Screen::Market
        | Screen::Tower { .. }
        | Screen::Replays { .. } => Some(MENU_THEME),
        Screen::Settings { from_fight, .. } if !from_fight => Some(MENU_THEME),
        Screen::Cutscene { .. } => Some(CUTSCENE_THEME),
        _ => None,
    }
}

impl App {
    pub(crate) async fn menu_bgm_tick(&mut self) {
        let want = theme_for(&self.screen);
        if want == CURRENT_THEME.with(|c| c.get()) {
            return;
        }
        CURRENT_THEME.with(|c| c.set(want));
        match want {
            Some(theme) => {
                self.audio.reset_bgm();
                self.audio.play_music(theme).await;
            }
            None => self.audio.stop_music(),
        }
    }
}
