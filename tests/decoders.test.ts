import { describe, it, expect } from 'vitest';
// @types/node is not installed in this project; these resolve at runtime under vitest.
// @ts-ignore
import fs from 'node:fs';
// @ts-ignore
import path from 'node:path';
import { zlibSync } from 'fflate';
import { decodePcx, decodePcxRle } from '../src/mugen/pcx';
import { decodeSff, decodeRle8, decodeRle5, decodeLz5, decodePng } from '../src/mugen/sff';
import { decodeSnd } from '../src/mugen/snd';
import { decodeAct, encodeAct } from '../src/mugen/act';
import { spriteToRGBA } from '../src/mugen/sprite';

// ---------------------------------------------------------------------------
// helpers / tiny encoders used to build synthetic files
// ---------------------------------------------------------------------------

class Writer {
  bytes: number[] = [];
  get length() { return this.bytes.length; }
  u8(v: number) { this.bytes.push(v & 0xff); return this; }
  u16(v: number) { return this.u8(v).u8(v >> 8); }
  u32(v: number) { return this.u16(v & 0xffff).u16(v >>> 16); }
  str(s: string) { for (const c of s) this.u8(c.charCodeAt(0)); return this; }
  raw(a: ArrayLike<number>) { for (let i = 0; i < a.length; i++) this.bytes.push(a[i]); return this; }
  pad(n: number) { while (this.bytes.length < n) this.bytes.push(0); return this; }
  set32(at: number, v: number) { for (let k = 0; k < 4; k++) this.bytes[at + k] = (v >>> (8 * k)) & 0xff; }
  out() { return new Uint8Array(this.bytes); }
}

/** Deterministic pseudo random generator. */
function rng(seed: number) {
  let s = seed >>> 0;
  return () => { s = (s * 1664525 + 1013904223) >>> 0; return s >>> 24; };
}

/** PCX RLE encoding of one 8-bit image (runs never cross scanlines). */
function encodePcx(w: number, h: number, px: Uint8Array, palette: Uint8Array | null): Uint8Array {
  const bpl = w + (w & 1); // even bytes-per-line, as the PCX spec requires
  const wr = new Writer();
  wr.u8(10).u8(5).u8(1).u8(8).u16(0).u16(0).u16(w - 1).u16(h - 1).u16(72).u16(72);
  wr.pad(65).u8(1).u16(bpl).u16(1).pad(128);
  for (let y = 0; y < h; y++) {
    const line = new Uint8Array(bpl);
    line.set(px.subarray(y * w, y * w + w));
    let x = 0;
    while (x < bpl) {
      const v = line[x];
      let n = 1;
      while (x + n < bpl && line[x + n] === v && n < 63) n++;
      if (n > 1 || v >= 0xc0) wr.u8(0xc0 | n).u8(v);
      else wr.u8(v);
      x += n;
    }
  }
  if (palette) wr.u8(0x0c).raw(palette);
  return wr.out();
}

function makeRgbPalette(seed: number): Uint8Array {
  const r = rng(seed);
  const p = new Uint8Array(768);
  for (let i = 0; i < 768; i++) p[i] = r();
  return p;
}

function encodeRle8(px: Uint8Array): Uint8Array {
  const wr = new Writer();
  let i = 0;
  while (i < px.length) {
    const v = px[i];
    let n = 1;
    while (i + n < px.length && px[i + n] === v && n < 63) n++;
    if (n > 1 || (v & 0xc0) === 0x40) wr.u8(0x40 | n).u8(v);
    else wr.u8(v);
    i += n;
  }
  return wr.out();
}

function paeth(a: number, b: number, c: number) {
  const p = a + b - c, pa = Math.abs(p - a), pb = Math.abs(p - b), pc = Math.abs(p - c);
  return pa <= pb && pa <= pc ? a : pb <= pc ? b : c;
}

function crc32(b: Uint8Array) {
  let c = ~0;
  for (const x of b) { c ^= x; for (let k = 0; k < 8; k++) c = (c >>> 1) ^ (0xedb88320 & -(c & 1)); }
  return ~c >>> 0;
}

