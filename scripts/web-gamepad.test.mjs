import { test } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import vm from 'node:vm';

const source = fs.readFileSync(new URL('../web/gamepad.js', import.meta.url), 'utf8');

class FakeMouseEvent {
  constructor(type, init = {}) {
    this.type = type;
    this.button = init.button ?? 0;
    this.buttons = init.buttons ?? 0;
    this.clientX = init.clientX ?? 0;
    this.clientY = init.clientY ?? 0;
  }
}

class FakePointerEvent extends FakeMouseEvent {
  constructor(type, init = {}) {
    super(type, init);
    this.pointerId = init.pointerId ?? 0;
    this.pointerType = init.pointerType ?? '';
  }
}

class FakeKeyboardEvent {
  constructor(type, init = {}) {
    this.type = type;
    Object.assign(this, init);
  }
}

function load(withPointerEvent = true) {
  const windowListeners = new Map();
  const dispatched = [];
  const canvas = { dispatchEvent: (e) => { dispatched.push(e); return true; } };
  const window = { addEventListener: (type, fn) => { const l = windowListeners.get(type) || []; l.push(fn); windowListeners.set(type, l); } };
  const context = {
    window,
    document: { addEventListener: () => {}, getElementById: (id) => (id === 'glcanvas' ? canvas : null) },
    MouseEvent: FakeMouseEvent,
    KeyboardEvent: FakeKeyboardEvent,
  };
  if (withPointerEvent) context.PointerEvent = FakePointerEvent;
  vm.runInNewContext(source, context);
  const fire = (type, event) => { for (const fn of windowListeners.get(type) || []) fn(event); };
  return { dispatched, fire };
}

const lastPointerUp = (dispatched) => dispatched.filter((e) => e.type === 'pointerup').pop();

test('releaseAll sends a real pointer event the engine can read', () => {
  const { dispatched, fire } = load();
  fire('pointerdown', new FakePointerEvent('pointerdown', { clientX: 420, clientY: 310 }));
  fire('blur', { type: 'blur' });
  const up = lastPointerUp(dispatched);
  assert.ok(up instanceof FakePointerEvent, 'pointerup must be a PointerEvent');
  assert.equal(up.pointerType, 'mouse');
  assert.equal(typeof up.pointerId, 'number');
  assert.equal(up.button, 0);
  assert.equal(up.buttons, 0);
  assert.equal(up.clientX, 420);
  assert.equal(up.clientY, 310);
});

test('pointerup keeps a pointerType even where PointerEvent is missing', () => {
  const { dispatched, fire } = load(false);
  fire('contextmenu', new FakeMouseEvent('contextmenu'));
  const up = lastPointerUp(dispatched);
  assert.ok(up, 'releaseAll must still release the pointer');
  assert.equal(up.pointerType, 'mouse');
  assert.equal(up.pointerId, 1);
});

test('releaseAll lifts every held key before releasing the pointer', () => {
  const { dispatched, fire } = load();
  fire('keydown', new FakeKeyboardEvent('keydown', { code: 'KeyQ', key: 'q', keyCode: 81 }));
  fire('blur', { type: 'blur' });
  const keyUps = dispatched.filter((e) => e.type === 'keyup');
  assert.deepEqual(keyUps.map((e) => e.code), ['KeyQ']);
  fire('blur', { type: 'blur' });
  assert.equal(dispatched.filter((e) => e.type === 'keyup').length, 1, 'held keys are cleared once');
});

test('gamepad buttons map to engine bits and idle pads are ignored', () => {
  const window = { addEventListener: () => {} };
  const gamepads = [null, { connected: false }, { connected: true, id: 'Pad', axes: [], buttons: [{ pressed: true }, { pressed: true }, { pressed: true }] }];
  const context = { window, document: { addEventListener: () => {} }, navigator: { getGamepads: () => gamepads } };
  vm.runInNewContext(source, context);
  assert.equal(window.ume_gp_count(), 1);
  assert.equal(window.ume_gp_name(0), 'Pad');
  assert.equal(window.ume_gp_state(0), 16 | 32 | 128);
  assert.equal(window.ume_gp_state(1), 0);
});
