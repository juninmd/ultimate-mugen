use super::{render_frame, CharDef, HashMap, NativeChar, NativeFrame, NativeMove, Sprite, Value};

pub(super) fn awakening_palette(nc: &NativeChar, palette: Option<&Value>) -> Option<Value> {
    let awake = nc.awakening_palette.as_ref()?;
    let mut colors = palette.and_then(|p| p.get("colors").unwrap_or(p).as_object()).cloned().unwrap_or_default();
    colors.extend(awake.get("colors").unwrap_or(awake).as_object()?.clone());
    colors.insert("_awakening".into(), Value::Bool(true));
    Some(Value::Object(colors))
}

pub(super) fn insert_awakened(sprites: &mut HashMap<(i32, i32), Sprite>, nc: &NativeChar, fr: &NativeFrame, index: i32, with_images: bool, palette: Option<&Value>) {
    if !with_images || fr.effect.is_some() { return; }
    if let Some(palette) = palette {
        let awake = render_frame(nc, fr, true, Some(palette));
        sprites.insert((2, index), Sprite { image: awake.image, width: awake.width, height: awake.height, axis_x: awake.axis_x, axis_y: awake.axis_y, scale: awake.scale });
    }
}

pub(super) fn voice_state(m: &NativeMove, sid: i32) -> String {
    let Some((group, index)) = m.voice.as_deref().and_then(|voice| {
        let (group, index) = voice.split_once(',')?;
        Some((group.trim().parse::<u16>().ok()?, index.trim().parse::<u16>().ok()?))
    }) else { return String::new() };
    format!("[State {sid}, voice]\ntype = PlaySnd\ntrigger1 = Time = 0\nvalue = {group}, {index}\nchannel = 4\nstopongethit = 1\n\n")
}

pub(super) fn insert_voices(d: &mut CharDef, nc: &NativeChar) {
    if nc.id != "kaito" { return; }
    d.sounds.insert((1000, 0), include_bytes!("../../../../assets/chars/voices/kaito/dragon.wav").to_vec());
    d.sounds.insert((1000, 1), include_bytes!("../../../../assets/chars/voices/kaito/rush.wav").to_vec());
    d.sounds.insert((1000, 2), include_bytes!("../../../../assets/chars/voices/kaito/gale.wav").to_vec());
    d.sounds.insert((1000, 3), include_bytes!("../../../../assets/chars/voices/kaito/awake.wav").to_vec());
}
