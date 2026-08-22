---
title: Ciclo 192 — Wikilink com alias e alvo por caminho
type: ciclo
ciclo: "192"
status: concluida
date: 2026-08-20
prioridade: alta
depende_de: [191]
tags:
- ciclo
---

# Ciclo 192 — Wikilink com alias e alvo por caminho

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

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

## Resultado

# 192 — Wikilink com alias e alvo por caminho

## O que mudou

- `crates/core/src/links.rs`: `split_wikilink`, `escapar_barra`,
  `desescapar_barra`; `extract_wikilink_targets` passa a usar o split
  comum. 8 testes.
- `ui/src/wikilink.rs`: `linkify_line` exibe o alias e leva o miolo CRU
  no href; `extract_titles_line` devolve o alvo. 4 testes.
- `ui/src/html_to_md.rs`: reconstrução preserva o alvo.
- `ui/src/components/editor.rs`: `resolver_alvo` (caminho exato →
  caminho sem `.md` → título → nome do arquivo) + rede de segurança da
  barra não escapada; autocompletar escapa a barra.
- `scripts/uitest/cenarios.mjs`: 2 cenários.

## Guardrails para `|` em nome de arquivo

`|` é válido no POSIX (só o Windows proíbe), então um vault feito no
Linux pode ter `com|barra.md` de verdade.

1. `\|` escapa uma barra literal no alvo.
2. Só a PRIMEIRA barra não escapada separa — texto exibido pode conter
   barra à vontade.
3. Quem GERA wikilink escapa sozinho (`escapar_barra`), então a pessoa
   nunca digita o escape na mão.
4. **Rede de segurança:** se o alvo pós-split não resolver, a string
   inteira é tentada antes de dar "não encontrada". É o que faz
   `[[com|barra]]` escrito sem escape ainda abrir o arquivo certo. Custa
   uma busca a mais só no caminho do erro.

O cenário de harness cria um `pages/com|barra.md` de verdade e confere
os dois caminhos.

## Achado durante a implementação

`html_to_md` reconstruía `[[texto visível]]`. Com alias, gravar a página
apagaria o alvo: `[[grafo.md|Grafo do Vault]]` viraria `[[Grafo do
Vault]]`. Por isso o href passou a carregar o miolo cru — ele é a única
coisa que sobrevive ao ciclo markdown → HTML → markdown.

## Validação

- `cargo test --workspace`: 0 falhas.
- `cd ui && cargo test`: 38 passaram.
- `trunk build`: `✅ success`; Tauri: 0 erros.
- `node scripts/uitest/run.mjs`: **27/27 em 151.2s**.
