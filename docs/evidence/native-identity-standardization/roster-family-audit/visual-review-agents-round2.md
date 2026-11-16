# Revisão visual do roster — segunda rodada

Data: 2026-11-12
Worktree: `fix/native-roster-low-kicks`, base `881b8db6`
Escopo: 379 entradas atuais em 14 pacotes na fila `visual-family-correction-queue.json`
Revisores: cinco subagentes com conjuntos de personagens sem sobreposição; todos somente leitura.

## Resultados

| Pacote | Itens na fila | Resultado estático desta rodada |
|---|---:|---|
| Akio | 36 | Comparação individual completa: 15 consistentes sem correção e 21 com discrepância de penteado/rosto/acabamento, mantendo em geral a roupa reconhecível. O finding anterior “roupa errada” era incorreto para `cjab0`, `cjab1`, `dfLie` e mais 12 arquivos; três super/starburst têm efeito ciano coerente. Formas não trocam cabelo/roupa. |
| Bruno | 33 | Comparação individual completa: 33 com desvio de acabamento e/ou equipamento, sem troca evidente de personagem. `br0/1` têm a maior divergência de roupa; `cjab0/1`, `clk`, `crouch0/1` preservam melhor o design e diferem sobretudo no traço/equipamento. Cinco candidatos anteriores foram rejeitados por deriva de cabelo/rosto e os originais foram restaurados. |
| Daichi | 19 | Roupa antiga com mangas e calça cinza difere do gi sem mangas e calça creme da âncora. `flip` e `getUp` exibem troca entre quadros adjacentes. |
| Garan | 4 | `hop`, `jumpDown`, `jumpTop`, `jumpUp` divergem do oni musculoso de pele vermelha, juba branca, chifres e hakama roxo da âncora `04_special.png`. |
| Haruki | 29 | Cabelo e rosto ainda lembram o personagem, mas colete/camisa roxa quadriculada e calça escura divergem do casaco longo roxo com acabamento dourado. Sem forma declarada que explique a troca. |
| Jin | 0 | Nenhum item na fila; a folha revisada parece coerente com a âncora. Isso não equivale a validação de todas as sequências em runtime. |
| Lian | 29 | 28 imagens mostram grande desvio de roupa e acabamento da túnica verde ornamentada, fita/coque e calça creme da âncora. `crouch0.png` é um candidato estático promissor, ainda sem validação runtime da sequência e dos vizinhos. |
| Marlow | 23 | Alvos auxiliares simplificam ou ocultam chapéu amarelo, cabelo ruivo, casaco azul longo, camisa branca e faixa vermelha. O desvio excede acabamento em várias poses. |
| Kaname | 70 | A âncora mostra homem de cabelo azul, haori branco e hakama azul-marinho; muitas auxiliares mostram ninja mascarado/armadurado. Não há forma declarada que explique a mudança de design. |
| Oboro | 40 | Contatos apontam divergência mais séria que acabamento: a âncora mostra homem de cabelo escuro, robe/hakama preto e vermelho e katana; várias auxiliares mostram figura mascarada feminina de roupa escura e espada rosa. Confirmar formas/trocas declaradas antes da correção. |
| Shion | 40 | Há diferença material de silhueta e roupa frente à túnica longa preto/vermelha, cabelo branco e calça larga da âncora; não foi concluída troca de identidade. |
| Suzume | 18 | A inconsistência visual é real, mas a descrição de identidade na fila está errada: `04_special.png` mostra cabelo roxo preso com laço vermelho, máscara/roupa ninja roxa e vermelha e kunai — não uma lutadora verde e creme. Corrigir a referência antes de gerar. |
| Sylene | 2 | A âncora é maga de cabelo longo gelo e vestido branco/ciano/azul; `airGuard.png` parece ninja mascarado de roupa azul/branca, com grande diferença de silhueta. |
| Valdric | 2 | `airGuard.png` difere em anatomia, escala, armadura e acabamento do guerreiro musculoso de cabelo roxo longo e armadura preta/vermelha/dourada. |
| Volta | 34 | A família clara e mais fina diverge do robô robusto grafite com placas cinza e luzes ciano da âncora. Investigar se é modelo/forma antes de normalizar. |

## Limites e próximos passos

- Os revisores cobriram todos os 379 itens por conjunto de personagens; os 69 alvos de Akio e Bruno foram comparados individualmente a 400×400. Em outros pacotes, a cobertura incluiu páginas completas e amostras nativas, não necessariamente cada arquivo. Onde consta “parece”, “muitas” ou “suspeito”, a classificação permanece provisória.
- A inspeção foi estática. Nenhuma sequência de animação dos itens pendentes foi validada em runtime nesta rodada.
- O estado atual da fila não deve ser usado como prompt único de lote: o finding compartilhado do Akio contradiz imagens atuais, e a descrição canônica de Suzume está incorreta.
- A geração não ocorreu: a rota local Grok exige login; a ferramenta de imagem da sessão atingiu o limite de uso e informa reset em 2026-11-12 19:37 UTC. Não foi criado nem integrado candidato desta rodada.
- A fila mantém 379 registros hash-pinned: 15 foram adjudicados sem correção; 363 têm correção visual confirmada pendente; um candidato integrado de Lian aguarda runtime completo. A revisão estática não substitui a validação das famílias afetadas.
