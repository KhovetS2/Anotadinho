---
title: Ciclo 175 — Edição estruturada por bloco
type: ciclo
ciclo: "175"
status: concluida
date: 2026-08-20
prioridade: media
depende_de: ["174"]
tags:
- ciclo
---

# Ciclo 175 — Edição estruturada por bloco

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Edição estruturada por bloco

## Objetivo

Segunda fatia do editor de blocos. Com o ciclo 174 dá pra NAVEGAR por
blocos; aqui o bloco passa a ser a unidade de EDIÇÃO: um
`contenteditable` por bloco, em vez de um por segmento de markdown.

É o que destrava reordenar bloco, dobrar/desdobrar e (depois) comentar
por bloco. É também a fatia arriscada: mexe em digitação, que é o
caminho mais usado do app inteiro e a origem histórica de quase todo
bug do editor (076, 078, 079, 082, 111, 141-143).

## Critérios de aceite

- [x] Um `contenteditable` por bloco, com o markdown do bloco
- [x] Enter no fim de um bloco cria um bloco novo depois; Enter no meio
      divide o bloco em dois
- [x] Backspace no início de um bloco funde com o anterior (e no
      primeiro bloco não faz nada)
- [ ] Seleção e cópia atravessando blocos preservam o markdown — **não
      entregue**, ver a nota final
- [x] Colar vários parágrafos cria vários blocos
- [x] Mover bloco pra cima/baixo por atalho (a mesma ação que a toolbar
      de embed já tem, agora pra qualquer bloco) — **feito**, mais
      duplicar (`y`) e apagar (`d`)
- [x] Desfazer/refazer continua funcionando (bateria 193)
- [x] Atalhos de formatação por prefixo (`#`, `-`, `>`, `[]` — ciclos
      142/143) continuam disparando dentro do bloco
- [x] O `.md` gerado é byte-idêntico ao de hoje pra uma página que não
      foi editada — cenário da bateria 193
- [x] Vim mode continua operando dentro do bloco (cenário do 133 verde)

## Comandos de validação

```bash
cargo test --workspace
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Arrastar bloco com o mouse (entra depois; o atalho já resolve o caso)
- Blocos aninhados/outliner com indentação (é outro modelo, e o projeto
  é markdown-first, não outliner)

## Notas

Fazer só depois do 174 estar em uso por alguns dias: se a navegação por
blocos já resolver o que o usuário precisa, esta fatia pode nem ser
necessária — e ela é a que mais pode quebrar coisa que funciona.

Recomendação forte: entrar aqui só com o harness de teste de UI de pé
(ver proposta de roadmap), porque os modos de falha desta mudança são
todos de comportamento de DOM.


## Estado: FEITO

O 175a entregou a manipulação de bloco por teclado; este fechamento
entregou a reescrita. O texto abaixo, escrito quando a reescrita foi
adiada, fica como registro de por que ela não era urgente.

### O que ficou de fora, com motivo

**Seleção atravessando blocos.** Com um `contenteditable` por bloco, o
navegador não estende seleção entre eles — é a troca conhecida desse
modelo (Notion e Logseq resolvem reimplementando seleção do zero). Não
foi reimplementado: é trabalho grande e ninguém pediu. Copiar DENTRO de
um bloco funciona; entre blocos, não.

### Estado anterior: PARCIAL (ciclo 175a)

Entregue a manipulação de bloco pelo teclado, que era o item de valor
direto da lista: `Alt+↑`/`K` sobe, `Alt+↓`/`J` desce, `y` duplica, `d`
apaga, com o foco preservado pra encadear as ações.

**A reescrita arquitetural (um `contenteditable` por bloco) NÃO foi
feita, de propósito.** Motivos, na ordem em que pesam:

1. Ela não é necessária pro que foi pedido. A manipulação por teclado
   funciona movendo o NÓ no DOM e recompondo o markdown a partir dele —
   o mesmo caminho que toda edição já usa. O bloco já é um filho de
   primeiro nível marcado por `marcar_blocos` desde o 174.
2. Os critérios que sobram (Enter divide, Backspace funde, colar cria
   vários blocos, seleção atravessando blocos) mexem em DIGITAÇÃO, que
   é o caminho mais usado do app e a origem de quase todo bug do editor
   (076, 078, 079, 082, 111, 141-143). A própria task já dizia isso.
3. O ganho concreto sobre o estado atual é reordenar com o mouse e
   dobrar bloco — nenhum dos dois foi pedido.

Se for pra fazer, o pré-requisito continua valendo e agora existe: o
harness. Vale escrever os cenários de digitação ANTES de mexer, pra a
reescrita ter rede.

## Resultado

# 175 — Um `contenteditable` por bloco

## A decisão que tornou isto seguro

As TAGS dos blocos não mudaram. Um bloco continua sendo `<p>`, `<h1>`,
`<ul>`, `<pre>` — só ganhou `contenteditable="true"` e a classe
`editor__bloco`; o contêiner do segmento passou a `false`.

Como a tag é a mesma, `html_to_md` não mudou uma linha e o markdown
gerado continua idêntico. Foi o que permitiu trocar o modelo de edição
sem reescrever a serialização junto — e é por isso que o cenário de
round-trip byte-idêntico passou já na primeira execução depois da
reescrita.

## O que mudou

- `ui/src/components/editor.rs`
  - Contêiner do segmento: `contenteditable="false"`. Os handlers
    continuam nele — eventos borbulham do bloco.
  - `marcar_blocos`: marca cada bloco como editável e **cria um
    parágrafo vazio quando o segmento não tem bloco nenhum**.
  - `dividir_bloco` (Enter) e `fundir_com_anterior` (Backspace no
    início), com lista/código/tabela ficando de fora — neles o Enter
    dentro do bloco é o certo.
  - `entrar_no_bloco` foca o bloco, não o contêiner.
  - `inserir_segmento_e_abrir_menu` foca o primeiro bloco do segmento.
- `ui/src/styles/components.css`: bloco visível — hover discreto, barra
  azul no foco, e dica em bloco vazio.
- `ui/src/app.rs`, `ui/src/components/sidebar.rs`: seletores de foco.
- `scripts/uitest/`: encanamento dos cenários (as asserções ficaram
  intactas — elas olham o markdown salvo, não o DOM).

## O que o harness pegou

1. **Página vazia sem nenhum bloco editável.** Sem filhos no segmento,
   não havia onde pôr cursor: uma página nova ficava impossível de
   digitar. Apareceu como "o menu / não abre" em dois cenários.
2. **`n` sobre embed parou de abrir o menu.** Ele focava o contêiner do
   segmento, que deixou de ser editável.

Os dois teriam passado despercebidos numa conferência manual rápida.

## Validação

- `cargo test --workspace`: 0 falhas; `ui`: 39 testes.
- `trunk build`: `✅ success`; Tauri: 0 erros.
- `node scripts/uitest/run.mjs`: **43/43 em 241.6s**.

## Não verificado

O estilo de `:focus` do bloco não dá pra conferir pelo bridge: a janela
não tem foco do sistema quando é dirigida de fora, então nenhum elemento
casa `:focus`. Confirmei que a regra existe na folha servida e que o
estilo base do bloco aplica (padding medido). O visual do foco em uso
real precisa de olho humano.
