use std::path::Path;
use ume_charforge::spritekit::{load_rgba, shared_palette, to_indexed, Rgba};

fn bounds(image: &Rgba) -> Option<(usize, usize, usize, usize)> {
    let mut result = (image.width, image.height, 0, 0);
    for y in 0..image.height {
        for x in 0..image.width {
            if image.pixel(x, y)[3] >= 40 {
                result.0 = result.0.min(x);
                result.1 = result.1.min(y);
                result.2 = result.2.max(x + 1);
                result.3 = result.3.max(y + 1);
            }
        }
    }
    (result.2 > result.0 && result.3 > result.1).then_some(result)
}

fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().collect();
    let source = load_rgba(Path::new(&args[1]))?;
    let sb = bounds(&source).ok_or("candidate has no opaque pixels")?;
    let (target_height, output) = if Path::new(&args[2]).is_file() {
        let target = load_rgba(Path::new(&args[2]))?;
        let tb = bounds(&target).ok_or("target has no opaque pixels")?;
        ((tb.3 - tb.1) as f64, &args[3])
    } else {
        (args[2].parse::<f64>().map_err(|error| error.to_string())?, &args[3])
    };
    let sw = sb.2 - sb.0;
    let sh = sb.3 - sb.1;
    let scale = (target_height / sh as f64).min(396.0 / sw as f64);
    let width = (sw as f64 * scale).round() as usize;
    let height = (sh as f64 * scale).round() as usize;
    let left = (400usize.saturating_sub(width)) / 2;
    let top = 391usize.saturating_sub(height);
    let mut canvas = Rgba { width: 400, height: 400, data: vec![0; 400 * 400 * 4] };
    for y in 0..height {
        for x in 0..width {
            let sx = sb.0 + (x as f64 / scale) as usize;
            let sy = sb.1 + (y as f64 / scale) as usize;
            let pixel = source.pixel(sx.min(source.width - 1), sy.min(source.height - 1));
            if pixel[3] < 40 { continue; }
            let at = ((top + y) * 400 + left + x) * 4;
            canvas.data[at..at + 4].copy_from_slice(&[pixel[0], pixel[1], pixel[2], 255]);
        }
    }
    let (palette, _) = shared_palette(&[Rgba { width: 400, height: 400, data: canvas.data.clone() }], 48);
    let indices = to_indexed(&canvas, &palette);
    for (index, color_index) in indices.iter().enumerate() {
        let pixel = &mut canvas.data[index * 4..index * 4 + 4];
        if *color_index == 0 {
            pixel.copy_from_slice(&[0, 0, 0, 0]);
        } else {
            pixel[..3].copy_from_slice(&palette[*color_index as usize]);
            pixel[3] = 255;
        }
    }
    let file = std::fs::File::create(output).map_err(|error| error.to_string())?;
    let mut encoder = png::Encoder::new(file, 400, 400);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().map_err(|error| error.to_string())?;
    writer.write_image_data(&canvas.data).map_err(|error| error.to_string())?;
    println!("scale {scale:.3}, output {width}x{height} at ({left},{top})");
    Ok(())
}
