/**
 * SFF (Elecbyte sprite file) decoder: versions 1.01 (PCX subfiles) and
 * 2.00 / 2.01 (raw, RLE8, RLE5, LZ5, PNG8/24/32).
 *
 * Decompression algorithms follow Ikemen GO (src/image.go).
 */
import { unzlibSync } from 'fflate';
import type { DecodedSff, DecodedSprite } from './sprite';
import { decodePcxRle, findPcxPalette, readPcxHeader, readRgbPalette, PCX_HEADER_SIZE } from './pcx';

const SIGNATURE = 'ElecbyteSpr\0';

class Reader {
  readonly dv: DataView;
  constructor(readonly buf: Uint8Array) {
    this.dv = new DataView(buf.buffer, buf.byteOffset, buf.byteLength);
  }
  u8(o: number): number { return o < this.buf.length ? this.buf[o] : 0; }
  u16(o: number): number { return o + 2 <= this.buf.length ? this.dv.getUint16(o, true) : 0; }
  i16(o: number): number { return o + 2 <= this.buf.length ? this.dv.getInt16(o, true) : 0; }
  u32(o: number): number { return o + 4 <= this.buf.length ? this.dv.getUint32(o, true) : 0; }
  u32be(o: number): number { return o + 4 <= this.buf.length ? this.dv.getUint32(o, false) : 0; }
  i32(o: number): number { return o + 4 <= this.buf.length ? this.dv.getInt32(o, true) : 0; }
}

function checkSignature(buf: Uint8Array, sig: string): boolean {
  if (buf.length < sig.length) return false;
  for (let i = 0; i < sig.length; i++) if (buf[i] !== sig.charCodeAt(i)) return false;
  return true;
}

export function decodeSff(buf: Uint8Array): DecodedSff {
  if (!checkSignature(buf, SIGNATURE)) throw new Error('SFF: invalid signature (expected ElecbyteSpr)');
  const verHi = buf[15];
  if (verHi === 1) return decodeSffV1(buf);
  if (verHi === 2) return decodeSffV2(buf);
  throw new Error(`SFF: unsupported version ${verHi}`);
}

// ---------------------------------------------------------------------------
// SFF v1
// ---------------------------------------------------------------------------

/**
 * SFF v1 layout:
 *   header: [0]sig(12) [12]ver(4) [16]numGroups u32 [20]numImages u32
 *           [24]firstSubfileOffset u32 [28]subheaderSize u32 [32]paletteType u8
 *   subfile: [0]next u32 [4]length u32 [8]axisX i16 [10]axisY i16 [12]group u16
 *            [14]index u16 [16]linkIndex u16 [18]samePalette u8 [19]comment(13)
 *            followed by PCX data (length bytes; 0 => linked sprite).
 */
