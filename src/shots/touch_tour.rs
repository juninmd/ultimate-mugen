use super::{find, screen, stage};
use crate::app::{App, Mode};

const LAYOUT: &str = "[touch]\nscale = 1.2\nopacity = 0.7\nleft_handed = true\n";

pub(crate) async fn run(app: &mut App, dir: &str) {
    let _ = std::fs::create_dir_all(dir);
    app.mode = Mode::Training;
    app.sel = (find(app, "kaito"), find(app, "suzume"), stage(app, "dojo"));
    app.team = None;
    app.c.touch.enabled = true;
    app.start_fight().await;
    screen(app, dir, "touch-default", 90).await;
    crate::storage::write("tweaks.toml", LAYOUT);
    crate::tweaks::reload_now();
    screen(app, dir, "touch-left-handed", 20).await;
    crate::storage::write("tweaks.toml", crate::tweaks::DEFAULT_TEXT);
    crate::tweaks::reload_now();
}
