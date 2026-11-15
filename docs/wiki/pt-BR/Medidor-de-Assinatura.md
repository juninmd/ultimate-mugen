# Medidor de Assinatura

[English](../Signature.md)

Cada lutador nativo pode declarar um bloco `signature` em `passives` no JSON. É um medidor próprio do lutador que muda o jeito de jogar, somado a fúria, acúmulos e enrage por tempo.

Chaves: `max`, `gainHit`, `gainSpecial`, `gainHurt`, `decayDelaySeconds`, `decayPerSecond`, `drainOnSuper`, `riskPerSecond` e `tiers` (até quatro degraus com `at`, `attack`, `defence` e `speed`). O formato é o mesmo da página em inglês.

O ataque continua limitado pelo teto global de [Passivos](../Passives.md). O medidor zera a cada round e aparece como uma faixa sob a barra de vida.

Lutadores atuais: **Kaito** (Tempestade, acumula e acelera) e **Volta** (Sobrecarga, forte com o medidor cheio, mas custa vida).
