use std::path::Path;
use std::time::Instant;
use ume_formats::sff::{decode_sff, DecodedSff};
use ume_formats::umsff::encode_umsff;

pub struct CookReport {
    pub sprites: usize,
    pub source_bytes: usize,
    pub cooked_bytes: usize,
    pub source_ms: f64,
    pub cooked_ms: f64,
}

fn same(left: &DecodedSff, right: &DecodedSff) -> Result<(), String> {
    if left.version != right.version
        || left.sprites.len() != right.sprites.len()
        || left.palettes.len() != right.palettes.len()
    {
        return Err("sprite or palette count differs".into());
    }
    for (a, b) in left.palettes.iter().zip(&right.palettes) {
        if (a.group, a.index) != (b.group, b.index) || a.data != b.data {
            return Err(format!("palette {},{} differs", a.group, a.index));
        }
    }
    for (a, b) in left.sprites.iter().zip(&right.sprites) {
        let key = (
            a.group,
            a.index,
            a.axis_x,
            a.axis_y,
            a.width,
            a.height,
            a.palette_index,
        );
        let equal =
            key == (
                b.group,
                b.index,
                b.axis_x,
                b.axis_y,
                b.width,
                b.height,
                b.palette_index,
            ) && a.indexed == b.indexed
                && a.rgba == b.rgba
                && a.palette == b.palette;
        if !equal {
            return Err(format!("sprite {},{} differs", a.group, a.index));
        }
    }
    Ok(())
}

fn timed<T>(work: impl FnOnce() -> T) -> (T, f64) {
    let start = Instant::now();
    let value = work();
    (value, start.elapsed().as_secs_f64() * 1000.0)
}

pub fn cook(source: &Path, target: &Path) -> Result<CookReport, String> {
    let bytes = std::fs::read(source).map_err(|e| format!("{}: {e}", source.display()))?;
    let (decoded, source_ms) = timed(|| decode_sff(&bytes));
    let decoded = decoded?;
    let cooked = encode_umsff(&decoded)?;
    let (back, cooked_ms) = timed(|| decode_sff(&cooked));
    same(&decoded, &back?).map_err(|e| format!("{}: {e}", source.display()))?;
    std::fs::write(target, &cooked).map_err(|e| format!("{}: {e}", target.display()))?;
    Ok(CookReport {
        sprites: decoded.sprites.len(),
        source_bytes: bytes.len(),
        cooked_bytes: cooked.len(),
        source_ms,
        cooked_ms,
    })
}
