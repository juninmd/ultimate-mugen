use super::*;

#[test]
fn loads_umf_json_from_folder_without_rebuild() {
    let dir = std::env::temp_dir().join(format!("ume-umf-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let mut v: serde_json::Value = serde_json::from_str(BUILTIN[0].1).unwrap();
    v["id"] = "folder_test".into();
    v["name"] = "Folder Test".into();
    std::fs::write(dir.join("folder_test.json"), v.to_string()).unwrap();
    std::fs::write(dir.join("broken.json"), "{").unwrap();
    let mut reg = Registry::new();
    let before = reg.chars.len();
    assert_eq!(reg.scan_umf_dir(&dir), 1);
    assert_eq!(reg.chars.len(), before + 1);
    let e = reg
        .chars
        .iter()
        .find(|c| c.id == "umf:folder_test")
        .unwrap();
    assert_eq!(e.name, "Folder Test");
    assert!(e.load(1).is_ok());
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn mugen_previews_stream_without_blocking_the_caller() {
    let zip = include_bytes!("../../assets/mugen/elecbyte.zip");
    let mut reg = Registry::new();
    reg.import_zip(zip).unwrap();
    let e = reg
        .chars
        .iter()
        .find(|c| c.mugen && !c.broken.get())
        .unwrap();
    let t = std::time::Instant::now();
    e.stream_preview(1);
    e.stream_portrait();
    let call_ms = t.elapsed().as_secs_f64() * 1000.0;
    assert!(call_ms < 25.0, "stream calls took {call_ms} ms");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    while (e.peek(1).is_none() || !e.has_portrait_attempt()) && std::time::Instant::now() < deadline
    {
        e.reap();
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(e.peek(1).is_some() && e.has_portrait_attempt() && !e.streaming());
}

#[test]
fn read_zip_matches_import_zip() {
    let zip = include_bytes!("../../assets/mugen/elecbyte.zip");
    let (pack, vfs) = read_zip(zip).unwrap();
    let mut reg = Registry::new();
    let r = reg.finish_import(pack.clone(), vfs);
    assert!(r.chars > 0 && !r.duplicate_pack);
    let (_, vfs2) = read_zip(zip).unwrap();
    assert!(reg.finish_import(pack, vfs2).duplicate_pack);
}

#[test]
fn importing_the_same_pack_twice_does_not_duplicate() {
    let zip = include_bytes!("../../assets/mugen/elecbyte.zip");
    let mut reg = Registry::new();
    let r1 = reg.import_zip(zip).unwrap();
    assert!(r1.chars > 0 && !r1.duplicate_pack);
    let (c, s) = (reg.chars.len(), reg.stages.len());
    let r2 = reg.import_zip(zip).unwrap();
    assert!(r2.duplicate_pack);
    assert_eq!((reg.chars.len(), reg.stages.len()), (c, s));
}

#[test]
fn same_content_from_another_source_is_skipped_and_dedupe_cleans_old_rosters() {
    let load = || {
        let mut v = Vfs::new();
        ume_formats::vfs::add_zip(
            &mut v,
            include_bytes!("../../assets/mugen/elecbyte.zip"),
            "",
        )
        .unwrap();
        v
    };
    let mut reg = Registry::new();
    reg.add_vfs_lazy(load(), "bundled");
    let (c, s) = (reg.chars.len(), reg.stages.len());
    reg.add_vfs_lazy(load(), "web123-0");
    reg.add_vfs_lazy(load(), "/some/other/dir");
    assert_eq!((reg.chars.len(), reg.stages.len()), (c, s));
    let dup_key = reg.chars.iter().find(|c| c.mugen).unwrap().key.clone();
    let e = CharEntry {
        key: dup_key,
        id: "mugen:legacy:chars/kfm/kfm.def".into(),
        name: "Legacy".into(),
        series: String::new(),
        author: String::new(),
        color: [0; 3],
        description: String::new(),
        mugen: true,
        boss: false,
        src: Src::Mugen {
            vfs: Arc::new(Vfs::new()),
            def: String::new(),
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
    reg.chars.push(e);
    assert_eq!(reg.dedupe(), 1);
    assert_eq!(reg.chars.len(), c);
}

#[test]
fn scanning_folders_repeatedly_is_idempotent() {
    let mut reg = Registry::new();
    reg.add_bundled_mugen();
    reg.scan_folders();
    let (c, s) = (reg.chars.len(), reg.stages.len());
    reg.scan_folders();
    reg.add_bundled_mugen();
    assert_eq!((reg.chars.len(), reg.stages.len()), (c, s));
}

#[test]
fn mugen_stages_produce_cached_previews() {
    let mut reg = Registry::new();
    reg.add_bundled_mugen();
    reg.scan_folders();
    let mugen_indices: Vec<usize> = reg
        .stages
        .iter()
        .enumerate()
        .filter(|(_, s)| s.mugen)
        .map(|(i, _)| i)
        .collect();
    assert!(!mugen_indices.is_empty());
    let mut produced = 0;
    for i in mugen_indices {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        let bundled = reg.stages[i].id.contains(":bundled:");
        loop {
            if let Some(p) = reg.stage_preview(i) {
                assert_eq!((p.w, p.h), (320, 180));
                assert!(p
                    .rgba
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .any(|px| px[..3] != [20, 20, 30]));
                assert_eq!(
                    reg.stage_preview(i).unwrap().id,
                    p.id,
                    "preview must be cached"
                );
                produced += 1;
                break;
            }
            // folder content may legitimately have no drawable layer: it must
            // still finish instead of spinning on the worker
            if reg.preview_settled(i) && !bundled {
                break;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "missing preview for {}",
                reg.stages[i].id
            );
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
    }
    assert!(produced > 0, "no stage produced a thumbnail");
}

#[test]
fn mugen_preview_handles_extreme_tiling_and_position_link_visibility() {
    let mut vfs = Vfs::new();
    ume_formats::vfs::add_zip(
        &mut vfs,
        include_bytes!("../../assets/mugen/elecbyte.zip"),
        "",
    )
    .unwrap();
    let mut st = load_stage(&vfs, "stages/stage0.def").unwrap();
    if st.layers.len() > 1 {
        st.layers[0].id = 77;
        st.layers[1].position_link = true;
        st.ctrl_defs.push(ume_formats::stage::BgCtrlDef {
            loop_time: -1,
            ids: vec![77],
            ctrls: vec![ume_formats::stage::BgCtrl {
                kind: ume_formats::stage::BgCtrlKind::Visible(false),
                start: 0,
                end: 0,
                loop_time: -1,
                ids: vec![77],
            }],
        });
    }
    let last = st.layers.last_mut().unwrap();
    last.tile = [i32::MAX, i32::MAX];
    last.start = [-1e20, 0.0];
    let prev = stages::mugen_preview(&st);
    if st.layers.len() <= 2 {
        assert!(prev.is_none());
    } else {
        assert!(prev.is_some());
    }
}

#[test]
fn broken_characters_are_quarantined_not_catalogued() {
    let mut vfs = Vfs::new();
    let def = "[Info]\nname = \"Whole\"\n[Files]\nsprite = w.sff\nanim = w.air\ncns = w.cns\n";
    vfs.insert("c/whole/whole.def".into(), def.as_bytes().to_vec());
    for f in ["w.sff", "w.air", "w.cns"] {
        vfs.insert(format!("c/whole/{f}"), b"x".to_vec());
    }
    vfs.insert(
        "c/broken/broken.def".into(),
        b"[Info]\nname = \"Broken\"\n[Files]\nsprite = gone.sff\nanim = b.air\ncns = b.cns\n"
            .to_vec(),
    );
    vfs.insert("c/broken/b.air".into(), b"x".to_vec());
    vfs.insert("c/broken/b.cns".into(), b"x".to_vec());
    let mut reg = Registry::new();
    let before = reg.chars.iter().filter(|c| c.mugen).count();
    reg.add_vfs_lazy(vfs, "pack");
    let names: Vec<&str> = reg
        .chars
        .iter()
        .filter(|c| c.mugen)
        .map(|c| c.name.as_str())
        .collect();
    assert_eq!(reg.chars.iter().filter(|c| c.mugen).count(), before + 1);
    assert!(names.contains(&"Whole"));
    assert_eq!(reg.quarantined.len(), 1);
    assert_eq!(reg.quarantined[0].name, "Broken");
    assert_eq!(
        reg.quarantined[0].problems,
        vec!["sprite 'gone.sff' não existe"]
    );
}
