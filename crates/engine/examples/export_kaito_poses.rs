use serde_json::Value;
use tiny_skia::Pixmap;
use ume_engine::chardef::ImageData;
use ume_engine::native::rig::{render_pose, set_art_res};

fn to_pixmap(img: &ImageData) -> Option<Pixmap> {
    let mut pm = Pixmap::new(img.w, img.h)?;
    for (d, s) in pm.data_mut().chunks_exact_mut(4).zip(img.rgba.chunks_exact(4)) {
        let a = s[3] as u32;
        for i in 0..3 { d[i] = ((s[i] as u32 * a + 127) / 255) as u8; }
        d[3] = s[3];
    }
    Some(pm)
}

fn main() {
    set_art_res(3.0);
    let path = format!("{}/../../assets/chars/kaito.json", env!("CARGO_MANIFEST_DIR"));
    let ch: Value = serde_json::from_str(&std::fs::read_to_string(&path).expect("character json")).expect("json");
    let style = &ch["art"]["style"];
    let poses = &ch["art"]["poses"];
    let anims = &ch["anims"];

    std::fs::create_dir_all("proto_sprites").unwrap();

    let targets = [
        ("before_01_idle.png", &anims["idle"]["frames"][0]),
        ("before_02_jab.png", &anims["jab"]["frames"][1]),
        ("before_03_kick.png", &anims["hkick"]["frames"][2]),
        ("before_04_uppercut.png", &anims["rise"]["frames"][1]),
        ("before_05_blast.png", &anims["gale"]["frames"][3]),
        ("before_06_victory.png", &anims["win"]["frames"][1]),
    ];

    for (name, fr) in &targets {
        let pose = match &fr["pose"] {
            Value::String(s) => poses.get(s).cloned().unwrap_or(Value::Null),
            v => v.clone(),
        };
        let r = render_pose(style, &pose, None, true, None);
        if let Some(src) = r.image.as_ref().and_then(|i| to_pixmap(i)) {
            let out_path = format!("proto_sprites/{name}");
            src.save_png(&out_path).expect("save png");
            println!("Exported {out_path} ({}x{})", src.width(), src.height());
        }
    }

    // Export Shion idle facing left
    let shion_p = format!("{}/../../assets/chars/shion.json", env!("CARGO_MANIFEST_DIR"));
    let shion_ch: Value = serde_json::from_str(&std::fs::read_to_string(&shion_p).unwrap()).unwrap();
    let s_pose = match &shion_ch["anims"]["idle"]["frames"][0]["pose"] {
        Value::String(s) => shion_ch["art"]["poses"][s.as_str()].clone(),
        v => v.clone(),
    };
    let s_r = render_pose(&shion_ch["art"]["style"], &s_pose, None, true, None);
    if let Some(src) = s_r.image.as_ref().and_then(|i| to_pixmap(i)) {
        src.save_png("proto_sprites/shion_idle.png").unwrap();
        println!("Exported proto_sprites/shion_idle.png");
    }
}
