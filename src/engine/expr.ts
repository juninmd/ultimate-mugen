/**
 * MUGEN expression language: tokenizer, parser (to AST) and closure compiler.
 * Supports int/float semantics, intervals, redirection, string comparisons,
 * special triggers (animelem, timemod, projhit...), and := assignments.
 */
export type Node =
  | { k: 'num'; v: number; float: boolean }
  | { k: 'bin'; op: string; a: Node; b: Node }
  | { k: 'un'; op: string; a: Node }
  | { k: 'trig'; name: string; args: Node[]; comp?: string; raw?: string }
  | { k: 'redir'; kind: string; arg?: Node; e: Node }
  | { k: 'cmpstr'; name: string; neg: boolean; values: string[] }
  | { k: 'interval'; a: Node; neg: boolean; lo: Node; hi: Node; loInc: boolean; hiInc: boolean }
  | { k: 'special'; name: string; op: string; a: Node; op2?: string; b?: Node }
  | { k: 'assign'; name: string; idx: Node; v: Node };

interface Tok { t: 'num' | 'id' | 'str' | 'op' | 'eof'; v: string; float?: boolean }

const OPS = [':=', '**', '!=', '<=', '>=', '&&', '||', '^^', '=', '<', '>', '+', '-', '*', '/', '%', '!', '~', '&', '|', '^', '(', ')', '[', ']', ','];

export function tokenize(src: string): Tok[] {
  const out: Tok[] = [];
  let i = 0;
  while (i < src.length) {
    const c = src[i];
    if (c === ' ' || c === '\t' || c === '\r' || c === '\n') { i++; continue; }
    if (c === '"') {
      const e = src.indexOf('"', i + 1);
      const end = e < 0 ? src.length : e;
      out.push({ t: 'str', v: src.slice(i + 1, end) });
      i = end + 1;
      continue;
    }
    if (/[0-9.]/.test(c) && (c !== '.' || /[0-9]/.test(src[i + 1] ?? ''))) {
      let j = i;
      while (j < src.length && /[0-9.]/.test(src[j])) j++;
      const s = src.slice(i, j);
      out.push({ t: 'num', v: s, float: s.includes('.') });
      i = j;
      continue;
    }
    if (/[A-Za-z_]/.test(c)) {
      let j = i;
      while (j < src.length && /[A-Za-z0-9_.]/.test(src[j])) j++;
      out.push({ t: 'id', v: src.slice(i, j) });
      i = j;
      continue;
    }
    const op = OPS.find((o) => src.startsWith(o, i));
    if (op) { out.push({ t: 'op', v: op }); i += op.length; continue; }
    // unknown char: skip
    i++;
  }
  out.push({ t: 'eof', v: '' });
  return out;
}

const REDIRS = new Set(['parent', 'root', 'helper', 'target', 'partner', 'enemy', 'enemynear', 'playerid', 'p2', 'stateowner']);
const STRCMP = new Set(['command', 'statetype', 'movetype', 'p2statetype', 'p2movetype', 'name', 'p1name', 'p2name', 'p3name', 'p4name', 'authorname', 'hitdefattr', 'physics', 'stagevar', 'gethitvar_dummy']);
const COMPONENT = new Set(['vel', 'pos', 'p2dist', 'p2bodydist', 'p1dist', 'rootdist', 'parentdist', 'screenpos', 'camerapos', 'hitvel', 'camerazoom', 'scale', 'offset']);
const RAWARG = new Set(['const', 'gethitvar', 'stagevar', 'sysvar_dummy', 'const240p', 'const480p', 'const720p', 'const1080p', 'constcoord']);
const SPECIAL_EQ = new Set(['animelem', 'timemod', 'projhit', 'projcontact', 'projguarded']);

