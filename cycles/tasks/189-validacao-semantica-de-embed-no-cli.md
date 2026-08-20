---
id: "189"
titulo: "Validação semântica de embed no CLI"
status: pending
criado: 2026-08-20
autor: humano
prioridade: alta
depende_de: [157]
estima_min: 90
agente_alvo: claude-opus
---

# Validação semântica de embed no CLI

# Objetivo

`anotadinho-cli embed set` aceita qualquer JSON que PARSEIA pro tipo.
Um agente pode gravar um card numa coluna que não existe, um item de
timeline com `end` antes do `start` ou uma galeria apontando pra asset
ausente: tudo isso passa pelo serde, vai pro disco e só quebra na
renderização. Este ciclo põe uma checagem semântica entre o parse e a
gravação.

## Critérios de aceite

- [ ] `EmbedData::validate(&self, ctx: &ValidationCtx) -> Vec<Problema>`
      em `crates/core/src/embed.rs`, com `Severidade::{Erro, Aviso}`.
- [ ] Regras cobertas: coluna inexistente no kanban; intervalo invertido
      em timeline/calendário; valor fora das `options` de coluna
      select/multiselect; número de células diferente do número de
      colunas na tabela; asset ausente na galeria; `action` de botão
      desconhecida e `template`/`path` inexistente.
- [ ] `embed set`/`add-card`/`add-row`/`add-event` recusam `Erro`
      (saída != 0, nada gravado) e imprimem `Aviso` sem bloquear.
- [ ] `--forcar` grava mesmo com erro, pra não travar caso legítimo.
- [ ] Subcomando `embed check <page>` valida o que já está no disco.
- [ ] Testes por regra, com `tempfile`.

## Comandos de validação

```bash
cargo build --workspace
cargo test --workspace
cargo test -p anotadinho-cli
```

## Não-objetivos

- Validar na UI neste ciclo — o alvo é o canal do agente, que é onde o
  erro entra sem ninguém ver.
- Corrigir automaticamente o que está errado.

## Notas

`ValidationCtx` carrega o que a checagem precisa do MUNDO (raiz do
vault, pra conferir asset e template). Sem contexto, as regras que
tocam disco não têm como rodar; com ele, elas ficam testáveis por
`tempfile` como as outras.
