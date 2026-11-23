const ASSET_STALL_MS = 30000;

async function readStreamed(response, onChunk) {
  const reader = response.body.getReader();
  const chunks = [];
  let size = 0;
  for (;;) {
    const { done, value } = await reader.read();
    if (done) break;
    onChunk();
    chunks.push(value);
    size += value.length;
  }
  const out = new Uint8Array(size);
  let offset = 0;
  for (const chunk of chunks) {
    out.set(chunk, offset);
    offset += chunk.length;
  }
  return out;
}

window.ume_fetch_asset = async (path) => {
  const controller = new AbortController();
  let timeout = setTimeout(() => controller.abort(), ASSET_STALL_MS);
  const rearm = () => {
    clearTimeout(timeout);
    timeout = setTimeout(() => controller.abort(), ASSET_STALL_MS);
  };
  try {
    const response = await fetch(new URL(path, document.baseURI), { signal: controller.signal });
    if (!response.ok) throw new Error(`${path}: HTTP ${response.status}`);
    if (!response.body) return new Uint8Array(await response.arrayBuffer());
    return await readStreamed(response, rearm);
  } finally {
    clearTimeout(timeout);
  }
};

const LITTLE_ENDIAN = new Uint8Array(new Uint32Array([1]).buffer)[0] === 1;

function isOpaque(data) {
  if (!LITTLE_ENDIAN) {
    for (let i = 3; i < data.length; i += 4) if (data[i] !== 255) return false;
    return true;
  }
  const words = new Uint32Array(data.buffer, data.byteOffset, data.length >> 2);
  for (let i = 0; i < words.length; i++) if (words[i] >>> 24 !== 255) return false;
  return true;
}

window.ume_decode_image = async (bytes) => {
  if (typeof createImageBitmap !== 'function' || typeof OffscreenCanvas !== 'function') return null;
  const bitmap = await createImageBitmap(new Blob([bytes], { type: 'image/png' }), {
    premultiplyAlpha: 'none',
    colorSpaceConversion: 'none',
  });
  try {
    const canvas = new OffscreenCanvas(bitmap.width, bitmap.height);
    const context = canvas.getContext('2d', { willReadFrequently: true });
    context.drawImage(bitmap, 0, 0);
    const image = context.getImageData(0, 0, bitmap.width, bitmap.height);
    const data = new Uint8Array(image.data.buffer);
    if (!isOpaque(data)) return null;
    return { width: image.width, height: image.height, data };
  } finally {
    bitmap.close();
  }
};
