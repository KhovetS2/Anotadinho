---
id: "219"
titulo: "O motivo da falha do agente não pode ser engolido pelo ruído"
status: done
criado: 2026-08-23
autor: humano
prioridade: alta
depende_de: ["218"]
estima_min: 45
agente_alvo: claude-opus-5
---

# O motivo da falha do agente não pode ser engolido pelo ruído

## Objetivo

A tela mostrou `o agente falhou: Reading additional input from
stdin...`. Isso não diz nada a ninguém, e mandou o usuário procurar
problema na configuração quando não havia nenhum.

## Critérios de aceite

- [x] O motivo reportado pelo agente no stream vence o ruído do stderr
- [x] O evento `error` do Codex também vira motivo, não só `turn.failed`
- [x] Sem nada no stream, o stderr continua servindo de pista
- [x] Sem nada em lugar nenhum, a mensagem diz o código de saída
- [x] Cenário de harness travando a regressão

## Comandos de validação

```bash
cargo test --workspace
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
node scripts/uitest/run.mjs
```

## Notas

### O que realmente aconteceu

Reproduzindo a chamada exata, o stdout tinha:

```
{"type":"error","message":"You've hit your usage limit. ..."}
{"type":"turn.failed","error":{"message":"You've hit your usage limit. ..."}}
```

A conta do Codex bateu o limite de uso. Não havia nada errado com a
configuração, com a pasta de trabalho nem com o sandbox.

### Por que a mensagem era inútil

Quando o processo saía com código != 0, o código preferia o stderr:

```rust
let detalhe = if stderr.is_empty() { stdout } else { stderr };
```

O stderr do Codex tem uma linha de ruído de inicialização ("Reading
additional input from stdin...", porque o stdin é `null`). Como ela não
é vazia, ganhava — e o motivo real, que estava no stream, era
descartado junto com o `Err` do leitor, num `unwrap_or_default()`.

A ordem agora é: o erro que o LEITOR entendeu, depois o texto do
stream, depois o stderr, e por último o código de saída. O agente diz o
motivo no stream; o stderr quase sempre é ruído.
