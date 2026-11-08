use crate::app::{App, Mode, Team};
use crate::gfx::*;
use crate::shots::{find, stage};
use std::time::Instant;

struct Samples(Vec<f32>);

impl Samples {
    fn new() -> Self {
        Samples(Vec::new())
    }
    fn push(&mut self, ms: f32) {
        self.0.push(ms);
    }
    fn pct(&self, p: f32) -> f32 {
        if self.0.is_empty() {
            return 0.0;
        }
        let mut v = self.0.clone();
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        v[((v.len() - 1) as f32 * p) as usize]
    }
    fn mean(&self) -> f32 {
        if self.0.is_empty() {
            0.0
        } else {
            self.0.iter().sum::<f32>() / self.0.len() as f32
        }
    }
    fn clear(&mut self) {
        self.0.clear();
    }
}

pub async fn run(app: &mut App, _dir: &str) {
    if std::env::var("UME_PERF_ROTATE").is_ok() {
        rotate(app).await;
        return;
    }
    let chars: Vec<String> = std::env::var("UME_CHARS")
        .unwrap_or_else(|_| "kaito,shion".into())
        .split(',')
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect();
    let stage_id = std::env::var("UME_STAGE").unwrap_or_else(|_| "dojo".into());
    let seconds: usize = std::env::var("UME_PERF_SECONDS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(10);
    let frames = seconds * 60;
    let (a, b) = (find(app, &chars[0]), find(app, &chars[1]));
    let per_side = (chars.len() / 2).max(1);
    app.team = (per_side > 1).then(|| {
        Team::new(
            chars[..per_side].iter().map(|c| find(app, c)).collect(),
            chars[per_side..per_side * 2]
                .iter()
                .map(|c| find(app, c))
                .collect(),
            true,
        )
    });
    app.mode = if app.team.is_some() {
        Mode::TeamSimul
    } else {
        Mode::Watch
    };
    app.sel = (a, b, stage(app, &stage_id));
    let load = Instant::now();
    app.start_fight().await;
    println!(
        "perf: load {:.1} ms · {} vs {} · {} · rss {:.0} MB",
        load.elapsed().as_secs_f64() * 1000.0,
        app.reg.chars[a].id,
        app.reg.chars[b].id,
        stage_id,
        rss_mb()
    );
    let mut step = Samples::new();
    let mut draw = Samples::new();
    let mut frame = Samples::new();
    let mut all_step = Samples::new();
    let mut all_draw = Samples::new();
    let mut all_frame = Samples::new();
    let mut uploads = 0f64;
    let mut upload_kb = 0f64;
    let mut vertices = 0f64;
    let mut calls = 0f64;
    let (mut all_uploads, mut all_upload_kb, mut all_vertices, mut all_calls) =
        (0f64, 0f64, 0f64, 0f64);
    let (mut fill_rgba, mut fill_index) = (0f32, 0f32);
    for i in 0..frames {
        let t = Instant::now();
        {
            let App {
                session, c, audio, ..
            } = app;
            session.as_mut().unwrap().step_visual(c, audio);
        }
        step.push(t.elapsed().as_secs_f32() * 1000.0);
        all_step.push(t.elapsed().as_secs_f32() * 1000.0);
        let t = Instant::now();
        app.draw_session_only();
        draw.push(t.elapsed().as_secs_f32() * 1000.0);
        all_draw.push(t.elapsed().as_secs_f32() * 1000.0);
        next_frame().await;
        let st = stats();
        uploads += st.uploads as f64;
        upload_kb += st.upload_kb as f64;
        vertices += st.vertices as f64;
        calls += st.draw_calls as f64;
        all_uploads += st.uploads as f64;
        all_upload_kb += st.upload_kb as f64;
        all_vertices += st.vertices as f64;
        all_calls += st.draw_calls as f64;
        fill_rgba = st.rgba_fill;
        fill_index = st.index_fill;
        frame.push(t.elapsed().as_secs_f32() * 1000.0);
        all_frame.push(t.elapsed().as_secs_f32() * 1000.0);
        if (i + 1) % 60 == 0 {
            let n = step.0.len() as f64;
            println!(
                "perf: {}s step p50 {:.2} p99 {:.2} max {:.2} · draw p50 {:.2} p99 {:.2} max {:.2} · frame p50 {:.2} p99 {:.2} max {:.2} ms · {:.0} v {:.1} dc {:.0} up {:.0} KB · atlas {:.0}%/{:.0}% · rss {:.0} MB",
                (i + 1) / 60, step.pct(0.5), step.pct(0.99), step.pct(1.0), draw.pct(0.5), draw.pct(0.99), draw.pct(1.0), frame.pct(0.5), frame.pct(0.99), frame.pct(1.0),
                vertices / n, calls / n, uploads / n, upload_kb / n, fill_rgba * 100.0, fill_index * 100.0, rss_mb()
            );
            step.clear();
            draw.clear();
            frame.clear();
            uploads = 0.0;
            upload_kb = 0.0;
            vertices = 0.0;
            calls = 0.0;
        }
    }
    let n = frames as f64;
    println!("perf: step avg {:.2} ms p99 {:.2} · draw avg {:.2} ms p99 {:.2} · frame avg {:.2} ms p99 {:.2} · {:.0} v {:.1} dc {:.0} up {:.0} KB · rss {:.0} MB",
        all_step.mean(), all_step.pct(0.99), all_draw.mean(), all_draw.pct(0.99), all_frame.mean(), all_frame.pct(0.99),
        all_vertices / n, all_calls / n, all_uploads / n, all_upload_kb / n, rss_mb());
}

async fn rotate(app: &mut App) {
    let stage_id = std::env::var("UME_STAGE").unwrap_or_else(|_| "dojo".into());
    let roster: Vec<usize> = (0..app.reg.chars.len())
        .filter(|&i| !app.reg.chars[i].mugen && !app.reg.chars[i].boss)
        .collect();
    println!(
        "perf: rotate start · {} fighters · rss {:.0} MB",
        roster.len(),
        rss_mb()
    );
    app.mode = Mode::Watch;
    app.team = None;
    for k in 0..roster.len() / 2 {
        let (a, b) = (roster[k * 2], roster[k * 2 + 1]);
        app.sel = (a, b, stage(app, &stage_id));
        let load = Instant::now();
        app.start_fight().await;
        for _ in 0..2 {
            app.draw_session_only();
            next_frame().await;
        }
        println!(
            "perf: rotate {} {} vs {} load {:.0} ms · rss {:.0} MB",
            k,
            app.reg.chars[a].id,
            app.reg.chars[b].id,
            load.elapsed().as_secs_f64() * 1000.0,
            rss_mb()
        );
    }
}

fn rss_mb() -> f64 {
    #[cfg(windows)]
    {
        #[repr(C)]
        struct Counters {
            cb: u32,
            fault: u32,
            peak_ws: usize,
            ws: usize,
            _rest: [usize; 6],
        }
        extern "system" {
            fn GetCurrentProcess() -> isize;
            fn K32GetProcessMemoryInfo(p: isize, c: *mut Counters, cb: u32) -> i32;
        }
        let mut c = Counters {
            cb: std::mem::size_of::<Counters>() as u32,
            fault: 0,
            peak_ws: 0,
            ws: 0,
            _rest: [0; 6],
        };
        if unsafe { K32GetProcessMemoryInfo(GetCurrentProcess(), &mut c, c.cb) } == 0 {
            0.0
        } else {
            c.ws as f64 / (1024.0 * 1024.0)
        }
    }
    #[cfg(not(windows))]
    {
        0.0
    }
}