/** PNG encoder: row y uses filter type y % 5 (exercises all filters). */
function encodePng(opts: {
  w: number; h: number; colorType: number; bitDepth?: number; data: Uint8Array;
  plte?: Uint8Array; trns?: Uint8Array;
}): Uint8Array {
  const { w, h, colorType, data } = opts;
  const bitDepth = opts.bitDepth ?? 8;
  const ch = { 0: 1, 2: 3, 3: 1, 4: 2, 6: 4 }[colorType]!;
  const stride = Math.ceil((w * ch * bitDepth) / 8);
  const bpp = Math.max(1, (ch * bitDepth) >> 3);
  const raw = new Uint8Array((stride + 1) * h);
  for (let y = 0; y < h; y++) {
    const ft = y % 5;
    raw[y * (stride + 1)] = ft;
    for (let x = 0; x < stride; x++) {
      const cur = data[y * stride + x];
      const a = x >= bpp ? data[y * stride + x - bpp] : 0;
      const b = y > 0 ? data[(y - 1) * stride + x] : 0;
      const c = x >= bpp && y > 0 ? data[(y - 1) * stride + x - bpp] : 0;
      const pred = [0, a, b, (a + b) >> 1, paeth(a, b, c)][ft];
      raw[y * (stride + 1) + 1 + x] = (cur - pred) & 0xff;
    }
  }
  const chunk = (t: string, d: Uint8Array) => {
    const o = new Uint8Array(12 + d.length);
    const dv = new DataView(o.buffer);
    dv.setUint32(0, d.length);
    for (let i = 0; i < 4; i++) o[4 + i] = t.charCodeAt(i);
    o.set(d, 8);
    dv.setUint32(8 + d.length, crc32(o.subarray(4, 8 + d.length)));
    return o;
  };
  const ih = new Uint8Array(13);
  new DataView(ih.buffer).setUint32(0, w);
  new DataView(ih.buffer).setUint32(4, h);
  ih[8] = bitDepth; ih[9] = colorType;
  const z = zlibSync(raw);
  const parts = [new Uint8Array([137, 80, 78, 71, 13, 10, 26, 10]), chunk('IHDR', ih)];
  if (opts.plte) parts.push(chunk('PLTE', opts.plte));
  if (opts.trns) parts.push(chunk('tRNS', opts.trns));
  // split IDAT in two chunks to test concatenation
  const half = z.length >> 1;
  parts.push(chunk('IDAT', z.subarray(0, half)), chunk('IDAT', z.subarray(half)), chunk('IEND', new Uint8Array()));
  const total = parts.reduce((s, p) => s + p.length, 0);
  const out = new Uint8Array(total);
  let o = 0;
  for (const p of parts) { out.set(p, o); o += p.length; }
  return out;
}

// ---------------------------------------------------------------------------
// PCX
// ---------------------------------------------------------------------------

describe('PCX', () => {
  it('round-trips RLE with odd width (bytesPerLine padding) and reads the 0x0C palette', () => {
    const w = 37, h = 11;
    const r = rng(1);
    const px = new Uint8Array(w * h);
    for (let i = 0; i < px.length; i++) px[i] = i % 7 < 3 ? 0xc5 : r() > 128 ? r() : 0; // runs + literals >= 0xC0
    const pal = makeRgbPalette(2);
    const img = decodePcx(encodePcx(w, h, px, pal));
    expect(img.width).toBe(w);
    expect(img.height).toBe(h);
    expect(img.bytesPerLine).toBe(38);
    expect(Array.from(img.pixels)).toEqual(Array.from(px));
    expect(img.palette).not.toBeNull();
    expect(img.palette![5 * 4]).toBe(pal[15]);
    expect(img.palette![255 * 4 + 2]).toBe(pal[767]);
    expect(img.palette![3]).toBe(0); // index 0 transparent
    expect(img.palette![7]).toBe(255);
  });

  it('cuts runs at scanline boundaries', () => {
    // width 2, bpl 2, two lines; a run of 4 would cross the line: MUGEN truncates it.
    const data = new Uint8Array([0xc4, 9, 0xc2, 7]);
    const { pixels, end } = decodePcxRle(data, 2, 2, 2);
    expect(Array.from(pixels)).toEqual([9, 9, 7, 7]);
    expect(end).toBe(4);
  });
});

