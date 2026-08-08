---
id: "095"
titulo: "Undo redo generico via snapshots de content_md"
status: done
criado: 2026-08-07
autor: humano
prioridade: media
depende_de: []
estima_min: 90
agente_alvo: claude-sonnet
---

# Undo/redo genérico via snapshots de content_md

## Objetivo

Décimo ciclo do conjunto grande. `Ctrl+Z`/`Ctrl+Shift+Z` desfazem/
refazem QUALQUER edição — texto solto E mutação de embed (mover card,
editar evento, etc) — com uma pilha de snapshots do markdown inteiro,
não um mecanismo por tipo de embed.

## Critérios de aceite

- [x] `ui/src/components/editor.rs`: `undo_stack`/`redo_stack`
      (`use_mut_ref<Vec<String>>`, cap. 20), `last_content_ref` (base de
      comparação pra decidir quando empilhar — não `content_md`, que
      nem sempre atualiza em sync com toda edição)
- [x] Empilha um snapshot novo só se >800ms desde o último (agrupa
      rajada de digitação numa pausa só, não um passo por tecla)
- [x] `render_gen` — força o Effect 2 (injeção de HTML nos trechos de
      markdown solto) a reagir a undo/redo mesmo quando path/
      has_embeds/segment_count não mudaram (só `content_md` mudando não
      dispara Effect 2 sozinho — embeds são declarativos e reagem
      normal, mas o markdown solto injetado via `set_inner_html`
      precisava desse empurrão a mais)
- [x] `Ctrl+Z`/`Ctrl+Shift+Z` checados ANTES da interceptação do vim
      mode (undo é ação de documento, não motion de texto — funciona
      igual com vim mode ligado ou desligado)
- [x] Histórico de undo/redo é por página (limpo ao trocar de página)
- [x] `cargo test --workspace`, `cd ui && cargo test --lib`,
      `trunk build`, `cargo build --manifest-path src-tauri/Cargo.toml`
      passam

## Comandos de validação

```bash
cargo test --workspace
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Undo/redo cross-página (cada página tem sua própria pilha,
  independente)
- Preservar a POSIÇÃO do cursor exata ao desfazer — restaura o
  conteúdo, cursor fica onde o navegador decidir depois do
  `set_inner_html` novo
- Limite configurável de profundidade — fixo em 20 snapshots

## Notas

O ponto de decisão mais delicado foi como forçar os trechos de markdown
solto (injetados imperativamente via `set_inner_html`, ver Effect 2) a
se atualizarem quando `content_md` muda por undo/redo, sem reintroduzir
o loop de re-render do ciclo 043 (que reagia a QUALQUER mudança de
`content_md`, inclusive digitação normal). Fix: `render_gen` — um
contador que só é incrementado explicitamente por `do_undo`/`do_redo`,
adicionado como 4º campo do guard `last_rendered` do Effect 2. Digitar
texto normal não toca nisso, então o guard continua funcionando como
antes pra esse caso; só undo/redo força a reinjeção.

Achado de metodologia de teste (mesma classe do ciclo 091): `Ctrl+Z`
não fez nada na primeira tentativa porque o foco estava em
`.app-root` (elemento errado) em vez do `<div contenteditable>` de
verdade — o handler de `on_keydown` do editor está ligado aos
segmentos/contenteditable especificamente, não ao container raiz do
app. Focar o elemento certo resolveu.

Validado ao vivo via MCP `tauri`: texto solto — digitar "primeira
versao", esperar, digitar mais, `Ctrl+Z` volta pro passo anterior,
`Ctrl+Shift+Z` refaz, `Ctrl+Z` 2x volta ao estado vazio original.
Embed — inserir calendário (semeia 1 evento de exemplo), adicionar um
2º evento, `Ctrl+Z` volta pra 1 evento, `Ctrl+Shift+Z` volta pra 2 —
confirma que embeds (declarativos) e texto solto (imperativo) undo
corretamente com o MESMO mecanismo. Mudança de teste revertida em
`VaultAnotadinho/pages/teste.md`.
