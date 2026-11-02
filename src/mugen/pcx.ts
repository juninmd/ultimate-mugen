/**
 * PCX decoder (8-bit, single plane), as embedded in SFF v1 files.
 *
 * Layout: 128-byte header, RLE (or raw) scanlines of `bytesPerLine` bytes each
 * (lines are padded to an even length; padding bytes are discarded), and an
 * optional 256-color VGA palette at the end introduced by a 0x0C marker byte.
 */

export interface PcxImage {
  width: number;
  height: number;
  bytesPerLine: number;
  /** Indexed pixels, width*height. */
  pixels: Uint8Array;
  /** RGBA 256*4 palette (index 0 alpha 0, others 255), or null if none found. */
  palette: Uint8ClampedArray | null;
  /** Offset (relative to the start of the PCX buffer) right after the image data. */
  dataEnd: number;
}

export interface PcxHeader {
  encoding: number;
  bitsPerPixel: number;
  width: number;
  height: number;
  planes: number;
  bytesPerLine: number;
}

export const PCX_HEADER_SIZE = 128;

export function readPcxHeader(buf: Uint8Array): PcxHeader {
  if (buf.length < PCX_HEADER_SIZE) throw new Error('PCX: buffer too small for header');
  const u16 = (o: number) => buf[o] | (buf[o + 1] << 8);
  const xmin = u16(4), ymin = u16(6), xmax = u16(8), ymax = u16(10);
  const width = Math.max(0, xmax - xmin + 1);
  const height = Math.max(0, ymax - ymin + 1);
  let bytesPerLine = u16(66);
  if (bytesPerLine < width) bytesPerLine = width; // broken headers: never drop real pixels
  return {
    encoding: buf[2],
    bitsPerPixel: buf[3],
    width,
    height,
    planes: buf[65] || 1,
    bytesPerLine,
  };
}

/**
 * Decode PCX RLE scanlines. Each code byte >= 0xC0 is a run of (code & 0x3F)
 * copies of the following byte; anything else is a literal pixel. Runs are cut
 * at scanline boundaries (same behavior as MUGEN/Ikemen). Bytes in the
 * [width, bytesPerLine) padding area are dropped.
 *
 * Returns the pixels and the offset just past the last consumed byte.
 */
export function decodePcxRle(
  data: Uint8Array,
  width: number,
  height: number,
  bytesPerLine: number,
  start = 0,
  end = data.length,
): { pixels: Uint8Array; end: number } {
  const out = new Uint8Array(width * height);
  let i = start;
  for (let y = 0; y < height; y++) {
    const row = y * width;
    let x = 0;
    while (x < bytesPerLine) {
      if (i >= end) return { pixels: out, end: i };
      let d = data[i++];
      let n = 1;
      if (d >= 0xc0) {
        n = d & 0x3f;
        d = i < end ? data[i++] : 0;
      }
      for (; n > 0 && x < bytesPerLine; n--, x++) {
        if (x < width) out[row + x] = d;
      }
    }
  }
  return { pixels: out, end: i };
}

/** Read a 768-byte RGB palette at `offset` into RGBA (index 0 transparent). */
export function readRgbPalette(buf: Uint8Array, offset: number): Uint8ClampedArray {
  const pal = new Uint8ClampedArray(256 * 4);
  for (let c = 0; c < 256; c++) {
    const o = offset + c * 3;
    pal[c * 4] = buf[o] ?? 0;
    pal[c * 4 + 1] = buf[o + 1] ?? 0;
    pal[c * 4 + 2] = buf[o + 2] ?? 0;
    pal[c * 4 + 3] = c === 0 ? 0 : 255;
  }
  return pal;
}

/**
 * Locate the 256-color palette of a PCX buffer. Preferred: the 0x0C marker
 * immediately after the image data. Fallbacks: a marker exactly 769 bytes
 * before the end, then (legacy SFF v1 quirk) the raw last 768 bytes.
 * Returns the offset of the first palette byte, or -1.
 */
export function findPcxPalette(buf: Uint8Array, dataEnd: number): number {
  if (dataEnd + 769 <= buf.length && buf[dataEnd] === 0x0c) return dataEnd + 1;
  if (buf.length >= PCX_HEADER_SIZE + 769 && buf[buf.length - 769] === 0x0c) return buf.length - 768;
  if (buf.length - 768 >= dataEnd) return buf.length - 768;
  return -1;
}

/** Decode an 8-bit single-plane PCX image. */
export function decodePcx(buf: Uint8Array, opts: { readPalette?: boolean } = {}): PcxImage {
  const h = readPcxHeader(buf);
  if (h.bitsPerPixel !== 8 || h.planes !== 1) {
    throw new Error(`PCX: unsupported format (${h.bitsPerPixel} bpp, ${h.planes} planes); only 8-bit 1-plane`);
  }
  let pixels: Uint8Array;
  let dataEnd: number;
  if (h.encoding === 1) {
    const r = decodePcxRle(buf, h.width, h.height, h.bytesPerLine, PCX_HEADER_SIZE);
    pixels = r.pixels;
    dataEnd = r.end;
  } else {
    pixels = new Uint8Array(h.width * h.height);
    for (let y = 0; y < h.height; y++) {
      const src = PCX_HEADER_SIZE + y * h.bytesPerLine;
      const avail = Math.max(0, Math.min(h.width, buf.length - src));
      pixels.set(buf.subarray(src, src + avail), y * h.width);
    }
    dataEnd = Math.min(buf.length, PCX_HEADER_SIZE + h.height * h.bytesPerLine);
  }
  let palette: Uint8ClampedArray | null = null;
  if (opts.readPalette !== false) {
    const po = findPcxPalette(buf, dataEnd);
    if (po >= 0) palette = readRgbPalette(buf, po);
  }
  return { width: h.width, height: h.height, bytesPerLine: h.bytesPerLine, pixels, palette, dataEnd };
}
