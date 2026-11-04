//! INI / AIR / VFS / zip / stage tests (synthetic).

mod common;
use common::*;
use ume_formats::air::{parse_air, Blend};
use ume_formats::ini::{find_section, nums, parse_float, parse_ini, parse_int, split_top, strip_comment, unquote};
use ume_formats::stage::load_stage;
use ume_formats::vfs::{add_zip, find_char_defs, find_stage_defs, norm_path, resolve, text, Vfs};

// ---------------------------------------------------------------- ini

#[test]
fn ini_basic() {
    let src = "\u{feff}top = 1\r\n[Info]\r\nName = \"Kung Fu Man\" ; comment\nname = dup\r[Files]\ncns = kfm.cns\n\n  ; only comment\n-1, 2 = x\nSprite=kfm.sff\n[ Data ]\nlife=1000\n";
    let secs = parse_ini(src);
    assert_eq!(secs.len(), 4);
    assert_eq!(secs[0].name, "");
    assert_eq!(secs[0].get("top"), Some("1"));
    let info = find_section(&secs, "INFO").unwrap();
    assert_eq!(info.name, "Info");
    assert_eq!(info.get("NAME"), Some("\"Kung Fu Man\""));
    assert_eq!(info.get_all("name").collect::<Vec<_>>(), vec!["\"Kung Fu Man\"", "dup"]);
    assert_eq!(unquote(info.get("name").unwrap()), "Kung Fu Man");
    let files = find_section(&secs, "files").unwrap();
    assert_eq!(files.entries.len(), 2); // "-1, 2 = x" is not a key
    assert_eq!(files.lines, vec!["cns = kfm.cns", "-1, 2 = x", "Sprite=kfm.sff"]);
    assert_eq!(files.get("sprite"), Some("kfm.sff"));
    assert_eq!(find_section(&secs, "data").unwrap().lname, "data");
}

#[test]
fn ini_helpers() {
    assert_eq!(strip_comment("a = \"x;y\" ; z"), "a = \"x;y\" ");
    assert_eq!(unquote("  \"abc\" trailing"), "abc");
    assert_eq!(unquote(" plain "), "plain");
    assert_eq!(unquote("\"open"), "open");
    assert_eq!(nums(Some("1, .5,abc, -2e1x"), &[9.0, 9.0, 7.0]), vec![1.0, 0.5, 7.0, -20.0]);
    assert_eq!(nums(None, &[320.0, 240.0]), vec![320.0, 240.0]);
    assert_eq!(nums(Some("  "), &[1.0]), vec![1.0]);
    assert_eq!(nums(Some("5,"), &[1.0, 2.0]), vec![5.0, 2.0]);
    assert_eq!(split_top("a, f(b, c), [d,e], \"x,y\" ,z"), vec!["a", "f(b, c)", "[d,e]", "\"x,y\"", "z"]);
    assert_eq!(parse_float("  -.25abc"), Some(-0.25));
    assert_eq!(parse_float("1e"), Some(1.0));
    assert_eq!(parse_float("."), None);
    assert_eq!(parse_int("12.9"), Some(12));
    assert_eq!(parse_int(" -0x1f"), Some(-31));
    assert_eq!(parse_int("x"), None);
}

// ---------------------------------------------------------------- air

const AIR: &str = r#"
; test
[Begin Action 0]
Clsn2Default: 2
 Clsn2[0] = -13,  0, 16,-79
 Clsn2[1] =   5,-79, -7,-93
0,0, 0,0, 10
Clsn1: 1
 Clsn1[0] = 10,-60,40,-50
0,1, 1,-2, 5, H, A
Loopstart
0,2, 0,0, 7, VH, AS128D64, 2, 0.5, 45
0,3, 0,0, -1,,S
[Begin Action 20] ; second
Clsn2: 1
 Clsn2[0] = 0,0,1,1
5,0, 0,0, 3,  , A1
Interpolate Offset
5,1, 0,0, 3
[Statedef 10]
0,0,0,0,1
[begin   action  -5]
1,1,0,0,abc
"#;

