use super::*;
use std::collections::{HashMap, HashSet};
use ume_market::view::{display_key, format_of, Filter, Format};

#[derive(Clone)]
pub(super) struct Row {
    pub repo_key: String,
    pub id: String,
    pub kind: Kind,
    pub name: String,
    pub bytes: u64,
    pub installable: bool,
    pub installed: Option<bool>,
    pub format: Option<Format>,
    pub outdated: bool,
}

impl Row {
    pub fn pair(&self) -> (String, String) {
        (self.repo_key.clone(), self.id.clone())
    }
}

impl App {
    pub(super) fn market_filter(&self) -> Filter {
        Filter { kind: self.market_ui.filter, format: self.market_ui.format, outdated_only: self.market_ui.outdated_only }
    }

    pub(super) fn market_rows(&mut self) -> Vec<Row> {
        let filter = self.market_filter();
        let mut outdated: HashMap<String, HashSet<String>> = HashMap::new();
        let mut rows: Vec<Row> = match self.market_ui.view {
            View::Installed => {
                for key in self.market.state.installed.iter().map(|i| i.repo_key.clone()) {
                    outdated.entry(key.clone()).or_insert_with(|| self.market.outdated(&key).into_iter().collect());
                }
                self.market
                    .state
                    .installed
                    .iter()
                    .map(|i| Row {
                        repo_key: i.repo_key.clone(),
                        id: i.item_id.clone(),
                        kind: i.kind,
                        name: i.name.clone(),
                        bytes: i.bytes,
                        installable: true,
                        installed: Some(i.active),
                        format: format_of(i.kind, &i.item_id, &i.name),
                        outdated: outdated.get(&i.repo_key).is_some_and(|set| set.contains(&i.item_id)),
                    })
                    .collect()
            }
            View::Browse => {
                let Some(key) = self.market.state.repos.get(self.market_ui.repo).map(|r| r.key.clone()) else {
                    return vec![];
                };
                self.market.catalog(&key);
                let stale: HashSet<String> = self.market.outdated(&key).into_iter().collect();
                let installed: HashMap<&str, bool> = self.market.state.installed.iter().filter(|i| i.repo_key == key).map(|i| (i.item_id.as_str(), i.active)).collect();
                let Some(cat) = self.market.catalog_cached(&key) else {
                    return vec![];
                };
                cat.items
                    .iter()
                    .map(|i| Row {
                        repo_key: key.clone(),
                        id: i.id.clone(),
                        kind: i.kind,
                        name: i.name.clone(),
                        bytes: i.bytes,
                        installable: i.installable,
                        installed: installed.get(i.id.as_str()).copied(),
                        format: format_of(i.kind, &i.id, &i.name),
                        outdated: stale.contains(&i.id),
                    })
                    .collect()
            }
        };
        self.market_ui.outdated_total = rows.iter().filter(|r| r.outdated).count();
        self.market_ui.unfiltered = rows.len();
        rows.retain(|r| filter.accepts(r.kind, r.format, r.outdated));
        rows.sort_by_cached_key(|r| display_key(r.kind, &r.name, r.format));
        rows
    }
}
