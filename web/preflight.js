(function () {
  const TEXTS = {
    pt: { nowasm: 'Este navegador não permite WebAssembly (modo bloqueio do iOS, navegador antigo ou modo de economia). Abra no Chrome ou Safari atualizado, sem o modo de bloqueio.', nogl: 'Este navegador não ofereceu WebGL2, que o jogo precisa. Ative a aceleração de hardware ou atualize o navegador.' },
    es: { nowasm: 'Este navegador no permite WebAssembly (modo de bloqueo de iOS, navegador antiguo o ahorro de energía). Ábrelo en Chrome o Safari actualizado, sin el modo de bloqueo.', nogl: 'Este navegador no ofreció WebGL2, que el juego necesita. Activa la aceleración por hardware o actualiza el navegador.' },
    en: { nowasm: 'This browser does not allow WebAssembly (iOS Lockdown Mode, an old browser or a battery saver). Open it in an up-to-date Chrome or Safari without Lockdown Mode.', nogl: 'This browser did not provide WebGL2, which the game needs. Turn on hardware acceleration or update the browser.' },
  };
  function hasWebGL2() {
    try {
      const c = document.createElement('canvas');
      const gl = c.getContext('webgl2');
      if (!gl) return false;
      const lose = gl.getExtension('WEBGL_lose_context');
      if (lose) lose.loseContext();
      return true;
    } catch (e) { return false; }
  }
  window.umePreflight = function (lang) {
    const t = TEXTS[(lang || 'en').slice(0, 2)] || TEXTS.en;
    if (typeof WebAssembly !== 'object' || typeof WebAssembly.instantiate !== 'function') return t.nowasm;
    if (!hasWebGL2()) return t.nogl;
    return null;
  };
})();
