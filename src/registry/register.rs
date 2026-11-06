use super::*;

impl Registry {
    pub(super) fn add_vfs_inner(
        &mut self,
        vfs: Vfs,
        tag: &str,
        validate: bool,
    ) -> (Vec<String>, Vec<String>, Vec<String>) {
        let vfs = Arc::new(vfs);
        let (mut cs, mut ss, mut errs) = (vec![], vec![], vec![]);
        // roster identity kept in sets: a linear scan per imported fighter is
        // quadratic and dominates the startup of large collections
        let mut char_ids: std::collections::HashSet<String> = self.chars.iter().map(|c| c.id.clone()).collect();
        let mut char_keys: std::collections::HashSet<String> = self.chars.iter().map(|c| c.key.clone()).collect();
        let mut stage_ids: std::collections::HashSet<String> = self.stages.iter().map(|s| s.id.clone()).collect();
        let mut stage_keys: std::collections::HashSet<String> = self.stages.iter().map(|s| s.key.clone()).collect();
        let char_defs = find_char_defs(&vfs);
        // header parse and content key hit different files: fan the I/O out
        let chart: Vec<(String, String, String)> = ume_formats::par::map(&char_defs, |def| {
            let secs = parse_ini(&vfs.get(def).map(|v| text(&v)).unwrap_or_default());
            let info = find_section(&secs, "info");
            let mut name = info
                .and_then(|s| s.get("displayname").or(s.get("name")))
                .map(unquote)
                .filter(|s| !s.is_empty())
                .unwrap_or(def.clone());
            name = pretty_name(&name);
            if ume_formats::ini::nums(info.and_then(|s| s.get("localcoord")), &[320.0])[0] > 320.0 {
                name.push_str(" HD");
            }
            let author = info
                .and_then(|s| s.get("author"))
                .map(unquote)
                .unwrap_or_default();
            (name, author, char_key(&vfs, def))
        });
        for (def, (name, author, key)) in char_defs.into_iter().zip(chart) {
            let id = format!("mugen:{tag}:{def}");
            if char_ids.contains(&id) || char_keys.contains(&key) {
                continue;
            }
            let e = CharEntry {
                key: key.clone(),
                id: id.clone(),
                name: name.clone(),
                series: "M.U.G.E.N".into(),
                author,
                color: [90, 110, 150],
                description: "Personagem M.U.G.E.N importado".into(),
                mugen: true,
                boss: false,
                src: Src::Mugen {
                    vfs: vfs.clone(),
                    def: def.clone(),
                },
                cache: RefCell::new(HashMap::new()),
                preview: RefCell::new(HashMap::new()),
                last_use: Cell::new(0),
                portrait: RefCell::new(None),
                broken: Cell::new(false),
                errs: RefCell::new(HashMap::new()),
                jobs: Default::default(),
                sprite_revision: Cell::new(0),
            };
            if !validate {
                char_ids.insert(id);
                char_keys.insert(key);
                self.chars.push(e);
                cs.push(name);
                continue;
            }
            match e.load(1) {
                Ok(_) => {
                    e.unload();
                    char_ids.insert(id);
                    char_keys.insert(key);
                    self.chars.push(e);
                    cs.push(name);
                }
                Err(x) => errs.push(format!("{def}: {x}")),
            }
        }
        let stage_defs = find_stage_defs(&vfs);
        let stages_meta: Vec<(String, String)> = ume_formats::par::map(&stage_defs, |def| {
            let secs = parse_ini(&vfs.get(def).map(|v| text(&v)).unwrap_or_default());
            let info = find_section(&secs, "info");
            let name = info
                .and_then(|s| s.get("displayname").or(s.get("name")))
                .map(unquote)
                .filter(|s| !s.is_empty())
                .unwrap_or(def.clone());
            (name, stage_key(&vfs, def))
        });
        for (def, (name, key)) in stage_defs.into_iter().zip(stages_meta) {
            let id = format!("mugenstage:{tag}:{def}");
            if stage_ids.contains(&id) || stage_keys.contains(&key) {
                continue;
            }
            if !validate {
                ss.push(name.clone());
                stage_ids.insert(id.clone());
                stage_keys.insert(key.clone());
                self.stages.push(StageEntry {
                    key,
                    id,
                    name,
                    thumb: [[40, 40, 60], [20, 20, 30], [60, 60, 90]],
                    mugen: true,
                    src: StageSrc::Mugen {
                        vfs: vfs.clone(),
                        def,
                    },
                    preview: RefCell::new(Preview::Unknown),
                    stage: RefCell::new(None),
                });
                continue;
            }
            match load_stage(&vfs, &def) {
                Ok(st) => {
                    let prev = stages::mugen_preview(&st);
                    let arc_st = Arc::new(st);
                    ss.push(arc_st.name.clone());
                    stage_ids.insert(id.clone());
                    stage_keys.insert(key.clone());
                    self.stages.push(StageEntry {
                        key,
                        id,
                        name: arc_st.name.clone(),
                        thumb: [[40, 40, 60], [20, 20, 30], [60, 60, 90]],
                        mugen: true,
                        src: StageSrc::Mugen {
                            vfs: vfs.clone(),
                            def,
                        },
                        preview: RefCell::new(Preview::Ready(prev)),
                        stage: RefCell::new(Some(arc_st)),
                    });
                }
                Err(x) => errs.push(format!("{def}: {x}")),
            }
        }
        (cs, ss, errs)
    }
}