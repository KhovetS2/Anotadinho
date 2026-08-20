---
id: "192"
titulo: "Wikilink com alias e alvo por caminho"
status: done
criado: 2026-08-20
autor: humano
prioridade: alta
depende_de: [191]
estima_min: 120
agente_alvo: claude-opus
---

# Wikilink com alias e alvo por caminho

## Objetivo

Permitir escolher separadamente o que o link APONTA e o que ele MOSTRA:
`[[pages/produto/grafo.md|Grafo do Vault]]`. Resolve o caso que motivou o
pedido — apontar pro arquivo com certeza, sem depender do título do
frontmatter — e o alvo passa a aceitar caminho além de título.

## Critérios de aceite

- [x] `links::split_wikilink` parte o miolo em (alvo, texto), com `\\|`
      escapando barra literal e só a PRIMEIRA barra separando.
- [x] `linkify` exibe o alias e leva o miolo cru no href.
- [x] `html_to_md` reconstrói o `[[...]]` sem perder o alvo, e honra
      edição do texto visível.
- [x] Clique resolve na ordem: caminho exato → caminho sem `.md` →
      título do frontmatter → nome do arquivo.
- [x] **Guardrail**: arquivo com `|` no nome abre escapado E sem escape
      (rede de segurança tenta a string inteira antes de desistir).
- [x] Autocompletar escapa a barra ao inserir.
- [x] Grafo liga pelo alvo, não pelo alias.

## Comandos de validação

```bash
cargo test --workspace
cd ui && cargo test && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
node scripts/uitest/run.mjs
```

## Não-objetivos

- Transformar link em embed: o `segment()` parte o documento por LINHA,
  então embed é sempre bloco — um link precisa viver no meio da frase.
- Trocar a resolução por título como padrão.

## Notas

A sintaxe `[[alvo|texto]]` é a do Obsidian/Logseq, então o `.md`
continua portátil. `links.rs` já cortava o alias pro grafo desde antes;
quem não sabia do pipe era o `linkify` — mais um par "corrigido num
caminho, não no outro".

Achado durante a implementação: `html_to_md` reconstruía `[[texto
visível]]`, então gravar um link com alias APAGARIA o alvo. Por isso o
href passou a levar o miolo cru em vez do alvo.
