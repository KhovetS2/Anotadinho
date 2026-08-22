---
title: Ciclo 172 — CLI observa mudanças no vault
type: ciclo
ciclo: "172"
status: concluida
date: 2026-08-20
prioridade: baixa
depende_de: ["157"]
tags:
- ciclo
---

# Ciclo 172 — CLI observa mudanças no vault

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# CLI observa mudanças no vault

# Objetivo

Depois dos ciclos 157/158 o agente lê e escreve o vault, mas só quando
ele decide olhar. Pra reagir (ex: rodar algo quando uma spec vira
`in-progress`), hoje só sobra polling.

## Critérios de aceite

- [x] `anotadinho-cli watch` fica aberto e imprime uma linha por
      mudança, JSON por linha (JSONL, que é o formato que um agente
      consome em stream): `{path, kind: created|modified|deleted, ts}`
- [x] `--folder` filtra por prefixo de path
- [x] `--property status` também emite o valor novo do campo, lendo a
      página alterada (fecha o caso "quando a spec virar in-progress")
- [x] Reaproveita o `VaultWatcher` do ciclo 012, sem um segundo
      mecanismo de observação
- [x] Debounce PARCIAL: eventos idênticos (mesmo path E mesmo tipo) no
      mesmo lote viram um só. Criar um arquivo ainda emite `created` +
      `modified`, porque são coisas diferentes de verdade e um agente
      pode querer distinguir — juntar os dois seria decidir por ele
- [x] Ctrl+C encerra pelo SIGINT padrão, código 130 (a convenção do
      shell). Converter pra 0 exigiria uma dependência só pra isso, e
      130 é o que um `while read` já entende como "o produtor parou"
- [x] Testes no `anotadinho-vault` (onde a lógica mora): evento traz
      path relativo e tipo; `drain_events` esvazia a fila; arquivo que
      não é `.md` não gera evento. Conferido de ponta a ponta rodando o
      comando de verdade contra um vault temporário

## Comandos de validação

```bash
cargo test -p anotadinho-cli
cargo build --workspace
```

## Não-objetivos

- Executar comando por conta própria quando o evento acontece (isso é
  do agente, não do editor de notas)
- Watch remoto/sync

## Notas

`cargo test -p anotadinho-vault`: 70 (+3).

Saída conferida ao vivo:

```
{"kind":"modified","path":"pages/a.md","status":"in-progress","ts":"..."}
{"kind":"created","path":"pages/b.md","status":null,"ts":"..."}
```

`ts` é o unix em segundos, cru — é o que um agente compara sem parsear
data, e evita puxar `chrono` só pra formatar uma linha.

O `VaultWatcher` ganhou uma fila de eventos AO LADO do flag booleano
que o app já usava: o app só quer saber "mudou alguma coisa" (pra
recarregar), o agente quer saber o quê — granularidades diferentes,
mesmo watcher.

Combina com o guia de agent-os: a seção de CLI ganha um exemplo de
`watch | while read` pro agente reagir.

## Resultado

# Ciclo 172 - done

## Resumo

`anotadinho-cli watch` fica aberto e emite uma linha JSON por mudança
no vault (JSONL), com `--folder` pra filtrar e `--property` pra já
trazer o valor novo do campo. Fecha o último buraco do loop do agente:
antes ele só sabia o que perguntasse.

## Arquivos criados/modificados

- `crates/vault/src/watcher.rs` — `VaultEvent`, fila de eventos,
  `drain_events` + 3 testes
- `crates/vault/src/lib.rs` — exporta `VaultEvent`
- `crates/cli/{Cargo.toml,src/main.rs}` — subcomando `watch`
- `VaultAnotadinho/pages/produto/guia-agent-os.md` — seção nova com o
  exemplo de `watch | while read`

## Testes adicionados

- evento traz path relativo e tipo válido
- `drain_events` esvazia a fila
- arquivo que não é `.md` não gera evento

## Problemas encontrados

- Criar arquivo emite `created` + `modified`: são eventos diferentes de
  verdade, e juntá-los seria decidir pelo agente. O debounce só junta
  eventos IDÊNTICOS no mesmo lote.
- Ctrl+C sai com 130 (SIGINT padrão) em vez de 0 — converter exigiria
  dependência nova pra ganho nenhum.

## Notas para próximos ciclos

- Restam 163, 176 e 171.
