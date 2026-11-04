# Testes de estresse e resiliência

## Cobertura

| Área | Teste | Comando |
|---|---|---|
| Parsers (SFF, AIR, INI/DEF/CNS/CMD, SND, ACT, PCX, PNG, zip, stage) | Fuzzer determinístico por mutação, usando o zip do kfm como semente. Falha em qualquer panic ou entrada acima de 1,5 s. | `UME_FUZZ_ITERS=4000 cargo test -p ume-formats --release --test fuzz` |
| Spam de Explod, Projectile e Helper a cada tick | Custo do tick continua limitado | `cargo test -p ume-engine --release --test stress -- --nocapture` |
| Caminhos quentes | Bench embutido: SFF, AIR, CNS, carga de personagem, tick com 2, 6 e 10 lutadores | `cargo run -p ume-engine --release --example hotpath_bench` |
| Rollback com latência | `crates/netplay/tests/rollback.rs` | `cargo test -p ume-netplay` |

## Achados e correções

- Um SFF corrompido podia declarar sprite 65535x65535: de 3 a 48 s e gigabytes de RAM. Sprites e PNGs acima de 16M pixels (`MAX_SPRITE_PIXELS`) agora viram sprite vazio.
- Explods e projéteis não tinham limite. Com Helpers criando coisas a cada tick, 240 ticks levavam 72 s, com custo quadrático. Ambos agora têm teto de 256; novos além disso são descartados, como no MUGEN. 2000 ticks com 61 lutadores levam 1,2 s (cerca de 0,6 ms por tick).

## Números (kfm720, release)

| Caminho | Tempo |
|---|---|
| Decodificar SFF | 1,1 ms |
| Parse de AIR | 1,0 ms |
| Parse e compilação de CNS | 3,2 ms |
| Carga do personagem sem imagens | 7,6 a 10 ms |
| Tick com 2 lutadores | 5 us |
| Tick com 6 lutadores | 15 us |
| Tick com 10 lutadores | 39 us |

A simulação está muito abaixo dos 16,6 ms por quadro; carga e renderização são onde vale otimizar.

## Comparação e próximos passos

- O GGRS tem `SyncTestSession` (resimula todo quadro e compara checksums) e detecção de desync; rodar isso no CI numa luta roteirizada pegaria não-determinismo cedo.
- O Ikemen GO fez o rollback salvando e restaurando o estado completo; o custo é o tamanho do estado, por isso evitar alocações por tick (já testado em `tick_alloc`) importa.
- Ainda não coberto: perda de pacotes e jitter no harness de netplay (hoje só latência), recuperação de device-lost da GPU, pouca memória, fuzzer de CNS/expressões e stress carregando muitos personagens.
