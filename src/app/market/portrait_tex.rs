use super::badges::kind_initials;
use super::rows::Row;
use super::*;
use crate::market::Art;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

const MAX_TEXTURES: usize = 96;

thread_local! {
    static TEXTURES: RefCell<HashMap<String, (usize, Texture2D)>> = RefCell::new(HashMap::new());
}

fn texture_for(key: &str, art: &Art) -> Texture2D {
    let stamp = Arc::as_ptr(&art.rgba) as usize;
    TEXTURES.with(|cache| {
        let mut cache = cache.borrow_mut();
        if cache.len() > MAX_TEXTURES {
            cache.clear();
        }
        if let Some((seen, tex)) = cache.get(key) {
            if *seen == stamp {
                return tex.clone();
            }
        }
        let tex = Texture2D::from_rgba_shared(art.width as u16, art.height as u16, art.rgba.clone());
        cache.insert(key.to_string(), (stamp, tex.clone()));
        tex
    })
}

fn cover(iw: f32, ih: f32, w: f32, h: f32) -> Rect {
    let (ia, ba) = (iw / ih, w / h);
    if ia > ba {
        let sw = ih * ba;
        Rect::new((iw - sw) / 2.0, 0.0, sw, ih)
    } else {
        let sh = iw / ba;
        Rect::new(0.0, (ih - sh) / 3.0, iw, sh)
    }
}

impl App {
    pub(super) fn market_draw_portrait(&mut self, row: &Row, x: f32, y: f32, w: f32, h: f32) {
        let art = self.market.portrait(&row.repo_key, &row.id);
        self.ui.rect(x, y, w, h, Color::new(0.04, 0.03, 0.09, 1.0));
        match art {
            Some(art) => {
                let tex = texture_for(&format!("{}\u{0}{}", row.repo_key, row.id), &art);
                draw_texture_ex(
                    &tex,
                    self.ui.x(x),
                    self.ui.y(y),
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(w * self.ui.s, h * self.ui.s)),
                        source: Some(cover(art.width as f32, art.height as f32, w, h)),
                        ..Default::default()
                    },
                );
            }
            None => {
                let tint = alpha(kind_color(row.kind), 0.5);
                self.ui.rect(x, y + h - 4.0, w, 4.0, tint);
                self.ui.text(&kind_initials(&row.name), x + w / 2.0, y + h / 2.0 + 14.0, 42.0, tint, Align::Center, true);
            }
        }
    }
}