// ---------------------------------------------------------------------------
// SFF v2 compressors
// ---------------------------------------------------------------------------

describe('RLE8', () => {
  it('decodes hand-made stream', () => {
    // 0x43 0x05 => 5,5,5 ; 0x07 literal ; 0x41 0x42 => one 0x42 (escaped literal); 0x80 literal
    const out = decodeRle8(new Uint8Array([0x43, 5, 7, 0x41, 0x42, 0x80]), 6);
    expect(Array.from(out)).toEqual([5, 5, 5, 7, 0x42, 0x80]);
  });
  it('round-trips random data', () => {
    const r = rng(3);
    const px = new Uint8Array(500).map((_, i) => (i % 13 < 6 ? 0x44 : r()));
    expect(Array.from(decodeRle8(encodeRle8(px), px.length))).toEqual(Array.from(px));
  });
});

describe('RLE5', () => {
  it('decodes hand-made packets', () => {
    const stream = new Uint8Array([
      2, 0x80 | 2, 7, (1 << 5) | 3, (0 << 5) | 5, // 7 x3, then 3 x2, 5 x1
      1, 0x00, // color 0 x2, no data bytes
      0, 0x80 | 1, 30, (7 << 5) | 31, // 30 x1, 31 x8
    ]);
    const out = decodeRle5(stream, 17);
    expect(Array.from(out)).toEqual([7, 7, 7, 3, 3, 5, 0, 0, 30, 31, 31, 31, 31, 31, 31, 31, 31]);
  });
});

describe('LZ5', () => {
  it('decodes RLE short/long, LZ short (incl. recycled-bit offset) and LZ long', () => {
    const stream = new Uint8Array([
      0xf8, // control: packets 0-2 RLE, 3-7 LZ
      0x41, // RLE short: 1 x2
      0x22, // RLE short: 2 x1
      0x03, 0x00, // RLE long: 3 x8
      0x02, 0x0a, // LZ short: len 3, offset 11 (top bits 00)
      0x01, 0x0d, // LZ short: len 2, offset 14 (top bits 00)
      0xc1, 0x00, // LZ short: len 2, offset 1 (top bits 11, overlapping copy)
      0xc1, // 4th LZ short: len 2, offset from recycled bits 0b00001111 + 1 = 16
      0x00, 0x13, 0x00, // LZ long: offset 20, len 3
      0x00, // control: all RLE
      0x3f, // RLE short: 31 x1
    ]);
    const out = decodeLz5(stream, 24);
    expect(Array.from(out)).toEqual([1, 1, 2, 3, 3, 3, 3, 3, 3, 3, 3, 1, 1, 2, 1, 1, 1, 1, 2, 3, 1, 1, 2, 31]);
  });
  it('uses the top bits of a long LZ packet as offset high bits', () => {
    const stream = new Uint8Array([0x04, 0x27, 0x01, 0xff, 0x40, 0x07, 0x00]);
    const out = decodeLz5(stream, 267);
    expect(out[0]).toBe(7);
    expect(out[263]).toBe(1);
    expect(Array.from(out.subarray(264))).toEqual([7, 1, 1]); // offset 264
  });
});

// ---------------------------------------------------------------------------
// PNG
// ---------------------------------------------------------------------------

