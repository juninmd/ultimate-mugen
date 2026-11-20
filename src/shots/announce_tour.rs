use super::{find, screen, stage};
use crate::app::{App, Mode};
use crate::i18n::{self, Lang};
use ume_engine::matchs::AnnounceKind;

fn step_until(app: &mut App, kind: AnnounceKind, max: usize) {
    let App { session, c, audio, .. } = app;
    let s = session.as_mut().unwrap();
    for _ in 0..max {
        if s.m.announce.as_ref().is_some_and(|a| a.kind == kind) {
            break;
        }
        s.step_visual(c, audio);
    }
}

pub(crate) async fn run(app: &mut App, dir: &str) {
    let _ = std::fs::create_dir_all(dir);
    for (lang, code) in [(Lang::En, "en"), (Lang::Pt, "pt"), (Lang::Es, "es")] {
        app.settings.lang = lang;
        i18n::set(lang);
        app.mode = Mode::Cpu;
        app.team = None;
        app.sel = (find(app, "kaito"), find(app, "akio"), stage(app, "dojo"));
        app.start_fight().await;
        step_until(app, AnnounceKind::Round, 900);
        screen(app, dir, &format!("{code}-round"), 1).await;
        step_until(app, AnnounceKind::Fight, 300);
        screen(app, dir, &format!("{code}-fight"), 1).await;
        {
            let App { session, c, audio, .. } = app;
            let s = session.as_mut().unwrap();
            s.m.fighters[1].life = 0.0;
            for _ in 0..900 {
                if s.m.announce.as_ref().is_some_and(|a| a.kind == AnnounceKind::Ko) {
                    break;
                }
                s.step_visual(c, audio);
            }
        }
        screen(app, dir, &format!("{code}-ko"), 1).await;
        step_until(app, AnnounceKind::Win, 1500);
        screen(app, dir, &format!("{code}-win"), 1).await;
    }
    app.settings.lang = Lang::En;
    i18n::set(Lang::En);
}
