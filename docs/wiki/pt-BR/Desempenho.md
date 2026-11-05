Onda 30 - Direções Relativas de Entrada Sem Desvios e Tabela de Salto em Bytes:
- Substitui desvios aninhados em `input::rel_dirs` por deslocamento binário branchless `ud | (4 << (is_back ^ 1))`.
- Substitui casamento de strings em `button_bit` por casamento em nível de byte na fatia, suportando insensibilidade a maiúsculas sem alocação.
- Implementa atribuição de direção branchless em `InputHistory::push`.

## Fast-path de bytecode de operação única e avaliação sem pilha

- Atalho de expressão com operação única: Em `bytecode::Code::run`, expressões com exatamente um opcode (`Op::Const(v)`, `Op::Trig0(i)`, `Op::Host(i)`) retornam imediatamente sem inicialização do array de pilha, avanço do program counter ou despacho no loop da máquina virtual.
- Consultas de constantes e parâmetros com custo zero: Números constantes e gatilhos sem argumentos (`Time`, `StateTime`, `AnimTime`, `StateNo`, etc.) executam em 1 ciclo de CPU sem overhead de frame de execução.