describe('PNG', () => {
  const w = 13, h = 10;
  it('decodes RGBA (type 6) with all filter types', () => {
    const r = rng(4);
    const data = new Uint8Array(w * h * 4).map(() => r());
    const png = decodePng(encodePng({ w, h, colorType: 6, data }));
    expect(png.width).toBe(w);
    expect(Array.from(png.rgba!)).toEqual(Array.from(data));
  });
  it('decodes RGB (type 2)', () => {
    const r = rng(5);
    const data = new Uint8Array(w * h * 3).map(() => r());
    const png = decodePng(encodePng({ w, h, colorType: 2, data }));
    for (let i = 0; i < w * h; i++) {
      expect([png.rgba![i * 4], png.rgba![i * 4 + 1], png.rgba![i * 4 + 2], png.rgba![i * 4 + 3]])
        .toEqual([data[i * 3], data[i * 3 + 1], data[i * 3 + 2], 255]);
    }
  });
  it('decodes indexed (type 3) with PLTE + tRNS, 8-bit and 4-bit', () => {
    const r = rng(6);
    const idx = new Uint8Array(w * h).map(() => r() & 15);
    const plte = makeRgbPalette(7).subarray(0, 16 * 3);
    const trns = new Uint8Array([0, 128]);
    const png = decodePng(encodePng({ w, h, colorType: 3, data: idx, plte, trns }));
    expect(Array.from(png.indexed!)).toEqual(Array.from(idx));
    expect(png.palette![3]).toBe(0);
    expect(png.palette![7]).toBe(128);
    expect(png.palette![11]).toBe(255);
    expect(png.palette![4 * 4 + 1]).toBe(plte[4 * 3 + 1]);
    // 4-bit packing
    const stride = Math.ceil(w / 2);
    const packed = new Uint8Array(stride * h);
    for (let y = 0; y < h; y++) for (let x = 0; x < w; x++) packed[y * stride + (x >> 1)] |= idx[y * w + x] << (x & 1 ? 0 : 4);
    const png4 = decodePng(encodePng({ w, h, colorType: 3, bitDepth: 4, data: packed, plte }));
    expect(Array.from(png4.indexed!)).toEqual(Array.from(idx));
  });
});

// ---------------------------------------------------------------------------
// SFF v1 (synthetic, round trip)
// ---------------------------------------------------------------------------

interface V1Spr { group: number; index: number; ax: number; ay: number; w?: number; h?: number; px?: Uint8Array; pal?: Uint8Array | null; samePal?: boolean; link?: number }

function encodeSffV1(sprites: V1Spr[]): Uint8Array {
  const wr = new Writer();
  wr.str('ElecbyteSpr\0').u8(0).u8(1).u8(0).u8(1);
  wr.u32(1).u32(sprites.length).u32(512).u32(32).u8(1).pad(512);
  sprites.forEach((s, i) => {
    const pcx = s.link !== undefined ? new Uint8Array() : encodePcx(s.w!, s.h!, s.px!, s.pal ?? null);
    const start = wr.length;
    const next = i === sprites.length - 1 ? 0 : start + 32 + pcx.length;
    wr.u32(next).u32(pcx.length).u16(s.ax).u16(s.ay).u16(s.group).u16(s.index).u16(s.link ?? 0).u8(s.samePal ? 1 : 0);
    wr.pad(start + 32).raw(pcx);
  });
  return wr.out();
}

describe('SFF v1', () => {
  it('round-trips PCX subfiles, linked sprites and shared palettes', () => {
    const r = rng(8);
    const mk = (w: number, h: number) => new Uint8Array(w * h).map((_, i) => (i % 5 === 0 ? 0 : r()));
    const palA = makeRgbPalette(10), palB = makeRgbPalette(11);
    const px0 = mk(21, 30), px1 = mk(8, 8), px3 = mk(16, 4);
    const buf = encodeSffV1([
      { group: 0, index: 0, ax: 10, ay: 29, w: 21, h: 30, px: px0, pal: palA },
      { group: 0, index: 1, ax: -3, ay: 4, w: 8, h: 8, px: px1, samePal: true },
      { group: 5, index: 2, ax: 7, ay: 8, link: 0 },
      { group: 9000, index: 0, ax: 0, ay: 0, w: 16, h: 4, px: px3, pal: palB },
    ]);
    const sff = decodeSff(buf);
    expect(sff.version).toBe(1);
    expect(sff.sprites).toHaveLength(4);
    const [s0, s1, s2, s3] = sff.sprites;
    expect([s0.group, s0.index, s0.axisX, s0.axisY, s0.width, s0.height]).toEqual([0, 0, 10, 29, 21, 30]);
    expect(Array.from(s0.indexed!)).toEqual(Array.from(px0));
    expect(s0.palette![4]).toBe(palA[3]);
    expect([s1.axisX, s1.axisY, s1.width, s1.height]).toEqual([-3, 4, 8, 8]);
    expect(Array.from(s1.indexed!)).toEqual(Array.from(px1));
    expect(s1.palette).toBe(s0.palette); // same palette as previous
    expect([s2.group, s2.index, s2.axisX, s2.axisY, s2.width, s2.height]).toEqual([5, 2, 7, 8, 21, 30]);
    expect(s2.indexed).toBe(s0.indexed); // linked
    expect(Array.from(s3.indexed!)).toEqual(Array.from(px3));
    expect(s3.palette![255 * 4 + 1]).toBe(palB[766]);
    expect(sff.palettes).toHaveLength(2);
    expect(s3.paletteIndex).toBe(1);
    const rgba = spriteToRGBA(s0);
    const c = s0.indexed![1];
    expect([rgba[4], rgba[5], rgba[6], rgba[7]]).toEqual(c ? [palA[c * 3], palA[c * 3 + 1], palA[c * 3 + 2], 255] : [0, 0, 0, 0]);
  });
});

