import { test } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import vm from 'node:vm';

const source = fs.readFileSync(new URL('../web/gamepad.js', import.meta.url), 'utf8');

class FakeMouseEvent {
  constructor(type, init = {}) {
    this.type = type;
    if ('button' in init) this.button = init.button;
    if ('buttons' in init) this.buttons = init.buttons;
    if ('clientX' in init) this.clientX = init.clientX;
    if ('clientY' in init) this.clientY = init.clientY;
  }
}

class FakePointerEvent extends FakeMouseEvent {
  constructor(type, init = {}) {
    super(type, init);
    if ('pointerId' in init) this.pointerId = init.pointerId;
    if ('pointerType' in init) this.pointerType = init.pointerType;
  }
}

class FakeKeyboardEvent {
  constructor(type, init = {}) {
    this.type = type;
    Object.assign(this, init);
  }
}

function load({ pointerEvents = true, globals = {} } = {}) {
  const windowListeners = new Map();
  const documentListeners = new Map();
  const dispatched = [];
  const collect = (store) => (type, fn) => { const l = store.get(type) || []; l.push(fn); store.set(type, l); };
  const canvas = {
    dispatchEvent: (e) => {
      if (!(e instanceof FakeMouseEvent || e instanceof FakeKeyboardEvent)) throw new TypeError('canvas dispatches Event objects');
      dispatched.push(e);
      return true;
    },
  };
  const window = { addEventListener: collect(windowListeners) };
  const document = { hidden: false, addEventListener: collect(documentListeners), getElementById: (id) => (id === 'glcanvas' ? canvas : null) };
  const context = { window, document, MouseEvent: FakeMouseEvent, KeyboardEvent: FakeKeyboardEvent, ...globals };
  if (pointerEvents) context.PointerEvent = FakePointerEvent;
  vm.runInNewContext(source, context);
  const fireOn = (store) => (type, event) => { for (const fn of store.get(type) || []) fn(event); };
  return { window, document, dispatched, fire: fireOn(windowListeners), fireDocument: fireOn(documentListeners) };
}

const pointerUps = (dispatched) => dispatched.filter((e) => e.type === 'pointerup');

test('every release trigger sends a real pointer event the engine can read', () => {
  const { dispatched, fire, fireDocument, document } = load();
  fire('pointerdown', new FakePointerEvent('pointerdown', { clientX: 420, clientY: 310 }));
  fire('blur', { type: 'blur' });
  fire('pointermove', new FakePointerEvent('pointermove', { clientX: 640, clientY: 360 }));
  fire('pagehide', { type: 'pagehide' });
  fire('contextmenu', new FakePointerEvent('contextmenu'));
  document.hidden = true;
  fireDocument('visibilitychange');
  const ups = pointerUps(dispatched);
  assert.equal(ups.length, 4, 'blur, pagehide, contextmenu and a hidden tab all release the pointer');
  assert.deepEqual(ups.map((e) => [e.clientX, e.clientY]), [[420, 310], [640, 360], [640, 360], [640, 360]], 'the release follows the pointer, pressed or hovering');
  for (const up of ups) {
    assert.ok(up instanceof FakePointerEvent, 'pointerup must be a PointerEvent');
    assert.equal(up.pointerType, 'mouse');
    assert.equal(typeof up.pointerId, 'number');
    assert.equal(up.button, 0);
    assert.equal(up.buttons, 0);
  }
});

test('pointerup keeps pointerType and the cursor where the PointerEvent constructor is missing', () => {
  const { dispatched, fire } = load({ pointerEvents: false });
  fire('pointerdown', new FakePointerEvent('pointerdown', { clientX: 500, clientY: 250 }));
  fire('contextmenu', new FakeMouseEvent('contextmenu'));
  const up = pointerUps(dispatched).pop();
  assert.ok(up instanceof FakeMouseEvent, 'the fallback dispatches the closest event the browser has');
  assert.equal(up.pointerType, 'mouse');
  assert.equal(typeof up.pointerId, 'number');
  assert.equal(up.clientX, 500);
  assert.equal(up.clientY, 250);
});

test('releaseAll lifts every held key before it releases the pointer', () => {
  const { dispatched, fire } = load();
  fire('keydown', new FakeKeyboardEvent('keydown', { code: 'KeyQ', key: 'q', keyCode: 81 }));
  fire('blur', { type: 'blur' });
  assert.deepEqual(dispatched.map((e) => e.type), ['keyup', 'mouseup', 'pointerup']);
  assert.equal(dispatched[0].code, 'KeyQ');
  assert.equal(dispatched[0].key, 'q');
  fire('blur', { type: 'blur' });
  assert.equal(dispatched.filter((e) => e.type === 'keyup').length, 1, 'held keys are cleared once');
});

test('gamepad buttons map to engine bits and idle pads are ignored', () => {
  const gamepads = [null, { connected: false }, { connected: true, id: 'Pad', axes: [], buttons: [{ pressed: true }, { pressed: true }, { pressed: true }] }];
  const { window } = load({ globals: { navigator: { getGamepads: () => gamepads } } });
  assert.equal(window.ume_gp_count(), 1);
  assert.equal(window.ume_gp_name(0), 'Pad');
  assert.equal(window.ume_gp_state(0), 16 | 32 | 128);
  assert.equal(window.ume_gp_state(1), 0);
});

test('a multi-file import keeps every file after the file input is cleared', async () => {
  const { window } = load();
  const pushed = [];
  let open = 0;
  const wasm = {
    ume_import_begin: () => { open++; },
    ume_import_end: () => { open--; },
    ume_import_pending: () => 0,
    ume_import: (bytes) => pushed.push(bytes.length),
  };
  window.umeStartImports(wasm);
  await new Promise((r) => setTimeout(r, 0));
  const picked = [
    { name: 'Aquaman.zip', arrayBuffer: async () => new Uint8Array(4).buffer },
    { name: 'Thor_Odinson.zip', arrayBuffer: async () => new Uint8Array(7).buffer },
  ];
  const live = { length: picked.length, 0: picked[0], 1: picked[1], [Symbol.iterator]: Array.prototype[Symbol.iterator] };
  window.umeImport(live);
  live.length = 0;
  delete live[0];
  delete live[1];
  await new Promise((r) => setTimeout(r, 20));
  assert.deepEqual(pushed, [4, 7], 'both packs reach the wasm queue even though the input was cleared');
  assert.equal(open, 0, 'the batch is closed once every file was handed over');
});

test('an unreadable file in a multi-file import does not drop the files after it', async () => {
  const warnings = [];
  const { window } = load({ globals: { console: { warn: (...args) => warnings.push(args.join(' ')) } } });
  const pushed = [];
  let open = 0;
  window.umeStartImports({
    ume_import_begin: () => { open++; },
    ume_import_end: () => { open--; },
    ume_import_pending: () => 0,
    ume_import: (bytes) => pushed.push(bytes.length),
  });
  await new Promise((r) => setTimeout(r, 0));
  window.umeImport([
    { name: 'a.zip', arrayBuffer: async () => new Uint8Array(3).buffer },
    { name: 'moved.zip', arrayBuffer: async () => { throw new Error('NotReadableError'); } },
    { name: 'c.zip', arrayBuffer: async () => new Uint8Array(5).buffer },
  ]);
  await new Promise((r) => setTimeout(r, 20));
  assert.deepEqual(pushed, [3, 5], 'the file after the unreadable one is still imported');
  assert.equal(open, 0, 'the batch is closed');
  assert.ok(warnings.some((w) => w.includes('moved.zip')), 'the skipped file is named in the console');
});
