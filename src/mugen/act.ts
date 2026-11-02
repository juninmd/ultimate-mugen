/**
 * ACT palette decoder (Photoshop color table as used by MUGEN characters).
 *
 * 256 RGB triplets (768 bytes; some files have 4 extra trailing bytes holding a
 * color count / transparent index, which are ignored). MUGEN stores colors in
 * REVERSE order: file entry i is palette index 255 - i (same as Ikemen GO's
 * readActPalette). Index 0 is transparent.
 */
export function decodeAct(buf: Uint8Array): Uint8ClampedArray {
  const pal = new Uint8ClampedArray(256 * 4);
  const count = Math.min(256, Math.floor(buf.length / 3));
  for (let i = 0; i < count; i++) {
    const dst = (255 - i) * 4;
    pal[dst] = buf[i * 3];
    pal[dst + 1] = buf[i * 3 + 1];
    pal[dst + 2] = buf[i * 3 + 2];
    pal[dst + 3] = 255;
  }
  pal[3] = 0;
  return pal;
}

/** Inverse of decodeAct: RGBA 256*4 palette → 768-byte MUGEN .act. */
export function encodeAct(pal: Uint8ClampedArray | Uint8Array): Uint8Array {
  const out = new Uint8Array(768);
  for (let i = 0; i < 256; i++) {
    const src = (255 - i) * 4;
    out[i * 3] = pal[src] ?? 0;
    out[i * 3 + 1] = pal[src + 1] ?? 0;
    out[i * 3 + 2] = pal[src + 2] ?? 0;
  }
  return out;
}