// ---------------------------------------------------------------------------
// SFF v2 (synthetic)
// ---------------------------------------------------------------------------

describe('SFF v2', () => {
  it('decodes a synthetic file with raw/RLE8/RLE5/LZ5/PNG8/PNG32, linked sprite and linked palette', () => {
    const w = 6, h = 4;
    const r = rng(12);
    const pxRaw = new Uint8Array(w * h).map(() => r());
    const pxRle8 = new Uint8Array(w * h).map((_, i) => (i < 10 ? 3 : r()));
    const rle5 = new Uint8Array([23, 0x80 | 0, 9]); // 24 x color 9
    const lz5 = new Uint8Array([0x00, 0x03, 16]); // RLE long: 3 x (16+8)=24
    const png8 = encodePng({ w, h, colorType: 3, data: pxRaw, plte: makeRgbPalette(1) });
    const rgbaData = new Uint8Array(w * h * 4).map(() => r());
    const png32 = encodePng({ w, h, colorType: 6, data: rgbaData });
    const withLen = (d: Uint8Array) => new Uint8Array([w * h, 0, 0, 0, ...d]);
    const blobs = [pxRaw, withLen(encodeRle8(pxRle8)), withLen(rle5), withLen(lz5), withLen(png8), withLen(png32)];
    const formats = [0, 2, 3, 4, 10, 12];
    const depths = [8, 8, 5, 5, 8, 32];
    const pal0 = new Uint8Array(1024).map((_, i) => (i * 7) & 0xff);

    const wr = new Writer();
    wr.str('ElecbyteSpr\0').u8(0).u8(1).u8(0).u8(2).pad(36);
    const nSpr = blobs.length + 1;
    const sprOfs = 72, palOfs = sprOfs + nSpr * 28, ldataOfs = palOfs + 2 * 16;
    wr.u32(sprOfs).u32(nSpr).u32(palOfs).u32(2).u32(ldataOfs).u32(0).u32(0).u32(0).pad(sprOfs);
    let dataOfs = pal0.length;
    blobs.forEach((b, i) => {
      wr.u16(1).u16(i).u16(w).u16(h).u16(i).u16(0xffff - 1).u16(0).u8(formats[i]).u8(depths[i]);
      wr.u32(dataOfs).u32(b.length).u16(i === 4 ? 1 : 0).u16(0);
      dataOfs += b.length;
    });
    wr.u16(2).u16(0).u16(0).u16(0).u16(0).u16(0).u16(0).u8(0).u8(0).u32(0).u32(0).u16(0).u16(0); // linked to sprite 0
    wr.u16(1).u16(1).u16(256).u16(0).u32(0).u32(1024); // palette 0
    wr.u16(1).u16(2).u16(256).u16(0).u32(0).u32(0); // palette 1 linked to 0
    wr.raw(pal0);
    for (const b of blobs) wr.raw(b);

    const sff = decodeSff(wr.out());
    expect(sff.version).toBe(2);
    expect(sff.palettes).toHaveLength(2);
    expect(sff.palettes[1].data).toBe(sff.palettes[0].data);
    expect(sff.palettes[0].data[3]).toBe(0);
    expect(sff.palettes[0].data[4 + 3]).toBe(pal0[7]); // v2.01 keeps stored alpha
    const s = sff.sprites;
    expect(s).toHaveLength(7);
    expect([s[1].axisX, s[1].axisY]).toEqual([1, -2]);
    expect(Array.from(s[0].indexed!)).toEqual(Array.from(pxRaw));
    expect(Array.from(s[1].indexed!)).toEqual(Array.from(pxRle8));
    expect(Array.from(s[2].indexed!)).toEqual(new Array(24).fill(9));
    expect(Array.from(s[3].indexed!)).toEqual(new Array(24).fill(3));
    expect(Array.from(s[4].indexed!)).toEqual(Array.from(pxRaw));
    expect(s[4].palette).toBe(sff.palettes[1].data); // SFF palette wins over PNG PLTE
    expect(Array.from(s[5].rgba!)).toEqual(Array.from(rgbaData));
    expect(s[5].paletteIndex).toBe(-1);
    expect(s[6].indexed).toBe(s[0].indexed);
    expect([s[6].group, s[6].width, s[6].height]).toEqual([2, w, h]);
  });
});

