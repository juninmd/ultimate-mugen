use super::screen;
use crate::app::{App, Screen};

pub async fn run(app: &mut App, dir: &str) {
    app.studio = None;
    app.screen = Screen::StoryStudio { cur: 5 };
    screen(app, dir, "studio-lista", 20).await;
    app.studio = Some(crate::app::StudioState::open().editing("Wolf: Fique perto. Hoje a rua é nossa.", 1));
    app.screen = Screen::StoryStudio { cur: 9 };
    screen(app, dir, "studio-editando", 20).await;
    app.studio = None;
    app.epi_studio = Some(crate::app::EpiState::open());
    app.screen = Screen::EpilogueStudio { cur: 1 };
    screen(app, dir, "epilogo-lista", 20).await;
    app.epi_studio = Some(crate::app::EpiState::open().editing("Dawn broke over the city and the shutters opened on a quiet street.", "kaito"));
    app.screen = Screen::EpilogueStudio { cur: 5 };
    screen(app, dir, "epilogo-editando", 20).await;
    app.epi_studio = None;
}
