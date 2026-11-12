#[path = "generate_missing_sprites/pixelize.rs"]
mod pixelize;

use std::path::Path;
use tiny_skia::Pixmap;

fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().collect();
    let input = Pixmap::load_png(Path::new(&args[1])).map_err(|error| error.to_string())?;
    let output = pixelize::apply(&input, &serde_json::Value::Null)
        .ok_or("sprite has no opaque pixels")?;
    output
        .save_png(Path::new(&args[2]))
        .map_err(|error| error.to_string())
}
