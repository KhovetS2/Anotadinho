---
id: "106"
titulo: "Navegacao por teclado na sidebar"
status: pending
criado: 2026-08-08
autor: humano
prioridade: media
depende_de: ["105"]
estima_min: 120
agente_alvo: claude-sonnet
---

# Navegação por teclado na sidebar

## Objetivo

Terceiro ciclo do tema "navegação 100% via teclado". A sidebar (lista
de páginas + árvore de pastas) hoje é 100% mouse — zero navegação por
seta (confirmado na auditoria). Este ciclo implementa a ação "Focar
sidebar" deixada pronta no ciclo 105: setas movem um item destacado
(incluindo entrar/sair de pastas), Enter abre a página destacada,
Escape sai da região e devolve o foco pro editor.

## Critérios de aceite

- [ ] `ui/src/components/sidebar.rs`: estado de "item destacado" (índice
      numa lista ACHATADA — pastas expandidas + páginas visíveis, na
      ordem em que aparecem na tela) — mesmo padrão de índice já usado
      pelo menu `/`/paleta de comandos
- [ ] `ArrowDown`/`ArrowUp` movem o destaque (scroll-into-view se sair
      da área visível, mesmo padrão do menu `/`); `ArrowRight` numa
      pasta expande (se colapsada) ou entra nela; `ArrowLeft` colapsa
      (se expandida) ou sobe pro pai
- [ ] `Enter` abre a página destacada (mesmo `on_page_selected` do
      clique); `Escape` sai da região sidebar, foco volta pro editor
      (ou pro corpo da página se nenhuma estiver aberta)
- [ ] Ativar via "Focar sidebar" (`GlobalKeymap`, ciclo 105) muda o
      destaque visualmente pro primeiro item e habilita a navegação por
      seta; clicar com o mouse continua funcionando exatamente como
      antes
- [ ] `cargo test --workspace`, `cd ui && cargo test --lib`,
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

- Navegação por teclado dentro da caixa de busca da sidebar (já filtra
  em tempo real digitando; setas aqui são só pra navegar a LISTA depois
  de focar a região, não pra dentro do campo de texto)
- Ações de mover/renomear/excluir página só com teclado (continuam só
  por menu/botão) — este ciclo é só navegação/seleção, não gestão
- Journals na mesma navegação por seta da árvore de Pages — v1 cobre só
  a seção Pages (lista + árvore); Journals fica pra depois se pedirem

## Notas

Depende do ciclo 105 (`GlobalKeymap` com a ação "Focar sidebar" já
existindo, mesmo que sem comportamento ainda). A "lista achatada" de
navegação precisa ser recalculada toda vez que uma pasta expande/
colapsa (igual a árvore de pastas já reage a isso via `<details
open>` nativo) — cuidado pra manter o índice destacado coerente quando
a lista muda de tamanho no meio da navegação.
