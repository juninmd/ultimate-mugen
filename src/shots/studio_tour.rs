use super::screen;
use crate::app::{App, Screen};

pub async fn run(app: &mut App, dir: &str) {
    app.studio = None;
    app.screen = Screen::StoryStudio { cur: 5 };
    screen(app, dir, "studio-lista", 20).await;
    app.studio = Some(crate::app::StudioState::open().editing("Wolf: Fique perto. Hoje a rua é nossa.", 1));
    app.screen = Screen::StoryStudio { cur: 7 };
    screen(app, dir, "studio-editando", 20).await;
    app.studio = None;
}
