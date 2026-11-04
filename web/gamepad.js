// Globals imported by the wasm module: Gamepad API (buttons/sticks), DualShock/DualSense/Xbox vibration,
// touch detection and the MUGEN .zip import button (packs persisted in IndexedDB).
(function () {
  const K = { U: 1, D: 2, L: 4, R: 8, a: 16, b: 32, c: 64, x: 128, y: 256, z: 512, s: 1024, d: 2048, w: 4096 };
  const held = new Map();
  function releaseAll() {
    const c = document.getElementById('glcanvas');
    if (!c) return;
    for (const [code, e] of held) c.dispatchEvent(new KeyboardEvent('keyup', { code, key: e.key, keyCode: e.keyCode, bubbles: true }));
    held.clear();
    for (const t of ['mouseup', 'pointerup']) { try { c.dispatchEvent(new MouseEvent(t, { button: 0, bubbles: true })); } catch (e) {} }
  }
  window.addEventListener('keydown', (e) => { if (e.code) held.set(e.code, { key: e.key, keyCode: e.keyCode }); }, true);
  window.addEventListener('keyup', (e) => { held.delete(e.code); }, true);
  window.addEventListener('blur', releaseAll);
  window.addEventListener('pagehide', releaseAll);
  window.addEventListener('contextmenu', releaseAll);
  document.addEventListener('visibilitychange', () => { if (document.hidden) releaseAll(); });
  function pads() { return navigator.getGamepads ? Array.from(navigator.getGamepads()).filter((p) => p && p.connected) : []; }
  function state(i) {
    const p = pads()[i]; if (!p) return 0;
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
  window.ume_is_touch = () => (('ontouchstart' in window) || navigator.maxTouchPoints > 0 || /[?&]touch=1/.test(location.search)) ? 1 : 0;

  window.umeStartImports = function (wasm) {
    const DB = 'ultimate-mugen', ST = 'packs';
    function db() { return new Promise((res, rej) => { const r = indexedDB.open(DB, 1); r.onupgradeneeded = () => r.result.createObjectStore(ST); r.onsuccess = () => res(r.result); r.onerror = () => rej(r.error); }); }
    async function store(name, buf) { try { const d = await db(); d.transaction(ST, 'readwrite').objectStore(ST).put(buf, name); } catch (e) {} }
    async function all() { try { const d = await db(); return await new Promise((res) => { const out = []; const rq = d.transaction(ST).objectStore(ST).openCursor(); rq.onsuccess = () => { const c = rq.result; if (c) { out.push(c.value); c.continue(); } else res(out); }; rq.onerror = () => res(out); }); } catch (e) { return []; } }
    async function send(file) { const buf = new Uint8Array(await file.arrayBuffer()); wasm.ume_import(buf); store(file.name + ':' + file.size, buf); }
    window.umeImport = (files) => { for (const f of files) send(f); };
    window.umeClearImports = async () => { const d = await db(); d.transaction(ST, 'readwrite').objectStore(ST).clear(); };
    all().then((bufs) => bufs.forEach((b) => wasm.ume_import(new Uint8Array(b))));
    const input = document.getElementById('ume-file');
    if (input) input.addEventListener('change', () => window.umeImport(input.files));
    window.addEventListener('dragover', (e) => e.preventDefault());
    window.addEventListener('drop', (e) => { e.preventDefault(); window.umeImport(e.dataTransfer.files); });
  };
})();
