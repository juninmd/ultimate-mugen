# Controle virtual (web, celulares e tablets)

A versão web mostra um gamepad na tela em todo aparelho com toque: analógico à esquerda, oito botões de ataque (`X Y Z ESP` / `A B C SUPER`) e uma fileira com `VOLTAR`, `OK`, `DESP.` e `II` (pausa) à direita. Ele é desenhado com formas simples de HTML e CSS (sem sprites) em `web/vpad.js`, acima do canvas, e fica visível em todas as telas: título, menus, seleção, VS, luta, pausa e história.

## Como funciona

- Cada dedo é rastreado pelo `pointerId`: o analógico e vários botões podem ser segurados ao mesmo tempo, e um dedo pode deslizar de um botão de ataque para outro.
- O próprio navegador decide qual botão foi tocado, então tamanho do canvas, densidade de pixels e escala de render não mudam onde o toque cai.
- O motor lê o controle uma vez por quadro com `ume_vpad_poll()` (bits da luta) e `ume_vpad_nav()` (`OK`/`VOLTAR`/`START` dos menus). O analógico e `A`/`B` também navegam nos menus, como um gamepad físico.
- Um toque mais curto que um quadro ainda vale por dois quadros.
- Os dedos são soltos em `pointerup`, `pointercancel`, `lostpointercapture`, perda de foco, aba escondida e `pagehide`. Um toque longo (`contextmenu`) não solta nada.
- O controle liga quando o aparelho informa toque (`maxTouchPoints`, `ontouchstart`, `pointer: coarse`) ou no primeiro toque, e nada o esconde depois. `?touch=1` força no desktop.

## Depurar num celular de verdade

Abra o jogo com `?debug=1`. Uma caixa no canto superior esquerdo mostra se o controle está ligado, a densidade de pixels, a viewport, os dedos segurados, os bits enviados ao motor e as contagens de `pointerdown`, `pointerup` e `pointercancel`.

## Testes

`node scripts/web-vpad-smoke.js web/dist` roda a build web real no Chromium com multitoque real (`Input.dispatchTouchEvent`): confere o controle visível e sem obstrução em cada tela, navega menu, seleção e cenários só pelo toque, anda, pula, ataca segurando o analógico, segura três dedos ao mesmo tempo, solta um dedo sem perder o analógico, pausa e volta. `VPAD_DEVICE` escolhe o aparelho (padrão `Pixel 7 landscape`; a CI também roda `Pixel 7` em retrato e `iPhone 13 landscape`). `VPAD_SHOTS=<pasta>` salva prints.
