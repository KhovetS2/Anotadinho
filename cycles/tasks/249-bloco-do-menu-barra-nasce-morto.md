---
id: "249"
titulo: "O bloco inserido pelo menu / nascia morto"
status: done
criado: 2026-09-03
autor: humano
prioridade: alta
depende_de: []
estima_min: 120
agente_alvo: claude-opus
---

# O bloco inserido pelo menu / nascia morto

## Objetivo

Reportado ao vivo, com três sintomas que pareciam soltos:

1. o convite "Digite ou use / para inserir" aparecendo **por cima** de um
   parágrafo que tem texto;
2. escolher um item do menu `/` e não conseguir editar o bloco inserido —
   nem ele, nem o resto da página — até sair da página e voltar;
3. salvar antes de sair e voltar dava `gravação recusada: isso apagaria
   as N letras`.

O ciclo 248 tinha tratado a ponta do 3 (o autosave não grava vazio, e a
trava passou a comparar o corpo). Era paliativo consciente: o autosave
não devia mesmo gravar vazio, mas ninguém tinha perguntado **por que o
markdown saía vazio**.

## O que estava errado

Medido na janela de verdade (`/` num bloco novo, item escolhido, DOM
inspecionado logo depois):

```
<h1>Título</h1>
```

O bloco entrou no DOM **cru**. Sem `contenteditable`, sem
`class="editor__bloco"`, sem `data-nav-block`, sem `tabindex`. Desde o
ciclo 175 o editável é o BLOCO, não o contêiner — quem dá essas marcas é
`marcar_blocos`, e `insert_element_at_cursor` nunca o chamava. Os outros
caminhos que criam bloco (Shift+Enter, `n`, duplicar) chamam:
`editor.rs:1931` e `editor.rs:4935`. Só o menu `/` não.

Daí saem os três sintomas:

- **(2a)** o bloco novo não aceita cursor nem teclado, e o nav-mode não o
  enxerga (é `[data-nav-block]` que ele percorre);
- **(2b)** pior: `range.set_start_after(el)` deixava o cursor no
  contêiner do segmento, que é `contenteditable="false"` — ou seja, em
  lugar nenhum. Medido: `selDentroDeEditavel: false`, âncora em
  `.editor__body`. Não era o bloco novo que travava, era o **editor
  inteiro**: nenhuma tecla tinha onde chegar;
- **(1)** `marcar_convite` só é revisto por `marcar_blocos`. Sem ele
  rodar, uma classe `--convite` posta quando o bloco estava vazio ficava
  lá depois de o bloco ganhar texto — e o CSS pintava a dica sem exigir
  vazio.

Sair da página e voltar "consertava" porque o efeito de render roda
`marcar_blocos` de novo em tudo.

## Achado de tabela junto (dano real, já no repositório)

`VaultAnotadinho/pages/arquitetura.md` voltou do editor com as fences do
usuário viradas em ```undefined. Causa: o highlight.js roda **em cima do
DOM editável** e escreve `class="language-undefined"` no `<code>` quando
não reconhece a linguagem; `html_to_md` lia essa classe como se fosse a
linguagem escolhida e a gravava. Uma vez no arquivo, não saía mais.

O mesmo padrão apareceu na validação, num segundo lugar: `marcar_blocos`
carimba `editor__bloco` em todo filho de primeiro nível — inclusive numa
`<figure>` de imagem, que é o único HTML que sobrevive ao round-trip com
a classe literal. Qualquer save posterior assava o marcador de runtime
dentro do arquivo.

Os dois são a mesma lição: **o que o app escreve no DOM pra funcionar não
pode voltar como conteúdo do usuário.**

## Critérios de aceite

- [x] Bloco escolhido no menu `/` entra marcado (`contenteditable`,
      `editor__bloco`, `data-nav-block`, `tabindex`) e aceita digitação
      na hora, sem sair da página e voltar
- [x] O cursor termina DENTRO de um bloco editável — nunca no contêiner
      do segmento nem no `<body>`
- [x] Embed escolhido no menu `/` (o marcador vira componente Yew num
      render seguinte, que refaz o DOM) pousa o foco num bloco depois do
      render, pela via do ciclo 195
- [x] A dica de bloco vazio não aparece em bloco com texto, qualquer que
      seja o estado da classe `--convite`
- [x] `language-undefined` do highlight.js não vira linguagem no arquivo
- [x] `editor__bloco` não é gravado dentro da `<figure>` de imagem
- [x] Cenários no harness que **escolhem** um item do menu e digitam
      depois — o vão por onde isto passou: havia cenário provando que o
      menu abre, nenhum provando o que ele insere

## Comandos de validação

```bash
cargo test --workspace
cargo build --workspace
cd ui && PATH="$HOME/.cargo/bin:$PATH" cargo check --target wasm32-unknown-unknown
node scripts/uitest/run.mjs
```
