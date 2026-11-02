/** Canvas helpers that work in browsers (and return null in node). */
export function makeCanvas(w: number, h: number): { canvas: HTMLCanvasElement; ctx: CanvasRenderingContext2D } | null {
  if (typeof document === 'undefined') return null;
  const c = document.createElement('canvas');
  c.width = Math.max(1, Math.ceil(w));
  c.height = Math.max(1, Math.ceil(h));
  const ctx = c.getContext('2d')!;
  return { canvas: c, ctx };
}

export function imageFromRGBA(rgba: Uint8ClampedArray, w: number, h: number): CanvasImageSource | null {
  const c = makeCanvas(w, h);
  if (!c || !w || !h) return null;
  const img = new ImageData(new Uint8ClampedArray(rgba.buffer.slice(rgba.byteOffset, rgba.byteOffset + w * h * 4)) as any, w, h);
  c.ctx.putImageData(img, 0, 0);
  return c.canvas;
}