#[test]
fn air_parse() {
    let a = parse_air(AIR);
    assert_eq!(a.len(), 3);
    let a0 = &a[&0];
    assert_eq!(a0.frames.len(), 4);
    assert_eq!(a0.loop_start, 2);
    assert_eq!(a0.total_time, -1);
    let f0 = &a0.frames[0];
    assert_eq!(f0.clsn2, vec![[-13.0, -79.0, 16.0, 0.0], [-7.0, -93.0, 5.0, -79.0]]);
    assert!(f0.clsn1.is_empty());
    let f1 = &a0.frames[1];
    assert_eq!((f1.group, f1.index, f1.x, f1.y, f1.time), (0, 1, 1.0, -2.0, 5));
    assert!(f1.flip_h && !f1.flip_v);
    assert_eq!(f1.blend, Blend::Add);
    assert_eq!(f1.clsn1, vec![[10.0, -60.0, 40.0, -50.0]]);
    assert_eq!(f1.clsn2.len(), 2); // default still applies
    let f2 = &a0.frames[2];
    assert!(f2.flip_h && f2.flip_v);
    assert_eq!(f2.blend, Blend::Add);
    assert_eq!(f2.alpha, Some((128, 64)));
    assert_eq!((f2.xscale, f2.yscale, f2.angle), (2.0, 0.5, 45.0));
    assert!(f2.clsn1.is_empty()); // one-frame Clsn1 consumed
    let f3 = &a0.frames[3];
    assert_eq!(f3.time, -1);
    assert_eq!(f3.blend, Blend::Sub);
    assert_eq!((f3.xscale, f3.yscale), (1.0, 1.0));

    let a20 = &a[&20];
    assert_eq!(a20.frames.len(), 2);
    assert_eq!(a20.total_time, 6);
    assert_eq!(a20.loop_start, 0);
    assert_eq!(a20.frames[0].blend, Blend::Add1);
    assert_eq!(a20.frames[0].clsn2.len(), 1);
    assert!(a20.frames[1].clsn2.is_empty());

    let a5 = &a[&-5];
    assert_eq!(a5.frames[0].time, 1); // NaN time -> 1
    assert!(!a.contains_key(&10));
}

#[test]
fn air_loopstart_past_end_resets() {
    let a = parse_air("[Begin Action 1]\n0,0,0,0,4\nLoopStart\n");
    assert_eq!(a[&1].loop_start, 0);
    assert_eq!(a[&1].total_time, 4);
}

// ---------------------------------------------------------------- vfs / zip

#[test]
fn vfs_norm_and_text() {
    assert_eq!(norm_path(".\\Chars\\\\KFM//Kfm.DEF"), "chars/kfm/kfm.def");
    assert_eq!(text(&[0xef, 0xbb, 0xbf, b'a', 0xe9]), "a\u{e9}");
}

