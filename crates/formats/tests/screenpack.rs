mod common;
use common::*;
use ume_formats::fnt::{decode_fnt, FontKind, Glyph};
use ume_formats::screenpack::{load_screenpack, parse_fight, parse_select, parse_storyboard, parse_system, SelectSlot};
use ume_formats::vfs::Vfs;

fn fnt_v1(map: &str) -> Vec<u8> {
    let px: Vec<u8> = (0..32 * 8).map(|i| (i % 7) as u8).collect();
    let pal = make_rgb_palette(3);
    let pcx = encode_pcx(32, 8, &px, Some(&pal));
    let txt = map.as_bytes();
    let mut w = Writer::new();
    w.str("ElecbyteFnt\0").u16(0).u16(1).u32(64).u32(pcx.len() as u32).u32(64 + pcx.len() as u32).u32(txt.len() as u32).pad(64);
    w.raw(&pcx).raw(txt);
    w.out()
}

#[test]
fn fnt_v1_variable_and_fixed() {
    let f = decode_fnt(&fnt_v1("[Def]\nSize = 8,8\nSpacing = 1,0\nColors = 4\nOffset = 0,1\nType = Variable\n[Map]\nA 0 6\nB 6 7\n0x20 13 4\n")).unwrap();
    assert_eq!(f.kind, FontKind::Variable);
    assert_eq!(f.size, [8, 8]);
    assert_eq!(f.offset, [0, 1]);
    assert_eq!(f.glyph('B'), Some(Glyph { x: 6, w: 7 }));
    assert_eq!(f.glyph(' '), Some(Glyph { x: 13, w: 4 }));
    assert_eq!(f.text_width("AB"), 6 + 7 + 1);
    assert_eq!(f.image.as_ref().map(|i| (i.width, i.height)), Some((32, 8)));
    assert!(f.palette_bank(1).is_some());

    let f = decode_fnt(&fnt_v1("[Def]\nSize = 4,8\nType = Fixed\n[Map]\na\nb\nz 7\n")).unwrap();
    assert_eq!(f.kind, FontKind::Fixed);
    assert_eq!(f.glyph('b'), Some(Glyph { x: 4, w: 4 }));
    assert_eq!(f.glyph('z'), Some(Glyph { x: 28, w: 4 }));
    assert_eq!(f.glyph('q'), None);
}

#[test]
fn fnt_v2_def() {
    let f = decode_fnt(b"[Def]\nType = truetype\nSize = 0,24\nFile = \"arial.ttf\"\n").unwrap();
    assert_eq!(f.kind, FontKind::TrueType);
    assert_eq!(f.file.as_deref(), Some("arial.ttf"));
    assert!(decode_fnt(b"garbage").is_err());
}

const SYSTEM: &str = "[Info]\nlocalcoord = 640,480\n[Files]\nspr = system.sff\nsnd = system.snd\nlogo.storyboard = logo.def\nselect = select.def\nfight = fight.def\nfont1 = f-4x6.fnt\nfont2 = missing.fnt\n[Music]\ntitle.bgm = sound/title.ogg\ntitle.bgm.loop = 1\ntitle.bgm.loopstart = 4410\ntitle.bgm.loopend = 88200\ntitle.bgm.volume = 80\n[Title Info]\nmenu.itemname.arcade = \"ARCADE\"\nmenu.itemname.versus = \"VS MODE\"\nmenu.itemname.exit = \"EXIT\"\n[TitleBGdef]\nbgclearcolor = 10,20,30\n[TitleBG 0]\ntype = normal\nspriteno = 5,1\nstart = 0,10\ndelta = 0.5\nvelocity = 1,0\n[TitleBG 1]\ntype = anim\nactionno = 100\n[Begin Action 100]\n0,0, 0,0, 5\n[Select Info]\nrows = 3\ncolumns = 8\nwrapping = 1\npos = 90,170\ncell.size = 27,27\ncell.spacing = 2\np2.cursor.startcell = 0,7\n";

#[test]
fn system_def() {
    let s = parse_system(SYSTEM);
    assert_eq!(s.localcoord, [640.0, 480.0]);
    assert_eq!(s.fonts.get(&1).map(String::as_str), Some("f-4x6.fnt"));
    assert_eq!(s.menu.iter().map(|m| m.label.as_str()).collect::<Vec<_>>(), vec!["ARCADE", "VS MODE", "EXIT"]);
    let bgm = &s.music["title"];
    assert_eq!((bgm.file.as_str(), bgm.loop_start, bgm.loop_end, bgm.volume, bgm.looping), ("sound/title.ogg", 4410, 88200, 80, true));
    let bg = s.background("titlebg");
    assert_eq!(bg.clear_color, Some([10, 20, 30]));
    assert_eq!(bg.layers.len(), 2);
    assert_eq!(bg.layers[0].sprite, (5, 1));
    assert_eq!(bg.layers[0].delta, [0.5, 0.5]);
    assert!(bg.layers[1].anim.is_some());
    let g = s.select_grid();
    assert_eq!((g.rows, g.columns, g.wrapping), (3, 8, true));
    assert_eq!(g.cell_spacing, [2.0, 2.0]);
    assert_eq!(g.p2_start, [0.0, 7.0]);
}

