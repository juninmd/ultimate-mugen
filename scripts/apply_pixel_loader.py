with open('crates/engine/src/native/compile.rs', 'r', encoding='utf-8') as f:
    lines = f.readlines()

idx_start = None
idx_end = None
for i, line in enumerate(lines):
    if line.strip().startswith('fn render_frame('):
        idx_start = i
    if idx_start is not None and line.strip() == '}' and i > idx_start:
        idx_end = i + 1
        break

print(f"Replacing lines {idx_start} to {idx_end}")

replacement = """fn try_load_pixel_sprite(nc: &NativeChar, fr: &NativeFrame) -> Option<RigRender> {
    let spr_name = if let Some(s) = &fr.sprite {
        s.as_str()
    } else if let Some(Value::String(p)) = &fr.pose {
        p.as_str()
    } else {
        "idle"
    };
    let candidates = [
        format!("assets/chars/sprites/{}/{}.png", nc.id, spr_name),
        format!("{}/../../assets/chars/sprites/{}/{}.png", env!("CARGO_MANIFEST_DIR"), nc.id, spr_name),
        format!("assets/chars/sprites/{}/idle.png", nc.id),
        format!("{}/../../assets/chars/sprites/{}/idle.png", env!("CARGO_MANIFEST_DIR"), nc.id),
    ];
    for p in candidates {
        if let Ok(bytes) = std::fs::read(&p) {
            if let Ok(dec) = ume_formats::png::decode_png(&bytes) {
                if let Some(rgba) = dec.rgba {
                    let img = ImageData::new(dec.width, dec.height, rgba);
                    let scale = 100.0 / (dec.height as f32).max(1.0);
                    return Some(RigRender {
                        image: Some(img),
                        width: dec.width as f32,
                        height: dec.height as f32,
                        axis_x: dec.width as f32 / 2.0,
                        axis_y: dec.height as f32,
                        scale,
                        hurt: vec![[-18.0, -96.0, 18.0, 0.0]],
                        hit: vec![],
                    });
                }
            }
        }
    }
    None
}

fn render_frame(nc: &NativeChar, fr: &NativeFrame, with_images: bool, palette: Option<&Value>) -> RigRender {
    let use_pixel = nc.art.kind == "sprites" || nc.art.kind == "pixel" || std::env::var("UME_PIXEL_ART").is_ok();
    if use_pixel {
        if let Some(r) = try_load_pixel_sprite(nc, fr) {
            return r;
        }
    }
    if nc.art.kind != "rig" {
        RigRender { image: None, width: 0.0, height: 0.0, axis_x: 0.0, axis_y: 0.0, scale: 1.0, hurt: vec![], hit: vec![] }
    } else if let Some(e) = &fr.effect {
        render_effect(e, with_images, palette)
    } else {
        let pose = match &fr.pose { Some(Value::String(s)) => nc.art.poses.get(s).cloned().unwrap_or(Value::Null), Some(v) => v.clone(), None => Value::Null };
        render_pose(&nc.art.style, &pose, fr.hit.as_ref(), with_images, palette)
    }
}
"""

new_lines = lines[:idx_start] + [replacement] + lines[idx_end:]
with open('crates/engine/src/native/compile.rs', 'w', encoding='utf-8') as f:
    f.writelines(new_lines)
print("Updated compile.rs successfully")
