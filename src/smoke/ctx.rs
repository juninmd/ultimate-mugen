use super::report::{self, Check, FeatureReport};
use super::capture;
use crate::app::App;
use crate::gfx::next_frame;

pub struct Ctx {
    pub out: String,
    pub features: Vec<FeatureReport>,
    current: Option<FeatureReport>,
    started_at: f64,
    frame_sum: f64,
}

impl Ctx {
    pub fn new(out: &str) -> Self {
        Ctx { out: out.to_string(), features: vec![], current: None, started_at: 0.0, frame_sum: 0.0 }
    }

    pub fn begin(&mut self, id: &'static str, title: &'static str) {
        self.started_at = crate::perf_clock();
        self.frame_sum = 0.0;
        self.current = Some(FeatureReport { id, title, ms: 0.0, frames: 0, frame_avg_ms: 0.0, frame_max_ms: 0.0, checks: vec![] });
        self.flush(false, None);
    }

    pub fn end(&mut self) {
        let Some(mut feature) = self.current.take() else { return };
        feature.ms = crate::perf_clock() - self.started_at;
        feature.frame_avg_ms = if feature.frames == 0 { 0.0 } else { self.frame_sum / feature.frames as f64 };
        let budget = frame_budget();
        if feature.frames > 0 {
            let ok = feature.frame_avg_ms <= budget;
            feature.checks.push(Check { name: "frame time".into(), ok, detail: format!("avg {:.1} ms, max {:.0} ms, budget {budget:.0} ms avg", feature.frame_avg_ms, feature.frame_max_ms), shot: None });
        }
        self.features.push(feature);
        self.flush(false, None);
    }

    pub fn check(&mut self, name: &str, ok: bool, detail: impl Into<String>) {
        self.push(Check { name: name.into(), ok, detail: detail.into(), shot: None });
    }

    fn push(&mut self, check: Check) {
        if let Some(f) = self.current.as_mut() {
            println!("[smoke] {} {}: {}", if check.ok { "ok  " } else { "FAIL" }, check.name, check.detail);
            f.checks.push(check);
        }
        self.flush(false, None);
    }

    pub async fn frames(&mut self, app: &mut App, n: usize) {
        for _ in 0..n {
            let t0 = crate::perf_clock();
            app.frame().await;
            next_frame().await;
            let dt = crate::perf_clock() - t0;
            self.frame_sum += dt;
            if let Some(f) = self.current.as_mut() {
                f.frames += 1;
                f.frame_max_ms = f.frame_max_ms.max(dt);
            }
        }
    }

    pub async fn shot(&mut self, app: &mut App, name: &str, warm: usize) {
        self.frames(app, warm).await;
        app.trans = 0.0;
        app.frame().await;
        let id = self.current.as_ref().map(|f| f.id).unwrap_or("smoke");
        let file = format!("shots/{id}-{name}.png");
        #[cfg(not(target_arch = "wasm32"))]
        let _ = std::fs::create_dir_all(format!("{}/shots", self.out));
        let (ok, detail) = match capture::take(app, &self.out, &file).await {
            Ok(Some(stats)) => (!stats.looks_blank(), stats.describe()),
            Ok(None) => (true, "captured by the driver".to_string()),
            Err(e) => (false, e),
        };
        let detail = if ok { detail } else { format!("blank or missing frame: {detail}") };
        self.push(Check { name: format!("{name} renders"), ok, detail, shot: Some(file) });
    }

    pub fn flush(&self, done: bool, panic: Option<&str>) {
        let running = self.current.as_ref().map(|f| f.id);
        let mut all: Vec<FeatureReport> = vec![];
        all.extend(self.features.iter().map(clone_feature));
        if let Some(f) = self.current.as_ref() {
            all.push(clone_feature(f));
        }
        let json = report::to_json(&all, running, done, panic);
        report::remember(&json);
        capture::publish(&json.to_string());
    }
}

fn clone_feature(f: &FeatureReport) -> FeatureReport {
    FeatureReport {
        id: f.id,
        title: f.title,
        ms: f.ms,
        frames: f.frames,
        frame_avg_ms: f.frame_avg_ms,
        frame_max_ms: f.frame_max_ms,
        checks: f.checks.iter().map(|c| Check { name: c.name.clone(), ok: c.ok, detail: c.detail.clone(), shot: c.shot.clone() }).collect(),
    }
}

fn frame_budget() -> f64 {
    #[cfg(not(target_arch = "wasm32"))]
    if let Some(v) = std::env::var("UME_SMOKE_FRAME_BUDGET_MS").ok().and_then(|v| v.parse().ok()) {
        return v;
    }
    500.0
}
