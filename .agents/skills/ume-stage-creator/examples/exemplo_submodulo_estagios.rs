use std::sync::OnceLock;
use ume_engine::chardef::ImageData;
use crate::stages::StageEntry;

pub const EXAMPLE_IDS: [&str; 2] = [
    "example_arena_a",
    "example_arena_b",
];

const THUMB_BYTES: [&[u8]; 2] = [
    include_bytes!("../../assets/stages/thumbs/colosseum.png"),
    include_bytes!("../../assets/stages/thumbs/texas.png"),
];

const STAGE_BYTES: [&[u8]; 2] = [
    include_bytes!("../../assets/stages/colosseum.png"),
    include_bytes!("../../assets/stages/texas.png"),
];

static PREVIEWS: [OnceLock<ImageData>; 2] = [OnceLock::new(), OnceLock::new()];
static STAGES: [OnceLock<ImageData>; 2] = [OnceLock::new(), OnceLock::new()];

pub fn illustration_preview(id: &str) -> Option<ImageData> {
    let index = EXAMPLE_IDS.iter().position(|&x| x == id)?;
    Some(PREVIEWS[index].get_or_init(|| decode_png(THUMB_BYTES[index])).clone())
}

pub fn illustration(id: &str) -> Option<ImageData> {
    let index = EXAMPLE_IDS.iter().position(|&x| x == id)?;
    Some(STAGES[index].get_or_init(|| decode_png(STAGE_BYTES[index])).clone())
}

pub fn builtin() -> Vec<StageEntry> {
    vec![
        crate::stages::entry(
            "example_arena_a",
            "Example Arena A",
            "Rome, Italy",
            "Example author",
            "Day",
            [[40, 30, 20], [80, 70, 60], [30, 25, 20]],
        ),
        crate::stages::entry(
            "example_arena_b",
            "Example Arena B",
            "Texas, USA",
            "Example author",
            "Sunset",
            [[50, 40, 30], [90, 80, 70], [40, 35, 30]],
        ),
    ]
}

fn decode_png(bytes: &[u8]) -> ImageData {
    let decoder = png::Decoder::new(bytes);
    let mut reader = decoder.read_info().expect("decode png");
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).expect("read frame");
    buf.truncate(info.buffer_size());
    let rgba = match info.color_type {
        png::ColorType::Rgb => {
            let mut rgba = Vec::with_capacity((info.width * info.height * 4) as usize);
            for chunk in buf.chunks_exact(3) {
                rgba.extend_from_slice(&[chunk[0], chunk[1], chunk[2], 255]);
            }
            rgba
        }
        png::ColorType::Rgba => buf,
        _ => panic!("unsupported color type"),
    };
    ImageData { w: info.width, h: info.height, rgba }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn previews_and_stages_decode() {
        for &id in EXAMPLE_IDS.iter() {
            let preview = illustration_preview(id).expect("preview decodes");
            assert_eq!(preview.w, 240);
            assert_eq!(preview.h, 135);
            let stage = illustration(id).expect("stage decodes");
            assert_eq!(stage.w, 1672);
            assert_eq!(stage.h, 941);
        }
    }
}
