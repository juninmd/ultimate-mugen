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
