# ZSS states / Estados ZSS

ZSS is the brace-based state syntax introduced by Ikemen GO. List a `.zss` file in a character's `.def` (`st`, `st1`…`st9`, `stcommon`) and it is compiled to the same state controllers as CNS.

ZSS é a sintaxe de estados com chaves criada pelo Ikemen GO. Liste um arquivo `.zss` no `.def` do personagem (`st`, `st1`…`st9`, `stcommon`) e ele vira os mesmos controllers de um CNS.

```
[StateDef 200; type: S; movetype: A; physics: S; anim: 200; ctrl: 0;]

if AnimElem = 2 {
	HitDef{ attr: S, NA; damage: 20, 0; animtype: Light }
}
if Time = 0 {
	PlaySnd{ value: 0, 0 }
} else {
	VarAdd{ v: 1; value: 1 }
}
persistent(0) ignorehitpause {
	PowerAdd{ value: 10 }
}
switch Var(1) {
case 1, 2:
	CtrlSet{ value: 1 }
default:
	CtrlSet{ value: 0 }
}
```

| Feature | Status |
|---|---|
| `[StateDef n; key: value;]`, negative states | Supported |
| `if` / `else if` / `else`, `switch` / `case` / `default` | Supported |
| `persistent(n)`, `ignorehitpause` | Supported |
| Any state controller `Name{ key: value; … }` | Supported |
| `let` locals (`let x = 3;`, `x += 1;`, `x++;`) | Supported (see below) |
| `for init; cond; step { }`, `while cond { }`, `break`, `continue` | Supported, capped at 2500 loop iterations per state run |
| `[Function name(a, b)]`, `call name(1, 2);`, `let r = call name(1);`, `return value;` | Supported, expanded at each call site |
| Anything else (unknown sections, unknown statements, `break` outside a loop, recursion, wrong argument count, expression errors) | Compile error: the character fails to load with the file name and line |

## Locals, loops and functions

```
[Function bump(n)]
let i = 0;
let total = 0;
while i < n {
	i++;
	if i = 3 { continue }
	if i > 6 { break }
	total += i;
}
return total;

[StateDef 1000]
let r = call bump(5);
VarSet{ v: 1; value: r }
```

- Locals live in 48 per-character slots (`local(n)` and `flocal(n)` internally). A local is an int or a float depending on the expression it was declared with (`let h = 1.5;` is a float), as in CNS typing. Write `x` or `$x`; `x` is left alone after `vel`, `pos`, `p2dist` and the other component triggers (`vel x`).
- A block `{ }` is a scope: locals declared inside are released at its end. Functions cannot see the caller's locals; parameters are locals initialised from the arguments.
- Functions are inlined at every call site, so the same function can be used from many states. A function may call other functions, but not itself, and calls nest at most 16 deep. A function may be defined after the state that calls it.
- `return value;` stores the result and leaves the function; `let r = call f(...)` keeps it. A function call cannot appear inside an expression, only as a statement or as the right side of `let` / `=`.
- Loops run inside one tick. A loop that has not finished after 2500 back jumps in one state run is cut off, so a runaway loop cannot hang a match or desync a replay.
- `var(n) := value;` statements are accepted.

Conditions are checked per controller, as in CNS. A controller that changes a value used by its own `if` affects the controllers that follow it.

As condições são avaliadas por controller, como no CNS. Um controller que muda um valor usado no próprio `if` afeta os controllers seguintes.

Variáveis locais (`let`), laços (`for`, `while`, `break`, `continue`) e funções (`[Function]`, `call`, `return`) são compilados junto com os controllers: as funções são expandidas em cada chamada, os locais usam 48 slots por personagem e um laço é cortado após 2500 voltas por execução de estado. Qualquer construção não suportada é erro de compilação e o personagem não carrega.
