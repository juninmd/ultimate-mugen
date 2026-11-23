use super::decode_cache::{self, Decoded};
use super::effects::FrameArt;
use super::frame_raster::sprite_of;
use super::pack::read_sprite;
use super::pixel::{native_frame_sprite_key, pose_stem, sprite_roots, BASE_STEM};
use super::pixel_crop::crop_art;
use super::recolor::Recolor;
use super::types::{NativeChar, NativeFrame};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use ume_engine::chardef::{ImageData, Sprite};

pub(super) fn candidates<'a>(roots: &'a [PathBuf], pose: &'a str, stem: &'a str) -> impl Iterator<Item = (&'a PathBuf, &'a str)> {
    [pose, stem, BASE_STEM, "idle"].into_iter().flat_map(move |name| roots.iter().map(move |root| (root, name)))
}

pub(super) fn open(root: &Path, name: &str) -> Option<(Arc<Decoded>, Arc<Value>)> {
    let bytes = read_sprite(root, &format!("{name}.png"))?;
    let decoded = decode_cache::png(root, name, &bytes)?;
    let meta = read_sprite(root, "meta.json").map_or_else(|| Arc::new(Value::Null), |bytes| decode_cache::meta(root, &bytes));
    Some((decoded, meta))
}

pub(super) fn hair_region(frame: &Value) -> Option<[u32; 4]> {
    frame.get("awakeningHair").and_then(|value| serde_json::from_value(value.clone()).ok())
}

pub(super) struct Source {
    decoded: Arc<Decoded>,
    hair: Option<[u32; 4]>,
}

pub(super) fn locate(nc: &NativeChar, frame: &NativeFrame) -> Option<Arc<Source>> {
    let key = native_frame_sprite_key(frame);
    let lower = key.to_ascii_lowercase();
    let roots = sprite_roots(nc.sprite_dir());
    let found = candidates(&roots, &key, pose_stem(&lower)).find_map(|(root, name)| {
        let (decoded, meta) = open(root, name)?;
        let hair = hair_region(meta.get("frames").and_then(|frames| frames.get(name)).unwrap_or(&Value::Null));
        Some(Arc::new(Source { decoded, hair }))
    });
    found
}

/// A recolored copy of `base` whose pixels are produced on first use: placement and size are the base's.
pub(super) fn sprite(base: &FrameArt, source: Option<&Arc<Source>>, recolor: Option<&Arc<Recolor>>) -> Sprite {
    let mut sprite = sprite_of(base);
    let (Some(plain), Some(source), Some(recolor)) = (base.image.clone(), source.cloned(), recolor.cloned()) else { return sprite };
    sprite.image = Some(ImageData::lazy(plain.w, plain.h, move || {
        let Decoded { width, height, rgba, bounds } = &*source.decoded;
        let cropped = crop_art(*width, *height, rgba.clone(), *bounds, Some(&recolor), source.hair);
        let pixels = cropped.image.rgba();
        let same_size = (cropped.image.w, cropped.image.h) == (plain.w, plain.h);
        if !same_size || **pixels == **plain.rgba() { plain.rgba().clone() } else { pixels.clone() }
    }));
    sprite
}
