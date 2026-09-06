---
id: "279"
titulo: "Passo 4c: a árvore manda na navegação"
status: aberto
criado: 2026-09-06
autor: agente
prioridade: alta
depende_de: ["278"]
estima_min: 240
---

# 279 — A árvore manda na navegação

## Onde o passo 4 está

- 4a (ciclo 276): a árvore decide o que mudou ao **gravar**.
- 4b (ciclo 278): a árvore responde a **política** do bloco.
- 4c (aqui): a árvore decide **para onde o foco vai**.

## O que ainda deriva do DOM

`items_in_group()` monta a lista de destinos com
`querySelectorAll("[data-nav-item][data-nav-parent=...]")`, filtra por
retângulo visível e anda com `index ± 1`. O núcleo tem
`navegacao::mover(raiz, cursor, Passo)` desde o ciclo 273 e não é
consultado por ninguém.

Enquanto for assim, o `navegacao.rs` é código que só os testes exercem —
e o porte CLI vai depender exatamente dele. Duas navegações que ninguém
compara voltam a divergir, como a política divergiu (ciclo 278).

## A ponte que falta

O caminho da árvore (`Caminho = Vec<usize>`) precisa existir no DOM.
Proposta: `marcar_blocos` estampa `data-nav-caminho="0"`, `"2.1"` — o
endereço da unidade, não um índice plano.

Com isso a navegação vira:

1. ler o caminho do elemento focado;
2. `navegacao::mover(raiz, cursor, passo)`;
3. focar o elemento com o caminho devolvido.

O ciclo 272 já provou que modelo e tela concordam sobre a lista de
blocos (bateria `--arvore`, 14 cenários), então o mapeamento por posição
tem base medida — não é suposição.

## O que isto resolve de verdade

O que a pessoa relatou no ciclo 273: andar entre unidades entrava
dentro, e dentro do calendário o foco ficava preso. `mover` não muda de
nível nunca, e devolver `None` na borda é "fica onde está" — a regra
existe no núcleo e a GUI hoje não a usa.

## Cuidados

- **Elemento invisível.** O filtro por retângulo não existe na árvore.
  O modelo precisa dizer o que é destino, ou a GUI continua filtrando
  depois — e essa decisão tem que ser explícita, não implícita.
- **Embeds.** No DOM o embed é contêiner marcado por atributo; na
  árvore é `Tipo::Embed`. O caminho tem que casar nos dois lados,
  inclusive para os filhos que o embed renderiza sozinho.
- **Medir antes.** Um cenário que reprove ANTES do conserto, ou o ciclo
  não vale — a lição dos ciclos 259, 276 e 278.

## Critérios de aceite

- [ ] O caminho da árvore está no DOM, e a bateria `--arvore` confere
- [ ] O movimento entre blocos passa por `navegacao::mover`
- [ ] Borda é "fica onde está", vindo do núcleo e não da GUI
- [ ] Um cenário foi visto reprovando antes

## Comandos de validação

```bash
cargo test --workspace
node scripts/uitest/run.mjs --arvore
node scripts/uitest/run.mjs
```
