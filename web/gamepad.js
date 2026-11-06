// Globals imported by the wasm module: Gamepad API (buttons/sticks), DualShock/DualSense/Xbox vibration,
// touch detection and the MUGEN .zip import button (packs persisted in IndexedDB).
(function () {
  const K = { U: 1, D: 2, L: 4, R: 8, a: 16, b: 32, c: 64, x: 128, y: 256, z: 512, s: 1024, d: 2048, w: 4096 };
  const held = new Map();
  // False while the page is blurred: pads keep reporting their last state (some browsers stop polling them
  // altogether), so a button held at that moment would stay pressed for the rest of the session.
  let pageActive = true;
  const cursor = { x: 0, y: 0 };
  const keepCursor = (e) => { cursor.x = e.clientX; cursor.y = e.clientY; };
  window.addEventListener('pointermove', keepCursor, true);
  window.addEventListener('pointerdown', keepCursor, true);
  // The engine reads pointerType/pointerId/offsetX from every pointer event, so a MouseEvent disguised
  // as a pointer event (pointerType undefined) crashes the wasm glue in passStringToWasm0.
  function pointerUp() {
    const init = { bubbles: true, cancelable: true, button: 0, buttons: 0, pointerId: 1, pointerType: 'mouse', isPrimary: true, clientX: cursor.x, clientY: cursor.y, screenX: cursor.x, screenY: cursor.y };
    const e = typeof PointerEvent === 'function' ? new PointerEvent('pointerup', init) : new MouseEvent('pointerup', init);
    if (e.pointerType === undefined) Object.defineProperty(e, 'pointerType', { value: init.pointerType });
    if (e.pointerId === undefined) Object.defineProperty(e, 'pointerId', { value: init.pointerId });
    return e;
  }
  function releaseAll() {
    const c = document.getElementById('glcanvas');
    if (!c) return;
    for (const [code, e] of held) c.dispatchEvent(new KeyboardEvent('keyup', { code, key: e.key, keyCode: e.keyCode, bubbles: true }));
    held.clear();
    try { c.dispatchEvent(new MouseEvent('mouseup', { button: 0, buttons: 0, bubbles: true })); } catch (e) {}
    try { c.dispatchEvent(pointerUp()); } catch (e) {}
  }
  window.addEventListener('keydown', (e) => { if (e.code) held.set(e.code, { key: e.key, keyCode: e.keyCode }); }, true);
  window.addEventListener('keyup', (e) => { held.delete(e.code); }, true);
  window.addEventListener('blur', () => { pageActive = false; releaseAll(); });
  window.addEventListener('focus', () => { pageActive = true; });
  window.addEventListener('pagehide', releaseAll);
  window.addEventListener('contextmenu', releaseAll);
  document.addEventListener('visibilitychange', () => { if (document.hidden) releaseAll(); });
  function pads() { return navigator.getGamepads ? Array.from(navigator.getGamepads()).filter((p) => p && p.connected) : []; }
  function state(i) {
    const p = pads()[i]; if (!p || !pageActive || document.hidden) return 0;
    const pr = (n) => !!(p.buttons[n] && p.buttons[n].pressed);
    const ax = p.axes[0] || 0, ay = p.axes[1] || 0; let b = 0;
    if (ay < -0.5 || pr(12)) b |= K.U; if (ay > 0.5 || pr(13)) b |= K.D; if (ax < -0.5 || pr(14)) b |= K.L; if (ax > 0.5 || pr(15)) b |= K.R;
    if (pr(2)) b |= K.x; if (pr(3)) b |= K.y; if (pr(5)) b |= K.z; if (pr(0)) b |= K.a; if (pr(1)) b |= K.b; if (pr(7)) b |= K.c;
    if (pr(4)) b |= K.d; if (pr(6)) b |= K.w; if (pr(9)) b |= K.s;
    return b;
  }
  function rumble(i, strong, weak, ms) {
    const p = pads()[i]; if (!p) return;
    const a = p.vibrationActuator || (p.hapticActuators && p.hapticActuators[0]);
    if (a && a.playEffect) a.playEffect(a.type || 'dual-rumble', { duration: ms, strongMagnitude: strong, weakMagnitude: weak }).catch(() => {});
    else if (a && a.pulse) a.pulse(Math.max(strong, weak), ms);
    else if (navigator.vibrate && ('ontouchstart' in window)) navigator.vibrate(ms);
  }
  window.ume_gp_count = () => pads().length;
  window.ume_gp_state = (i) => state(i);
  window.ume_gp_name = (i) => { const p = pads()[i]; return p ? p.id : ''; };
  window.ume_gp_rumble = (i, s, w, ms) => rumble(i, s, w, ms);
  window.ume_set_import_visible = (v) => { const b = document.getElementById('imp'); if (b) b.style.display = v ? '' : 'none'; };
  window.ume_storage_get = (k) => { try { return localStorage.getItem('ume.' + k) || ''; } catch (e) { return ''; } };
  window.ume_storage_set = (k, v) => { try { localStorage.setItem('ume.' + k, v); } catch (e) {} };
  window.ume_is_touch = () => (('ontouchstart' in window) || navigator.maxTouchPoints > 0 || /[?&]touch=1/.test(location.search)) ? 1 : 0;

  window.umeStartImports = function (wasm) {
    const DB = 'ultimate-mugen', ST = 'packs';
    function db() { return new Promise((res, rej) => { const r = indexedDB.open(DB, 1); r.onupgradeneeded = () => r.result.createObjectStore(ST); r.onsuccess = () => res(r.result); r.onerror = () => rej(r.error); }); }
    async function digest(buf) {
      try { const h = await crypto.subtle.digest('SHA-256', buf); return Array.from(new Uint8Array(h), (x) => x.toString(16).padStart(2, '0')).join(''); }
      catch (e) { let h = 2166136261; for (let i = 0; i < buf.length; i++) { h ^= buf[i]; h = Math.imul(h, 16777619) >>> 0; } return 'f' + buf.length + '-' + h.toString(16); }
    }
    const seen = new Set();
    async function store(key, buf) { try { const d = await db(); d.transaction(ST, 'readwrite').objectStore(ST).put(buf, key); } catch (e) {} }
    async function keys() { try { const d = await db(); return await new Promise((res) => { const rq = d.transaction(ST).objectStore(ST).getAllKeys(); rq.onsuccess = () => res(rq.result || []); rq.onerror = () => res([]); }); } catch (e) { return []; } }
    async function get(key) { try { const d = await db(); return await new Promise((res) => { const rq = d.transaction(ST).objectStore(ST).get(key); rq.onsuccess = () => res(rq.result); rq.onerror = () => res(null); }); } catch (e) { return null; } }
    async function remove(key) { try { const d = await db(); d.transaction(ST, 'readwrite').objectStore(ST).delete(key); } catch (e) {} }
    async function importSlot() { while (wasm.ume_import_pending() >= 2) await new Promise((r) => setTimeout(r, 40)); }
    async function accept(buf) {
      const id = await digest(buf);
      if (seen.has(id)) return false;
      seen.add(id);
      wasm.ume_import(buf);
      await importSlot();
      return id;
    }
    async function send(file) { const buf = new Uint8Array(await file.arrayBuffer()); const id = await accept(buf); if (id) store(id, buf); }
    window.umeImport = (files) => {
      wasm.ume_import_begin();
      (async () => { try { for (const f of files) await send(f); } finally { wasm.ume_import_end(); } })();
    };
    window.umeClearImports = async () => { seen.clear(); const d = await db(); d.transaction(ST, 'readwrite').objectStore(ST).clear(); };
    (async () => {
      const ks = await keys();
      wasm.ume_import_begin();
      try {
        for (const key of ks) {
          const value = await get(key);
          if (!value) continue;
          const buf = new Uint8Array(value);
          const id = await accept(buf);
          if (key !== id) { await remove(key); if (id) await store(id, buf); }
        }
      } finally { wasm.ume_import_end(); }
    })();
    const input = document.getElementById('ume-file');
    if (input) input.addEventListener('change', () => window.umeImport(input.files));
    window.addEventListener('dragover', (e) => e.preventDefault());
    window.addEventListener('drop', (e) => { e.preventDefault(); window.umeImport(e.dataTransfer.files); });
  };
})();
