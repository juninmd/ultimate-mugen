use super::screen;
use crate::app::{App, Screen};

pub(super) async fn run(app: &mut App, dir: &str) {
    for zip in std::env::var("UME_ROSTER_ZIPS").unwrap_or_default().split(':').filter(|z| !z.is_empty()) {
        if let Ok(bytes) = std::fs::read(zip) {
            let _ = app.reg.import_zip(&bytes);
        }
    }
    app.screen = Screen::Mugen { view: 0, page: 0 };
    app.roster_tour_focus(0, false, false);
    screen(app, dir, "roster-mugen", 40).await;
    app.roster_tour_focus(3, false, true);
    screen(app, dir, "roster-all", 40).await;
    app.roster_tour_focus(5, true, false);
    screen(app, dir, "sheet-mugen", 90).await;
    app.roster_tour_focus(4, true, true);
    screen(app, dir, "sheet-native", 60).await;
    app.roster_tour_focus(21, false, false);
    screen(app, dir, "roster-page2", 40).await;
}
