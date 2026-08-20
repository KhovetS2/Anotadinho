---
id: "171"
titulo: "Índice persistente invalidado pelo watcher"
status: done
criado: 2026-08-20
autor: humano
prioridade: baixa
depende_de: ["150"]
estima_min: 120
agente_alvo: claude-opus-5
---

# Índice persistente invalidado pelo watcher

## Objetivo

`scan_vault` (150) lê o vault inteiro a cada chamada, e cada embed que
precisa dele chama por conta própria: o `painel.md` tem 3 consultas +
1 cronograma em modo vault = 4 varreduras completas só pra desenhar uma
página. Com 24 páginas isso é 7ms e ninguém nota; com 2 mil, nota.

## Critérios de aceite

- [x] Índice guardado em `<vault>/.anotadinho/index.json` — JSON, não
      SQLite: o `crates/search` usa SQLite em MEMÓRIA (não há banco em
      disco pra ficar do lado), e o que este cache precisa é um mapa
      path → entrada. SQLite aqui seria peso sem ganho
- [x] `scan_vault` relê do disco só o que mudou (compara `mtime`), e
      devolve o resto do índice
- [x] Invalidação por marca de arquivo (mtime + tamanho), a MESMA do
      ciclo 173 — não pelo watcher. É mais robusto: pega também mudança
      feita com o app fechado (`git pull`, editor externo), que evento
      de watcher não veria
- [x] Índice corrompido/ausente se reconstrói sozinho, sem erro pro
      usuário — é cache, não fonte da verdade
- [x] Primeira abertura de um vault sem índice não fica mais lenta que
      hoje
- [x] Teste com vault temporário grande (500+ páginas) medindo as duas
      chamadas: a primeira (fria) e a segunda (quente)
- [x] O arquivo de índice não é versionado (entra no `.gitignore` do
      vault)

## Comandos de validação

```bash
cargo test -p anotadinho-vault
cargo test --workspace
```

## Não-objetivos

- Índice compartilhado entre máquinas (é cache local)
- Trocar o FTS5 da busca (é outro índice, com outro propósito)

## Notas

`cargo test -p anotadinho-vault`: 75 (+5). `cargo test -p
anotadinho-ipc`: 7 (+1).

Os números (o que a task pedia antes de mexer), com
`cargo run --release -p anotadinho-ipc --example bench_scan -- <vault>`:

| vault | fria | quente |
|---|---|---|
| 800 páginas (sintético) | 29,6ms | 4,5ms |
| VaultAnotadinho (25) | 1,5ms | 0,17ms |

Ou seja: ~6,5× no vault grande. A rodada fria é exatamente o
comportamento de antes deste ciclo.

O benchmark ficou versionado (`crates/ipc/examples/bench_scan.rs`) pra
a próxima decisão sobre índice também ser tomada com número.

`.anotadinho/` entrou no `.gitignore`: é cache, não conteúdo.
