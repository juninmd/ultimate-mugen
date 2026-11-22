# Akio — revisão visual nativa

## Objetivo

Uniformizar os quadros de combate que destoavam da aparência canônica de Akio e manter sua leitura visual consistente ao longo de defesa, dano, queda, recuperação, salto e rasteira.

## Decisões de arte

- `walk0.png` foi usado como âncora de identidade e paleta.
- 25 sprites foram refeitos depois de comparar o pacote completo em folhas de contato, em tamanho nativo e sobre fundos claros e escuros.
- Os quadros aprovados mantêm canvas transparente de 400×400, escala e linha dos pés do quadro original, com alpha binário.
- O retrato e `meta.json` permaneceram intactos. As poses de carga com aura foram preservadas porque o kit as declara como parte dos golpes.

As folhas `before/` e `after/` cobrem o conjunto inteiro de 123 PNGs de combate e permitem revisar identidade, silhueta e consistência entre poses. As capturas `runtime/` mostram o personagem compilado em luta no desktop e no navegador; o GIF percorre normais, especiais, supers e habilidades.

## Revisões funcionais

- Auditoria de movimentos do Akio: 26 normais, 13 especiais, 3 supers; projéteis, arremesso, cancelamentos, voo, velocidade e formas registrados sem problemas estáticos.
- Laboratório nativo: 52 estados de ataque conectaram.
- Entradas reais: todos os 59 comandos de botão e os 63/63 estados de habilidade do Akio foram alcançados.
- Auditoria do elenco: 6.708 quadros de caixas alinhados a pixels visíveis e 973 quadros de golpe dentro das proporções; zero problemas.
- Teste aéreo do elenco: 337/337 variantes conectaram e voltaram ao chão.
- Banco de vozes: todos os clipes foram decodificados; nenhum arquivo de voz precisou de alteração.
- Web smoke: `WEB SMOKE OK`; a captura do combate antes do desfoque foi verificada a 1280×720 (média de luminância 97,3; desvio padrão 46,8).
- `node scripts/file-size.mjs` e `git diff --check` passaram.

O wrapper `scripts/build-web.sh` não iniciou porque o disco WSL desta máquina (`ext4.vhdx`) está ausente. O build wasm, o empacotamento de sprites e o mesmo script de smoke foram executados manualmente no Windows.
