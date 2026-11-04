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
| `let` locals (compiled to `var(59)` downwards, 20 slots) | Supported |
| `for` / `while` (unrolled up to 32 iterations per tick, then bounded) | Supported |
| `break` / `continue` | Supported |
| `call name(args)` and `[Function name(params)]` (inlined, arity checked) | Supported |

A ZSS file that fails to compile (wrong arity, unknown function, malformed loop, too many locals) is reported as a compile error and not loaded; it never silently drops logic. Loops are bounded to 32 iterations per tick; a `while` that never changes its condition stops at the cap instead of hanging the match.

Conditions are checked per controller, as in CNS. A controller that changes a value used by its own `if` affects the controllers that follow it.

As condições são avaliadas por controller, como no CNS. Um controller que muda um valor usado no próprio `if` afeta os controllers seguintes.
