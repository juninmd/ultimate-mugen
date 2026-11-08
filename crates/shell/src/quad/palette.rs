//! Palette atlas handles: an indexed sprite samples one 256-colour row of `PALETTE_ROWS`.
//!
//! Rows are shared by content (`ume_render::PaletteRows`), so a mirror match, a `RemapPal` that lands
//! on an existing palette, or a re-warm after an atlas reset reuse the row and skip the upload.
use super::*;

struct PaletteInner {
    slot: Cell<Option<(u32, u32)>>,
    colors: Box<[[u8; 4]; 256]>,
}

#[derive(Clone)]
pub struct Palette(Rc<PaletteInner>);

impl Palette {
    pub fn from_rgba(pal: &[u8]) -> Palette {
        let mut colors = Box::new([[0u8; 4]; 256]);
        let rgba = pal.len() >= 1024;
        for (i, c) in colors.iter_mut().enumerate().skip(1) {
            let at = |k: usize| pal.get(i * 4 + k).copied().unwrap_or(0);
            *c = [at(0), at(1), at(2), if rgba { at(3) } else { 255 }];
        }
        Palette(Rc::new(PaletteInner { slot: Cell::new(None), colors }))
    }

    pub(super) fn row(&self, f: &mut Frame) -> Option<u32> {
        let gen = index_generation();
        if let Some((g, row)) = self.0.slot.get() {
            if g == gen {
                return Some(row);
            }
        }
        let (row, fresh) = match f.gpu.palette_rows.row(&self.0.colors) {
            Some(slot) => slot,
            None => {
                INDEX_FULL.with(|a| a.set(true));
                return None;
            }
        };
        if fresh {
            f.gpu.renderer.set_palette(&f.gpu.queue, row, &self.0.colors);
            note_upload(256 * 4);
        }
        self.0.slot.set(Some((gen, row)));
        Some(row)
    }

    pub fn is_resident(&self) -> bool {
        self.0.slot.get().is_some_and(|(g, _)| g == index_generation())
    }
}
