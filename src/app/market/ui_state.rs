use std::collections::HashSet;
use ume_market::view::Format;
use ume_market::Kind;

#[derive(Clone, Copy, PartialEq)]
pub enum View {
    Browse,
    Installed,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Pane {
    Repos,
    Items,
}

pub struct TextEntry {
    pub token: bool,
    pub buf: String,
    pub suggestion: usize,
}

#[derive(Default)]
pub struct Pointer {
    pub active: bool,
    pub hit: Option<usize>,
    pub last_y: f32,
    pub travelled: f32,
    pub carry: f32,
}

pub struct MarketUi {
    pub view: View,
    pub pane: Pane,
    pub repo: usize,
    pub repo_scroll: usize,
    pub cur: usize,
    pub scroll: usize,
    pub filter: Option<Kind>,
    pub format: Option<Format>,
    pub outdated_only: bool,
    pub outdated_total: usize,
    pub unfiltered: usize,
    pub marked: HashSet<(String, String)>,
    pub entry: Option<TextEntry>,
    pub backspace_hold: f32,
    pub opened: bool,
    pub pointer: Pointer,
}

impl Default for MarketUi {
    fn default() -> Self {
        MarketUi {
            view: View::Browse,
            pane: Pane::Items,
            repo: 0,
            repo_scroll: 0,
            cur: 0,
            scroll: 0,
            filter: None,
            format: None,
            outdated_only: false,
            outdated_total: 0,
            unfiltered: 0,
            marked: HashSet::new(),
            entry: None,
            backspace_hold: 0.0,
            opened: false,
            pointer: Pointer::default(),
        }
    }
}
