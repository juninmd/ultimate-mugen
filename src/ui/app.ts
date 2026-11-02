import { Registry, type CharEntry, type StageEntry } from '../game/registry';
import { FightSession } from '../game/session';
import { KeyboardState, KeyboardSource, GamepadSource, MergedSource, type KeyMap, type InputSource } from '../engine/input';
import { WebAudioSink } from '../engine/audio';
import { TouchControls } from './touch';

const P1_SOLO: KeyMap = {
  KeyW: 'U', KeyS: 'D', KeyA: 'L', KeyD: 'R', ArrowUp: 'U', ArrowDown: 'D', ArrowLeft: 'L', ArrowRight: 'R',
  KeyU: 'x', KeyI: 'y', KeyO: 'z', KeyJ: 'a', KeyK: 'b', KeyL: 'c', KeyZ: 'a', KeyX: 'b', KeyC: 'c', KeyV: 'x', KeyB: 'y', KeyN: 'z',
  Digit1: 'd', Digit2: 'w', Digit3: 'e', Enter: 's',
};
const P1_DUO: KeyMap = { KeyW: 'U', KeyS: 'D', KeyA: 'L', KeyD: 'R', KeyU: 'x', KeyI: 'y', KeyO: 'z', KeyJ: 'a', KeyK: 'b', KeyL: 'c', Digit1: 'd', Digit2: 'w', Digit3: 'e' };
const P2_DUO: KeyMap = {
  ArrowUp: 'U', ArrowDown: 'D', ArrowLeft: 'L', ArrowRight: 'R', Numpad4: 'x', Numpad5: 'y', Numpad6: 'z', Numpad1: 'a', Numpad2: 'b', Numpad3: 'c',
  Numpad7: 'd', Numpad8: 'w', Numpad9: 'e',
};

type Mode = 'arcade' | 'cpu' | 'versus' | 'training' | 'watch';
const MODE_LABEL: Record<Mode, string> = { arcade: 'Arcade', cpu: 'Versus CPU', versus: '2 Jogadores', training: 'Treino', watch: 'CPU vs CPU' };

