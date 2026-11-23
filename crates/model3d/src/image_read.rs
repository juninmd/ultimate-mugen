use crate::accessor::Source;
use crate::error::ModelError;

#[derive(Clone, Debug)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

fn decode(bytes: &[u8]) -> Result<Image, ModelError> {
    let mut decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder
        .read_info()
        .map_err(|_| ModelError::Unsupported("png image"))?;
    let mut buf = vec![
        0;
        reader
            .output_buffer_size()
            .ok_or(ModelError::OutOfRange("png size"))?
    ];
    let info = reader
        .next_frame(&mut buf)
        .map_err(|_| ModelError::Unsupported("png frame"))?;
    let raw = &buf[..info.buffer_size()];
    let rgba = match info.color_type {
        png::ColorType::Rgba => raw.to_vec(),
        png::ColorType::Rgb => raw
            .chunks_exact(3)
            .flat_map(|p| [p[0], p[1], p[2], 255])
            .collect(),
        png::ColorType::GrayscaleAlpha => raw
            .chunks_exact(2)
            .flat_map(|p| [p[0], p[0], p[0], p[1]])
            .collect(),
        png::ColorType::Grayscale => raw.iter().flat_map(|&g| [g, g, g, 255]).collect(),
        png::ColorType::Indexed => return Err(ModelError::Unsupported("indexed png after expand")),
    };
    Ok(Image {
        width: info.width,
        height: info.height,
        rgba,
    })
}

pub fn read_images(src: &Source) -> Vec<Option<Image>> {
    src.json["images"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|image| {
            let view = src.json["bufferViews"].get(image["bufferView"].as_u64()? as usize)?;
            let start = view["byteOffset"].as_u64().unwrap_or(0) as usize;
            let end = start + view["byteLength"].as_u64()? as usize;
            decode(src.bin.get(start..end)?).ok()
        })
        .collect()
}