class Parser {
  p = 0;
  constructor(public toks: Tok[], public src: string) {}
  peek(o = 0) { return this.toks[Math.min(this.p + o, this.toks.length - 1)]; }
  next() { return this.toks[this.p++] ?? this.toks[this.toks.length - 1]; }
  isOp(v: string, o = 0) { const t = this.peek(o); return t.t === 'op' && t.v === v; }
  expect(v: string) {
    if (!this.isOp(v)) throw new Error(`Expected '${v}' in "${this.src}"`);
    this.p++;
  }

  // precedence table: lower index = lower precedence
  static LEVELS: string[][] = [['||'], ['^^'], ['&&'], ['|'], ['^'], ['&'], ['=', '!=', ':='], ['<', '<=', '>', '>='], ['+', '-'], ['*', '/', '%'], ['**']];

  parseExpr(level = 0): Node {
    if (level >= Parser.LEVELS.length) return this.parseUnary();
    let a = this.parseExpr(level + 1);
    const ops = Parser.LEVELS[level];
    for (;;) {
      const t = this.peek();
      if (t.t !== 'op' || !ops.includes(t.v)) break;
      this.p++;
      if ((t.v === '=' || t.v === '!=') && (this.isOp('[') || this.isOp('('))) {
        const iv = this.tryInterval(a, t.v === '!=');
        if (iv) { a = iv; continue; }
      }
      if (t.v === '**') {
        const b = this.parseExpr(level); // right assoc
        a = { k: 'bin', op: t.v, a, b };
        continue;
      }
      const b = this.parseExpr(level + 1);
      a = { k: 'bin', op: t.v, a, b };
    }
    return a;
  }

  tryInterval(a: Node, neg: boolean): Node | null {
    const save = this.p;
    const open = this.next().v;
    try {
      const lo = this.parseExpr(0);
      if (!this.isOp(',')) { this.p = save; return null; }
      this.p++;
      const hi = this.parseExpr(0);
      const close = this.peek();
      if (close.t !== 'op' || (close.v !== ']' && close.v !== ')')) { this.p = save; return null; }
      this.p++;
      return { k: 'interval', a, neg, lo, hi, loInc: open === '[', hiInc: close.v === ']' };
    } catch {
      this.p = save;
      return null;
    }
  }

  parseUnary(): Node {
    const t = this.peek();
    if (t.t === 'op' && (t.v === '!' || t.v === '~' || t.v === '-' || t.v === '+')) {
      this.p++;
      const a = this.parseUnary();
      if (t.v === '+') return a;
      if (t.v === '-' && a.k === 'num') return { k: 'num', v: -a.v, float: a.float };
      // unary binds tighter than ** in MUGEN? Treat -x**2 as (-x)**2 (MUGEN behaviour)
      return { k: 'un', op: t.v, a };
    }
    let a = this.parsePrimary();
    return a;
  }