#[test]
fn zip_round_trip_and_discovery() {
    let def = b"[Info]\nname=\"Test\"\n[Files]\ncns = t.cns\nsprite= t.sff\nanim=sub\\t.air\n";
    let stage = b"[Info]\nname=Stg\n[Camera]\nboundleft=-100\n[BGdef]\nspr=missing.sff\n[BG 0]\ntype=normal\nspriteno=1\ndelta=.5\n";
    let big: Vec<u8> = (0..20000u32).map(|i| (i % 251) as u8).collect();
    let zip = encode_zip(&[
        ("Test/", b"", false),
        ("Test/T.def", def, true),
        ("Test/sub/T.air", b"[Begin Action 0]\n0,0,0,0,1\n", false),
        ("Test/t.sff", &big, true),
        ("Test/stage.def", stage, false),
    ]);
    let mut vfs = Vfs::new();
    let n = add_zip(&mut vfs, &zip, "").unwrap();
    assert_eq!(n, 4);
    assert_eq!(vfs["test/t.def"], def.to_vec());
    assert_eq!(vfs["test/t.sff"], big);
    assert_eq!(find_char_defs(&vfs), vec!["test/t.def"]);
    assert_eq!(find_stage_defs(&vfs), vec!["test/stage.def"]);
    assert_eq!(resolve(&vfs, "test", "sub\\T.air").map(|b| b.len()), Some("[Begin Action 0]\n0,0,0,0,1\n".len()));
    assert_eq!(resolve(&vfs, "test", "\"t.sff\"").map(|b| b.len()), Some(20000));
    assert!(resolve(&vfs, "test", "elsewhere/T.AIR").is_some()); // basename fallback
    assert!(resolve(&vfs, "test", "nope.cns").is_none());
    assert!(resolve(&vfs, "test", "").is_none());

    let mut vfs2 = Vfs::new();
    add_zip(&mut vfs2, &zip, "Packs\\One/").unwrap();
    assert!(vfs2.contains_key("packs/one/test/t.def"));

    assert!(add_zip(&mut Vfs::new(), b"definitely not a zip file", "").is_err());

    let st = load_stage(&vfs, "Test/Stage.def").unwrap();
    assert_eq!(st.name, "Stg");
    assert_eq!(st.local_w, 320.0);
    assert_eq!(st.bound_left, -100.0);
    assert_eq!(st.bound_right, 150.0);
    assert_eq!(st.zoffset, 200.0);
    assert_eq!(st.layers.len(), 1);
    assert_eq!(st.layers[0].sprite, (1, 0));
    assert_eq!(st.layers[0].delta, [0.5, 0.5]);
    assert!(st.sprites.is_empty());
    assert!(load_stage(&vfs, "nope.def").is_err());
}

#[test]
fn stage_multi_layer_and_camera_bounds() {
    let stage_def = br#"
[Info]
name = "Dojo"
displayname = "The Great Dojo"

[StageInfo]
zoffset = 215
localcoord = 640, 480

[Camera]
boundleft = -220
boundright = 220
boundhigh = -80
boundlow = 10
verticalfollow = 0.5
tension = 75

[PlayerInfo]
p1startx = -80
p2startx = 80

[BGdef]
spr = dojo.sff

[BG Background]
type = normal
spriteno = 0, 0
start = 0, 0
delta = 0.75, 0.75
mask = 1
tile = 1, 0
tilespacing = 0, 0

[BG AnimatedLantern]
type = anim
actionno = 5
start = 50, -40
delta = 1.0, 1.0
layerno = 1
trans = add

[Begin Action 5]
0,1, 0,0, 5
0,2, 0,0, 5
"#;

    let mut vfs = Vfs::new();
    vfs.insert("stages/dojo.def".into(), stage_def.to_vec());

    let st = load_stage(&vfs, "stages/dojo.def").unwrap();
    assert_eq!(st.name, "The Great Dojo");
    assert_eq!(st.local_w, 640.0);
    assert_eq!(st.zoffset, 215.0);
    assert_eq!(st.bound_left, -220.0);
    assert_eq!(st.bound_right, 220.0);
    assert_eq!(st.bound_high, -80.0);
    assert_eq!(st.bound_low, 10.0);
    assert_eq!(st.vertical_follow, 0.5);
    assert_eq!(st.tension, 75.0);
    assert_eq!(st.p1x, -80.0);
    assert_eq!(st.p2x, 80.0);

    assert_eq!(st.layers.len(), 2);

    // Layer 0: Normal static BG
    assert_eq!(st.layers[0].kind, "normal");
    assert_eq!(st.layers[0].sprite, (0, 0));
    assert_eq!(st.layers[0].delta, [0.75, 0.75]);
    assert!(st.layers[0].mask);
    assert_eq!(st.layers[0].tile, [1, 0]);
    assert_eq!(st.layers[0].layer, 0);

    // Layer 1: Animated BG
    assert_eq!(st.layers[1].kind, "anim");
    assert_eq!(st.layers[1].start, [50.0, -40.0]);
    assert_eq!(st.layers[1].layer, 1);
    assert_eq!(st.layers[1].trans, "add");
    assert!(st.layers[1].anim.is_some());
    let anim = st.layers[1].anim.as_ref().unwrap();
    assert_eq!(anim.frames.len(), 2);
    assert_eq!(anim.total_time, 10);
}

