# Rush Cancel

Todo lutador (nativo UMF e M.U.G.E.N) pode **cancelar um golpe conectado para o estado neutro** gastando meia barra de energia. É o prolongador de combos universal da engine: nenhum dado de personagem é necessário.

## Como usar

Aperte a tecla de energia (**R**, a entrada `s` / Start) enquanto o golpe em andamento acabou de acertar ou ser defendido.

| Regra | Valor |
|---|---|
| Custo | 500 de energia |
| Janela | 1 a 45 ticks depois do contato (`CONTACT_WINDOW`) |
| Buffer de entrada | O toque é lembrado por 6 ticks, então apertar um pouco antes ainda funciona |
| Recarga | 40 ticks entre cancelamentos |
| Condição | `movetype = A`, o golpe acertou ou foi defendido, o lutador está vivo e o round corre |
| Não vale | Golpe no vazio, helpers, lutador deitado, pausa de super |

A mesma tecla continua carregando energia parado: um lutador com controle e sem ataque em andamento carrega como antes. Depois de um cancelamento a tecla fica mascarada até ser solta uma vez, então segurá-la nunca vira carga.

## O que acontece

1. 500 de energia são gastos e o lutador vai para o estado neutro da sua postura (`0` em pé, `11` agachado, `50` no ar) com controle.
2. A pausa e o tremor de golpe do atacante são zerados e o contador de air dash é reabastecido, então o próximo golpe ou dash começa na hora.
3. Todo rival em estado de dano ou defesa ganha 8 ticks extras de stun (`VICTIM_STUN_BONUS`), o que permite a continuação conectar.
4. A partida entra em câmera lenta por 15 ticks, o lutador pisca em azul e deixa um rastro de afterimage.
5. O **Overdrive** começa: por 150 ticks o lutador causa 20 % a mais de dano (`effective_attack`, aplicado sobre os passivos e abaixo dos tetos deles).

## Whiff cancel

Apertar Start durante um golpe que não tocou em nada também cancela, por um preço maior. Custa 1000 de barra (o Rush Cancel custa 500), vale de 10 a 60 ticks depois do início do golpe e divide a recarga do Rush Cancel. Devolve o controle como o Rush Cancel, mas sem Overdrive, sem câmera lenta e sem contar como Rush Cancel. Golpe que conectou ou foi bloqueado continua usando o Rush Cancel mais barato. Constantes em `crates/engine/src/rush/whiff.rs`.

## Dash cancel

Um golpe normal que conectou (fraco a forte, em pé ou agachado) pode ser cancelado em dash para frente ou para trás tocando duas vezes a direção em até 20 ticks do contato, por 250 de barra. É mais barato que o Rush Cancel, não ativa Overdrive e não usa o cooldown. Exige contato (acerto ou defesa) e barra suficiente, então um normal que errou não pode ser cancelado. Código: `crates/engine/src/rush/dash.rs`.

## Defesa perfeita

Apertar para trás até 6 ticks antes do golpe acertar, sem já estar segurando a guarda, torna o bloqueio perfeito: sem dano de chip e com o atordoamento da guarda um terço menor. Segurar cedo é guarda normal, e soltar trava a janela por 12 ticks, então martelar o botão não funciona. Vale só para nativos no chão e não dá barra (barra por guarda sustentava o voo da Shion sem fim). Código: `crates/engine/src/rush/perfect.rs`.

## Mapa da engine

| Peça | Arquivo |
|---|---|
| Estado, buffer, constantes | `crates/engine/src/rush.rs` |
| O cancelamento (`Match::rush_step`, `rush_ready`) | `crates/engine/src/rush/cancel.rs` |
| Consulta da janela para o HUD (`Match::rush_window`: `None` sem cancelamento possível, senão a fração da janela que resta) | `crates/engine/src/rush/cancel.rs` |
| Marcador "RUSH CANCEL" com barra de janela decrescente sob a barra de vida | `src/hud/rush_marker.rs` (`UME_SHOTS_ONLY=rush` captura) |
| Evento para a apresentação | `Event::Rush { fighter, side, guarded }` em `crates/engine/src/matchs/types.rs` |
| Efeitos (anéis, brilho, faíscas, rótulo, brasas do Overdrive) | `src/vfx/rush.rs` |
| Checksum de rollback | `overdrive`, `cooldown` e `cancels` em `crates/netplay/src/checksum.rs` |
| Testes | `cargo test --release -p ume-engine --test rush_cancel` |

`rush_step` roda uma vez por tick de simulação depois da fase de entrada e não aloca (o rastro de afterimage só cresce quando há um cancelamento). Tudo é dado simples em `Fighter::rush`, então `Match` continua `Clone` e replays e rollback não mudam.
