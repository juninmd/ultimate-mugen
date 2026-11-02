/** Decoded sprite, independent of the source format. */
export interface DecodedSprite {
  group: number;
  index: number;
  /** Axis (pivot) offset: sprite is drawn at (x - axisX, y - axisY). */
  axisX: number;
  axisY: number;
  width: number;
  height: number;
  /** Indexed pixels (8-bit) when the source is paletted; null for true-color. */
  indexed: Uint8Array | null;
  /** RGBA pixels for true-color (PNG 24/32) sprites, else null. */
  rgba: Uint8ClampedArray | null;
  /** Palette as RGBA (256*4) used by this sprite (own or shared), when paletted. */
  palette: Uint8ClampedArray | null;
  /** Index into the file's palette list (SFF v2) or -1. */
  paletteIndex: number;
}

export interface DecodedSff {
  version: 1 | 2;
  sprites: DecodedSprite[];
  /** Palettes found in the file (SFF v2 palette table, or SFF v1 per-sprite palettes deduped). RGBA 256*4. */
  palettes: { group: number; index: number; data: Uint8ClampedArray }[];
}

/** Convert an indexed sprite into RGBA using a palette; index 0 is transparent. */
export function spriteToRGBA(s: DecodedSprite, palette?: Uint8ClampedArray | null): Uint8ClampedArray {
  if (s.rgba) return s.rgba;
  const pal = palette ?? s.palette;
  const out = new Uint8ClampedArray(s.width * s.height * 4);
  if (!s.indexed || !pal) return out;
  for (let i = 0; i < s.indexed.length; i++) {
    const c = s.indexed[i];
    if (c === 0) continue;
    out[i * 4] = pal[c * 4];
    out[i * 4 + 1] = pal[c * 4 + 1];
    out[i * 4 + 2] = pal[c * 4 + 2];
    out[i * 4 + 3] = 255;
  }
  return out;
}