  parsePrimary(): Node {
    const t = this.next();
    if (t.t === 'num') return { k: 'num', v: parseFloat(t.v), float: !!t.float };
    if (t.t === 'str') return { k: 'trig', name: '__str', args: [], raw: t.v };
    if (t.t === 'op' && t.v === '(') {
      const e = this.parseExpr(0);
      this.expect(')');
      return e;
    }
    if (t.t !== 'id') throw new Error(`Unexpected '${t.v}' in "${this.src}"`);
    const name = t.v.toLowerCase();

    // redirection
    if (REDIRS.has(name)) {
      let arg: Node | undefined;
      const save = this.p;
      if (this.isOp('(')) {
        this.p++;
        arg = this.parseExpr(0);
        this.expect(')');
      }
      if (this.isOp(',')) {
        this.p++;
        const e = this.parseUnaryNoRedirOps();
        return { k: 'redir', kind: name, arg, e };
      }
      this.p = save;
      if (name === 'p2') return { k: 'trig', name: 'p2', args: [] };
    }

    // string comparison triggers
    if (STRCMP.has(name) && (this.isOp('=') || this.isOp('!='))) {
      const neg = this.next().v === '!=';
      const vals: string[] = [];
      const v = this.next();
      vals.push(v.v);
      if (name === 'hitdefattr') {
        while (this.isOp(',') && this.peek(1).t === 'id' && /^[NSHAP][ATP]$/i.test(this.peek(1).v)) {
          this.p++;
          vals.push(this.next().v);
        }
      }
      return { k: 'cmpstr', name, neg, values: vals };
    }

    // animelem = n[, op t], timemod = m, r, projhit = 1, < 4
    if (SPECIAL_EQ.has(name) || /^proj(hit|contact|guarded)\d+$/.test(name)) {
      let tname = name;
      if (this.isOp('=') || this.isOp('!=')) {
        const op = this.next().v;
        const a = this.parseExpr(8);
        if (this.isOp(',')) {
          this.p++;
          let op2 = '=';
          const tt = this.peek();
          if (tt.t === 'op' && ['=', '!=', '<', '<=', '>', '>='].includes(tt.v)) { op2 = tt.v; this.p++; }
          const b = this.parseExpr(8);
          return { k: 'special', name: tname, op, a, op2, b };
        }
        return { k: 'special', name: tname, op, a };
      }
    }

    // component triggers: vel x
    if (COMPONENT.has(name)) {
      const c = this.peek();
      if (c.t === 'id' && /^[xyz]$/i.test(c.v)) {
        this.p++;
        return { k: 'trig', name, args: [], comp: c.v.toLowerCase() };
      }
    }

    if (this.isOp('(')) {
      this.p++;
      if (RAWARG.has(name)) {
        // raw identifier argument
        let depth = 1;
        let raw = '';
        while (depth > 0 && this.peek().t !== 'eof') {
          const x = this.next();
          if (x.t === 'op' && x.v === '(') depth++;
          if (x.t === 'op' && x.v === ')') { depth--; if (depth === 0) break; }
          raw += x.v;
        }
        return { k: 'trig', name, args: [], raw: raw.toLowerCase() };
      }
      const args: Node[] = [];
      if (!this.isOp(')')) {
        args.push(this.parseExpr(0));
        while (this.isOp(',')) { this.p++; args.push(this.parseExpr(0)); }
      }
      this.expect(')');
      if ((name === 'var' || name === 'fvar' || name === 'sysvar' || name === 'sysfvar') && this.isOp(':=')) {
        this.p++;
        const v = this.parseExpr(0);
        return { k: 'assign', name, idx: args[0], v };
      }
      return { k: 'trig', name, args };
    }
    return { k: 'trig', name, args: [] };
  }

  /** After a redirect "p2," the trigger itself (including its own =/!= string compare) */
  parseUnaryNoRedirOps(): Node {
    return this.parseUnary();
  }
}

export function parseExpr(src: string): Node {
  const p = new Parser(tokenize(src), src);
  const n = p.parseExpr(0);
  return n;
}

/** Parse comma-separated list of expressions (e.g. "pausetime = 12, 12"). */
export function parseExprList(src: string): Node[] {
  const p = new Parser(tokenize(src), src);
  const out: Node[] = [];
  if (p.peek().t === 'eof') return out;
  out.push(p.parseExpr(0));
  while (p.isOp(',')) { p.p++; if (p.peek().t === 'eof') break; out.push(p.parseExpr(0)); }
  return out;
}

// ---------------------------------------------------------------- compiler

export type Fn<H> = (h: H) => number;
export interface Compiled<H> { f: Fn<H>; float: boolean }

export interface TriggerResolver<H> {
  trigger(node: Extract<Node, { k: 'trig' }>, c: (n: Node) => Compiled<H>): Compiled<H>;
  redirect(kind: string, arg: Fn<H> | undefined): (h: H) => H | undefined;
  cmpstr(node: Extract<Node, { k: 'cmpstr' }>): Fn<H>;
  special(node: Extract<Node, { k: 'special' }>, c: (n: Node) => Compiled<H>): Fn<H>;
  assign(name: string, idx: Fn<H>, v: Compiled<H>): Fn<H>;
}