function decodeSffV1(buf: Uint8Array): DecodedSff {
  const r = new Reader(buf);
  const count = r.u32(20);
  let ofs = r.u32(24);
  const sprites: DecodedSprite[] = [];
  const palettes: DecodedSff['palettes'] = [];
  const palKeys = new Map<string, number>();
  let prev: DecodedSprite | null = null;

  const internPalette = (pal: Uint8ClampedArray, group: number, index: number): { data: Uint8ClampedArray; idx: number } => {
    let key = '';
    for (let i = 0; i < pal.length; i++) key += String.fromCharCode(pal[i]);
    const found = palKeys.get(key);
    if (found !== undefined) return { data: palettes[found].data, idx: found };
    const idx = palettes.length;
    palettes.push({ group, index, data: pal });
    palKeys.set(key, idx);
    return { data: pal, idx };
  };

  for (let i = 0; i < count; i++) {
    if (ofs <= 0 || ofs + 32 > buf.length) break;
    const next = r.u32(ofs);
    const length = r.u32(ofs + 4);
    const spr: DecodedSprite = {
      group: r.u16(ofs + 12),
      index: r.u16(ofs + 14),
      axisX: r.i16(ofs + 8),
      axisY: r.i16(ofs + 10),
      width: 0,
      height: 0,
      indexed: null,
      rgba: null,
      palette: null,
      paletteIndex: -1,
    };
    const link = r.u16(ofs + 16);
    const samePal = r.u8(ofs + 18) !== 0;
    const dataStart = ofs + 32;

    if (length === 0) {
      const src = link < sprites.length ? sprites[link] : null;
      if (src) copyImage(spr, src);
    } else {
      let blockEnd = next > dataStart ? next : dataStart + length;
      blockEnd = Math.min(blockEnd, buf.length);
      const pcx = buf.subarray(dataStart, blockEnd);
      if (pcx.length >= PCX_HEADER_SIZE) {
        const h = readPcxHeader(pcx);
        spr.width = h.width;
        spr.height = h.height;
        let dataEnd: number;
        if (h.encoding === 1) {
          const d = decodePcxRle(pcx, h.width, h.height, h.bytesPerLine, PCX_HEADER_SIZE);
          spr.indexed = d.pixels;
          dataEnd = d.end;
        } else {
          const px = new Uint8Array(h.width * h.height);
          for (let y = 0; y < h.height; y++) {
            const s = PCX_HEADER_SIZE + y * h.bytesPerLine;
            px.set(pcx.subarray(s, Math.min(s + h.width, pcx.length)), y * h.width);
          }
          spr.indexed = px;
          dataEnd = PCX_HEADER_SIZE + h.height * h.bytesPerLine;
        }
        let pal: Uint8ClampedArray | null = null;
        if (samePal && prev) {
          pal = prev.palette;
          spr.paletteIndex = prev.paletteIndex;
        } else {
          const po = findPcxPalette(pcx, dataEnd);
          if (po >= 0) pal = readRgbPalette(pcx, po);
          else if (prev) {
            pal = prev.palette;
            spr.paletteIndex = prev.paletteIndex;
          }
          if (pal && spr.paletteIndex < 0) {
            const p = internPalette(pal, spr.group, spr.index);
            pal = p.data;
            spr.paletteIndex = p.idx;
          }
        }
        spr.palette = pal;
      }
      prev = spr;
    }
    sprites.push(spr);
    ofs = next;
  }
  return { version: 1, sprites, palettes };
}

function copyImage(dst: DecodedSprite, src: DecodedSprite): void {
  dst.width = src.width;
  dst.height = src.height;
  dst.indexed = src.indexed;
  dst.rgba = src.rgba;
  if (!dst.palette) dst.palette = src.palette;
  if (dst.paletteIndex < 0) dst.paletteIndex = src.paletteIndex;
}

// ---------------------------------------------------------------------------
// SFF v2
// ---------------------------------------------------------------------------

/**
 * SFF v2 layout:
 *   header: [0]sig(12) [12]verlo3 [13]verlo2 [14]verlo1 [15]verhi
 *           [36]spriteListOffset [40]spriteCount [44]paletteListOffset
 *           [48]paletteCount [52]ldataOffset [56]ldataLength
 *           [60]tdataOffset [64]tdataLength
 *   sprite node (28 bytes): group u16, index u16, w u16, h u16, axisX i16,
 *     axisY i16, linkIndex u16, format u8, colorDepth u8, dataOffset u32,
 *     dataLength u32, paletteIndex u16, flags u16 (bit0: 0=ldata, 1=tdata)
 *   palette node (16 bytes): group u16, index u16, numColors u16,
 *     linkIndex u16, dataOffset u32 (in ldata), dataLength u32 (0 => linked)
 */
export const SFF2_FORMAT = {
  RAW: 0,
  INVALID: 1,
  RLE8: 2,
  RLE5: 3,
  LZ5: 4,
  PNG8: 10,
  PNG24: 11,
  PNG32: 12,
} as const;

