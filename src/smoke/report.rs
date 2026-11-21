use serde_json::{json, Value};
use std::sync::Mutex;

pub struct Check {
    pub name: String,
    pub ok: bool,
    pub detail: String,
    pub shot: Option<String>,
}

pub struct FeatureReport {
    pub id: &'static str,
    pub title: &'static str,
    pub ms: f64,
    pub frames: usize,
    pub frame_avg_ms: f64,
    pub frame_max_ms: f64,
    pub checks: Vec<Check>,
}

impl FeatureReport {
    pub fn ok(&self) -> bool {
        !self.checks.is_empty() && self.checks.iter().all(|c| c.ok)
    }
}

static LATEST: Mutex<String> = Mutex::new(String::new());

pub fn to_json(features: &[FeatureReport], running: Option<&str>, done: bool, panic: Option<&str>) -> Value {
    let failed = features.iter().filter(|f| !f.ok()).count();
    json!({
        "schema": 1,
        "platform": platform(),
        "done": done,
        "running": running,
        "panic": panic,
        "passed": panic.is_none() && failed == 0 && done,
        "features": features.iter().map(feature_json).collect::<Vec<_>>(),
    })
}

fn feature_json(f: &FeatureReport) -> Value {
    json!({
        "id": f.id,
        "title": f.title,
        "ok": f.ok(),
        "ms": f.ms.round(),
        "frames": f.frames,
        "frame_avg_ms": (f.frame_avg_ms * 10.0).round() / 10.0,
        "frame_max_ms": f.frame_max_ms.round(),
        "checks": f.checks.iter().map(|c| json!({"name": c.name, "ok": c.ok, "detail": c.detail, "shot": c.shot})).collect::<Vec<_>>(),
    })
}

pub fn platform() -> String {
    format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH)
}

pub fn remember(json: &Value) {
    if let Ok(mut latest) = LATEST.lock() {
        *latest = json.to_string();
    }
}

pub fn latest() -> String {
    LATEST.lock().map(|l| l.clone()).unwrap_or_default()
}

pub fn markdown(report: &Value) -> String {
    let verdict = if report["passed"].as_bool().unwrap_or(false) { "PASS" } else { "FAIL" };
    let mut out = format!("# Smoke report: {verdict}\n\nPlatform: {}\n\n", report["platform"].as_str().unwrap_or("?"));
    if let Some(p) = report["panic"].as_str() {
        out.push_str(&format!("**Panic during `{}`:** {p}\n\n", report["running"].as_str().unwrap_or("?")));
    }
    out.push_str("| Feature | Result | Frames | Avg ms | Max ms |\n|---|---|---|---|---|\n");
    let features = report["features"].as_array().cloned().unwrap_or_default();
    for f in &features {
        let mark = if f["ok"].as_bool().unwrap_or(false) { "pass" } else { "FAIL" };
        out.push_str(&format!("| {} | {mark} | {} | {} | {} |\n", f["title"].as_str().unwrap_or(""), f["frames"], f["frame_avg_ms"], f["frame_max_ms"]));
    }
    for f in features.iter().filter(|f| !f["ok"].as_bool().unwrap_or(false)) {
        out.push_str(&format!("\n## {} failed\n\n", f["title"].as_str().unwrap_or("")));
        for c in f["checks"].as_array().into_iter().flatten().filter(|c| !c["ok"].as_bool().unwrap_or(false)) {
            let shot = c["shot"].as_str().map(|s| format!(" ![]({s})")).unwrap_or_default();
            out.push_str(&format!("- {}: {}{shot}\n", c["name"].as_str().unwrap_or(""), c["detail"].as_str().unwrap_or("")));
        }
    }
    out
}
