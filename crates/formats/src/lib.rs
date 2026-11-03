//! Decoders / parsers for M.U.G.E.N file formats.
//!
//! Ported from the (verified) TypeScript reference implementation in
//! `src/mugen/*.ts`. Pure Rust, no platform dependencies (builds for
//! `wasm32-unknown-unknown`).
//!
//! * [`ini`]   – generic MUGEN INI parser (DEF/CNS/CMD/AIR/stage files)
//! * [`air`]   – animation (.air) parser
//! * [`sff`]   – SFF v1 (PCX) and v2 (raw/RLE8/RLE5/LZ5/PNG) sprite files
//! * [`pcx`]   – 8-bit PCX decoder (SFF v1 subfiles)
//! * [`png`]   – minimal PNG decoder (SFF v2 PNG sprites)
//! * [`snd`]   – SND sound containers
//! * [`act`]   – ACT palettes
//! * [`vfs`]   – in-memory virtual file system (zip / directory) + file resolution
//! * [`stage`] – stage .def loader (data only, no rendering)

pub mod act;
pub mod air;
pub mod fnt;
pub mod ini;
pub mod pcx;
pub mod png;
pub mod sff;
pub mod screenpack;
pub mod snd;
pub mod stage;
pub mod vfs;

/// Little-endian bounds-checked reader: out-of-range reads return 0.
#[derive(Clone, Copy)]
pub(crate) struct Reader<'a>(pub &'a [u8]);

impl<'a> Reader<'a> {
    #[inline]
    fn bytes<const N: usize>(&self, o: usize) -> Option<[u8; N]> {
        let end = o.checked_add(N)?;
        self.0.get(o..end).map(|s| s.try_into().unwrap())
    }
    #[inline]
    pub fn u8(&self, o: usize) -> u8 {
        self.0.get(o).copied().unwrap_or(0)
    }
    #[inline]
    pub fn u16(&self, o: usize) -> u16 {
        self.bytes::<2>(o).map(u16::from_le_bytes).unwrap_or(0)
    }
    #[inline]
    pub fn i16(&self, o: usize) -> i16 {
        self.bytes::<2>(o).map(i16::from_le_bytes).unwrap_or(0)
    }
    #[inline]
    pub fn u32(&self, o: usize) -> u32 {
        self.bytes::<4>(o).map(u32::from_le_bytes).unwrap_or(0)
    }
    #[inline]
    pub fn i32(&self, o: usize) -> i32 {
        self.bytes::<4>(o).map(i32::from_le_bytes).unwrap_or(0)
    }
    #[inline]
    pub fn u64(&self, o: usize) -> u64 {
        self.bytes::<8>(o).map(u64::from_le_bytes).unwrap_or(0)
    }
    #[inline]
    pub fn u32be(&self, o: usize) -> u32 {
        self.bytes::<4>(o).map(u32::from_be_bytes).unwrap_or(0)
    }
}