function decodeSffV2(buf: Uint8Array): DecodedSff {
  const r = new Reader(buf);
  const verlo2 = buf[13];
  const sprOfs = r.u32(36);
  const sprCount = r.u32(40);
  const palOfs = r.u32(44);
  const palCount = r.u32(48);
  const ldata = r.u32(52);
  const tdata = r.u32(60);

  // --- palettes
  const palettes: DecodedSff['palettes'] = [];
  for (let i = 0; i < palCount; i++) {
    const o = palOfs + i * 16;
    if (o + 16 > buf.length) break;
    const group = r.u16(o), index = r.u16(o + 2), link = r.u16(o + 6); // o+4: numColors (unused)
    const dOfs = r.u32(o + 8), dLen = r.u32(o + 12);
    let data: Uint8ClampedArray;
    if (dLen === 0 && link < palettes.length) {
      data = palettes[link].data;
    } else {
      data = new Uint8ClampedArray(256 * 4);
      const start = ldata + dOfs;
      const n = Math.min(256, Math.floor(dLen / 4));
      for (let c = 0; c < n; c++) {
        const s = start + c * 4;
        if (s + 3 >= buf.length) break;
        data[c * 4] = buf[s];
        data[c * 4 + 1] = buf[s + 1];
        data[c * 4 + 2] = buf[s + 2];
        // v2.00: 4th byte is unused; v2.01 stores alpha (Ikemen behavior).
        data[c * 4 + 3] = verlo2 === 0 ? 255 : buf[s + 3];
      }
      data[3] = 0; // color 0 is always transparent
    }
    palettes.push({ group, index, data });
  }

  // --- sprites
  const sprites: DecodedSprite[] = [];
  for (let i = 0; i < sprCount; i++) {
    const o = sprOfs + i * 28;
    if (o + 28 > buf.length) break;
    const width = r.u16(o + 4), height = r.u16(o + 6);
    const link = r.u16(o + 12);
    const format = r.u8(o + 14);
    const depth = r.u8(o + 15);
    const dOfs = r.u32(o + 16), dLen = r.u32(o + 20);
    const palIdx = r.u16(o + 24);
    const flags = r.u16(o + 26);
    const spr: DecodedSprite = {
      group: r.u16(o),
      index: r.u16(o + 2),
      axisX: r.i16(o + 8),
      axisY: r.i16(o + 10),
      width,
      height,
      indexed: null,
      rgba: null,
      palette: null,
      paletteIndex: palIdx,
    };
    if (dLen === 0) {
      if (link < sprites.length) {
        const src = sprites[link];
        spr.width = src.width;
        spr.height = src.height;
        spr.indexed = src.indexed;
        spr.rgba = src.rgba;
      }
    } else {
      const start = ((flags & 1) === 0 ? ldata : tdata) + dOfs;
      const data = buf.subarray(Math.min(start, buf.length), Math.min(start + dLen, buf.length));
      decodeV2Image(spr, format, depth, data, palettes[palIdx]?.data ?? null);
    }
    if (spr.rgba) spr.paletteIndex = -1;
    else spr.palette = palettes[spr.paletteIndex]?.data ?? spr.palette;
    sprites.push(spr);
  }
  return { version: 2, sprites, palettes };
}

function decodeV2Image(
  spr: DecodedSprite,
  format: number,
  depth: number,
  data: Uint8Array,
  sffPalette: Uint8ClampedArray | null,
): void {
  const w = spr.width, h = spr.height, n = w * h;
  // Compressed formats are prefixed by a u32 uncompressed length.
  const body = data.subarray(Math.min(4, data.length));
  switch (format) {
    case SFF2_FORMAT.RAW: {
      if (depth === 24 || depth === 32) {
        const bpp = depth / 8;
        const out = new Uint8ClampedArray(n * 4);
        for (let p = 0; p < n; p++) {
          out[p * 4] = data[p * bpp] ?? 0;
          out[p * 4 + 1] = data[p * bpp + 1] ?? 0;
          out[p * 4 + 2] = data[p * bpp + 2] ?? 0;
          out[p * 4 + 3] = bpp === 4 ? (data[p * bpp + 3] ?? 0) : 255;
        }
        spr.rgba = out;
      } else {
        const out = new Uint8Array(n);
        out.set(data.subarray(0, Math.min(n, data.length)));
        spr.indexed = out;
      }
      return;
    }
    case SFF2_FORMAT.RLE8:
      spr.indexed = decodeRle8(body, n);
      return;
    case SFF2_FORMAT.RLE5:
      spr.indexed = decodeRle5(body, n);
      return;
    case SFF2_FORMAT.LZ5:
      spr.indexed = decodeLz5(body, n);
      return;
    case SFF2_FORMAT.PNG8:
    case SFF2_FORMAT.PNG24:
    case SFF2_FORMAT.PNG32: {
      const png = decodePng(body);
      spr.width = png.width;
      spr.height = png.height;
      if (format === SFF2_FORMAT.PNG8 && png.indexed) {
        spr.indexed = png.indexed;
        if (!sffPalette && png.palette) spr.palette = png.palette;
      } else {
        spr.rgba = png.rgba ?? pngIndexedToRgba(png);
      }
      return;
    }
    default:
      // INVALID (1) or unknown: leave an empty sprite.
      spr.indexed = new Uint8Array(n);
  }
}

