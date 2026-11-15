use super::{find, screen, stage};
use crate::app::{App, Mode};

const JAB_STATE: i32 = 200;

pub(crate) async fn run(app: &mut App, dir: &str) {
    let _ = std::fs::create_dir_all(dir);
    app.mode = Mode::Training;
    app.sel = (find(app, "kaito"), find(app, "suzume"), stage(app, "dojo"));
    app.team = None;
    app.start_fight().await;
    {
        let App { session, c, audio, .. } = app;
        let s = session.as_mut().unwrap();
        for _ in 0..600 {
            if s.m.round_state == 2 && s.m.fighters[0].ctrl {
                break;
            }
            s.step_visual(c, audio);
        }
        let center = s.m.camera.x;
        s.m.fighters[0].pos.x = center - 17.0;
        s.m.fighters[1].pos.x = center + 17.0;
        s.m.fighters[0].facing = 1.0;
        s.m.fighters[1].facing = -1.0;
        s.m.fighters[0].power = 1000.0;
        let def = s.m.fighters[0].def.clone();
        s.m.change_state(0, JAB_STATE, def);
        for _ in 0..90 {
            s.step_visual(c, audio);
            if s.m.fighters[0].move_hit > 0 && s.m.fighters[0].move_contact > 0 {
                break;
            }
        }
    }
    screen(app, dir, "rush-cancel-marker", 2).await;
}
