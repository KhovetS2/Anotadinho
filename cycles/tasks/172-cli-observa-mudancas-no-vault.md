---
id: "172"
titulo: "CLI observa mudanças no vault"
status: done
criado: 2026-08-20
autor: humano
prioridade: baixa
depende_de: ["157"]
estima_min: 75
agente_alvo: claude-opus-5
---

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
