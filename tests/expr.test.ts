import { describe, it, expect } from 'vitest';
import { parseExpr, compileNode, type TriggerResolver } from '../src/engine/expr';
import { parseCommand, matchCommand } from '../src/engine/commands';
import { InputHistory, K } from '../src/engine/input';

const host = { time: 7, vars: [0, 5, 9], name: 'Kung Fu Man', st: 'S' };
const r: TriggerResolver<typeof host> = {
  trigger(n, c) {
    const args = n.args.map(c);
    if (n.name === 'time') return { f: (h) => h.time, float: false };
    if (n.name === 'var') return { f: (h) => h.vars[args[0].f(h)] ?? 0, float: false };
    if (n.name === 'ifelse') return { f: (h) => (args[0].f(h) ? args[1].f(h) : args[2].f(h)), float: false };
    if (n.name === 'vel') return { f: () => (n.comp === 'y' ? -2.5 : 1.5), float: true };
    return { f: () => 0, float: false };
  },
  redirect: () => (h) => ({ ...h, time: 100 }),
  cmpstr: (n) => (h) => ((n.name === 'statetype' ? h.st === n.values[0].toUpperCase() : h.name.toLowerCase() === n.values[0].toLowerCase()) !== n.neg ? 1 : 0),
  special: () => () => 1,
  assign: (name, idx, v) => (h) => (h.vars[idx(h)] = v.f(h)),
};
const ev = (s: string) => compileNode(parseExpr(s), r).f(host);

describe('expressions', () => {
  it('integer semantics', () => { expect(ev('7/2')).toBe(3); expect(ev('7.0/2')).toBe(3.5); expect(ev('-7/2')).toBe(-3); expect(ev('2**3')).toBe(8); expect(ev('7%3')).toBe(1); });
  it('precedence & logic', () => { expect(ev('1 + 2 * 3')).toBe(7); expect(ev('time = 7 && var(1) = 5')).toBe(1); expect(ev('!time || 0')).toBe(0); expect(ev('1 ^^ 1')).toBe(0); });
  it('intervals', () => { expect(ev('time = [5, 7]')).toBe(1); expect(ev('time = [5, 7)')).toBe(0); expect(ev('time != (7, 9]')).toBe(1); });
  it('components & funcs', () => { expect(ev('vel y < 0')).toBe(1); expect(ev('ifelse(time > 3, 10, 20)')).toBe(10); });
  it('string compares & redirection', () => { expect(ev('statetype = S')).toBe(1); expect(ev('statetype != A')).toBe(1); expect(ev('name = "kung fu man"')).toBe(1); expect(ev('p2, time')).toBe(100); expect(ev('enemynear(0), time = 100')).toBe(1); });
  it('assignment', () => { expect(ev('var(2) := 42')).toBe(42); expect(host.vars[2]).toBe(42); });
});

describe('commands', () => {
  const feed = (h: InputHistory, seq: number[]) => { for (const b of seq) h.push(b, 1); };
  it('QCF + x', () => {
    const c = parseCommand('qcf', '~D, DF, F, x');
    const h = new InputHistory();
    feed(h, [0, K.D, K.D, K.D | K.R, K.R, K.R | K.x]);
    expect(matchCommand(h, c)).toBe(true);
  });
  it('rejects wrong order', () => {
    const c = parseCommand('qcf', '~D, DF, F, x');
    const h = new InputHistory();
    feed(h, [0, K.R, K.D | K.R, K.D, K.x]);
    expect(matchCommand(h, c)).toBe(false);
  });
  it('facing left flips F/B', () => {
    const c = parseCommand('qcf', '~D, DF, F, x');
    const h = new InputHistory();
    for (const b of [0, K.D, K.D, K.D | K.L, K.L, K.L | K.x]) h.push(b, -1);
    expect(matchCommand(h, c)).toBe(true);
  });
  it('dash F,F and hold', () => {
    const h = new InputHistory();
    feed(h, [0, K.R, 0, K.R]);
    expect(matchCommand(h, parseCommand('FF', 'F, F', 10))).toBe(true);
    expect(matchCommand(h, parseCommand('hf', '/$F', 1))).toBe(true);
  });
  it('simultaneous a+b and charge', () => {
    const h = new InputHistory();
    feed(h, [0, K.a, K.a | K.b]);
    expect(matchCommand(h, parseCommand('ab', 'a+b'))).toBe(true);
    const h2 = new InputHistory();
    feed(h2, [0, ...Array(40).fill(K.L), K.R, K.R | K.x]);
    expect(matchCommand(h2, parseCommand('chg', '~30$B, F, x', 20))).toBe(true);
  });
});