#[test]
fn stage_layer_hd_parameters() {
    let stage_def = br#"
[BGdef]
spr = none.sff

[BG Hd]
type = normal
spriteno = 0, 0
delta = 0.5, 0.25
scalestart = 2, 3
scaledelta = 0.01, 0.02
angle = 15
xshear = 0.5
window = 10, 20, 110, 120
maskwindow = 0, 40, 100, 200
windowdelta = 0.3, 0.6
spacing = 4, 6

[BG Plain]
type = normal
spriteno = 1, 0
delta = 1, 1
tilespacing = 7, 8
spacing = 1, 1
maskwindow = 5, 6, 7, 8
"#;
    let mut vfs = Vfs::new();
    vfs.insert("stages/hd.def".into(), stage_def.to_vec());
    let st = load_stage(&vfs, "stages/hd.def").unwrap();
    let hd = &st.layers[0];
    assert_eq!(hd.scale_start, [2.0, 3.0]);
    assert_eq!(hd.scale_delta, [0.01, 0.02]);
    assert_eq!(hd.angle, 15.0);
    assert_eq!(hd.x_shear, 0.5);
    assert_eq!(hd.window, Some([10.0, 20.0, 110.0, 120.0]));
    assert_eq!(hd.mask_window, Some([0.0, 40.0, 100.0, 200.0]));
    assert_eq!(hd.clip_window(), Some([10.0, 40.0, 100.0, 120.0]));
    assert_eq!(hd.window_scroll(), [0.3, 0.6]);
    assert_eq!(hd.tile_spacing, [4.0, 6.0]);

    let camera = [100.0, 50.0];
    let scale = hd.scale_at(camera);
    assert!((scale[0] - (2.0 + 0.01 * 100.0 * 0.5)).abs() < 1e-5);
    assert!((scale[1] - (3.0 + 0.02 * 50.0 * 0.25)).abs() < 1e-5);
    assert!(hd.scale_at([200.0, 50.0])[0] > scale[0]);

    let plain = &st.layers[1];
    assert_eq!(plain.scale_start, [1.0, 1.0]);
    assert_eq!(plain.scale_at([300.0, 300.0]), [1.0, 1.0]);
    assert_eq!(plain.angle, 0.0);
    assert_eq!(plain.x_shear, 0.0);
    assert_eq!(plain.window, None);
    assert_eq!(plain.clip_window(), Some([5.0, 6.0, 7.0, 8.0]));
    assert_eq!(plain.window_scroll(), [1.0, 1.0]);
    assert_eq!(plain.tile_spacing, [7.0, 8.0]);
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn add_dir_expands_zips() {
    use ume_formats::vfs::add_dir;
    let root = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("ume_formats_add_dir");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("Chars/Pack")).unwrap();
    std::fs::write(root.join("Chars/Loose.DEF"), b"x").unwrap();
    std::fs::write(root.join("Chars/Pack/c.zip"), encode_zip(&[("Guy/guy.def", b"[Files]\ncns=a\nsprite=b\n", true)])).unwrap();
    std::fs::write(root.join("Chars/Pack/bad.zip"), b"not a zip").unwrap();
    let mut vfs = Vfs::new();
    let n = add_dir(&mut vfs, &root, "My\\Root").unwrap();
    assert_eq!(n, 3);
    let keys: Vec<&str> = vfs.keys().map(|k| k.as_str()).collect();
    assert_eq!(keys, vec!["my/root/chars/loose.def", "my/root/chars/pack/bad.zip", "my/root/chars/pack/guy/guy.def"]);
    assert_eq!(find_char_defs(&vfs), vec!["my/root/chars/pack/guy/guy.def"]);
    let _ = std::fs::remove_dir_all(&root);
}
