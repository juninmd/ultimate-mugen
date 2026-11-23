use std::cell::RefCell;
use std::collections::HashMap;
use ume_engine::chardef::{ImageData, Sprite};

thread_local! {
    static PORTRAITS: RefCell<HashMap<String, Option<Sprite>>> = RefCell::new(HashMap::new());
}

fn decode(bytes: &[u8]) -> Option<Sprite> {
    let png = ume_formats::png::decode_png(bytes).ok()?;
    let rgba = png.rgba?;
    Some(Sprite {
        image: Some(ImageData::new(png.width, png.height, rgba)),
        width: png.width as f32,
        height: png.height as f32,
        axis_x: 0.0,
        axis_y: 0.0,
        scale: 1.0,
    })
}

pub(super) fn portrait(character: &str) -> Option<Sprite> {
    let spec = super::specs::spec(character, false)?;
    PORTRAITS.with(|cache| {
        if let Some(found) = cache.borrow().get(character) {
            return found.clone();
        }
        cache
            .borrow_mut()
            .entry(character.to_string())
            .or_insert_with(|| decode(spec.portrait))
            .clone()
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn ichigo_portrait_decodes() {
        let p = super::portrait("ichigo3d").expect("portrait");
        assert!(p.width > 100.0);
    }
}
