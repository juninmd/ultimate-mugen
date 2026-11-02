/** Tiny IndexedDB wrapper for imported MUGEN packs. */
const DB = 'ultimate-mugen';
const STORE = 'packs';

export interface StoredPack { id: string; name: string; added: number; files: Record<string, Uint8Array> }

function open(): Promise<IDBDatabase> {
  return new Promise((res, rej) => {
    const r = indexedDB.open(DB, 1);
    r.onupgradeneeded = () => r.result.createObjectStore(STORE, { keyPath: 'id' });
    r.onsuccess = () => res(r.result);
    r.onerror = () => rej(r.error);
  });
}
export async function savePack(p: StoredPack) {
  try {
    const db = await open();
    await new Promise<void>((res, rej) => { const tx = db.transaction(STORE, 'readwrite'); tx.objectStore(STORE).put(p); tx.oncomplete = () => res(); tx.onerror = () => rej(tx.error); });
  } catch (e) { console.warn('savePack failed', e); }
}
export async function loadPacks(): Promise<StoredPack[]> {
  try {
    const db = await open();
    return await new Promise((res, rej) => { const tx = db.transaction(STORE, 'readonly'); const rq = tx.objectStore(STORE).getAll(); rq.onsuccess = () => res(rq.result as StoredPack[]); rq.onerror = () => rej(rq.error); });
  } catch { return []; }
}
export async function deletePack(id: string) {
  try {
    const db = await open();
    await new Promise<void>((res) => { const tx = db.transaction(STORE, 'readwrite'); tx.objectStore(STORE).delete(id); tx.oncomplete = () => res(); });
  } catch { /* */ }
}