// ---------------------------------------------------------------------------
// SND / ACT
// ---------------------------------------------------------------------------

describe('SND', () => {
  it('walks the subfile linked list', () => {
    const wr = new Writer();
    wr.str('ElecbyteSnd\0').u16(0).u16(1).u32(2).u32(512).pad(512);
    const a = new Uint8Array([82, 73, 70, 70, 1, 2, 3]);
    const b = new Uint8Array([82, 73, 70, 70, 9]);
    wr.u32(512 + 16 + a.length).u32(a.length).u32(5).u32(1).raw(a);
    wr.u32(0).u32(b.length).u32(-1 >>> 0).u32(42).raw(b);
    const m = decodeSnd(wr.out());
    expect([...m.keys()]).toEqual(['5,1', '-1,42']);
    expect(Array.from(m.get('5,1')!)).toEqual(Array.from(a));
    expect(Array.from(m.get('-1,42')!)).toEqual(Array.from(b));
  });
});

describe('ACT', () => {
  it('stores colors in reverse order', () => {
    const raw = new Uint8Array(768);
    raw.set([10, 20, 30], 0); // file entry 0 => palette index 255
    raw.set([1, 2, 3], 255 * 3); // file entry 255 => palette index 0
    raw.set([4, 5, 6], 254 * 3); // => index 1
    const pal = decodeAct(raw);
    expect(Array.from(pal.subarray(255 * 4, 256 * 4))).toEqual([10, 20, 30, 255]);
    expect(Array.from(pal.subarray(0, 4))).toEqual([1, 2, 3, 0]);
    expect(Array.from(pal.subarray(4, 8))).toEqual([4, 5, 6, 255]);
    expect(Array.from(encodeAct(pal))).toEqual(Array.from(raw));
  });
});

// ---------------------------------------------------------------------------
// Integration with real files (Ikemen GO screenpack), when available
// ---------------------------------------------------------------------------

const SP = '/tmp/claude-0/-home-user-ultimate-mugen/fa14b17d-4289-5fed-86c2-aacfb04ddae6/scratchpad/sp';
const read = (p: string) => new Uint8Array(fs.readFileSync(path.join(SP, p)));

