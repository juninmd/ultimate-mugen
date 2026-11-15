use super::*;
use std::cell::RefCell;

struct ViewKey {
    filter: u8,
    sort: usize,
    query: String,
    has_roster: bool,
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
        let hit = VIEW.with(|c| {
            c.borrow().as_ref().and_then(|(k, v)| {
                (k.filter == self.sel_filter
                    && k.sort == self.settings.sort
                    && k.has_roster == has_roster
                    && k.query == query
                    && k.base == base)
                    .then(|| v.clone())
            })
        });
        if let Some(v) = hit {
            return v;
        }
        let mut v: Vec<usize> = base
            .iter()
            .copied()
            .filter(|&i| self.category_matches(i))
            .collect();
        if v.is_empty() {
            v = base.clone();
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
        let key = ViewKey {
            filter: self.sel_filter,
            sort: self.settings.sort,
            query,
            has_roster,
            base,
        };
        VIEW.with(|c| *c.borrow_mut() = Some((key, v.clone())));
        v
    }
}
