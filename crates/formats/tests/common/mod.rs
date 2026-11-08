//! Tiny encoders used to build synthetic files (port of the helpers in
//! tests/decoders.test.ts).
#![allow(dead_code)]

#[derive(Default)]
pub struct Writer {
    pub bytes: Vec<u8>,
}

impl Writer {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn len(&self) -> usize {
        self.bytes.len()
    }
    pub fn u8(&mut self, v: u32) -> &mut Self {
        self.bytes.push(v as u8);
        self
    }
    pub fn u16(&mut self, v: u32) -> &mut Self {
        self.bytes.extend_from_slice(&(v as u16).to_le_bytes());
        self
    }
    pub fn u32(&mut self, v: u32) -> &mut Self {
        self.bytes.extend_from_slice(&v.to_le_bytes());
        self
    }
    pub fn str(&mut self, s: &str) -> &mut Self {
        self.bytes.extend_from_slice(s.as_bytes());
        self
    }
    pub fn raw(&mut self, a: &[u8]) -> &mut Self {
        self.bytes.extend_from_slice(a);
        self
    }
    pub fn pad(&mut self, n: usize) -> &mut Self {
        if self.bytes.len() < n {
            self.bytes.resize(n, 0);
        }
        self
    }
    pub fn out(&self) -> Vec<u8> {
        self.bytes.clone()
    }
}

/// Deterministic pseudo random generator (same as the TS tests).
pub fn rng(seed: u32) -> impl FnMut() -> u8 {
    let mut s = seed;
    move || {
        s = s.wrapping_mul(1664525).wrapping_add(1013904223);
        (s >> 24) as u8
    }
}

/// PCX RLE encoding of one 8-bit image (runs never cross scanlines).
pub fn encode_pcx(w: usize, h: usize, px: &[u8], palette: Option<&[u8]>) -> Vec<u8> {
    let bpl = w + (w & 1);
    let mut wr = Writer::new();
    wr.u8(10).u8(5).u8(1).u8(8).u16(0).u16(0).u16(w as u32 - 1).u16(h as u32 - 1).u16(72).u16(72);
    wr.pad(65).u8(1).u16(bpl as u32).u16(1).pad(128);
    for y in 0..h {
        let mut line = vec![0u8; bpl];
        line[..w].copy_from_slice(&px[y * w..y * w + w]);
        let mut x = 0;
        while x < bpl {
            let v = line[x];
            let mut n = 1;
            while x + n < bpl && line[x + n] == v && n < 63 {
                n += 1;
            }
            if n > 1 || v >= 0xc0 {
                wr.u8(0xc0 | n as u32).u8(v as u32);
            } else {
                wr.u8(v as u32);
            }
            x += n;
        }
    }
    if let Some(p) = palette {
        wr.u8(0x0c).raw(p);
    }
    wr.out()
}

pub fn make_rgb_palette(seed: u32) -> Vec<u8> {
    let mut r = rng(seed);
    (0..768).map(|_| r()).collect()
}

pub fn encode_rle8(px: &[u8]) -> Vec<u8> {
    let mut wr = Writer::new();
    let mut i = 0;
    while i < px.len() {
        let v = px[i];
        let mut n = 1;
        while i + n < px.len() && px[i + n] == v && n < 63 {
            n += 1;
        }
        if n > 1 || (v & 0xc0) == 0x40 {
            wr.u8(0x40 | n as u32).u8(v as u32);
        } else {
            wr.u8(v as u32);
        }
        i += n;
    }
    wr.out()
}

fn paeth(a: i32, b: i32, c: i32) -> i32 {
    let p = a + b - c;
    let (pa, pb, pc) = ((p - a).abs(), (p - b).abs(), (p - c).abs());
    if pa <= pb && pa <= pc {
        a
    } else if pb <= pc {
        b
    } else {
        c
    }
}

pub fn crc32(b: &[u8]) -> u32 {
    let mut c = !0u32;
    for &x in b {
        c ^= x as u32;
        for _ in 0..8 {
            c = (c >> 1) ^ (0xedb8_8320 & (c & 1).wrapping_neg());
        }
    }
    !c
}

