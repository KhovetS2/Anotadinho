---
title: "Ciclo 157 — CLI embed: agente lê e escreve embeds"
type: ciclo
ciclo: "157"
status: concluida
date: 2026-08-19
prioridade: alta
depende_de: ["149"]
tags:
- ciclo
---

# Ciclo 157 — CLI embed: agente lê e escreve embeds

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# CLI embed: agente lê e escreve embeds

## Objetivo

Fecha o loop agente↔UI. Hoje o `anotadinho-cli` lê/escreve página
inteira e propriedade de frontmatter, mas não sabe o que é um embed:
pra mover um card do board um agente precisa reescrever o `.md`
inteiro montando YAML na mão — o caminho que já corrompeu arquivo no
ciclo 064 e cujo conserto (serializar só por derive de serde) é
justamente o que o `crates/core` agora expõe depois do ciclo 149.

## Critérios de aceite

- [x] `anotadinho-cli embed list <page>` — índice, tipo e um resumo de
      cada embed da página (nº de cards/eventos/linhas); `--json`
      devolve `[{ index, type, summary }]`
- [x] `anotadinho-cli embed get <page> <idx>` — o CONTEÚDO do embed
      como está no arquivo (YAML pra 8 dos 9 tipos; tabela markdown pro
      `table`, cujo formato nasceu como tabela markdown comum). Com
      `--json` vem envelopado: `{index, type, body}`. Ver Notas: JSON
      tipado por embed não existe porque `TableEmbedData` não é uma
      struct de serde
- [x] `anotadinho-cli embed set <page> <idx>` — lê o conteúdo de um
      arquivo (`--file`) ou do stdin, PARSEIA no tipo do embed daquele
      índice e reescreve por `EmbedData::to_fence_text` +
      `embed::join`. Frontmatter e markdown ao redor ficam intocados;
      os outros embeds da página são re-serializados (normalização,
      igual a qualquer escrita pelo app — ver Notas)
- [x] Atalhos: `embed add-card <page> <idx> --column <col> --title <t>`,
      `embed add-row <page> <idx> --values a,b,c`,
      `embed add-event <page> <idx> --date <YYYY-MM-DD> --title <t>`
- [x] Tipo errado (ex: `add-card` num embed de tabela) sai com código
      != 0 e mensagem clara, sem tocar no arquivo
- [x] Índice fora do intervalo idem
- [x] Testes em `crates/cli/tests/cli.rs` (padrão existente com
      `tempfile`/`assert_cmd`): list/get/set round-trip; `add-card`
      preservando o markdown ao redor; erro de tipo; erro de índice;
      e um teste de IDEMPOTÊNCIA: a primeira rodada de `get | set`
      normaliza (aspas de YAML, espaçamento do wrapper — o mesmo que
      qualquer escrita pelo app faz), e da segunda em diante o arquivo
      não muda mais

## Comandos de validação

```bash
cargo build --workspace
cargo test --workspace
cargo test -p anotadinho-cli
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Watch/streaming de mudanças pro agente
- Criar embed novo numa página pelo CLI (`embed add <type>`) — entra
  depois se o fluxo pedir; este ciclo é sobre operar embeds existentes
- Edição concorrente com a janela aberta: o app já tem watcher
  (ciclo 012) e recarrega, mas conflito de escrita simultânea não é
  resolvido aqui

## Notas

`cargo test -p anotadinho-cli`: 22 (13 + 9 novos). `cargo test
--workspace`: 241. `trunk build` e `cargo build --manifest-path
src-tauri/Cargo.toml`: OK.

Formato de intercâmbio: o CONTEÚDO do embed, não um JSON por tipo. O
`table` guarda tabela markdown (com um preâmbulo YAML opcional de
configuração de coluna) e `TableEmbedData` não deriva `Serialize` — um
JSON tipado por embed teria que inventar uma representação paralela e
mantê-la em sincronia com o parser. O conteúdo cru é o que já
round-tripa, e `set` sempre passa pelo parser antes de gravar, que é a
garantia que importa: o agente nunca escreve texto direto no arquivo.

Uma escrita normaliza TODOS os embeds da página, não só o alterado
(`join` re-serializa cada segmento). É o mesmo comportamento de
qualquer edição pelo app; vale saber antes de olhar o primeiro `git
diff`.

Validação ao vivo: `embed add-card pages/exemplos-embeds.md 0 --column
Todo` com o app aberto e, ao ABRIR a página, o card estava lá na coluna
Todo.

Correção (2026-08-20): a redação original dizia "sem recarregar nada
(watcher do ciclo 012)" — errado. Quem releu o arquivo foi o clique na
página; o watcher não recarrega página aberta, e `write_page` nem
compara versão antes de gravar. Ver task 173.

O teste de "get | set sem alteração => arquivo idêntico" é o que
protege contra regressão silenciosa de formatação — o mesmo tipo de
bug dos ciclos 076/078/111.

## Resultado

# Ciclo 157 - done

## Resumo

Fecha o loop agente↔UI. O `anotadinho-cli` passou a enxergar os embeds
inline: listar, ler, substituir e as mutações típicas (card, linha,
evento). Antes disso, um agente que quisesse mover um card tinha que
reescrever o `.md` inteiro montando YAML por concatenação — o caminho
que corrompeu arquivo no ciclo 064.

Toda escrita passa pelo parser do tipo e sai por
`EmbedData::to_fence_text` + `embed::join`: o agente nunca escreve
texto direto no arquivo.

## Arquivos criados/modificados

- `crates/cli/src/main.rs` — subcomando `embed` (list/get/set/add-card/
  add-row/add-event), `PageDoc`, `embed_summary`, `embed_body`
- `crates/cli/tests/cli.rs` — 9 testes novos

## Testes adicionados

- `list` mostra índice, tipo e resumo
- `get` devolve o conteúdo do embed
- `get | set` é idempotente da segunda rodada em diante, e o entorno
  sobrevive
- `add-card` preserva frontmatter, os 3 trechos de markdown ao redor e
  o outro embed da página
- `add-event` no calendário
- tipo errado falha SEM tocar no arquivo; coluna inexistente falha;
  índice fora do intervalo diz quantos embeds a página tem
- `add-row` valida a quantidade de células contra as colunas

## Problemas encontrados

- O formato de intercâmbio virou o conteúdo do embed, não JSON por
  tipo: `TableEmbedData` não é struct de serde (o formato dele é tabela
  markdown), e um JSON paralelo teria que ser mantido em sincronia com
  o parser.
- O teste de round-trip byte-exato falhou de início — corretamente: a
  primeira escrita normaliza um arquivo escrito à mão. Virou teste de
  idempotência, que é a propriedade que importa.
- O status deste ciclo foi commitado num segundo commit: o commit do
  código saiu antes de o arquivo de status existir.

## Notas para próximos ciclos

- 158 (CLI de query) fecha a leitura: o agente vê o mesmo recorte que o
  humano.
- Uma escrita normaliza todos os embeds da página, não só o alterado.

## Correção (2026-08-20)

Este status dizia que o card escrito pelo CLI "apareceu na coluna Todo
do board sem recarregar nada (watcher do ciclo 012)". **Está errado.**
Naquela validação eu tinha CLICADO na página depois de escrever, e é o
clique que relê o arquivo. O watcher não recarrega página aberta: o
polling de `check_changes` só atualiza a lista da sidebar.

Pior: `write_page` não compara versão nenhuma antes de gravar, então
uma edição feita pelo CLI com a página aberta é sobrescrita no autosave
seguinte, em silêncio. Virou a **task 173**.