const trunc = (x: number) => (x < 0 ? Math.ceil(x) : Math.floor(x)) | 0;

export function compileNode<H>(n: Node, r: TriggerResolver<H>): Compiled<H> {
  const c = (x: Node) => compileNode(x, r);
  switch (n.k) {
    case 'num': { const v = n.v; return { f: () => v, float: n.float }; }
    case 'un': {
      const a = c(n.a);
      if (n.op === '!') return { f: (h) => (a.f(h) ? 0 : 1), float: false };
      if (n.op === '~') return { f: (h) => ~trunc(a.f(h)), float: false };
      return { f: (h) => -a.f(h), float: a.float };
    }
    case 'bin': {
      const a = c(n.a), b = c(n.b);
      const fl = a.float || b.float;
      const A = a.f, B = b.f;
      switch (n.op) {
        case '+': return { f: (h) => A(h) + B(h), float: fl };
        case '-': return { f: (h) => A(h) - B(h), float: fl };
        case '*': return { f: fl ? (h) => A(h) * B(h) : (h) => Math.imul(A(h) | 0, B(h) | 0), float: fl };
        case '/': return fl
          ? { f: (h) => { const d = B(h); return d === 0 ? 0 : A(h) / d; }, float: true }
          : { f: (h) => { const d = B(h); return d === 0 ? 0 : trunc(A(h) / d); }, float: false };
        case '%': return { f: (h) => { const d = trunc(B(h)); return d === 0 ? 0 : trunc(A(h)) % d; }, float: false };
        case '**': return { f: fl ? (h) => Math.pow(A(h), B(h)) : (h) => trunc(Math.pow(A(h), B(h))), float: fl };
        case '=': return { f: (h) => (A(h) === B(h) ? 1 : 0), float: false };
        case '!=': return { f: (h) => (A(h) !== B(h) ? 1 : 0), float: false };
        case '<': return { f: (h) => (A(h) < B(h) ? 1 : 0), float: false };
        case '<=': return { f: (h) => (A(h) <= B(h) ? 1 : 0), float: false };
        case '>': return { f: (h) => (A(h) > B(h) ? 1 : 0), float: false };
        case '>=': return { f: (h) => (A(h) >= B(h) ? 1 : 0), float: false };
        case '&&': return { f: (h) => (A(h) && B(h) ? 1 : 0), float: false };
        case '||': return { f: (h) => (A(h) || B(h) ? 1 : 0), float: false };
        case '^^': return { f: (h) => (!A(h) !== !B(h) ? 1 : 0), float: false };
        case '&': return { f: (h) => trunc(A(h)) & trunc(B(h)), float: false };
        case '|': return { f: (h) => trunc(A(h)) | trunc(B(h)), float: false };
        case '^': return { f: (h) => trunc(A(h)) ^ trunc(B(h)), float: false };
        case ':=': return { f: B, float: b.float };
      }
      throw new Error('bad op ' + n.op);
    }
    case 'interval': {
      const a = c(n.a).f, lo = c(n.lo).f, hi = c(n.hi).f;
      const { loInc, hiInc, neg } = n;
      return {
        f: (h) => {
          const v = a(h), l = lo(h), u = hi(h);
          const inside = (loInc ? v >= l : v > l) && (hiInc ? v <= u : v < u);
          return inside !== neg ? 1 : 0;
        },
        float: false,
      };
    }
    case 'redir': {
      const arg = n.arg ? c(n.arg).f : undefined;
      const get = r.redirect(n.kind, arg);
      const e = c(n.e);
      return { f: (h) => { const t = get(h); return t === undefined ? 0 : e.f(t); }, float: e.float };
    }
    case 'cmpstr': return { f: r.cmpstr(n), float: false };
    case 'special': return { f: r.special(n, c), float: false };
    case 'assign': return { f: r.assign(n.name, c(n.idx).f, c(n.v)), float: n.name.includes('f') };
    case 'trig': return r.trigger(n, c);
  }
}
