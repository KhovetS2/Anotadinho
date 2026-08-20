---
id: "157"
titulo: "CLI embed: agente lê e escreve embeds"
status: done
criado: 2026-08-19
autor: humano
prioridade: alta
depende_de: ["149"]
estima_min: 120
agente_alvo: claude-sonnet
---

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
Todo` com o app aberto — o card apareceu na coluna Todo do board sem
recarregar nada (watcher do ciclo 012). Loop agente↔UI fechado.

O teste de "get | set sem alteração => arquivo idêntico" é o que
protege contra regressão silenciosa de formatação — o mesmo tipo de
bug dos ciclos 076/078/111.