// ---------------------------------------------------------------------------
// SFF v2 decompressors (ported from Ikemen GO image.go)
// ---------------------------------------------------------------------------

/**
 * RLE8: a byte with top bits 01 (b & 0xC0 == 0x40) is a run of (b & 0x3F)
 * copies of the next byte; any other byte is a literal pixel.
 */
export function decodeRle8(src: Uint8Array, size: number): Uint8Array {
  const out = new Uint8Array(size);
  let i = 0, j = 0;
  const len = src.length;
  while (j < size && i < len) {
    let d = src[i++];
    let n = 1;
    if ((d & 0xc0) === 0x40) {
      n = d & 0x3f;
      d = i < len ? src[i++] : 0;
    }
    for (; n > 0 && j < size; n--) out[j++] = d;
  }
  return out;
}

/**
 * RLE5 packet: [runLen u8] [dataLen: bit7 = color follows, bits0-6 = count]
 * [color u8 if bit7, else 0]; the color is written runLen+1 times. Then
 * `count` bytes follow, each `rrrccccc`: color c (5 bits) written r+1 times.
 */
export function decodeRle5(src: Uint8Array, size: number): Uint8Array {
  const out = new Uint8Array(size);
  let i = 0, j = 0;
  const len = src.length;
  while (j < size && i < len) {
    let rl = src[i++];
    if (i >= len) break;
    const dlb = src[i++];
    let dl = dlb & 0x7f;
    let c = 0;
    if (dlb & 0x80) {
      if (i >= len) break;
      c = src[i++];
    }
    for (;;) {
      if (j < size) out[j++] = c;
      rl--;
      if (rl < 0) {
        dl--;
        if (dl < 0) break;
        if (i >= len) return out;
        const b = src[i++];
        c = b & 0x1f;
        rl = b >> 5;
      }
    }
  }
  return out;
}

/**
 * LZ5: a control byte supplies 8 flags (LSB first); 0 = RLE packet, 1 = LZ packet.
 *  RLE: `ccc ddddd` → color d repeated c times; if c == 0, the next byte + 8 is the count.
 *  LZ short (b & 0x3F != 0): copy (b & 0x3F) + 1 bytes; offset = next byte + 1,
 *    except every 4th short packet, whose offset is built from the top two bits
 *    of the previous 3 short packets plus its own (recycled bits), + 1.
 *  LZ long (b & 0x3F == 0): offset = ((b & 0xC0) << 2 | next) + 1, length = next2 + 3.
 */
export function decodeLz5(src: Uint8Array, size: number): Uint8Array {
  const out = new Uint8Array(size);
  const len = src.length;
  if (len === 0) return out;
  let i = 0, j = 0;
  let ct = src[i++];
  let cts = 0;
  let rb = 0, rbc = 0;
  const rd = () => (i < len ? src[i++] : 0);
  while (j < size) {
    if (i >= len) break;
    let d = src[i++];
    if (ct & (1 << cts)) {
      let n: number;
      if ((d & 0x3f) === 0) {
        d = ((d << 2) | rd()) + 1;
        n = rd() + 2;
      } else {
        rb |= (d & 0xc0) >> rbc;
        rbc += 2;
        n = d & 0x3f;
        if (rbc < 8) {
          d = rd() + 1;
        } else {
          d = rb + 1;
          rb = 0;
          rbc = 0;
        }
      }
      // copies n + 1 bytes (may overlap)
      for (; n >= 0 && j < size; n--) {
        out[j] = j >= d ? out[j - d] : 0;
        j++;
      }
    } else {
      let n: number;
      if ((d & 0xe0) === 0) {
        n = rd() + 8;
      } else {
        n = d >> 5;
        d &= 0x1f;
      }
      for (; n > 0 && j < size; n--) out[j++] = d;
    }
    cts++;
    if (cts >= 8) {
      if (i >= len) break;
      ct = src[i++];
      cts = 0;
    }
  }
  return out;
}

