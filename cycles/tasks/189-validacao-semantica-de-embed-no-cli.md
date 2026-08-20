---
id: "189"
titulo: "Validação semântica de embed no CLI"
status: done
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

- [x] `EmbedData::validate(&self, ctx: &ValidationCtx) -> Vec<Problema>`
      em `crates/core/src/embed.rs`, com `Severidade::{Erro, Aviso}`.
- [x] Regras cobertas: coluna inexistente no kanban; intervalo invertido
      em timeline/calendário; valor fora das `options` de coluna
      **select**; data/número malformados em coluna tipada; número de
      células diferente do número de colunas na tabela; asset ausente na
      galeria; `action` de botão desconhecida e `template`/`path`
      inexistente; `width: 0` em coluna; `columns: 0` em galeria.
- [x] `embed set`/`add-card`/`add-row`/`add-event` recusam `Erro`
      (saída != 0, nada gravado) e imprimem `Aviso` sem bloquear.
- [x] `--forcar` grava mesmo com erro, pra não travar caso legítimo.
- [x] Subcomando `embed check <page>` valida o que já está no disco, com
      saída != 0 quando há erro (serve pra hook de commit).
- [x] Testes por regra: 13 em `core::embed`, 6 em `crates/cli`.

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
tocam disco ficam em SILÊNCIO em vez de acusar falso positivo — é o que
deixa a validação utilizável também na UI, que não alcança o disco.

**Uma regra da task original foi descartada com motivo:** valor fora das
`options` numa coluna `multiselect` NÃO é erro. A lista de opções de
multiselect cresce sozinha por design (digitar um valor novo na célula
cadastra a opção), então "fora da lista" é uso normal. Só `select`, que
é fechada, entra na regra. Tem teste pros dois lados.

A validação roda sobre o embed TOCADO, depois da mutação e antes do
save. Validar a página inteira faria um embed inválido pré-existente
travar uma edição que não tem nada a ver com ele.

A regra de "número de células diferente do de colunas" só é alcançável
por construção direta: o pulldown-cmark completa linha curta ao parsear.
Coberta por teste de unidade no core, não pelo CLI.
