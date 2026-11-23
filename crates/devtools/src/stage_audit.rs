use serde_json::{json, Value};
use std::path::Path;

#[derive(Clone, Debug)]
pub struct StageAuditEntry {
    pub id: String,
    pub width: u32,
    pub height: u32,
    pub bytes: u64,
    pub has_thumb: bool,
    pub anim_frames: usize,
    pub issues: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct StageAuditReport {
    pub stages: Vec<StageAuditEntry>,
    pub total_issues: usize,
}

fn png_dimensions(path: &Path) -> Result<(u32, u32), String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    if bytes.len() < 24 || &bytes[0..8] != b"\x89PNG\r\n\x1a\n" {
        return Err(format!("{}: not a valid PNG header", path.display()));
    }
    let w = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
    let h = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
    Ok((w, h))
}

fn count_anim_frames(anim_dir: &Path) -> usize {
    let Ok(rd) = std::fs::read_dir(anim_dir) else {
        return 0;
    };
    rd.flatten()
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|s| s.to_str())
                .is_some_and(|ext| {
                    ext.eq_ignore_ascii_case("png") || ext.eq_ignore_ascii_case("jpg")
                })
        })
        .count()
}

pub fn audit(root: &Path) -> Result<StageAuditReport, String> {
    let rd = std::fs::read_dir(root).map_err(|e| format!("{}: {e}", root.display()))?;
    let mut paths: Vec<_> = rd
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.is_file()
                && p.extension()
                    .and_then(|s| s.to_str())
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("png"))
        })
        .collect();
    paths.sort();

    let mut stages = Vec::new();
    let mut total_issues = 0;
    for path in paths {
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        if stem.ends_with("_platform") {
            continue;
        }
        let (width, height) = png_dimensions(&path)?;
        let bytes = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        let has_thumb = root.join("thumbs").join(format!("{stem}.png")).exists();
        let anim_frames = count_anim_frames(&root.join("animation").join(stem));
        let mut issues = Vec::new();
        if width < 640 || height < 360 {
            issues.push(format!("low resolution ({width}x{height})"));
        }
        let aspect = if height > 0 {
            width as f32 / height as f32
        } else {
            0.0
        };
        if aspect < 1.5 || aspect > 2.4 {
            issues.push(format!("unusual aspect ratio ({aspect:.2})"));
        }
        if !has_thumb {
            issues.push("missing thumbnail".into());
        }
        if anim_frames == 1 {
            issues.push("animation folder has only 1 frame".into());
        }
        total_issues += issues.len();
        stages.push(StageAuditEntry {
            id: stem.to_string(),
            width,
            height,
            bytes,
            has_thumb,
            anim_frames,
            issues,
        });
    }
    Ok(StageAuditReport {
        stages,
        total_issues,
    })
}

pub fn format_text(rep: &StageAuditReport) -> String {
    let mut out = format!(
        "Stage Audit: {} stages, {} issue(s)\n",
        rep.stages.len(),
        rep.total_issues
    );
    out.push_str("STAGE                      DIM         SIZE    THUMB  ANIM  STATUS\n");
    for s in &rep.stages {
        let dim = format!("{}x{}", s.width, s.height);
        let kb = s.bytes / 1024;
        let status = if s.issues.is_empty() {
            "OK".to_string()
        } else {
            s.issues.join("; ")
        };
        out.push_str(&format!(
            "{:<25}  {:<10}  {:>5}KB  {:^5}  {:>4}  {}\n",
            s.id,
            dim,
            kb,
            if s.has_thumb { "yes" } else { "no" },
            s.anim_frames,
            status
        ));
    }
    out
}

pub fn to_json(rep: &StageAuditReport) -> Value {
    let items: Vec<Value> = rep
        .stages
        .iter()
        .map(|s| {
            json!({
                "id": s.id, "width": s.width, "height": s.height,
                "bytes": s.bytes, "has_thumb": s.has_thumb,
                "anim_frames": s.anim_frames, "issues": s.issues,
            })
        })
        .collect();
    json!({ "count": rep.stages.len(), "total_issues": rep.total_issues, "stages": items })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audits_repo_stages_directory() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/stages");
        let rep = audit(&dir).expect("audit assets/stages");
        assert!(
            rep.stages.len() >= 10,
            "expected at least 10 stages, found {}",
            rep.stages.len()
        );
        assert!(rep.stages.iter().all(|s| s.width >= 640 && s.height >= 360));
        let txt = format_text(&rep);
        assert!(txt.contains("colosseum"));
        let val = to_json(&rep);
        assert_eq!(val["count"].as_u64(), Some(rep.stages.len() as u64));
    }
}
