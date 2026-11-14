(function () {
  const DB = 'ultimate-mugen-market';
  const STORE = 'kv';
  let wasm = null;
  const waiting = [];

  function open() {
    return new Promise((resolve, reject) => {
      const r = indexedDB.open(DB, 1);
      r.onupgradeneeded = () => r.result.createObjectStore(STORE);
      r.onsuccess = () => resolve(r.result);
      r.onerror = () => reject(r.error);
    });
  }

  function request(db, mode, run) {
    return new Promise((resolve, reject) => {
      const tx = db.transaction(STORE, mode);
      const rq = run(tx.objectStore(STORE));
      tx.oncomplete = () => resolve(rq ? rq.result : undefined);
      tx.onerror = () => reject(tx.error);
      tx.onabort = () => reject(tx.error);
    });
  }

  function deliver(key) {
    open()
      .then((db) => request(db, 'readonly', (s) => s.get(key)))
      .then((value) => wasm.ume_market_deliver(key, value ? new Uint8Array(value) : new Uint8Array()))
      .catch(() => wasm.ume_market_deliver(key, new Uint8Array()));
  }

  window.ume_market_put = (key, data) => {
    const copy = new Uint8Array(data);
    open().then((db) => request(db, 'readwrite', (s) => s.put(copy, key))).catch(() => {});
  };

  window.ume_market_del = (key) => {
    open().then((db) => request(db, 'readwrite', (s) => s.delete(key))).catch(() => {});
  };

  window.ume_market_get = (key) => {
    if (wasm) deliver(key);
    else waiting.push(key);
  };

  window.ume_prompt = (message, fallback) => {
    const value = window.prompt(message, fallback || '');
    return value == null ? '' : value;
  };

  window.umeStartMarket = async function (module) {
    wasm = module;
    try {
      const db = await open();
      const keys = await request(db, 'readonly', (s) => s.getAllKeys());
      for (const key of keys) {
        if (typeof key !== 'string' || !(key === 'state.json' || key.startsWith('catalog/'))) continue;
        const value = await request(db, 'readonly', (s) => s.get(key));
        if (value) wasm.ume_market_hydrate(key, new Uint8Array(value));
      }
    } catch (e) {}
    wasm.ume_market_hydrate_done();
    waiting.splice(0).forEach(deliver);
  };
})();
