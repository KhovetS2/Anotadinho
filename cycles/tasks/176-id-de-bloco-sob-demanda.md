---
id: "176"
titulo: "Id de bloco sob demanda"
status: done
criado: 2026-08-20
autor: humano
prioridade: baixa
depende_de: ["174", "170"]
estima_min: 120
agente_alvo: claude-opus-5
---

# Id de bloco sob demanda

## Objetivo

Terceira fatia. Referenciar um bloco ESPECÍFICO ao longo do tempo
(transcluir `![[página^id]]`, backlink que aponta pro parágrafo, e não
só pra página) exige um identificador estável no arquivo.

A preocupação do usuário — "não quero um arquivo poluído" — é o
requisito central deste ciclo: **id só é escrito no bloco que alguém
de fato referenciou**, nunca em todos.

## Critérios de aceite

- [x] Com um bloco focado (nav-mode do ciclo 174), a tecla `c` grava um
      `^id` curto no fim daquela linha — e só nela — e põe
      `![[Página^id]]` na área de transferência
- [x] Bloco nunca referenciado continua sem marca nenhuma no `.md`
- [x] `^id` é renderizado como marca discreta (`.bloco-id`: menor,
      apagada, sem seleção; opaca no hover). NÃO dá pra esconder de
      vez — ver Notas: o markdown é recomposto a partir do DOM ao
      salvar, então o que sai da renderização some do arquivo
- [x] Id sobrevive porque é texto do arquivo como qualquer outro:
      mover o bloco leva o id junto, e editar o texto ao redor não o
      toca
- [x] Dois blocos de texto IDÊNTICO no mesmo arquivo geram ids
      diferentes (desempate por tentativa) — tem teste
- [x] `anotadinho-cli` sabe resolver `página^id` (ler o bloco)
- [x] Testes no core: extrair id, escrever id só onde pedido,
      round-trip, colisão

## Comandos de validação

```bash
cargo test -p anotadinho-core
cargo test -p anotadinho-cli
cargo test --workspace
```

## Não-objetivos

- Migrar o vault pra ter id em todo bloco (é exatamente o que NÃO se
  quer)
- Id em bloco dentro de embed (o embed já tem estrutura própria)

## Notas

`cargo test -p anotadinho-core`: 166 (+7). `cargo test -p
anotadinho-cli`: 37 (+2). Harness (177): 14/14.

**O achado do ciclo**: a primeira versão ESCONDIA o `^id` na
renderização, e ele sumia do arquivo no salvamento seguinte — o editor
recompõe o markdown a partir do DOM, então o que não está no DOM não
existe. O id ficou no DOM, embrulhado num `<span class="bloco-id">`
que o CSS apaga. Foi o harness que pegou isso.

Segunda pegadinha, mesma raiz: escrever o id só em `content_md` não
bastava — o guard de renderização não via motivo pra reinjetar o HTML,
então o DOM continuava sem o id e o save o descartava. Precisou do
`render_gen` (o mesmo empurrão que a recarga do ciclo 173 usa).

A linha do bloco é achada pelo TEXTO, não pelo índice do filho no DOM:
um parágrafo pode ocupar várias linhas do markdown e uma lista ocupa
uma por item. Se o texto não for encontrado, nada é gravado — melhor
não fazer do que marcar a linha errada.

Mesma convenção do Obsidian (`^id` no fim da linha), que é a mais
compatível com vault existente e a menos intrusiva — e mantém o `.md`
legível fora do app, que é premissa do projeto.
