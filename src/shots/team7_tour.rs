use super::{find, screen, stage};
use crate::app::{App, Mode, Team};

const SIDE_A: [&str; 7] = ["akio", "bruno", "daichi", "garan", "haruki", "jin", "kaito"];
const SIDE_B: [&str; 7] = [
    "kaname", "lian", "marlow", "shion", "suzume", "sylene", "valdric",
];

pub(crate) async fn run(app: &mut App, dir: &str) {
    let _ = std::fs::create_dir_all(dir);
    let stage_id = std::env::var("UME_STAGE").unwrap_or_else(|_| "dojo".into());
    let team_a: Vec<usize> = SIDE_A.iter().map(|c| find(app, c)).collect();
    let team_b: Vec<usize> = SIDE_B.iter().map(|c| find(app, c)).collect();
    app.mode = Mode::TeamSimul;
    app.team_n = 7;
    app.team = Some(Team::new(team_a.clone(), team_b.clone(), true));
    app.sel = (team_a[0], team_b[0], stage(app, &stage_id));
    app.start_fight().await;
    {
        let App {
            session, c, audio, ..
        } = app;
        let s = session.as_mut().unwrap();
        for _ in 0..30 {
            s.step_visual(c, audio);
        }
        let xs: Vec<String> =
            s.m.fighters
                .iter()
                .filter(|f| !f.is_helper)
                .map(|f| format!("{:.0}", f.pos.x))
                .collect();
        println!("team7: spawn x {}", xs.join(" "));
    }
    screen(app, dir, "team7-spawn", 2).await;
    {
        let App {
            session, c, audio, ..
        } = app;
        let s = session.as_mut().unwrap();
        for _ in 0..600 {
            s.step_visual(c, audio);
        }
    }
    screen(app, dir, "team7-fight", 2).await;
}
