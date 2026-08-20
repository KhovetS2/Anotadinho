---
id: "172"
titulo: "CLI observa mudanças no vault"
status: pending
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

- [ ] `anotadinho-cli watch` fica aberto e imprime uma linha por
      mudança, JSON por linha (JSONL, que é o formato que um agente
      consome em stream): `{path, kind: created|modified|deleted, ts}`
- [ ] `--folder` filtra por prefixo de path
- [ ] `--property status` também emite o valor novo do campo, lendo a
      página alterada (fecha o caso "quando a spec virar in-progress")
- [ ] Reaproveita o `VaultWatcher` do ciclo 012, sem um segundo
      mecanismo de observação
- [ ] Debounce: salvar uma vez não vira 3 eventos (editor grava
      arquivo temporário + rename em algumas plataformas)
- [ ] Ctrl+C encerra limpo, com código 0
- [ ] Teste com vault temporário: escrever um arquivo e ver o evento
      sair no stdout

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

Combina com o guia de agent-os: a seção de CLI ganha um exemplo de
`watch | while read` pro agente reagir.
