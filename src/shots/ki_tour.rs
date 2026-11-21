use super::*;

pub(super) async fn run(app: &mut App, dir: &str) {
    for (name, power, transform) in [
        ("h8-ki-ready", 2600.0, false),
        ("h9-ki-half", 1450.0, false),
        ("h10-ki-form", 1600.0, true),
    ] {
        capture(app, dir, name, power, transform).await;
    }
}

async fn capture(app: &mut App, dir: &str, name: &str, power: f32, transform: bool) {
    app.mode = Mode::Cpu;
    app.team = None;
    app.sel = (find(app, "kaito"), find(app, "shion"), stage(app, "dojo"));
    app.start_fight().await;
    {
        let App {
            session, c, audio, ..
        } = app;
        let s = session.as_mut().unwrap();
        for _ in 0..600 {
            if s.m.round_state == 2 && s.m.fighters[0].ctrl {
                break;
            }
            s.step_visual(c, audio);
        }
        for f in s.m.fighters.iter_mut() {
            f.power = power;
        }
        if transform {
            s.m.fighters[0].power = 3000.0;
            s.m.try_transform(0);
            for _ in 0..150 {
                s.step_visual(c, audio);
            }
            s.m.fighters[0].power = power;
        }
        for _ in 0..2 {
            s.step_visual(c, audio);
        }
    }
    snap(app, dir, name, |a| a.draw_session_only()).await;
}