#[test]
fn select_def() {
    let s = parse_select("[Characters]\nkfm, stages/kfm.def, music=sound/kfm.mp3, order=2\nrandomselect\nempty\n\"Evil Ken/evil.def\"\n[ExtraStages]\nstages/stage0.def\n[Options]\narcade.maxmatches = 6,1,1,0\n");
    assert_eq!(s.slots.len(), 4);
    match &s.slots[0] {
        SelectSlot::Char { path, stage, params } => {
            assert_eq!(path, "kfm");
            assert_eq!(stage.as_deref(), Some("stages/kfm.def"));
            assert_eq!(params.get("order").map(String::as_str), Some("2"));
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(s.slots[1], SelectSlot::Random);
    assert_eq!(s.slots[2], SelectSlot::Empty);
    assert_eq!(s.chars().collect::<Vec<_>>(), vec!["kfm", "Evil Ken/evil.def"]);
    assert_eq!(s.extra_stages, vec!["stages/stage0.def"]);
    assert_eq!(s.options.get("arcade.maxmatches").map(String::as_str), Some("6,1,1,0"));
}

#[test]
fn fight_def() {
    let f = parse_fight("[Files]\nsff = fight.sff\nfightfx.air = fightfx.air\nfont2 = num1.fnt\n[Lifebar]\np1.pos = 140,12\np1.range.x = 0,-127\np1.bg0.spr = 10,0\np1.mid.spr = 11,0\np1.front.anim = 12\np2.pos = 180,12\np2.bg0.spr = 10,0\np2.bg0.facing = -1\n[Round]\nround.default.font = 3,0,0\nround.default.text = \"Round %i\"\nround.default.offset = 160,120\n");
    assert_eq!(f.sff.as_deref(), Some("fight.sff"));
    assert_eq!(f.fonts.get(&2).map(String::as_str), Some("num1.fnt"));
    let p1 = f.life_bar(1);
    assert_eq!(p1.pos, [140.0, 12.0]);
    assert_eq!(p1.range_x, [0.0, -127.0]);
    assert_eq!(p1.bg0.spr, Some((10, 0)));
    assert_eq!(p1.front.anim, Some(12));
    assert!(!p1.bg1.is_visible());
    assert_eq!(f.life_bar(2).bg0.facing, -1);
    let r = f.element("round", "round.default");
    assert_eq!(r.font, Some([3, 0, 0]));
    assert_eq!(r.text.as_deref(), Some("Round %i"));
    assert_eq!(r.offset, [160.0, 120.0]);
}

#[test]
fn storyboard() {
    let s = parse_storyboard("[SceneDef]\nspr = logo.sff\nstartscene = 0\n[Scene 0]\nfadein.time = 20\nfadeout.time = 30\nclearcolor = 0,0,0\nend.time = 120\nlayerall.pos = 160,120\nlayer0.anim = 0\nlayer0.starttime = 5\nlayer1.text = \"Presents\"\nlayer1.font = 1,0,0\nbgm = intro.ogg\n[Scene 1]\nend.time = 60\n[Begin Action 0]\n0,0, 0,0, -1\n");
    assert_eq!(s.spr.as_deref(), Some("logo.sff"));
    assert_eq!(s.scenes.len(), 2);
    let sc = &s.scenes[0];
    assert_eq!((sc.fade_in, sc.fade_out, sc.end_time), (20, 30, 120));
    assert_eq!(sc.layers.len(), 2);
    assert_eq!(sc.layers[0].element.anim, Some(0));
    assert_eq!(sc.layers[0].start_time, 5);
    assert_eq!(sc.layers[1].element.text.as_deref(), Some("Presents"));
    assert_eq!(sc.bgm.as_deref(), Some("intro.ogg"));
    assert_eq!(s.total_ticks(), 180);
    assert!(s.anims.contains_key(&0));
}

#[test]
fn load_from_vfs_reports_missing_files_without_failing() {
    let mut vfs = Vfs::new();
    vfs.insert("data/system.def".into(), SYSTEM.as_bytes().to_vec());
    vfs.insert("data/select.def".into(), b"[Characters]\nkfm\n".to_vec());
    vfs.insert("data/fight.def".into(), b"[Files]\nfont1 = f-4x6.fnt\n[Lifebar]\np1.pos = 1,2\n".to_vec());
    vfs.insert("font/f-4x6.fnt".into(), fnt_v1("[Def]\nSize = 4,6\nType = Fixed\n[Map]\na\n"));
    vfs.insert("data/logo.def".into(), b"[SceneDef]\n[Scene 0]\nend.time = 10\n".to_vec());
    let sp = load_screenpack(&vfs, "data/system.def").unwrap();
    assert_eq!(sp.select.chars().collect::<Vec<_>>(), vec!["kfm"]);
    assert_eq!(sp.fight.life_bar(1).pos, [1.0, 2.0]);
    assert!(sp.fonts.contains_key(&1) && sp.fight_fonts.contains_key(&1));
    assert_eq!(sp.logo.as_ref().map(|l| l.total_ticks()), Some(10));
    assert!(sp.warnings.iter().any(|w| w.contains("missing.fnt")));
    assert!(sp.warnings.iter().any(|w| w.contains("system.sff")));
    assert!(load_screenpack(&vfs, "nope/system.def").is_err());
}
