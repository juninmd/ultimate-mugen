use super::{Issue, Severity};
use std::collections::BTreeSet;
use std::path::Path;
use ume_engine::matchs::{CameraParams, StageInfo};
use ume_formats::stage::{load_stage, MugenStage};
use ume_formats::vfs::{add_dir, resolve, Vfs};

pub struct LoadedStage {
    pub stage: MugenStage,
    pub issues: Vec<Issue>,
}

pub fn load(dir: &str, def: &str) -> Result<LoadedStage, String> {
    let mut vfs = Vfs::new();
    add_dir(&mut vfs, Path::new(dir), "c").map_err(|e| format!("{dir}: {e}"))?;
    let stage = load_stage(&vfs, def)?;
    let mut issues = static_issues(&stage, &vfs);
    if let Some(problem) = inverted_bounds(&vfs, def, &stage.name) {
        issues.push(problem);
    }
    Ok(LoadedStage { stage, issues })
}

fn inverted_bounds(vfs: &Vfs, def: &str, name: &str) -> Option<Issue> {
    let text = ume_formats::vfs::text(vfs.get(def)?);
    let sections = ume_formats::ini::parse_ini_no_lines(&text);
    let camera = ume_formats::ini::find_section(&sections, "camera")?;
    let num = |key: &str, default: f32| ume_formats::ini::nums(camera.get(key), &[default])[0];
    let (left, right) = (num("boundleft", -150.0), num("boundright", 150.0));
    let (high, low) = (num("boundhigh", -25.0), num("boundlow", 0.0));
    if left <= right && high <= low {
        return None;
    }
    Some(Issue::new(
        Severity::Critical,
        "stage",
        "stage-bounds-inverted",
        name,
        format!("[Camera] boundleft {left} / boundright {right} and boundhigh {high} / boundlow {low} are not ordered; the loader sorts them so the camera clamp cannot panic"),
    ))
}

pub fn info(st: &MugenStage) -> StageInfo {
    let scl = 320.0 / if st.local_w > 0.0 { st.local_w } else { 320.0 };
    let scl_y = 240.0 / if st.local_h > 0.0 { st.local_h } else { 240.0 };
    StageInfo {
        name: st.name.clone(),
        bound_left: st.bound_left * scl,
        bound_right: st.bound_right * scl,
        bound_high: st.bound_high * scl_y,
        bound_low: st.bound_low * scl_y,
        p1x: st.p1x * scl,
        p2x: st.p2x * scl,
        zoffset: st.zoffset * scl_y,
        vertical_follow: st.vertical_follow,
        tension: st.tension * scl,
        start_x: st.start_x * scl,
        start_y: st.start_y * scl_y,
        zoom_out: st.zoom_out,
        zoom_in: st.zoom_in,
        camera: CameraParams {
            vertical_follow_delta: st.vertical_follow_delta * scl_y,
            y_scroll_speed: st.y_scroll_speed,
            zoom_in_speed: st.zoom_in_speed,
            zoom_out_speed: st.zoom_out_speed,
            lowest_cap: st.lowest_cap,
            dynamic_bound_high: st.dynamic_bound_high,
            bound_high_delta: st.bound_high_delta * scl_y,
        },
        auto_turn: st.auto_turn,
    }
}

fn static_issues(st: &MugenStage, vfs: &Vfs) -> Vec<Issue> {
    let mut issues = Vec::new();
    let name = st.name.clone();
    if st.layers.is_empty() {
        issues.push(Issue::new(Severity::Critical, "stage", "stage-no-layers", &name, "the .def declares no [BG] layer"));
    }
    if st.sprites.is_empty() {
        issues.push(Issue::new(Severity::Critical, "stage", "stage-no-sprites", &name, "no sprite could be decoded from the stage .sff"));
    }
    if st.bound_right - st.bound_left < 120.0 {
        issues.push(Issue::new(Severity::Major, "stage", "stage-bounds-tight", &name, format!("fightable width {:.0} (boundleft {} boundright {})", st.bound_right - st.bound_left, st.bound_left, st.bound_right)));
    }
    if (st.p2x - st.p1x).abs() < 40.0 {
        issues.push(Issue::new(Severity::Major, "stage", "stage-spawn-overlap", &name, format!("p1startx {} and p2startx {} put both fighters in the same place", st.p1x, st.p2x)));
    }
    let mut missing: BTreeSet<(i32, i32)> = BTreeSet::new();
    let mut missing_anim: BTreeSet<i32> = BTreeSet::new();
    for layer in &st.layers {
        if layer.kind == "anim" {
            if layer.anim.is_none() {
                missing_anim.insert(layer.id);
            }
            if let Some(anim) = &layer.anim {
                for frame in &anim.frames {
                    if !st.sprites.contains_key(&(frame.group, frame.index)) {
                        missing.insert((frame.group, frame.index));
                    }
                }
            }
            continue;
        }
        if !st.sprites.contains_key(&layer.sprite) {
            missing.insert(layer.sprite);
        }
    }
    for (group, index) in missing.iter().take(6) {
        issues.push(Issue::new(Severity::Major, "stage", "stage-layer-sprite-missing", &name, format!("layer sprite {group},{index} is not in the stage .sff (the layer renders empty)")));
    }
    if missing.len() > 6 {
        issues.push(Issue::new(Severity::Major, "stage", "stage-layer-sprite-missing", &name, format!("{} more layer sprites are missing", missing.len() - 6)));
    }
    for id in missing_anim.iter().take(4) {
        issues.push(Issue::new(Severity::Major, "stage", "stage-layer-anim-missing", &name, format!("layer id {id} is type = anim but its actionno is not defined")));
    }
    if let Some(music) = &st.music {
        let base = "";
        let found = resolve(vfs, base, &music.file).is_some() || resolve(vfs, "sound", &music.file).is_some();
        if !found {
            issues.push(Issue::new(Severity::Info, "stage", "stage-music-outside-vfs", &name, format!("bgmusic {:?} is not next to the stage (it plays only when the sound root is mounted)", music.file)));
        }
    }
    if st.shadow.intensity <= 0.0 {
        issues.push(Issue::new(Severity::Info, "stage", "stage-no-shadow", &name, "shadow intensity is 0, fighters cast no shadow"));
    }
    issues
}