const h = (html: string) => { const t = document.createElement('template'); t.innerHTML = html.trim(); return t.content.firstElementChild as HTMLElement; };
const esc = (s: string) => s.replace(/[&<>"]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]!));

export class App {
  reg = new Registry();
  root: HTMLElement;
  screen: HTMLElement | null = null;
  kb = new KeyboardState(window);
  audio = new WebAudioSink();
  session: FightSession | null = null;
  touch: TouchControls | null = null;
  canvas: HTMLCanvasElement;
  mode: Mode = 'cpu';
  aiLevel = 4;
  sel: { p1: CharEntry | null; p2: CharEntry | null; stage: StageEntry | null } = { p1: null, p2: null, stage: null };
  arcade: { ladder: CharEntry[]; idx: number } | null = null;
  touchEnabled: boolean;
  navHandler: ((e: KeyboardEvent) => void) | null = null;

  constructor(root: HTMLElement) {
    this.root = root;
    this.canvas = document.createElement('canvas');
    this.canvas.className = 'game';
    const params = new URLSearchParams(location.search);
    this.touchEnabled = params.get('touch') === '1' || (params.get('touch') !== '0' && ('ontouchstart' in window || navigator.maxTouchPoints > 0));
    window.addEventListener('keydown', (e) => this.navHandler?.(e));
    (window as any).__ume = this;
  }

  async init() {
    await this.reg.loadStored();
    const p = new URLSearchParams(location.search);
    if (p.get('auto') === 'fight') {
      const find = (id: string | null, i: number) => this.reg.chars.find((c) => c.id === id) ?? this.reg.chars[i % this.reg.chars.length];
      this.mode = (p.get('mode') as Mode) || 'watch';
      this.sel.p1 = find(p.get('p1'), 0);
      this.sel.p2 = find(p.get('p2'), 1);
      this.sel.stage = this.reg.stages.find((s) => s.id === p.get('stage')) ?? this.reg.stages[0];
      this.startFight();
      return;
    }
    const s = p.get('screen');
    if (s === 'select') { this.mode = 'cpu'; this.showSelect(); return; }
    if (s === 'menu') { this.showMenu(); return; }
    if (s === 'import') { this.showImport(); return; }
    this.showTitle();
  }

  setScreen(el: HTMLElement) {
    this.screen?.remove();
    this.screen = el;
    this.root.appendChild(el);
    this.navHandler = null;
  }

  // ------------------------------------------------------------ title & menu
  showTitle() {
    const el = h(`<div class="screen title">
      <div class="title-bg"></div>
      <div class="logo"><span class="l1">ULTIMATE</span><span class="l2">MUGEN</span><span class="l3">ENGINE · NEXT GEN</span></div>
      <div class="press">${this.touchEnabled ? 'TOQUE PARA COMEÇAR' : 'PRESSIONE ENTER'}</div>
      <div class="foot">v0.1 · Web · Desktop · Mobile · Compatível com personagens M.U.G.E.N</div>
    </div>`);
    this.setScreen(el);
    const go = () => { this.audio.ensure(); this.audio.synth('confirm'); this.showMenu(); };
    el.addEventListener('pointerdown', go);
    this.navHandler = (e) => { if (e.code === 'Enter' || e.code === 'Space' || e.code === 'KeyJ') go(); };
  }

  showMenu() {
    const items: [string, string, () => void][] = [
      ['Arcade', 'Enfrente uma sequência de oponentes', () => { this.mode = 'arcade'; this.showSelect(); }],
      ['Versus CPU', 'Você contra o computador', () => { this.mode = 'cpu'; this.showSelect(); }],
      ['2 Jogadores', 'Mesmo teclado/controles', () => { this.mode = 'versus'; this.showSelect(); }],
      ['Treino', 'Boneco de treino, barra infinita', () => { this.mode = 'training'; this.showSelect(); }],
      ['CPU vs CPU', 'Assista a IA lutar', () => { this.mode = 'watch'; this.showSelect(); }],
      ['Importar MUGEN', 'Adicione personagens e cenários .zip', () => this.showImport()],
      ['Criar Personagem', 'Formato UMF simplificado', () => this.showCreator()],
      ['Controles', 'Teclado, controle e toque', () => this.showControls()],
    ];
    const el = h(`<div class="screen menu">
      <div class="title-bg dim"></div>
      <div class="menu-head"><div class="logo small"><span class="l1">ULTIMATE</span><span class="l2">MUGEN</span></div></div>
      <div class="menu-list">${items.map(([t, d], i) => `<button class="mi" data-i="${i}"><span class="mt">${t}</span><span class="md">${d}</span></button>`).join('')}</div>
      <div class="menu-side">
        <div class="stat"><b>${this.reg.chars.length}</b> lutadores</div>
        <div class="stat"><b>${this.reg.stages.length}</b> cenários</div>
        <div class="stat"><b>${this.reg.chars.filter((c) => c.kind === 'mugen').length}</b> MUGEN importados</div>
        <label class="diff">Dificuldade CPU <input type="range" min="1" max="8" value="${this.aiLevel}"><span>${this.aiLevel}</span></label>
      </div>
    </div>`);
    this.setScreen(el);
    const btns = Array.from(el.querySelectorAll<HTMLButtonElement>('.mi'));
    btns.forEach((b, i) => b.addEventListener('click', () => { this.audio.synth('confirm'); items[i][2](); }));
    const range = el.querySelector('input') as HTMLInputElement;
    range.addEventListener('input', () => { this.aiLevel = +range.value; (el.querySelector('.diff span') as HTMLElement).textContent = range.value; });
    let cur = 0;
    const focus = () => btns.forEach((b, i) => b.classList.toggle('focus', i === cur));
    focus();
    btns.forEach((b, i) => b.addEventListener('pointerenter', () => { cur = i; focus(); }));
    this.navHandler = (e) => {
      if (e.code === 'ArrowDown' || e.code === 'KeyS') { cur = (cur + 1) % btns.length; this.audio.synth('select'); focus(); }
      else if (e.code === 'ArrowUp' || e.code === 'KeyW') { cur = (cur + btns.length - 1) % btns.length; this.audio.synth('select'); focus(); }
      else if (e.code === 'Enter' || e.code === 'KeyJ' || e.code === 'Space') btns[cur].click();
      else if (e.code === 'Escape') this.showTitle();
    };
  }

  // ------------------------------------------------------------ character select
  showSelect() {
    const chars = this.reg.chars;
    const el = h(`<div class="screen select">
      <div class="title-bg dim"></div>
      <div class="sel-top"><button class="back">‹ Voltar</button><div class="sel-mode">${MODE_LABEL[this.mode]}</div><div class="sel-hint">Escolha seu lutador</div></div>
      <div class="sel-main">
        <div class="preview p1"><canvas width="360" height="420"></canvas><div class="pv-name"></div><div class="pv-series"></div></div>
        <div class="grid">${chars.map((c, i) => `<button class="card" data-i="${i}" style="--c:${c.color}"><canvas width="160" height="160"></canvas><span>${esc(c.name)}</span>${c.kind === 'mugen' ? '<i>MUGEN</i>' : ''}</button>`).join('')}
          <button class="card import-card"><div class="plus">+</div><span>Importar</span></button></div>
        <div class="preview p2"><canvas width="360" height="420"></canvas><div class="pv-name"></div><div class="pv-series"></div></div>
      </div>
      <div class="sel-desc"></div>
    </div>`);
    this.setScreen(el);
    const cards = Array.from(el.querySelectorAll<HTMLButtonElement>('.card:not(.import-card)'));
    cards.forEach((card, i) => {
      const cv = card.querySelector('canvas')!;
      const p = chars[i].portrait();
      if (p) cv.getContext('2d')!.drawImage(p, 0, 0, 160, 160);
    });
    (el.querySelector('.import-card') as HTMLElement).addEventListener('click', () => this.showImport());
    (el.querySelector('.back') as HTMLElement).addEventListener('click', () => this.showMenu());
    const twoHumans = this.mode === 'versus';
    const cursors = [0, Math.min(1, chars.length - 1)];
    let picking = 0; // 0 = P1, 1 = P2
    const picked: (CharEntry | null)[] = [null, null];
    const cols = () => { const g = el.querySelector('.grid') as HTMLElement; const w = cards[0]?.offsetWidth || 90; return Math.max(1, Math.floor((g.clientWidth + 8) / (w + 8))); };
    const previews = [this.makePreview(el.querySelector('.preview.p1 canvas')!, false), this.makePreview(el.querySelector('.preview.p2 canvas')!, true)];
    const refresh = () => {
      cards.forEach((c, i) => { c.classList.toggle('c1', cursors[0] === i && (picking === 0 || !!picked[0])); c.classList.toggle('c2', cursors[1] === i && picking === 1); });
      for (let s = 0; s < 2; s++) {
        const pe = el.querySelector(`.preview.p${s + 1}`) as HTMLElement;
        const c = picked[s] ?? (s === picking ? chars[cursors[s]] : null);
        pe.classList.toggle('locked', !!picked[s]);
        (pe.querySelector('.pv-name') as HTMLElement).textContent = c ? c.name : (s === 1 ? '???' : '');
        (pe.querySelector('.pv-series') as HTMLElement).textContent = c ? `${c.series} · ${c.author}` : '';
        previews[s].set(c, s === 1 && picked[0]?.id === c?.id ? 2 : 1);
      }
      const c = chars[cursors[picking]];
      (el.querySelector('.sel-desc') as HTMLElement).textContent = c?.description ?? (c?.kind === 'mugen' ? 'Personagem M.U.G.E.N importado' : '');
      (el.querySelector('.sel-hint') as HTMLElement).textContent = picking === 0 ? 'Jogador 1: escolha seu lutador' : (twoHumans ? 'Jogador 2: escolha' : 'Escolha o oponente');
    };
    const confirm = () => {
      const c = chars[cursors[picking]];
      this.audio.synth('confirm');
      picked[picking] = c;
      if (picking === 0 && this.mode === 'arcade') {
        this.sel.p1 = c;
        const others = chars.filter((x) => x.id !== c.id && x.kind === 'native').sort(() => Math.random() - 0.5);
        this.arcade = { ladder: others.slice(0, 6), idx: 0 };
        this.sel.p2 = this.arcade.ladder[0] ?? c;
        previews.forEach((p) => p.stop());
        this.showStageSelect();
        return;
      }
      if (picking === 0) { picking = 1; refresh(); return; }
      this.sel.p1 = picked[0]; this.sel.p2 = picked[1];
      previews.forEach((p) => p.stop());
      this.showStageSelect();
    };
    cards.forEach((card, i) => {
      card.addEventListener('click', () => { if (cursors[picking] === i) confirm(); else { cursors[picking] = i; this.audio.synth('select'); refresh(); } });
      card.addEventListener('dblclick', () => { cursors[picking] = i; confirm(); });
    });
    this.navHandler = (e) => {
      const n = cards.length, cc = cols();
      const mv = (d: number) => { cursors[picking] = (cursors[picking] + d + n) % n; this.audio.synth('select'); refresh(); };
      const p2keys = twoHumans && picking === 1;
      const k = e.code;
      if (k === (p2keys ? 'ArrowRight' : 'KeyD') || (!twoHumans && k === 'ArrowRight')) mv(1);
      else if (k === (p2keys ? 'ArrowLeft' : 'KeyA') || (!twoHumans && k === 'ArrowLeft')) mv(-1);
      else if (k === (p2keys ? 'ArrowDown' : 'KeyS') || (!twoHumans && k === 'ArrowDown')) mv(cc);
      else if (k === (p2keys ? 'ArrowUp' : 'KeyW') || (!twoHumans && k === 'ArrowUp')) mv(-cc);
      else if (k === 'Enter' || k === 'KeyJ' || k === 'KeyU' || k === 'Space' || k === 'Numpad1' || k === 'Numpad4') confirm();
      else if (k === 'Escape' || k === 'Backspace') {
        if (picking === 1) { picking = 0; picked[0] = null; refresh(); } else { previews.forEach((p) => p.stop()); this.showMenu(); }
      }
    };
    refresh();
  }

  /** Animated idle preview on a canvas. */
  makePreview(cv: HTMLCanvasElement, mirror: boolean) {
    const ctx = cv.getContext('2d')!;
    let cur: { c: CharEntry; pal: number } | null = null;
    let t = 0, raf = 0;
    const draw = () => {
      raf = requestAnimationFrame(draw);
      ctx.clearRect(0, 0, cv.width, cv.height);
      if (!cur) return;
      let d;
      try { d = cur.c.load(cur.pal); } catch { return; }
      const a = d.anims.get(cur.c.kind === 'native' ? 181 : 0) && t % 600 > 400 ? d.anims.get(181)! : d.anims.get(0);
      if (!a || !a.frames.length) return;
      const tot = a.totalTime > 0 ? a.totalTime : 1;
      let tm = Math.floor(t / 1) % tot, idx = 0, acc = 0;
      for (let i = 0; i < a.frames.length; i++) { const ft = a.frames[i].time; if (ft < 0 || tm < acc + ft) { idx = i; break; } acc += ft; idx = i; }
      const fr = a.frames[idx];
      const spr = d.sprites.get(`${fr.group},${fr.index}`);
      if (!spr?.image) return;
      const s = (spr.scale ?? 1) * (320 / d.localW);
      const k = 2.9 * (d.source === 'mugen' ? Math.min(1, 110 / Math.max(60, spr.axisY * s)) : 1);
      ctx.save();
      ctx.translate(cv.width / 2, cv.height - 30);
      ctx.fillStyle = 'rgba(0,0,0,0.35)'; ctx.beginPath(); ctx.ellipse(0, 0, 60, 10, 0, 0, Math.PI * 2); ctx.fill();
      ctx.scale((mirror ? -1 : 1) * k * s, k * s);
      ctx.drawImage(spr.image, -spr.axisX + fr.x / (spr.scale ?? 1), -spr.axisY + fr.y / (spr.scale ?? 1), spr.width, spr.height);
      ctx.restore();
      t++;
    };
    raf = requestAnimationFrame(draw);
    return { set: (c: CharEntry | null, pal: number) => { if (cur?.c !== c || cur?.pal !== pal) { cur = c ? { c, pal } : null; t = 0; } }, stop: () => cancelAnimationFrame(raf) };
  }

  // ------------------------------------------------------------ stage select
  showStageSelect() {
    const stages = this.reg.stages;
    const el = h(`<div class="screen stages">
      <div class="title-bg dim"></div>
      <div class="sel-top"><button class="back">‹ Voltar</button><div class="sel-mode">Escolha o cenário</div></div>
      <div class="stage-grid">
        <button class="stage-card" data-i="-1" style="background:conic-gradient(#ff2fd0,#22e8ff,#ffe23a,#ff2fd0)"><span>Aleatório</span></button>
        ${stages.map((s, i) => `<button class="stage-card" data-i="${i}" style="background:${s.thumb}"><span>${esc(s.name)}</span>${s.kind === 'mugen' ? '<i>MUGEN</i>' : ''}</button>`).join('')}
      </div>
    </div>`);
    this.setScreen(el);
    (el.querySelector('.back') as HTMLElement).addEventListener('click', () => this.showSelect());
    const cards = Array.from(el.querySelectorAll<HTMLButtonElement>('.stage-card'));
    let cur = 0;
    const pick = (i: number) => { this.audio.synth('confirm'); this.sel.stage = i < 0 ? stages[Math.floor(Math.random() * stages.length)] : stages[i]; this.showVs(); };
    const focus = () => cards.forEach((c, i) => c.classList.toggle('focus', i === cur));
    cards.forEach((c, i) => c.addEventListener('click', () => pick(i - 1)));
    focus();
    this.navHandler = (e) => {
      if (['ArrowRight', 'KeyD'].includes(e.code)) { cur = (cur + 1) % cards.length; focus(); }
      else if (['ArrowLeft', 'KeyA'].includes(e.code)) { cur = (cur + cards.length - 1) % cards.length; focus(); }
      else if (['ArrowDown', 'KeyS'].includes(e.code)) { cur = Math.min(cards.length - 1, cur + 3); focus(); }
      else if (['ArrowUp', 'KeyW'].includes(e.code)) { cur = Math.max(0, cur - 3); focus(); }
      else if (['Enter', 'KeyJ', 'Space'].includes(e.code)) pick(cur - 1);
      else if (e.code === 'Escape') this.showSelect();
    };
  }

  showVs() {
    const { p1, p2 } = this.sel;
    if (!p1 || !p2) return;
    const el = h(`<div class="screen vs">
      <div class="vs-side l" style="--c:${p1.color}"><canvas width="300" height="300"></canvas><div class="vs-name">${esc(p1.name)}</div><div class="vs-series">${esc(p1.series)}</div></div>
      <div class="vs-mid">VS</div>
      <div class="vs-side r" style="--c:${p2.color}"><canvas width="300" height="300"></canvas><div class="vs-name">${esc(p2.name)}</div><div class="vs-series">${esc(p2.series)}</div></div>
      ${this.arcade ? `<div class="vs-stage">Arcade · Luta ${this.arcade.idx + 1}/${this.arcade.ladder.length}</div>` : `<div class="vs-stage">${esc(this.sel.stage?.name ?? '')}</div>`}
    </div>`);
    this.setScreen(el);
    const cvs = el.querySelectorAll('canvas');
    const pa = p1.portrait(), pb = p2.portrait();
    if (pa) cvs[0].getContext('2d')!.drawImage(pa, 0, 0, 300, 300);
    if (pb) { const c = cvs[1].getContext('2d')!; c.translate(300, 0); c.scale(-1, 1); c.drawImage(pb, 0, 0, 300, 300); }
    this.audio.synth('super');
    const t = setTimeout(() => this.startFight(), 2200);
    const skip = () => { clearTimeout(t); this.startFight(); };
    el.addEventListener('pointerdown', skip);
    this.navHandler = (e) => { if (e.code === 'Enter' || e.code === 'Space') skip(); };
  }

  // ------------------------------------------------------------ fight
  inputs(): { p1: InputSource | null; p2: InputSource | null } {
    const duo = this.mode === 'versus';
    const p1Sources: InputSource[] = [new KeyboardSource(this.kb, duo ? P1_DUO : P1_SOLO), new GamepadSource(0)];
    if (this.touchEnabled && this.touch) p1Sources.push(this.touch.src);
    const p1 = this.mode === 'watch' ? null : new MergedSource(p1Sources);
    const p2 = duo ? new MergedSource([new KeyboardSource(this.kb, P2_DUO), new GamepadSource(1)]) : null;
    return { p1, p2 };
  }

  startFight() {
    const { p1, p2, stage } = this.sel;
    if (!p1 || !p2 || !stage) return;
    const el = h(`<div class="screen fight"><button class="pause-btn" aria-label="Pausar">❚❚</button></div>`);
    this.setScreen(el);
    el.prepend(this.canvas);
    if (this.touchEnabled && this.mode !== 'watch') {
      this.touch = new TouchControls(el);
    } else this.touch = null;
    const inp = this.inputs();
    const level = this.mode === 'arcade' && this.arcade ? Math.min(8, 2 + this.arcade.idx + Math.floor(this.aiLevel / 3)) : this.aiLevel;
    this.session?.stop();
    const s = new FightSession(this.canvas, { p1, p2, stage, p1Input: inp.p1, p2Input: inp.p2, aiLevel: level, training: this.mode === 'training' }, this.audio);
    this.session = s;
    if (this.touch) {
      const t = this.touch;
      t.getFacing = () => s.match.roots[0]?.facing ?? 1;
      t.isMugen = () => s.match.roots[0]?.def.source === 'mugen';
    }
    s.debug = new URLSearchParams(location.search).get('debug') === '1';
    s.onMatchOver = (w) => this.showResults(w);
    s.start();
    (el.querySelector('.pause-btn') as HTMLElement).addEventListener('click', () => this.pause());
    this.navHandler = (e) => {
      if (e.code === 'Escape' || e.code === 'KeyP') this.pause();
      if (e.code === 'F2') s.debug = !s.debug;
    };
  }

  pause() {
    const s = this.session;
    if (!s || s.paused) return;
    s.paused = true;
    const f = s.match.roots[0];
    const moves = f?.def.movelist ?? [];
    const ov = h(`<div class="overlay pause">
      <div class="panel">
        <h2>PAUSA</h2>
        <button data-a="resume">Continuar</button>
        <button data-a="boxes">Hitboxes: ${s.debug ? 'ON' : 'OFF'}</button>
        <button data-a="restart">Reiniciar luta</button>
        <button data-a="select">Trocar personagens</button>
        <button data-a="menu">Menu principal</button>
        <div class="movelist"><h3>Golpes — ${esc(f?.def.displayName ?? '')}</h3>${moves.length ? moves.map((m) => `<div class="mv"><span>${esc(m.name)}</span><code>${esc(prettyInput(m.input))}</code></div>`).join('') : '<div class="mv">Lista de golpes indisponível (MUGEN): use os comandos do .cmd</div>'}</div>
      </div></div>`);
    this.screen!.appendChild(ov);
    const close = () => { ov.remove(); s.paused = false; this.navHandler = prev; };
    const prev = this.navHandler;
    ov.querySelectorAll<HTMLButtonElement>('button').forEach((b) => b.addEventListener('click', () => {
      const a = b.dataset.a;
      if (a === 'resume') close();
      else if (a === 'boxes') { s.debug = !s.debug; b.textContent = `Hitboxes: ${s.debug ? 'ON' : 'OFF'}`; }
      else if (a === 'restart') { ov.remove(); this.startFight(); }
      else if (a === 'select') { s.stop(); this.showSelect(); }
      else if (a === 'menu') { s.stop(); this.showMenu(); }
    }));
    this.navHandler = (e) => { if (e.code === 'Escape' || e.code === 'KeyP') close(); };
  }

  showResults(winnerSide: number) {
    const s = this.session;
    if (!s) return;
    const w = winnerSide === 0 ? this.sel.p1! : this.sel.p2!;
    const humanWon = winnerSide === 0 || this.mode === 'versus';
    if (this.mode === 'arcade' && this.arcade && winnerSide === 0) {
      this.arcade.idx++;
      if (this.arcade.idx < this.arcade.ladder.length) {
        s.stop();
        this.sel.p2 = this.arcade.ladder[this.arcade.idx];
        this.sel.stage = this.reg.stages[Math.floor(Math.random() * this.reg.stages.length)];
        this.showVs();
        return;
      }
    }
    const done = this.mode === 'arcade' && this.arcade && this.arcade.idx >= this.arcade.ladder.length;
    const ov = h(`<div class="overlay results">
      <div class="panel wide" style="--c:${w.color}">
        <canvas width="220" height="220"></canvas>
        <div class="res-txt"><div class="res-win">${done ? 'ARCADE COMPLETO!' : `${esc(w.name)} VENCE!`}</div><div class="res-sub">${this.mode === 'arcade' && !humanWon ? 'Fim de jogo' : ''}</div>
        <div class="res-btns"><button data-a="again">Revanche</button><button data-a="select">Personagens</button><button data-a="menu">Menu</button></div></div>
      </div></div>`);
    this.screen!.appendChild(ov);
    const p = w.portrait(); if (p) (ov.querySelector('canvas') as HTMLCanvasElement).getContext('2d')!.drawImage(p, 0, 0, 220, 220);
    ov.querySelectorAll<HTMLButtonElement>('button').forEach((b) => b.addEventListener('click', () => {
      s.stop();
      if (b.dataset.a === 'again') { if (this.arcade) this.arcade.idx = Math.min(this.arcade.idx, this.arcade.ladder.length - 1); this.startFight(); }
      else if (b.dataset.a === 'select') this.showSelect(); else this.showMenu();
    }));
    this.navHandler = (e) => { if (e.code === 'Enter') (ov.querySelector('button') as HTMLButtonElement).click(); };
  }

  // ------------------------------------------------------------ import / creator / controls
  showImport() {
    const el = h(`<div class="screen info">
      <div class="title-bg dim"></div>
      <div class="sel-top"><button class="back">‹ Voltar</button><div class="sel-mode">Importar personagens e cenários M.U.G.E.N</div></div>
      <div class="info-body">
        <div class="drop"><div class="drop-icon">⇪</div><div><b>Arraste aqui</b> arquivos <code>.zip</code> ou a pasta do personagem</div>
          <div class="drop-btns"><label class="btn">Escolher .zip / arquivos<input type="file" multiple accept=".zip,.def,.sff,.air,.cmd,.cns,.snd,.act,.st" hidden class="f1"></label>
          <label class="btn">Escolher pasta<input type="file" webkitdirectory hidden class="f2"></label></div>
        </div>
        <div class="log"></div>
        <div class="tips">
          <h3>Compatibilidade</h3>
          <p>O motor lê nativamente os formatos do M.U.G.E.N 1.0/1.1: <b>.def</b>, <b>.sff</b> (v1 PCX e v2 com RLE8/RLE5/LZ5/PNG), <b>.air</b> (Clsn1/Clsn2), <b>.cmd</b>, <b>.cns</b> (Statedefs, triggers e ~90 state controllers), <b>.snd</b> e paletas <b>.act</b>. Cenários (stages) .def/.sff também.</p>
          <p>Os personagens ficam salvos no navegador (IndexedDB) e aparecem na seleção. Funciona offline como app (PWA).</p>
          <p class="warn">Importe apenas personagens que você tem direito de usar. Personagens de anime/jogos feitos por fãs pertencem aos seus autores e detentores dos direitos.</p>
        </div>
      </div></div>`);
    this.setScreen(el);
    (el.querySelector('.back') as HTMLElement).addEventListener('click', () => this.showMenu());
    const log = el.querySelector('.log') as HTMLElement;
    const handle = async (files: { name: string; data: Uint8Array }[]) => {
      if (!files.length) return;
      log.innerHTML = `<div>Processando ${files.length} arquivo(s)…</div>`;
      try {
        const r = await this.reg.importFiles(files, files[0].name);
        log.innerHTML = `${r.chars.map((c) => `<div class="ok">✔ Personagem: ${esc(c)}</div>`).join('')}${r.stages.map((c) => `<div class="ok">✔ Cenário: ${esc(c)}</div>`).join('')}${r.errors.map((c) => `<div class="err">✖ ${esc(c)}</div>`).join('')}${!r.chars.length && !r.stages.length && !r.errors.length ? '<div class="err">Nenhum personagem (.def com [Files]) encontrado.</div>' : ''}`;
      } catch (e) { log.innerHTML = `<div class="err">Erro: ${esc((e as Error).message)}</div>`; }
    };
    const read = async (list: FileList | File[]) => Promise.all(Array.from(list).map(async (f) => ({ name: (f as any).webkitRelativePath || f.name, data: new Uint8Array(await f.arrayBuffer()) })));
    (el.querySelector('.f1') as HTMLInputElement).addEventListener('change', async (e) => handle(await read((e.target as HTMLInputElement).files!)));
    (el.querySelector('.f2') as HTMLInputElement).addEventListener('change', async (e) => handle(await read((e.target as HTMLInputElement).files!)));
    const drop = el.querySelector('.drop') as HTMLElement;
    drop.addEventListener('dragover', (e) => { e.preventDefault(); drop.classList.add('over'); });
    drop.addEventListener('dragleave', () => drop.classList.remove('over'));
    drop.addEventListener('drop', async (e) => {
      e.preventDefault(); drop.classList.remove('over');
      const out: { name: string; data: Uint8Array }[] = [];
      const items = Array.from(e.dataTransfer?.items ?? []);
      const walk = async (entry: any, path: string): Promise<void> => {
        if (entry.isFile) { const f: File = await new Promise((r) => entry.file(r)); out.push({ name: path + f.name, data: new Uint8Array(await f.arrayBuffer()) }); }
        else if (entry.isDirectory) { const rd = entry.createReader(); const ents: any[] = await new Promise((r) => rd.readEntries(r)); for (const en of ents) await walk(en, path + entry.name + '/'); }
      };
      for (const it of items) { const en = (it as any).webkitGetAsEntry?.(); if (en) await walk(en, ''); }
      if (!out.length && e.dataTransfer?.files) out.push(...await read(e.dataTransfer.files));
      handle(out);
    });
    this.navHandler = (e) => { if (e.code === 'Escape') this.showMenu(); };
  }

  showCreator() {
    const el = h(`<div class="screen info">
      <div class="title-bg dim"></div>
      <div class="sel-top"><button class="back">‹ Voltar</button><div class="sel-mode">Criar personagem — formato UMF</div></div>
      <div class="info-body doc">
        <p>Um personagem inteiro é <b>um único arquivo</b> JSON/TypeScript: atributos, arte (esqueleto procedural ou spritesheet), animações nomeadas e uma lista de golpes. O motor compila isso para a mesma máquina de estados usada pelos personagens MUGEN — sem precisar escrever CNS.</p>
<pre>{
  "format": "umf/1", "id": "kaze", "name": "Kaze",
  "stats": { "life": 1000, "walk": [2.6, 2.2], "jump": [-8.6, -2.6, 0, 2.6] },
  "anims": { "idle": { "frames": [{ "pose": "idle1", "ticks": 8 }, { "pose": "idle2", "ticks": 8 }] }, ... },
  "moves": [
    { "id": "jab", "name": "Soco Rápido", "input": "x", "kind": "normal", "anim": "jab",
      "hits": [{ "at": 2, "damage": 30, "level": "light" }] },
    { "id": "hadou", "name": "Onda de Vento", "input": "~D, DF, F, x", "kind": "special", "simple": "sp1",
      "anim": "fireball", "projectile": { "at": 3, "anim": "windball", "speed": [5, 0], "offset": [40, -60], "damage": 70 } },
    { "id": "super", "name": "Tempestade", "input": "~D, DF, F, D, DF, F, x", "kind": "super", "cost": 1000, ... }
  ]
}</pre>
        <ul>
          <li><b>Hitboxes automáticas</b>: a caixa de dano vem da parte do corpo ativa no quadro (<code>hit: { part: "handR" }</code>); as hurtboxes são calculadas do esqueleto.</li>
          <li><b>Comandos</b> usam a sintaxe MUGEN (<code>~D, DF, F, x</code>) e ganham automaticamente um <b>botão simples</b> no celular (<code>simple: "sp1"</code>).</li>
          <li><b>Cancelamentos</b>, custo de barra, super pause, invencibilidade, projéteis e teleporte são parâmetros declarativos.</li>
          <li>Personagens MUGEN continuam funcionando lado a lado — mesma VM, mesma colisão.</li>
        </ul>
        <p>Veja <code>docs/CHARACTER_FORMAT.md</code> e os 10 lutadores em <code>src/chars/</code> como exemplos.</p>
      </div></div>`);
    this.setScreen(el);
    (el.querySelector('.back') as HTMLElement).addEventListener('click', () => this.showMenu());
    this.navHandler = (e) => { if (e.code === 'Escape') this.showMenu(); };
  }

  showControls() {
    const el = h(`<div class="screen info">
      <div class="title-bg dim"></div>
      <div class="sel-top"><button class="back">‹ Voltar</button><div class="sel-mode">Controles</div></div>
      <div class="info-body doc cols">
        <div><h3>Teclado (1 jogador)</h3><p>Mover: <kbd>WASD</kbd> ou <kbd>Setas</kbd><br>Socos: <kbd>U</kbd> fraco · <kbd>I</kbd> forte · <kbd>O</kbd><br>Chutes: <kbd>J</kbd> fraco · <kbd>K</kbd> forte · <kbd>L</kbd><br>Especiais simples: <kbd>1</kbd> <kbd>2</kbd> <kbd>3</kbd> · Super: <kbd>1</kbd>+<kbd>2</kbd><br>Pausa: <kbd>Esc</kbd> · Hitboxes: <kbd>F2</kbd></p></div>
        <div><h3>2 Jogadores</h3><p>P1: <kbd>WASD</kbd> + <kbd>U I O</kbd> / <kbd>J K L</kbd><br>P2: <kbd>Setas</kbd> + <kbd>Num 4 5 6</kbd> / <kbd>Num 1 2 3</kbd></p></div>
        <div><h3>Controle (gamepad)</h3><p>Padrão Xbox/PlayStation: ✕/A chute fraco, ○/B chute forte, □/X soco fraco, △/Y soco forte, R1 soco 3, R2 chute 3.</p></div>
        <div><h3>Celular / Tablet</h3><p>Analógico virtual à esquerda, botões LP/HP/LK/HK à direita e <b>SP1–SP3 / SUPER</b>: especiais com um toque (modo simples), funcionam também com personagens MUGEN.</p></div>
      </div></div>`);
    this.setScreen(el);
    (el.querySelector('.back') as HTMLElement).addEventListener('click', () => this.showMenu());
    this.navHandler = (e) => { if (e.code === 'Escape') this.showMenu(); };
  }
}

function prettyInput(s: string) {
  return s.replace(/~/g, '').replace(/\s/g, '').split(',').map((t) => ({ D: '↓', U: '↑', F: '→', B: '←', DF: '↘', DB: '↙', UF: '↗', UB: '↖' } as Record<string, string>)[t] ?? t.replace('x', 'LP').replace('y', 'HP').replace('a', 'LK').replace('b', 'HK')).join(' ');
}
