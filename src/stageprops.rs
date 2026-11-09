use crate::gfx::*;
use crate::stages::StageKind;
use serde::Deserialize;
use std::cell::OnceCell;
use std::collections::HashMap;
use std::sync::OnceLock;
use ume_formats::png::decode_png;

const DEFAULT_TEXT: &str = include_str!("../assets/stageprops.toml");
const FILE: &str = "stageprops.toml";
const TORCH_PNG: &[u8] = include_bytes!("../assets/props/torch.png");
const BAMBOO_PNG: &[u8] = include_bytes!("../assets/props/bamboo.png");

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(default)]
pub struct Prop {
    pub sprite: String,
    pub u: f32,
    pub base: f32,
    pub height: f32,
    pub depth: f32,
    pub front: bool,
    pub flip: bool,
    pub alpha: f32,
    pub tint: [f32; 3],
    pub sway: f32,
    pub glow: f32,
    pub glow_color: [f32; 3],
}

impl Default for Prop {
    fn default() -> Self {
        Prop { sprite: String::new(), u: 0.5, base: 1.0, height: 0.2, depth: 0.0, front: false, flip: false, alpha: 1.0, tint: [1.0; 3], sway: 0.0, glow: 0.0, glow_color: [1.0; 3] }
    }
}

type Table = HashMap<String, Vec<Prop>>;

fn parse(text: &str) -> Option<Table> {
    toml::from_str(text).ok()
}

fn table() -> &'static Table {
    static TABLE: OnceLock<Table> = OnceLock::new();
    TABLE.get_or_init(|| crate::storage::read(FILE).and_then(|t| parse(&t)).or_else(|| parse(DEFAULT_TEXT)).unwrap_or_default())
}

fn name(kind: StageKind) -> &'static str {
    match kind {
        StageKind::Dojo => "dojo",
        StageKind::Neon => "neon",
        StageKind::Bamboo => "bamboo",
        StageKind::Plains => "plains",
        StageKind::Spirit => "spirit",
        StageKind::Volcano => "volcano",
        StageKind::Custom => "custom",
    }
}

struct Sheet {
    tex: Texture2D,
    cell: (f32, f32),
    frames: usize,
    cols: usize,
    fps: f32,
}

fn load(png: &[u8]) -> Option<Texture2D> {
    let d = decode_png(png).ok()?;
    let rgba = d.rgba.clone().unwrap_or_else(|| ume_formats::png::png_indexed_to_rgba(&d));
    Some(Texture2D::from_rgba8(d.width as u16, d.height as u16, &rgba))
}

fn sheet(sprite: &str) -> Option<Sheet> {
    let (png, cell, frames, cols, fps) = match sprite {
        "torch" => (TORCH_PNG, (32.0, 32.0), 6, 3, 9.0),
        "bamboo" => (BAMBOO_PNG, (0.0, 0.0), 1, 1, 1.0),
        _ => return None,
    };
    let tex = load(png)?;
    let cell = if cell.0 > 0.0 { cell } else { (tex.width(), tex.height()) };
    Some(Sheet { tex, cell, frames, cols, fps })
}

thread_local! {
    static SHEETS: std::cell::RefCell<HashMap<String, OnceCell<Option<Sheet>>>> = Default::default();
}

pub fn draw(g: &crate::render::Gfx, kind: StageKind, front: bool, cam_x: f32, view: (f32, f32), t: f32) {
    let level = crate::vfx::quality::level();
    if level == 0 || g.fx_gain <= 0.0 {
        return;
    }
    let Some(props) = table().get(name(kind)) else { return };
    let s = g.view_scale;
    for p in props.iter().filter(|p| p.front == front) {
        SHEETS.with(|cache| {
            let mut cache = cache.borrow_mut();
            let slot = cache.entry(p.sprite.clone()).or_default();
            let Some(sh) = slot.get_or_init(|| sheet(&p.sprite)).as_ref() else { return };
            let h = p.height * view.1 * s;
            let w = h * sh.cell.0 / sh.cell.1;
            let x = g.ox + (p.u * view.0 - cam_x * p.depth) * s - w * 0.5;
            let y = g.oy + p.base * view.1 * s - h;
            let frame = if sh.frames > 1 { ((t * sh.fps) as usize + (p.u * 7.0) as usize) % sh.frames } else { 0 };
            let sway = if level >= 2 { p.sway * (t * 0.9 + p.u * 6.0).sin() } else { 0.0 };
            if p.glow > 0.0 && level >= 2 {
                let flick = 0.8 + 0.2 * (t * 11.0 + p.u * 13.0).sin() * (t * 4.3).cos();
                set_blend(GfxBlend::Add);
                for k in 0..3 {
                    let r = h * (0.9 - 0.25 * k as f32);
                    let c = p.glow_color;
                    draw_ellipse(x + w * 0.5, y + h * 0.3, r, r * 0.9, 0.0, Color::new(c[0], c[1], c[2], p.glow * 0.1 * flick * g.fx_gain.min(1.0)));
                }
                set_blend(GfxBlend::Alpha);
            }
            let (col, row) = (frame % sh.cols, frame / sh.cols);
            draw_texture_ex(
                &sh.tex,
                x,
                y,
                Color::new(p.tint[0], p.tint[1], p.tint[2], p.alpha * g.fx_gain.min(1.0)),
                DrawTextureParams {
                    dest_size: Some(vec2(w, h)),
                    source: Some(Rect::new(col as f32 * sh.cell.0, row as f32 * sh.cell.1, sh.cell.0, sh.cell.1)),
                    rotation: sway.to_radians(),
                    flip_x: p.flip,
                    pivot: Some(vec2(x + w * 0.5, y + h)),
                    ..Default::default()
                },
            );
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_props_parse() {
        let t = parse(DEFAULT_TEXT).unwrap();
        assert_eq!(t["dojo"].len(), 2);
        assert!(t["bamboo"].iter().all(|p| p.front));
        assert!(t.values().flatten().all(|p| sheet_known(&p.sprite)));
    }

    fn sheet_known(s: &str) -> bool {
        matches!(s, "torch" | "bamboo")
    }
}
