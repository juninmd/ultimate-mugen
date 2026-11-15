use super::*;
use std::cell::{Cell, RefCell};

const FAVORITES_FILE: &str = "favorites.txt";
const RECENT_FILE: &str = "recent.txt";
const RECENT_MAX: usize = 24;
pub(crate) const FILTER_FAVORITES: u8 = 4;
pub(crate) const FILTER_RECENT: u8 = 5;

thread_local! {
    static FAVORITES: RefCell<Option<Vec<String>>> = const { RefCell::new(None) };
    static RECENT: RefCell<Option<Vec<String>>> = const { RefCell::new(None) };
    static REVISION: Cell<u64> = const { Cell::new(0) };
}

fn load(name: &str) -> Vec<String> {
    crate::storage::read(name)
        .unwrap_or_default()
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect()
}

fn with_list<R>(
    cell: &'static std::thread::LocalKey<RefCell<Option<Vec<String>>>>,
    file: &str,
    f: impl FnOnce(&mut Vec<String>) -> R,
) -> R {
    cell.with(|c| f(c.borrow_mut().get_or_insert_with(|| load(file))))
}

fn save(name: &str, list: &[String]) {
    crate::storage::write(name, &list.join("\n"));
    REVISION.with(|r| r.set(r.get() + 1));
}

pub(crate) fn marks_revision() -> u64 {
    REVISION.with(|r| r.get())
}

pub(crate) fn star(ui: &Ui, cx: f32, cy: f32, r: f32, color: Color) {
    let point = |k: usize| {
        let a = -std::f32::consts::FRAC_PI_2 + k as f32 * std::f32::consts::PI / 5.0;
        let len = if k % 2 == 0 { r } else { r * 0.45 };
        vec2(ui.x(cx + a.cos() * len), ui.y(cy + a.sin() * len))
    };
    let center = vec2(ui.x(cx), ui.y(cy));
    for k in 0..10 {
        draw_triangle(center, point(k), point(k + 1), color);
    }
}

impl App {
    pub(crate) fn is_favorite(&self, i: usize) -> bool {
        let key = &self.reg.chars[i].key;
        with_list(&FAVORITES, FAVORITES_FILE, |list| {
            list.iter().any(|k| k == key)
        })
    }

    pub(crate) fn toggle_favorite(&mut self, i: usize) {
        let key = self.reg.chars[i].key.clone();
        let now = with_list(&FAVORITES, FAVORITES_FILE, |list| {
            let had = list.iter().position(|k| *k == key);
            match had {
                Some(at) => {
                    list.remove(at);
                }
                None => list.push(key.clone()),
            }
            save(FAVORITES_FILE, list);
            had.is_none()
        });
        self.audio.sfx(if now { "confirm" } else { "select" });
    }

    pub(crate) fn note_recent(&self, i: usize) {
        let key = self.reg.chars[i].key.clone();
        with_list(&RECENT, RECENT_FILE, |list| {
            list.retain(|k| *k != key);
            list.insert(0, key);
            list.truncate(RECENT_MAX);
            save(RECENT_FILE, list);
        });
    }

    pub(crate) fn recent_rank(&self, i: usize) -> Option<usize> {
        let key = &self.reg.chars[i].key;
        with_list(&RECENT, RECENT_FILE, |list| {
            list.iter().position(|k| k == key)
        })
    }

    pub(crate) fn draw_favorite_mark(&self, cx: f32, cy: f32) {
        self.ui
            .rect(cx, cy, 16.0, 16.0, Color::new(0.0, 0.0, 0.0, 0.55));
        star(&self.ui, cx + 8.0, cy + 8.5, 6.5, hex("#ffcc33"));
    }

    pub(crate) fn favorite_button(&mut self, i: usize) {
        let (x, y, w, h) = (1084.0, 63.0, 156.0, 20.0);
        let on = self.is_favorite(i);
        let hover = self.hover_rect(x, y, w, h);
        let fill = if on {
            Color::new(1.0, 0.8, 0.2, 0.25)
        } else if hover {
            Color::new(1.0, 1.0, 1.0, 0.16)
        } else {
            Color::new(1.0, 1.0, 1.0, 0.07)
        };
        self.ui.rect(x, y, w, h, fill);
        let color = if on {
            hex("#ffcc33")
        } else {
            Color::new(1.0, 1.0, 1.0, 0.45)
        };
        star(&self.ui, x + 12.0, y + 10.5, 7.0, color);
        let text = if on {
            crate::i18n::tr3("Favorito · [F]", "Favorite · [F]", "Favorito · [F]")
        } else {
            crate::i18n::tr3("Favoritar · [F]", "Add favorite · [F]", "Favorito · [F]")
        };
        self.ui.text(
            text,
            x + 24.0,
            y + 15.0,
            13.0,
            if on {
                WHITE
            } else {
                Color::new(1.0, 1.0, 1.0, 0.6)
            },
            Align::Left,
            false,
        );
        if self.clicked_rect(x, y, w, h) || is_key_pressed(KeyCode::F) {
            self.toggle_favorite(i);
        }
    }
}