// ---------------------------------------------------------------------------
// Minimal PNG decoder
// ---------------------------------------------------------------------------

export interface DecodedPng {
  width: number;
  height: number;
  colorType: number;
  bitDepth: number;
  /** For color type 3: one palette index per pixel. */
  indexed: Uint8Array | null;
  /** RGBA 256*4 from PLTE (+tRNS), for color type 3. */
  palette: Uint8ClampedArray | null;
  /** For non-indexed color types: straight (non-premultiplied) RGBA. */
  rgba: Uint8ClampedArray | null;
}

const PNG_SIG = [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];

/**
 * Decode a non-interlaced PNG. Supports color types 0 (gray), 2 (RGB),
 * 3 (indexed, bit depth 1/2/4/8), 4 (gray+alpha) and 6 (RGBA); bit depth 8
 * (16-bit samples are reduced to their high byte). All five filter types.
 */
export function decodePng(buf: Uint8Array): DecodedPng {
  for (let k = 0; k < 8; k++) if (buf[k] !== PNG_SIG[k]) throw new Error('PNG: bad signature');
  const r = new Reader(buf);
  let p = 8;
  let width = 0, height = 0, bitDepth = 0, colorType = 0, interlace = 0;
  let plte: Uint8Array | null = null;
  let trns: Uint8Array | null = null;
  const idat: Uint8Array[] = [];
  while (p + 8 <= buf.length) {
    const len = r.u32be(p);
    const type = String.fromCharCode(buf[p + 4], buf[p + 5], buf[p + 6], buf[p + 7]);
    const data = buf.subarray(p + 8, Math.min(p + 8 + len, buf.length));
    if (type === 'IHDR') {
      width = r.u32be(p + 8);
      height = r.u32be(p + 12);
      bitDepth = buf[p + 16];
      colorType = buf[p + 17];
      interlace = buf[p + 20];
    } else if (type === 'PLTE') plte = data;
    else if (type === 'tRNS') trns = data;
    else if (type === 'IDAT') idat.push(data);
    else if (type === 'IEND') break;
    p += 12 + len;
  }
  if (!width || !height) throw new Error('PNG: missing IHDR');
  if (interlace) throw new Error('PNG: interlaced images are not supported');
  const channels = ({ 0: 1, 2: 3, 3: 1, 4: 2, 6: 4 } as Record<number, number>)[colorType];
  if (!channels) throw new Error(`PNG: unsupported color type ${colorType}`);
  if (colorType === 3 ? ![1, 2, 4, 8].includes(bitDepth) : ![8, 16].includes(bitDepth)) {
    throw new Error(`PNG: unsupported bit depth ${bitDepth} for color type ${colorType}`);
  }

  let total = 0;
  for (const c of idat) total += c.length;
  const z = new Uint8Array(total);
  let zo = 0;
  for (const c of idat) { z.set(c, zo); zo += c.length; }
  const raw = unzlibSync(z);

  const bitsPP = channels * bitDepth;
  const bpp = Math.max(1, bitsPP >> 3); // filter byte distance
  const stride = Math.ceil((width * bitsPP) / 8);
  const img = new Uint8Array(stride * height);
  let prevRow = new Uint8Array(stride);
  for (let y = 0; y < height; y++) {
    const ro = y * (stride + 1);
    if (ro + 1 + stride > raw.length) throw new Error('PNG: truncated image data');
    const ft = raw[ro];
    const line = img.subarray(y * stride, (y + 1) * stride);
    line.set(raw.subarray(ro + 1, ro + 1 + stride));
    unfilter(ft, line, prevRow, bpp);
    prevRow = line;
  }

  const n = width * height;
  if (colorType === 3) {
    const indexed = new Uint8Array(n);
    if (bitDepth === 8) {
      indexed.set(img.subarray(0, n));
    } else {
      const perByte = 8 / bitDepth, mask = (1 << bitDepth) - 1;
      for (let y = 0; y < height; y++) {
        for (let x = 0; x < width; x++) {
          const b = img[y * stride + Math.floor(x / perByte)];
          const shift = 8 - bitDepth * ((x % perByte) + 1);
          indexed[y * width + x] = (b >> shift) & mask;
        }
      }
    }
    let palette: Uint8ClampedArray | null = null;
    if (plte) {
      palette = new Uint8ClampedArray(256 * 4);
      const cnt = Math.min(256, Math.floor(plte.length / 3));
      for (let c = 0; c < cnt; c++) {
        palette[c * 4] = plte[c * 3];
        palette[c * 4 + 1] = plte[c * 3 + 1];
        palette[c * 4 + 2] = plte[c * 3 + 2];
        palette[c * 4 + 3] = trns && c < trns.length ? trns[c] : 255;
      }
    }
    return { width, height, colorType, bitDepth, indexed, palette, rgba: null };
  }

  const rgba = new Uint8ClampedArray(n * 4);
  if (bitDepth === 8 && colorType === 6) {
    rgba.set(img.subarray(0, n * 4));
    return { width, height, colorType, bitDepth, indexed: null, palette: null, rgba };
  }
  const bs = bitDepth === 16 ? 2 : 1; // bytes per sample; take high byte
  // tRNS for gray/RGB: single transparent color (compared on 8-bit high bytes)
  const tr = trns && (colorType === 0 || colorType === 2) ? trns : null;
  for (let y = 0; y < height; y++) {
    const row = y * stride;
    for (let x = 0; x < width; x++) {
      const s = row + x * channels * bs;
      const d = (y * width + x) * 4;
      let R: number, G: number, B: number, A = 255;
      switch (colorType) {
        case 0:
          R = G = B = img[s];
          if (tr && tr.length >= 2 && (bs === 1 ? img[s] === tr[1] : img[s] === tr[0] && img[s + 1] === tr[1])) A = 0;
          break;
        case 4:
          R = G = B = img[s];
          A = img[s + bs];
          break;
        case 2:
          R = img[s]; G = img[s + bs]; B = img[s + 2 * bs];
          // tRNS holds 16-bit big-endian samples: compare the low byte at 8-bit depth, high byte at 16-bit
          if (tr && tr.length >= 6) {
            const h = bs === 2 ? 0 : 1;
            if (R === tr[h] && G === tr[2 + h] && B === tr[4 + h]) A = 0;
          }
          break;
        default: // 6
          R = img[s]; G = img[s + bs]; B = img[s + 2 * bs]; A = img[s + 3 * bs];
      }
      rgba[d] = R; rgba[d + 1] = G; rgba[d + 2] = B; rgba[d + 3] = A;
    }
  }
  return { width, height, colorType, bitDepth, indexed: null, palette: null, rgba };
}

