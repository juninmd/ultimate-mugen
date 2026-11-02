/**
 * SND (ElecbyteSnd) decoder.
 *
 * Header: [0]"ElecbyteSnd\0" [12]verLo u16 [14]verHi u16 [16]numSounds u32
 *         [20]firstSubfileOffset u32
 * Subfile: [0]nextSubfileOffset u32 [4]length u32 [8]group i32 [12]index i32
 *          [16] `length` bytes of sound data (normally a RIFF/WAVE file).
 */

const SIGNATURE = 'ElecbyteSnd\0';

/** Key used in the map returned by decodeSnd. */
export function sndKey(group: number, index: number): string {
  return `${group},${index}`;
}

/**
 * Decode an SND file into a map `${group},${index}` → sound file bytes (WAV).
 * Each value is a copy with its own ArrayBuffer (usable with decodeAudioData).
 * Duplicate keys keep the first occurrence (MUGEN/Ikemen behavior).
 */
export function decodeSnd(buf: Uint8Array): Map<string, Uint8Array> {
  for (let i = 0; i < SIGNATURE.length; i++) {
    if (buf[i] !== SIGNATURE.charCodeAt(i)) throw new Error('SND: invalid signature (expected ElecbyteSnd)');
  }
  const dv = new DataView(buf.buffer, buf.byteOffset, buf.byteLength);
  const count = dv.getUint32(16, true);
  let ofs = dv.getUint32(20, true);
  const out = new Map<string, Uint8Array>();
  const seen = new Set<number>();
  for (let i = 0; i < count; i++) {
    if (ofs < 24 || ofs + 16 > buf.length || seen.has(ofs)) break;
    seen.add(ofs);
    const next = dv.getUint32(ofs, true);
    const length = dv.getUint32(ofs + 4, true);
    const group = dv.getInt32(ofs + 8, true);
    const index = dv.getInt32(ofs + 12, true);
    const start = ofs + 16;
    const end = Math.min(buf.length, start + length);
    const key = sndKey(group, index);
    if (!out.has(key) && end > start) out.set(key, buf.slice(start, end));
    ofs = next;
  }
  return out;
}
