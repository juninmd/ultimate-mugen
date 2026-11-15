use super::{find, screen};
use crate::app::{App, Mode, Screen};
use crate::i18n::{Lang, LANGS};

pub(super) async fn run(app: &mut App, dir: &str) {
    for (lang, _) in LANGS {
        app.settings.lang = lang;
        crate::i18n::set(lang);
        let code = lang.code();
        app.screen = Screen::Menu { cur: 5 };
        screen(app, dir, &format!("{code}-menu"), 40).await;
        app.screen = Screen::Mugen { view: 0, page: 0 };
        screen(app, dir, &format!("{code}-import"), 12).await;
        app.screen = Screen::Market;
        screen(app, dir, &format!("{code}-market"), 12).await;
        app.screen = Screen::Controls;
        screen(app, dir, &format!("{code}-controls"), 12).await;
        app.screen = Screen::Replays { cur: 0 };
        screen(app, dir, &format!("{code}-replays"), 12).await;
        for tab in [0usize, 3, 4] {
            app.screen = Screen::Settings { tab, cur: 1, from_fight: false };
            screen(app, dir, &format!("{code}-settings-{tab}"), 6).await;
        }
        app.mode = Mode::Cpu;
        app.screen = Screen::Select { cur: [find(app, "kaito"), find(app, "akio")], picking: 0, picked: [None, None] };
        screen(app, dir, &format!("{code}-select"), 60).await;
        app.screen = Screen::Stages { cur: 1 };
        screen(app, dir, &format!("{code}-stages"), 8).await;
        app.screen = Screen::Title;
        screen(app, dir, &format!("{code}-title"), 30).await;
    }
    app.settings.lang = Lang::En;
    crate::i18n::set(Lang::En);
}
