(function () {
  const K = { U: 1, D: 2, L: 4, R: 8, A: 16, B: 32, C: 64, X: 128, Y: 256, Z: 512, D2: 2048, W: 4096, T: 16384 };
  const NAV = { OK: 16, BACK: 32, START: 64 };
  const DEAD = 0.28, KEEP = 0.18, AXIS = 0.414, TRAVEL = 0.32, TAP_FRAMES = 2;
  const lang = (navigator.language || 'en').slice(0, 2);
  const t3 = (pt, en, es) => (lang === 'pt' ? pt : lang === 'es' ? es : en);
  const SYSTEM = [
    [t3('VOLTAR', 'BACK', 'VOLVER'), 0, NAV.BACK, 'back'],
    ['OK', 0, NAV.OK, 'ok'],
    [t3('DESP.', 'AWAKE', 'DESP.'), K.T, 0, 'awake'],
    ['II', 0, NAV.START, 'start'],
  ];
  const ATTACKS = [
    ['X', K.X, 'punch'], ['Y', K.Y, 'punch'], ['Z', K.Z, 'punch'], [t3('ESP', 'SP', 'ESP'), K.D2, 'special'],
    ['A', K.A, 'kick'], ['B', K.B, 'kick'], ['C', K.C, 'kick'], ['SUPER', K.D2 | K.W, 'special'],
  ];
  const CSS = `
#vpad{position:fixed;inset:0;z-index:6;pointer-events:none;display:none;--u:clamp(40px,10.5vmin,68px);--s:clamp(112px,30vmin,184px);--g:calc(var(--u)*.2);font:800 calc(var(--u)*.34)/1 system-ui,sans-serif;color:#fff;text-shadow:0 1px 2px #000}
#vpad.on{display:block}
#vpad .vp-hit{position:absolute;pointer-events:auto;touch-action:none;user-select:none;-webkit-user-select:none;-webkit-touch-callout:none;-webkit-tap-highlight-color:transparent}
#vp-stick{left:0;bottom:0;width:calc(var(--s)*1.5 + env(safe-area-inset-left));height:calc(var(--s)*1.5 + env(safe-area-inset-bottom))}
#vp-base{position:absolute;left:calc(env(safe-area-inset-left) + var(--s)*.22);bottom:calc(env(safe-area-inset-bottom) + var(--s)*.22);width:var(--s);height:var(--s);box-sizing:border-box;border-radius:50%;border:2px solid rgba(255,255,255,.6);background:radial-gradient(circle,rgba(255,255,255,.1) 0,rgba(10,6,24,.45) 70%)}
#vp-knob{position:absolute;left:28%;top:28%;width:44%;height:44%;border-radius:50%;background:rgba(255,255,255,.6);box-shadow:0 0 0 2px rgba(0,0,0,.35);will-change:transform}
#vp-stick.on #vp-knob{background:rgba(34,232,255,.85)}
#vp-right{position:absolute;right:calc(env(safe-area-inset-right) + var(--g));bottom:calc(env(safe-area-inset-bottom) + var(--g));display:grid;grid-template-columns:repeat(4,var(--u));gap:var(--g);pointer-events:none}
#vpad #vp-right .vp-hit{position:relative;display:flex;align-items:center;justify-content:center;box-sizing:border-box;border:2px solid rgba(255,255,255,.6);width:var(--u);height:var(--u);border-radius:50%}
#vpad #vp-right .vp-sys{height:calc(var(--u)*.72);border-radius:999px;font-size:calc(var(--u)*.24)}
.vp-punch{background:rgba(58,140,255,.5)}.vp-kick{background:rgba(255,77,109,.5)}.vp-special{background:rgba(255,183,3,.5)}
.vp-back{background:rgba(90,84,110,.6)}.vp-ok{background:rgba(61,220,151,.55)}.vp-awake{background:rgba(160,90,255,.55)}.vp-start{background:rgba(255,204,51,.55)}
#vpad .vp-hit.on{transform:scale(.88);filter:brightness(1.7);border-color:#fff}
#vp-debug{position:fixed;left:4px;top:4px;z-index:20;margin:0;padding:4px 6px;font:11px monospace;color:#9fe;background:rgba(0,0,0,.75);pointer-events:none;white-space:pre}`;

  const owners = new Map();
  const latched = new Map();
  const stats = { downs: 0, ups: 0, cancels: 0, polls: 0 };
  let on = false, root = null, stick = null, knob = null, base = null, lastBits = 0, lastNav = 0, debugBox = null;

  const forced = /[?&]touch=1/.test(location.search || '');
  const coarse = () => { try { return matchMedia('(pointer: coarse)').matches || matchMedia('(any-pointer: coarse)').matches; } catch (e) { return false; } };
  const touchCapable = () => forced || navigator.maxTouchPoints > 0 || 'ontouchstart' in window || coarse();

  function button(label, bits, nav, kind, extra) {
    const b = document.createElement('div');
    b.className = `vp-hit vp-${kind}${extra}`;
    b.textContent = label;
    b.dataset.bits = String(bits);
    b.dataset.nav = String(nav);
    b.dataset.slide = nav ? '' : '1';
    b.setAttribute('aria-hidden', 'true');
    return b;
  }

  function build() {
    if (root) return;
    const style = document.createElement('style');
    style.textContent = CSS;
    document.head.appendChild(style);
    root = document.createElement('div');
    root.id = 'vpad';
    stick = document.createElement('div');
    stick.id = 'vp-stick';
    stick.className = 'vp-hit';
    base = document.createElement('div');
    base.id = 'vp-base';
    knob = document.createElement('div');
    knob.id = 'vp-knob';
    base.appendChild(knob);
    stick.appendChild(base);
    const right = document.createElement('div');
    right.id = 'vp-right';
    for (const [label, bits, nav, kind] of SYSTEM) right.appendChild(button(label, bits, nav, kind, ' vp-sys'));
    for (const [label, bits, kind] of ATTACKS) right.appendChild(button(label, bits, 0, kind, ''));
    root.append(stick, right);
    root.addEventListener('pointerdown', down);
    for (const type of ['touchstart', 'touchmove', 'touchend', 'contextmenu', 'selectstart', 'dblclick']) {
      root.addEventListener(type, (e) => { if (e.cancelable) e.preventDefault(); }, { passive: false });
    }
    document.body.appendChild(root);
    if (on) root.classList.add('on');
  }

  function show() {
    if (on) return;
    on = true;
    if (root) root.classList.add('on');
  }

  function stickBits(o) {
    const r = base.getBoundingClientRect();
    const radius = r.width / 2 || 1;
    const dx = o.x - (r.left + radius), dy = o.y - (r.top + radius);
    const len = Math.hypot(dx, dy);
    const travel = Math.min(len, radius * TRAVEL * 2);
    const k = len > 0 ? travel / len : 0;
    knob.style.transform = `translate(${dx * k}px,${dy * k}px)`;
    let bits = 0;
    if (len > radius * (o.bits ? KEEP : DEAD)) {
      const ax = Math.abs(dx), ay = Math.abs(dy);
      if (ax > ay * AXIS) bits |= dx > 0 ? K.R : K.L;
      if (ay > ax * AXIS) bits |= dy > 0 ? K.D : K.U;
    }
    o.bits = bits;
  }

  function press(el) {
    el._n = (el._n || 0) + 1;
    el.classList.add('on');
    latched.set(el, TAP_FRAMES);
  }

  function release(el) {
    el._n = Math.max(0, (el._n || 0) - 1);
    if (!el._n) el.classList.remove('on');
  }

  function down(e) {
    if (e.pointerType === 'mouse' && e.button !== 0) return;
    const el = e.target.closest && e.target.closest('.vp-hit');
    if (!el || owners.has(e.pointerId)) return;
    if (e.cancelable) e.preventDefault();
    try { el.setPointerCapture(e.pointerId); } catch (err) {}
    stats.downs++;
    const o = { el, x: e.clientX, y: e.clientY, bits: 0 };
    owners.set(e.pointerId, o);
    if (el === stick) { stick.classList.add('on'); stickBits(o); } else { press(el); buzz(); }
    debug();
  }

  function move(e) {
    const o = owners.get(e.pointerId);
    if (!o) return;
    o.x = e.clientX; o.y = e.clientY;
    if (o.el === stick) { stickBits(o); return; }
    if (!o.el.dataset.slide) return;
    const hit = document.elementFromPoint(o.x, o.y);
    const next = hit && hit.closest && hit.closest('#vp-right .vp-hit');
    if (next && next !== o.el && next.dataset.slide) { release(o.el); o.el = next; press(next); buzz(); }
  }

  function up(e) {
    const o = owners.get(e.pointerId);
    if (!o) return;
    owners.delete(e.pointerId);
    if (e.type === 'pointercancel') stats.cancels++; else stats.ups++;
    if (o.el === stick) {
      if (![...owners.values()].some((x) => x.el === stick)) { stick.classList.remove('on'); knob.style.transform = ''; }
    } else {
      release(o.el);
    }
    debug();
  }

  function releaseAll() {
    for (const id of [...owners.keys()]) up({ pointerId: id, type: 'pointerup' });
  }

  function buzz() {
    try { if (navigator.vibrate) navigator.vibrate(8); } catch (e) {}
  }

  function poll() {
    stats.polls++;
    let bits = 0, nav = 0;
    for (const o of owners.values()) {
      if (o.el === stick) bits |= o.bits;
      else { bits |= +o.el.dataset.bits; nav |= +o.el.dataset.nav; }
    }
    for (const [el, n] of latched) {
      bits |= +el.dataset.bits;
      nav |= +el.dataset.nav;
      if (n <= 1) latched.delete(el); else latched.set(el, n - 1);
    }
    lastBits = bits;
    lastNav = nav;
    return bits;
  }

  function debug() {
    if (!debugBox) return;
    debugBox.textContent = `vpad on=${on} touchCapable=${touchCapable()} dpr=${window.devicePixelRatio} ${innerWidth}x${innerHeight}\nfingers=${owners.size} bits=${lastBits} nav=${lastNav} ${JSON.stringify(stats)}`;
  }

  window.addEventListener('pointermove', move, true);
  window.addEventListener('pointerup', up, true);
  window.addEventListener('pointercancel', up, true);
  window.addEventListener('lostpointercapture', up, true);
  window.addEventListener('pointerdown', (e) => { if (e.pointerType === 'touch') show(); }, true);
  window.addEventListener('touchstart', show, { capture: true, passive: true });
  window.addEventListener('blur', releaseAll);
  window.addEventListener('pagehide', releaseAll);
  document.addEventListener('visibilitychange', () => { if (document.hidden) releaseAll(); });
  const guardCanvas = (e) => { if (e.target && e.target.id === 'glcanvas' && e.cancelable) e.preventDefault(); };
  for (const type of ['touchstart', 'touchmove']) window.addEventListener(type, guardCanvas, { capture: true, passive: false });
  document.addEventListener('gesturestart', (e) => e.preventDefault());
  document.addEventListener('dblclick', (e) => e.preventDefault(), { passive: false });

  if (touchCapable()) on = true;
  const start = () => {
    build();
    if (/[?&]debug=1/.test(location.search || '')) {
      debugBox = document.createElement('pre');
      debugBox.id = 'vp-debug';
      document.body.appendChild(debugBox);
      setInterval(debug, 250);
    }
  };
  if (document.body) start(); else document.addEventListener('DOMContentLoaded', start);

  window.ume_vpad_on = () => (on ? 1 : 0);
  window.ume_vpad_poll = () => poll();
  window.ume_vpad_nav = () => lastNav;
  window.__vpad = { state: () => ({ on, fingers: owners.size, bits: lastBits, nav: lastNav, stats: { ...stats } }) };
})();
