---
id: "157"
titulo: "CLI embed: agente lê e escreve embeds"
status: pending
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

- [ ] `anotadinho-cli embed list <page>` — índice, tipo e um resumo de
      cada embed da página (nº de cards/eventos/linhas); `--json`
      devolve `[{ index, type, summary }]`
- [ ] `anotadinho-cli embed get <page> <idx>` — os dados do embed em
      JSON (default) ou YAML (`--yaml`)
- [ ] `anotadinho-cli embed set <page> <idx>` — lê JSON de um arquivo
      (`--file`) ou de stdin, valida contra o tipo do embed naquele
      índice e reescreve a página por `EmbedData::to_fence_text` +
      `embed::join`, preservando byte a byte todo o resto do arquivo
      (frontmatter, markdown ao redor, outros embeds)
- [ ] Atalhos: `embed add-card <page> <idx> --column <col> --title <t>`,
      `embed add-row <page> <idx> --values a,b,c`,
      `embed add-event <page> <idx> --date <YYYY-MM-DD> --title <t>`
- [ ] Tipo errado (ex: `add-card` num embed de tabela) sai com código
      != 0 e mensagem clara, sem tocar no arquivo
- [ ] Índice fora do intervalo idem
- [ ] Testes em `crates/cli/tests/cli.rs` (padrão existente com
      `tempfile`/`assert_cmd`): list/get/set round-trip; `add-card`
      preservando o markdown ao redor; erro de tipo; erro de índice;
      e um teste que roda `set` com o JSON devolvido por `get` sem
      alteração e confirma que o arquivo fica idêntico

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

O teste de "get | set sem alteração => arquivo idêntico" é o que
protege contra regressão silenciosa de formatação — o mesmo tipo de
bug dos ciclos 076/078/111.
