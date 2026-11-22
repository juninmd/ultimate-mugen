use super::sim::play;
use crate::routes::Route;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use ume_engine::chardef::CharDef;
use ume_engine::snapshot::{frame, sheet};

const SHOTS: usize = 6;
const EVERY: usize = 4;

fn shots(def: &Arc<CharDef>, route: &Route) -> Vec<tiny_skia::Pixmap> {
    let mut start = None;
    let mut out = vec![];
    play(def, route, |m, t| {
        m.events.clear();
        if start.is_none() && m.fighters[0].state_no == route.state {
            start = Some(t);
        }
        if let Some(s) = start {
            if (t - s) % EVERY == 0 {
                out.push(frame(m, false));
            }
        }
    });
    let n = out.len();
    if n <= SHOTS {
        return out;
    }
    (0..SHOTS).map(|k| out[k * (n - 1) / (SHOTS - 1)].clone()).collect()
}

pub fn file_name(route: &Route) -> String {
    let safe: String = route.command.chars().map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() } else { '_' }).collect();
    format!("{}_{safe}.png", route.when)
}

pub fn evidence(orig: &Arc<CharDef>, umf: &Arc<CharDef>, routes: &[Route], dir: &Path) -> Result<Vec<PathBuf>, String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let mut written = vec![];
    for route in routes {
        let (mut a, b) = (shots(orig, route), shots(umf, route));
        if a.is_empty() {
            continue;
        }
        let blank = tiny_skia::Pixmap::new(ume_engine::snapshot::W, ume_engine::snapshot::H).ok_or("pixmap")?;
        a.resize(SHOTS, blank.clone());
        let mut all = a;
        all.extend(b);
        all.resize(SHOTS * 2, blank);
        let path = dir.join(file_name(route));
        sheet(&all, SHOTS).save_png(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        written.push(path);
    }
    Ok(written)
}
