use super::{find, frames, snap, stage};
use crate::app::{App, Mode};

/// Replay determinism, end to end through the app: fight N ticks, replay the recording, compare the final game state
pub(crate) async fn replay_determinism(app: &mut App, dir: &str) {
    fn hash(s: &crate::session::Session) -> u64 {
        let mut h = 1469598103934665603u64;
        for f in &s.m.fighters {
            for v in [
                f.life.to_bits() as u64,
                f.power.to_bits() as u64,
                f.pos.x.to_bits() as u64,
                f.pos.y.to_bits() as u64,
                f.state_no as u64,
                f.state_time as u64,
            ] {
                h ^= v;
                h = h.wrapping_mul(1099511628211);
            }
        }
        h
    }
    app.mode = Mode::Watch;
    app.sel = (find(app, "akio"), find(app, "lian"), stage(app, "plains"));
    app.team = None;
    app.replaying = None;
    app.start_fight().await;
    let h1 = {
        let App {
            session, c, audio, ..
        } = app;
        let s = session.as_mut().unwrap();
        for _ in 0..700 {
            s.step_visual(c, audio);
        }
        hash(s)
    };
    app.capture_replay();
    app.start_replay().await;
    let (h2, is_replay) = {
        let App {
            session, c, audio, ..
        } = app;
        let s = session.as_mut().unwrap();
        for _ in 0..700 {
            s.step_visual(c, audio);
        }
        (hash(s), s.is_replay)
    };
    println!("replay-check: is_replay={is_replay} identical={}", h1 == h2);
    for _ in 0..4 {
        frames(app, 1).await;
    }
    snap(app, dir, "43-replay", |a| a.draw_session_only()).await;
    app.replaying = None;
}
