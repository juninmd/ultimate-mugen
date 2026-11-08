// ---------------------------------------------------------------------------
// SFF v2 decompressors (ported from Ikemen GO image.go)
// ---------------------------------------------------------------------------

/// RLE8: a byte with top bits 01 (`b & 0xC0 == 0x40`) is a run of `b & 0x3F`
/// copies of the next byte; any other byte is a literal pixel.
pub fn decode_rle8(src: &[u8], size: usize) -> Vec<u8> {
    let mut out = vec![0u8; size];
    let (mut i, mut j) = (0usize, 0usize);
    let len = src.len();
    while j < size && i < len {
        let d = src[i];
        i += 1;
        if d & 0xc0 == 0x40 {
            let n = (d & 0x3f) as usize;
            let val = if i < len {
                let v = src[i];
                i += 1;
                v
            } else {
                0
            };
            let e = (j + n).min(size);
            if val != 0 {
                out[j..e].fill(val);
            }
            j = e;
        } else {
            if d != 0 {
                out[j] = d;
            }
            j += 1;
        }
    }
    out
}

/// RLE5 packet: `[runLen u8] [dataLen: bit7 = color follows, bits0-6 = count]
/// [color u8 if bit7, else 0]`; the color is written runLen+1 times. Then
/// `count` bytes follow, each `rrrccccc`: color c (5 bits) written r+1 times.
pub fn decode_rle5(src: &[u8], size: usize) -> Vec<u8> {
    let mut out = vec![0u8; size];
    let (mut i, mut j) = (0usize, 0usize);
    let len = src.len();
    while j < size && i < len {
        let mut rl = src[i] as usize;
        i += 1;
        if i >= len {
            break;
        }
        let dlb = src[i];
        i += 1;
        let mut dl = (dlb & 0x7f) as i32;
        let mut c = 0u8;
        if dlb & 0x80 != 0 {
            if i >= len {
                break;
            }
            c = src[i];
            i += 1;
        }
        loop {
            let count = rl + 1;
            let e = (j + count).min(size);
            if c != 0 {
                out[j..e].fill(c);
            }
            j = e;
            dl -= 1;
            if dl < 0 {
                break;
            }
            if i >= len {
                return out;
            }
            let b = src[i];
            i += 1;
            c = b & 0x1f;
            rl = (b >> 5) as usize;
        }
    }
    out
}

/// LZ5: a control byte supplies 8 flags (LSB first); 0 = RLE packet, 1 = LZ packet.
///  RLE: `ccc ddddd` -> color d repeated c times; if c == 0, the next byte + 8 is the count.
///  LZ short (`b & 0x3F != 0`): copy `(b & 0x3F) + 1` bytes; offset = next byte + 1,
///    except every 4th short packet, whose offset is built from the top two bits
///    of the previous 3 short packets plus its own (recycled bits), + 1.
///  LZ long (`b & 0x3F == 0`): offset = `((b & 0xC0) << 2 | next) + 1`, length = next2 + 3.
pub fn decode_lz5(src: &[u8], size: usize) -> Vec<u8> {
    let mut out = vec![0u8; size];
    let len = src.len();
    if len == 0 {
        return out;
    }
    let mut i = 0usize;
    let mut j = 0usize;
    let mut ct = src[i];
    i += 1;
    let mut cts = 0u32;
    let (mut rb, mut rbc) = (0usize, 0u32);
    macro_rules! rd {
        () => {
            if i < len {
                i += 1;
                src[i - 1] as usize
            } else {
                0
            }
        };
    }
    while j < size {
        if i >= len {
            break;
        }
        let d = src[i] as usize;
        i += 1;
        if ct & (1 << cts) != 0 {
            let (off, n);
            if d & 0x3f == 0 {
                off = ((d << 2) | rd!()) + 1;
                n = rd!() + 2;
            } else {
                rb |= (d & 0xc0) >> rbc;
                rbc += 2;
                n = d & 0x3f;
                if rbc < 8 {
                    off = rd!() + 1;
                } else {
                    off = rb + 1;
                    rb = 0;
                    rbc = 0;
                }
            }
            // copies n + 1 bytes (may overlap)
            let e = (j + n + 1).min(size);
            if off > j {
                // reads before the start of the buffer yield 0
                while j < e {
                    out[j] = if j >= off { out[j - off] } else { 0 };
                    j += 1;
                }
            } else if off == 1 {
                let val = out[j - 1];
                out[j..e].fill(val);
                j = e;
            } else if off >= e - j {
                out.copy_within(j - off..e - off, j);
                j = e;
            } else {
                while j < e {
                    out[j] = out[j - off];
                    j += 1;
                }
            }
        } else {
            let (n, c);
            if d & 0xe0 == 0 {
                n = rd!() + 8;
                c = d as u8;
            } else {
                n = d >> 5;
                c = (d & 0x1f) as u8;
            }
            let e = (j + n).min(size);
            out[j..e].fill(c);
            j = e;
        }
        cts += 1;
        if cts >= 8 {
            if i >= len {
                break;
            }
            ct = src[i];
            i += 1;
            cts = 0;
        }
    }
    out
}