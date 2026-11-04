import { test } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import vm from 'node:vm';

const source = fs.readFileSync(new URL('../web/gamepad.js', import.meta.url), 'utf8');

class Events {
  constructor() { this.listeners = new Map(); }
  addEventListener(type, fn) { const list = this.listeners.get(type) || []; list.push(fn); this.listeners.set(type, list); }
  fire(type, event = { type }) { for (const fn of this.listeners.get(type) || []) fn(event); }
}

class FakeEvent {
  constructor(type, init = {}) { this.type = type; Object.assign(this, init); }
}

function load({ pad }) {
  const window = new Events();
  const document = new Events();
  document.hidden = false;
  document.getElementById = (id) => (id === 'glcanvas' ? { dispatchEvent() {} } : null);
  const navigator = { getGamepads: () => [pad] };
  vm.runInNewContext(source, { window, document, navigator, MouseEvent: FakeEvent, KeyboardEvent: FakeEvent, PointerEvent: FakeEvent });
  return { window, document };
}

const pressed = (buttons) => ({ connected: true, id: 'Pad', axes: [], buttons: buttons.map((pressed) => ({ pressed })) });
const START = 9;
const SQUARE = 2;

test('a button held when the page blurs does not stay pressed', () => {
  const { window } = load({ pad: pressed([0, 0, 1, 0, 0, 0, 0, 0, 0, 1]) });
  assert.equal(window.ume_gp_state(0), 128 | 1024, 'square and start while the page is active');
  window.fire('blur');
  assert.equal(window.ume_gp_state(0), 0, 'the pad is ignored while another window has focus');
  window.fire('focus');
  assert.equal(window.ume_gp_state(0), 128 | 1024, 'pressing again after coming back works');
});

test('a hidden tab reads no gamepad input', () => {
  const { window, document } = load({ pad: pressed([0, 0, 0, 0, 0, 0, 0, 0, 0, 1]) });
  assert.equal(window.ume_gp_state(0), 1024);
  document.hidden = true;
  assert.equal(window.ume_gp_state(0), 0, 'a background tab never fights on its own');
});

test('disconnected pads stay ignored', () => {
  const { window } = load({ pad: { connected: false, id: 'Pad', axes: [], buttons: [{ pressed: true }] } });
  assert.equal(window.ume_gp_count(), 0);
  assert.equal(window.ume_gp_state(0), 0);
  assert.equal(window.ume_gp_name(0), '');
});
