use super::*;

pub fn installed_notice(prior: Option<bool>, name: &str) -> String {
    let verb = match prior {
        Some(_) => crate::i18n::tr3("Atualizado", "Updated", "Actualizado"),
        None => crate::i18n::tr3("Instalado", "Installed", "Instalado"),
    };
    format!("{verb}: {name}")
}

impl Market {
    pub fn loading_installed(&self) -> bool {
        !self.ready || !self.requested.is_empty() || self.has_registry_work()
    }

    pub fn outdated(&self, repo_key: &str) -> Vec<String> {
        let Some(catalog) = self.catalogs.get(repo_key) else {
            return Vec::new();
        };
        self.state
            .installed
            .iter()
            .filter(|i| i.repo_key == repo_key)
            .filter_map(|i| {
                catalog
                    .find(&i.item_id)
                    .filter(|it| it.installable && !it.version.is_empty() && it.version != i.version)
            })
            .map(|it| it.id.clone())
            .collect()
    }

    pub(super) fn queue_updates(&mut self, repo_key: &str) -> usize {
        let ids = self.outdated(repo_key);
        self.queue_install(repo_key, &ids)
    }

    pub(super) fn prior_active(&self, repo_key: &str, item_id: &str) -> Option<bool> {
        self.state.installed_item(repo_key, item_id).map(|i| i.active)
    }

    pub(super) fn register_if_active(&mut self, tag: &str, kind: Kind, files: Files, prior: Option<bool>) {
        if prior != Some(false) {
            self.register(tag, kind, files);
        }
    }

    pub(super) fn auto_check(&mut self) {
        let now = ume_market::now_secs();
        let due = self.auto_checked == 0 || now.saturating_sub(self.auto_checked) > STALE_AFTER_SECS;
        if !self.ready || !due {
            return;
        }
        self.auto_checked = now;
        let mut keys: Vec<String> = self.state.installed.iter().map(|i| i.repo_key.clone()).collect();
        keys.sort();
        keys.dedup();
        for key in keys {
            self.refresh(&key);
        }
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use ume_market::MemPersist;
    use ume_market::catalog::FileRef;

    fn item(id: &str, version: &str, installable: bool) -> Item {
        let file = FileRef { path: format!("{id}.ogg"), size: 4, sha: "ab".into() };
        Item { id: id.into(), kind: Kind::Music, name: id.into(), files: vec![file], bytes: 4, installable, version: version.into() }
    }

    fn market() -> Market {
        let mut m = Market::with_persist(Arc::new(MemPersist::new()));
        m.finish_hydration();
        m.state.add_repo(RepoRef::parse("a/b").unwrap());
        m
    }

    fn install(m: &mut Market, it: &Item) {
        let files: Files = vec![(it.files[0].path.clone(), b"OggS".to_vec())];
        finish_install(&m.events, &m.persist.clone(), "a/b".into(), it.clone(), Ok(files));
        m.poll();
    }

    fn catalog_of(items: Vec<Item>) -> Catalog {
        Catalog { items, ..Default::default() }
    }

    #[test]
    fn only_changed_installed_items_are_outdated() {
        let mut m = market();
        for it in [item("music:same", "v1", true), item("music:changed", "v1", true), item("music:legacy", "", true), item("music:big", "v1", true)] {
            install(&mut m, &it);
        }
        m.state.installed.iter_mut().find(|i| i.item_id == "music:legacy").unwrap().version = String::new();
        let fresh = catalog_of(vec![
            item("music:same", "v1", true),
            item("music:changed", "v2", true),
            item("music:legacy", "v9", true),
            item("music:big", "v2", false),
            item("music:new", "v1", true),
        ]);
        m.catalogs.insert("a/b".into(), fresh);
        let mut ids = m.outdated("a/b");
        ids.sort();
        assert_eq!(ids, vec!["music:changed", "music:legacy"]);
        assert!(m.outdated("other/repo").is_empty());
    }

    #[test]
    fn unversioned_catalog_items_never_trigger_updates() {
        let mut m = market();
        install(&mut m, &item("music:a", "v1", true));
        m.catalogs.insert("a/b".into(), catalog_of(vec![item("music:a", "", true)]));
        assert!(m.outdated("a/b").is_empty());
    }

    #[test]
    fn updating_records_the_new_version_and_keeps_deactivated_items_off() {
        let mut m = market();
        let (on, off) = (item("music:on", "v1", true), item("music:off", "v1", true));
        install(&mut m, &on);
        install(&mut m, &off);
        m.set_active([("a/b", "music:off")], false);
        m.tracks.clear();
        m.registered.clear();
        install(&mut m, &item("music:on", "v2", true));
        install(&mut m, &item("music:off", "v2", true));
        let state = |id: &str| m.state.installed_item("a/b", id).unwrap().clone();
        assert_eq!((state("music:on").version.as_str(), state("music:on").active), ("v2", true));
        assert_eq!((state("music:off").version.as_str(), state("music:off").active), ("v2", false));
        assert_eq!(m.tracks.len(), 1);
        assert_eq!(m.state.installed.len(), 2);
    }

    #[test]
    fn notice_tells_updates_from_installs() {
        assert!(installed_notice(None, "x").ends_with(": x"));
        assert!(installed_notice(Some(true), "x").ends_with(": x"));
        assert_ne!(installed_notice(None, "x"), installed_notice(Some(true), "x"));
    }
}