pub struct PngOpts<'a> {
    pub w: usize,
    pub h: usize,
    pub color_type: u8,
    pub bit_depth: u8,
    pub data: &'a [u8],
    pub plte: Option<&'a [u8]>,
    pub trns: Option<&'a [u8]>,
}

/// PNG encoder: row y uses filter type y % 5 (exercises all filters).
pub fn encode_png(o: PngOpts) -> Vec<u8> {
    let ch = match o.color_type {
        0 => 1,
        2 => 3,
        3 => 1,
        4 => 2,
        _ => 4,
    };
    let bd = o.bit_depth as usize;
    let stride = (o.w * ch * bd).div_ceil(8);
    let bpp = ((ch * bd) >> 3).max(1);
    let data = o.data;
    let mut raw = vec![0u8; (stride + 1) * o.h];
    for y in 0..o.h {
        let ft = y % 5;
        raw[y * (stride + 1)] = ft as u8;
        for x in 0..stride {
            let cur = data[y * stride + x] as i32;
            let a = if x >= bpp { data[y * stride + x - bpp] as i32 } else { 0 };
            let b = if y > 0 { data[(y - 1) * stride + x] as i32 } else { 0 };
            let c = if x >= bpp && y > 0 { data[(y - 1) * stride + x - bpp] as i32 } else { 0 };
            let pred = [0, a, b, (a + b) >> 1, paeth(a, b, c)][ft];
            raw[y * (stride + 1) + 1 + x] = (cur - pred) as u8;
        }
    }
    fn chunk(out: &mut Vec<u8>, t: &[u8; 4], d: &[u8]) {
        out.extend_from_slice(&(d.len() as u32).to_be_bytes());
        let s = out.len();
        out.extend_from_slice(t);
        out.extend_from_slice(d);
        let crc = crc32(&out[s..]);
        out.extend_from_slice(&crc.to_be_bytes());
    }
    let mut ih = vec![0u8; 13];
    ih[0..4].copy_from_slice(&(o.w as u32).to_be_bytes());
    ih[4..8].copy_from_slice(&(o.h as u32).to_be_bytes());
    ih[8] = o.bit_depth;
    ih[9] = o.color_type;
    let z = miniz_oxide::deflate::compress_to_vec_zlib(&raw, 6);
    let mut out = vec![137, 80, 78, 71, 13, 10, 26, 10];
    chunk(&mut out, b"IHDR", &ih);
    if let Some(p) = o.plte {
        chunk(&mut out, b"PLTE", p);
    }
    if let Some(t) = o.trns {
        chunk(&mut out, b"tRNS", t);
    }
    // split IDAT in two chunks to test concatenation
    let half = z.len() / 2;
    chunk(&mut out, b"IDAT", &z[..half]);
    chunk(&mut out, b"IDAT", &z[half..]);
    chunk(&mut out, b"IEND", &[]);
    out
}

/// Minimal zip writer: `(name, data, deflate?)`.
pub fn encode_zip(files: &[(&str, &[u8], bool)]) -> Vec<u8> {
    let mut out = Writer::new();
    let mut cd = Writer::new();
    for (name, data, deflate) in files {
        let comp = if *deflate { miniz_oxide::deflate::compress_to_vec(data, 6) } else { data.to_vec() };
        let crc = crc32(data);
        let ofs = out.len() as u32;
        let method = if *deflate { 8 } else { 0 };
        out.u32(0x0403_4b50).u16(20).u16(0).u16(method).u16(0).u16(0).u32(crc);
        out.u32(comp.len() as u32).u32(data.len() as u32).u16(name.len() as u32).u16(0).str(name).raw(&comp);
        cd.u32(0x0201_4b50).u16(20).u16(20).u16(0).u16(method).u16(0).u16(0).u32(crc);
        cd.u32(comp.len() as u32).u32(data.len() as u32).u16(name.len() as u32).u16(0).u16(0);
        cd.u16(0).u16(0).u32(0).u32(ofs).str(name);
    }
    let cd_ofs = out.len() as u32;
    let cd_len = cd.len() as u32;
    out.raw(&cd.bytes);
    out.u32(0x0605_4b50).u16(0).u16(0).u16(files.len() as u32).u16(files.len() as u32);
    out.u32(cd_len).u32(cd_ofs).u16(7).str("comment");
    out.out()
}
