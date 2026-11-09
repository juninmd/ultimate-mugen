use crate::pack::{lanczos_resize, Image};
use crate::reel_font::draw_text;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use ume_charforge::spritekit::encode_rgb_png;
use ume_formats::png::{decode_png, png_indexed_to_rgba};

const HOLD_FRAMES: usize = 4;
const CAPTION_HEIGHT: usize = 22;

pub struct ReelOptions {
    pub width: usize,
    pub colors: u32,
    pub fps: u32,
    pub skip_normals: bool,
}

struct Clip {
    files: Vec<PathBuf>,
    movement: String,
}

type Heroes = BTreeMap<String, BTreeMap<u32, Clip>>;

fn parse_name(name: &str) -> Option<(String, u32, String)> {
    let stem = name.strip_prefix("C-")?.strip_suffix(".png")?;
    let (rest, frame) = stem.rsplit_once('-')?;
    frame.parse::<u32>().ok()?;
    let (hero, rest) = rest.split_once('-')?;
    let (index, movement) = rest.split_once('-')?;
    let valid = !hero.is_empty() && hero.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
    (valid && !movement.is_empty()).then(|| Some((hero.to_string(), index.parse().ok()?, movement.to_string())))?
}

fn collect(source: &Path, skip_normals: bool) -> Result<Heroes, String> {
    let mut heroes = Heroes::new();
    let mut names: Vec<PathBuf> = std::fs::read_dir(source).map_err(|e| format!("{}: {e}", source.display()))?.flatten().map(|e| e.path()).collect();
    names.sort();
    for file in names {
        let Some((hero, index, movement)) = file.file_name().and_then(|n| n.to_str()).and_then(parse_name) else { continue };
        if skip_normals && movement.starts_with("normal-") {
            continue;
        }
        let clip = heroes.entry(hero).or_default().entry(index).or_insert(Clip { files: Vec::new(), movement: String::new() });
        clip.files.push(file);
        clip.movement = movement;
    }
    Ok(heroes)
}

fn captioned(file: &Path, label: &str, width: usize) -> Result<(Vec<u8>, usize), String> {
    let png = decode_png(&std::fs::read(file).map_err(|e| e.to_string())?).map_err(|e| format!("{}: {e}", file.display()))?;
    let source = Image { width: png.width as usize, height: png.height as usize, rgba: png.rgba.clone().unwrap_or_else(|| png_indexed_to_rgba(&png)) };
    let height = (source.height as f64 * width as f64 / source.width as f64).round() as usize;
    let scaled = lanczos_resize(&source, width, height);
    let total = height + CAPTION_HEIGHT;
    let mut rgb = vec![0u8; width * total * 3];
    for (i, px) in scaled.rgba.chunks_exact(4).enumerate() {
        let a = px[3] as u32;
        rgb[i * 3..i * 3 + 3].iter_mut().zip(&px[..3]).for_each(|(d, s)| *d = ((*s as u32 * a + 127) / 255) as u8);
    }
    rgb[width * height * 3..].chunks_exact_mut(3).for_each(|p| p.copy_from_slice(&[8, 8, 20]));
    draw_text(&mut rgb, width, 6, height + 4, label, [255, 214, 77], 2);
    Ok((rgb, total))
}

fn build(hero: &str, clips: &BTreeMap<u32, Clip>, out_dir: &Path, options: &ReelOptions) -> Result<(PathBuf, usize), String> {
    let target = out_dir.join(format!("{}-abilities.gif", hero.replace('_', "-")));
    let scratch = std::env::temp_dir().join(format!("ume-reel-{}-{hero}", std::process::id()));
    std::fs::create_dir_all(&scratch).map_err(|e| e.to_string())?;
    let mut count = 0;
    for (number, clip) in clips.values().enumerate() {
        let label = format!("{}  {:02}/{}  {}", hero.replace('_', " "), number + 1, clips.len(), clip.movement.replace('-', " "));
        let mut files = clip.files.clone();
        files.sort();
        for (position, file) in files.iter().enumerate() {
            let (rgb, height) = captioned(file, &label, options.width)?;
            let png = encode_rgb_png(options.width, height, &rgb, 6);
            for _ in 0..if position + 1 == files.len() { HOLD_FRAMES } else { 1 } {
                std::fs::write(scratch.join(format!("{count:05}.png")), &png).map_err(|e| e.to_string())?;
                count += 1;
            }
        }
    }
    let filter = format!("split[a][b];[a]palettegen=max_colors={}:stats_mode=diff[p];[b][p]paletteuse=dither=bayer:bayer_scale=5:diff_mode=rectangle", options.colors);
    let status = Command::new("ffmpeg").args(["-y", "-loglevel", "error", "-framerate", &options.fps.to_string(), "-i"]).arg(scratch.join("%05d.png")).args(["-vf", &filter, "-loop", "0"]).arg(&target).status().map_err(|e| format!("ffmpeg is required to encode the GIFs: {e}"))?;
    let _ = std::fs::remove_dir_all(&scratch);
    status.success().then_some((target, count)).ok_or_else(|| format!("ffmpeg exited with {status}"))
}

pub fn make_reels(source: &Path, out_dir: &Path, options: &ReelOptions) -> Result<Vec<String>, String> {
    std::fs::create_dir_all(out_dir).map_err(|e| e.to_string())?;
    let mut lines = Vec::new();
    for (hero, clips) in collect(source, options.skip_normals)? {
        let (target, count) = build(&hero, &clips, out_dir, options)?;
        let size = std::fs::metadata(&target).map(|m| m.len()).unwrap_or(0);
        lines.push(format!("{} {count} frames {:.2} MB", target.display(), size as f64 / 1e6));
    }
    Ok(lines)
}
