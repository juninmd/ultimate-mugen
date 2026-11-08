//! Live fight capture for the screenshot tour: `UME_TOUR=duel UME_CHARS=akio,kaname ultimate-mugen --shots <dir> --res 1280x720 --mute`
//! lets two CPU fighters play a real round and saves one frame every `UME_DUEL_STRIDE` ticks (default 3) for `UME_DUEL_FRAMES`
//! frames (default 150) as `<dir>/D-<a>-<b>-<n>.png`. `UME_BOXES=1` draws the hit and hurt boxes, `UME_DUEL_SKIP=<ticks>` skips the
//! first ticks of the round, `UME_STAGE` picks the arena.
use crate::app::{App, Mode};
use crate::gfx::*;
use crate::shots::{find, stage};

fn setting(name: &str, default: usize) -> usize {
    std::env::var(name).ok().and_then(|value| value.parse().ok()).unwrap_or(default)
}

pub async fn run(app: &mut App, dir: &str) {
    let names: Vec<String> = std::env::var("UME_CHARS").unwrap_or_default().split(',').map(|s| s.trim().to_lowercase()).filter(|s| !s.is_empty()).collect();
    let [first, second] = [names.first().cloned().unwrap_or_else(|| "akio".into()), names.get(1).cloned().unwrap_or_else(|| "kaname".into())];
    let arena = std::env::var("UME_STAGE").unwrap_or_else(|_| "dojo".into());
    let (frames, stride, skip) = (setting("UME_DUEL_FRAMES", 150), setting("UME_DUEL_STRIDE", 3).max(1), setting("UME_DUEL_SKIP", 0));
    app.mode = Mode::Watch;
    app.sel = (find(app, &first), find(app, &second), stage(app, &arena));
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
        s.r.debug = std::env::var("UME_BOXES").is_ok();
        for _ in 0..skip {
            s.step_visual(c, audio);
        }
    }
    let mut saved = 0;
    for tick in 0..frames * stride {
        {
            let App { session, c, audio, .. } = app;
            session.as_mut().unwrap().step_visual(c, audio);
        }
        if tick % stride == 0 {
            app.draw_session_only();
            get_screen_data().export_png(&format!("{dir}/D-{first}-{second}-{saved:04}.png"));
            next_frame().await;
            saved += 1;
        }
    }
    let m = &app.session.as_ref().unwrap().m;
    println!("{first} vs {second}: {saved} frames, life {:.0}/{:.0} vs {:.0}/{:.0}, hits taken {} / {}", m.fighters[0].life, m.fighters[0].life_max, m.fighters[1].life, m.fighters[1].life_max, m.fighters[0].received_hits, m.fighters[1].received_hits);
}