describe.skipIf(!fs.existsSync(SP))('real files (Ikemen screenpack)', () => {
  it('decodes kfm.sff (SFF v2, LZ5) with a sane sprite 0,0', () => {
    const sff = decodeSff(read('chars/kfm/kfm.sff'));
    expect(sff.version).toBe(2);
    expect(sff.sprites.length).toBeGreaterThan(100);
    expect(sff.palettes.length).toBeGreaterThan(0);
    const s = sff.sprites.find((x) => x.group === 0 && x.index === 0)!;
    expect(s).toBeDefined();
    expect(s.width).toBeGreaterThan(20);
    expect(s.width).toBeLessThan(200);
    expect(s.height).toBeGreaterThan(50);
    expect(s.height).toBeLessThan(250);
    expect(s.axisX).toBeGreaterThan(0);
    expect(s.axisX).toBeLessThan(s.width);
    expect(s.axisY).toBeGreaterThan(s.height * 0.8); // feet near the bottom
    expect(s.axisY).toBeLessThanOrEqual(s.height + 2);
    expect(s.indexed!.length).toBe(s.width * s.height);
    expect(s.palette).not.toBeNull();
    const opaque = s.indexed!.reduce((n, v) => n + (v ? 1 : 0), 0);
    expect(opaque / s.indexed!.length).toBeGreaterThan(0.2);
    expect(opaque / s.indexed!.length).toBeLessThan(0.9);
    // index 0 transparent at the corners
    expect(s.indexed![0]).toBe(0);
  });

  it('decodes every SFF in the screenpack with consistent buffers', () => {
    const files: string[] = [];
    const walk = (d: string) => {
      for (const e of fs.readdirSync(path.join(SP, d), { withFileTypes: true })) {
        const rel = path.join(d, e.name);
        if (e.isDirectory()) walk(rel);
        else if (e.name.toLowerCase().endsWith('.sff')) files.push(rel);
      }
    };
    for (const d of ['chars', 'stages', 'data', 'font']) if (fs.existsSync(path.join(SP, d))) walk(d);
    expect(files.length).toBeGreaterThan(0);
    const problems: string[] = [];
    for (const f of files) {
      const buf = read(f);
      const sff = decodeSff(buf);
      if (!sff.sprites.length) problems.push(`${f}: no sprites`);
      for (const s of sff.sprites) {
        const id = `${f} ${s.group},${s.index}`;
        if (s.indexed && s.indexed.length !== s.width * s.height) problems.push(`${id}: indexed size`);
        if (s.rgba && s.rgba.length !== s.width * s.height * 4) problems.push(`${id}: rgba size`);
        if (s.indexed && s.width * s.height > 0 && !s.palette) problems.push(`${id}: no palette`);
      }
      if (sff.version === 2) {
        // uncompressed sizes recorded in the file must match the sprite size
        const dv = new DataView(buf.buffer, buf.byteOffset);
        const so = dv.getUint32(36, true), ldata = dv.getUint32(52, true), tdata = dv.getUint32(60, true);
        sff.sprites.forEach((s, i) => {
          const o = so + i * 28, fmt = buf[o + 14], len = dv.getUint32(o + 20, true);
          if (len && fmt >= 2 && fmt <= 4) {
            const start = (dv.getUint16(o + 26, true) & 1 ? tdata : ldata) + dv.getUint32(o + 16, true);
            if (dv.getUint32(start, true) !== s.width * s.height) problems.push(`${f} ${s.group},${s.index}: length prefix`);
          }
        });
      }
    }
    expect(problems).toEqual([]);
  }, 30000);

  it('decodes kfm intro.sff (SFF v1, PCX)', () => {
    const p = 'chars/kfm/intro.sff';
    if (!fs.existsSync(path.join(SP, p))) return;
    const sff = decodeSff(read(p));
    expect(sff.version).toBe(1);
    for (const s of sff.sprites) {
      expect(s.width).toBeGreaterThan(0);
      expect(s.palette).not.toBeNull();
    }
  });

  it('decodes kfm.snd into WAV entries', () => {
    const m = decodeSnd(read('chars/kfm/kfm.snd'));
    expect(m.size).toBeGreaterThan(0);
    for (const [k, v] of m) {
      expect(k).toMatch(/^-?\d+,-?\d+$/);
      expect(String.fromCharCode(...v.subarray(0, 4))).toBe('RIFF');
      expect(String.fromCharCode(...v.subarray(8, 12))).toBe('WAVE');
    }
  });

  it('ACT files match the SFF palette table (reverse order)', () => {
    const act = path.join(SP, 'work/gofx/5300-0.act');
    const sffPath = path.join(SP, 'data/gofx/gofx.sff');
    if (!fs.existsSync(act) || !fs.existsSync(sffPath)) return;
    const pal = decodeAct(new Uint8Array(fs.readFileSync(act)));
    const sff = decodeSff(new Uint8Array(fs.readFileSync(sffPath)));
    const ref = sff.palettes.find((p) => p.group === 5300 && p.index === 0)!.data;
    for (let c = 1; c < 256; c++) {
      expect([pal[c * 4], pal[c * 4 + 1], pal[c * 4 + 2]]).toEqual([ref[c * 4], ref[c * 4 + 1], ref[c * 4 + 2]]);
    }
  });
});