function unfilter(ft: number, line: Uint8Array, prev: Uint8Array, bpp: number): void {
  const len = line.length;
  switch (ft) {
    case 0:
      return;
    case 1: // Sub
      for (let x = bpp; x < len; x++) line[x] = (line[x] + line[x - bpp]) & 0xff;
      return;
    case 2: // Up
      for (let x = 0; x < len; x++) line[x] = (line[x] + prev[x]) & 0xff;
      return;
    case 3: // Average
      for (let x = 0; x < len; x++) {
        const a = x >= bpp ? line[x - bpp] : 0;
        line[x] = (line[x] + ((a + prev[x]) >> 1)) & 0xff;
      }
      return;
    case 4: // Paeth
      for (let x = 0; x < len; x++) {
        const a = x >= bpp ? line[x - bpp] : 0;
        const b = prev[x];
        const c = x >= bpp ? prev[x - bpp] : 0;
        const pp = a + b - c;
        const pa = Math.abs(pp - a), pb = Math.abs(pp - b), pc = Math.abs(pp - c);
        const pred = pa <= pb && pa <= pc ? a : pb <= pc ? b : c;
        line[x] = (line[x] + pred) & 0xff;
      }
      return;
    default:
      throw new Error(`PNG: bad filter type ${ft}`);
  }
}

function pngIndexedToRgba(png: DecodedPng): Uint8ClampedArray {
  const n = png.width * png.height;
  const out = new Uint8ClampedArray(n * 4);
  if (!png.indexed || !png.palette) return out;
  for (let k = 0; k < n; k++) {
    const c = png.indexed[k] * 4;
    out[k * 4] = png.palette[c];
    out[k * 4 + 1] = png.palette[c + 1];
    out[k * 4 + 2] = png.palette[c + 2];
    out[k * 4 + 3] = png.palette[c + 3];
  }
  return out;
}
