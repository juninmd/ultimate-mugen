use super::*;
use std::cell::RefCell;

struct ViewKey {
    filter: u8,
    sort: usize,
    query: String,
    has_roster: bool,
    marks: (u64, Option<String>),
    base: Vec<usize>,
}

thread_local! {
    static VIEW: RefCell<Option<(ViewKey, Vec<usize>)>> = const { RefCell::new(None) };
}

impl App {
    pub(crate) fn cached_char_view(&self) -> Vec<usize> {
        let base = self.playable();
        let query = super::select_search::query();
        let has_roster = self.sp.is_some();
        let marks = (
            super::select_marks::marks_revision(),
            super::select_author::author_filter(),
        );
        let hit = VIEW.with(|c| {
            c.borrow().as_ref().and_then(|(k, v)| {
                (k.filter == self.sel_filter
                    && k.sort == self.settings.sort
                    && k.has_roster == has_roster
                    && k.marks == marks
                    && k.query == query
                    && k.base == base)
                    .then(|| v.clone())
            })
        });
        if let Some(v) = hit {
            return v;
        }
        let v = self.compute_char_view(&base);
        let key = ViewKey {
            filter: self.sel_filter,
            sort: self.settings.sort,
            query,
            has_roster,
            marks,
            base,
        };
        VIEW.with(|c| *c.borrow_mut() = Some((key, v.clone())));
        v
    }

    pub(crate) fn compute_char_view(&self, base: &[usize]) -> Vec<usize> {
        let mut v: Vec<usize> = base
            .iter()
            .copied()
            .filter(|&i| self.category_matches(i))
            .collect();
        if v.is_empty() {
            v = base.to_vec();
        }
        if let Some(sp) = &self.sp {
            v.sort_by_key(|&i| {
                let e = &self.reg.chars[i];
                sp.roster_pos(&e.id)
                    .or_else(|| sp.roster_pos(&e.name))
                    .unwrap_or(usize::MAX)
            });
        }
        self.sort_view(&mut v);
        if self.sel_filter == super::select_marks::FILTER_RECENT {
            v.sort_by_key(|&i| self.recent_rank(i).unwrap_or(usize::MAX));
        }
        v
    }
}

pub(crate) fn last_view_len() -> usize {
    VIEW.with(|c| c.borrow().as_ref().map_or(1, |(_, v)| v.len().max(1)))
}
