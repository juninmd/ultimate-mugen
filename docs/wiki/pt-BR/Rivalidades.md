# Rivalidades

[English](../Rivalries.md)

Um lutador nativo pode declarar `rivals` no JSON, com o id do oponente como chave. Cada entrada pode trocar a fala do início do round e dar um pequeno bônus de ataque naquele confronto.

- `intro`: fala de quem começa, no lugar da `intro` genérica.
- `taunt`: resposta, no lugar da `taunt` genérica.
- `attack`: multiplicador de ataque só contra aquele oponente, limitado a 0,9 até 1,15.

O bônus multiplica o ataque depois dos passivos e respeita o teto global de [Passivos](../Passives.md). Lutadores sem `rivals` agem como antes.

Pares atuais, baseados na [Lore](../Lore.md): Kaito e Akio, Suzume contra Oboro, Sylene contra Valdric e Kaname, Bruno e Marlow contra Valdric, Shion contra Garan.
