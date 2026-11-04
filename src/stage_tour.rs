//! Stage showcase for the screenshot tour: `UME_SHOTS_ONLY=stages xvfb-run -a cargo run --release -- --shots <dir> --res 1280x720 --mute`
//! renders every built-in stage calm, after a heavy hit + landing, during a super and after a KO (impulses fed through `StageReact::react`).
//! `UME_STAGE=<id>` limits the tour to one stage.
use crate::app::{App, Mode};
use crate::gfx::*;
use crate::shots::{find, stage};
use crate::stagefx::{Impulse, ImpulseKind};

async fn draw_frames(app: &mut App, n: usize, shot: Option<String>) {
    let mut tot = 0.0;
    for i in 0..n {
        app.draw_session_only();
        if i + 1 == n {
            if let Some(p) = &shot {
                get_screen_data().export_png(p);
                println!("shot {p}");
            }
        }
        if i >= 4 {
            tot += get_frame_time();
        }
        next_frame().await;
    }
    println!("  avg frame {:.1} ms", tot / (n.max(5) - 4) as f32 * 1000.0);
}

pub async fn run(app: &mut App, dir: &str) {
    let only = std::env::var("UME_STAGE").ok();
    for (st, p1, p2) in [
        ("dojo", "kaito", "shion"),
        ("neon", "volta", "suzume"),
        ("bamboo", "lian", "sylene"),
        ("plains", "daichi", "jin"),
        ("spirit", "akio", "bruno"),
        ("volcano", "garan", "marlow"),
        ("brazil", "bruno", "lian"),
    ] {
        if only.as_deref().is_some_and(|o| o != st) {
            continue;
        }
        app.mode = Mode::Watch;
        app.sel = (find(app, p1), find(app, p2), stage(app, st));
        let t0 = std::time::Instant::now();
        app.start_fight().await;
        println!("start_fight {st}: {:.2}s", t0.elapsed().as_secs_f32());
        {
            let App {
                session, c, audio, ..
            } = app;
            let s = session.as_mut().unwrap();
            for _ in 0..240 {
                s.step_visual(c, audio);
            }
        }
        if st == "dojo" {
            // no explicit impulses: check the automatic detection from the Match (hits, landings, supers, KO)
            let before = app.session.as_ref().unwrap().r.fx.applied;
            for k in 0..900 {
                {
                    let App {
                        session, c, audio, ..
                    } = app;
                    session.as_mut().unwrap().step_visual(c, audio);
                }
                if k % 2 == 0 {
                    app.draw_session_only();
                    next_frame().await;
                }
            }
            println!(
                "auto-detected impulses on {st}: {}",
                app.session.as_ref().unwrap().r.fx.applied - before
            );
        }
        draw_frames(
            app,
            std::env::var("UME_FRAMES")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(40),
            Some(format!("{dir}/S-{st}-1-calm.png")),
        )
        .await;
        let (x0, x1) = {
            let s = app.session.as_ref().unwrap();
            (s.m.fighters[0].pos.x, s.m.fighters[1].pos.x)
        };
        app.session.as_mut().unwrap().r.fx.react(&[
            Impulse {
                x: x1,
                strength: 0.95,
                kind: ImpulseKind::Hit,
            },
            Impulse {
                x: x0,
                strength: 0.8,
                kind: ImpulseKind::Land,
            },
        ]);
        draw_frames(app, 12, Some(format!("{dir}/S-{st}-2-hit.png"))).await;
        app.session.as_mut().unwrap().r.fx.react(&[Impulse {
            x: x0,
            strength: 1.0,
            kind: ImpulseKind::Super,
        }]);
        draw_frames(app, 14, Some(format!("{dir}/S-{st}-3-super.png"))).await;
        app.session.as_mut().unwrap().r.fx.react(&[Impulse {
            x: x1,
            strength: 1.0,
            kind: ImpulseKind::Ko,
        }]);
        draw_frames(app, 26, Some(format!("{dir}/S-{st}-4-ko.png"))).await;
        {
            let m = &mut app.session.as_mut().unwrap().m;
            m.camera.y = m.stage.bound_high;
            m.camera.x = m.stage.bound_left;
        }
        draw_frames(app, 2, Some(format!("{dir}/S-{st}-5-camera-high-left.png"))).await;
        {
            let m = &mut app.session.as_mut().unwrap().m;
            m.camera.y = m.stage.bound_low;
            m.camera.x = m.stage.bound_right;
        }
        draw_frames(app, 2, Some(format!("{dir}/S-{st}-6-camera-low-right.png"))).await;
    }
}
