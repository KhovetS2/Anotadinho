---
id: "171"
titulo: "Índice persistente invalidado pelo watcher"
status: pending
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

- [ ] Índice guardado (SQLite, ao lado do que `crates/search` já usa)
      com `path`, `mtime` e o `PageIndexEntry` serializado
- [ ] `scan_vault` relê do disco só o que mudou (compara `mtime`), e
      devolve o resto do índice
- [ ] `VaultWatcher` (ciclo 012) invalida a entrada da página alterada
- [ ] Índice corrompido/ausente se reconstrói sozinho, sem erro pro
      usuário — é cache, não fonte da verdade
- [ ] Primeira abertura de um vault sem índice não fica mais lenta que
      hoje
- [ ] Teste com vault temporário grande (500+ páginas) medindo as duas
      chamadas: a primeira (fria) e a segunda (quente)
- [ ] O arquivo de índice não é versionado (entra no `.gitignore` do
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

Fazer só quando houver vault grande de verdade pra medir — otimizar sem
número é chute. A task existe pra o problema estar escrito quando o
número aparecer.
