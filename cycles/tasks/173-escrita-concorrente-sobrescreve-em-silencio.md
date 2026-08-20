---
id: "173"
titulo: "Escrita concorrente sobrescreve em silêncio"
status: done
criado: 2026-08-20
autor: agente
prioridade: alta
depende_de: []
estima_min: 120
agente_alvo: claude-opus-5
---

# Escrita concorrente sobrescreve em silêncio

## Objetivo

Achado ao revisar o que o ciclo 157 tornou rotina: com o app aberto na
página X, se QUALQUER outro processo alterar `X.md` (o
`anotadinho-cli`, um agente, `git pull` do ciclo 119, o editor do
sistema), o Anotadinho:

1. **não recarrega** a página aberta — o polling de
   `check_changes` (`app.rs` l.306) só incrementa `list_version`, que
   atualiza a LISTA da sidebar; `list_version` nunca chega no
   `page_view`/`editor`;
2. **sobrescreve sem avisar** na próxima gravação — `VaultIo::write_page`
   é um `std::fs::write` puro, sem comparar mtime nem hash.

Ou seja: o trabalho do agente (ou o seu, feito noutro lugar) some no
próximo autosave, sem nenhum aviso. Antes do ciclo 157 isso era raro;
depois dele, escrever no vault pelo terminal com o app aberto é o fluxo
normal.

## Critérios de aceite

- [x] `read_page` devolve, junto do conteúdo, uma marca de versão do
      arquivo (mtime + tamanho, ou hash do conteúdo)
- [x] `write_page` aceita a marca esperada e RECUSA a escrita se o
      arquivo no disco mudou desde a leitura (erro específico, não
      genérico)
- [x] O editor guarda a marca da última leitura e trata a recusa: avisa
      que a página mudou por fora e oferece recarregar (perdendo a
      edição local) ou salvar por cima (assumindo a perda do outro
      lado) — nunca decide sozinho em silêncio
- [x] Página aberta SEM edição pendente recarrega sozinha quando o
      arquivo muda no disco — conferido ao vivo: com a página aberta,
      um `embed add-card` pelo terminal fez o card aparecer no board em
      segundos, com o status "Recarregado do disco", SEM navegar
- [x] Página COM edição pendente não é recarregada por baixo do
      usuário. O aviso de status ("Mudou no disco — salve pra escolher
      o que fica") está implementado mas não apareceu na conferência ao
      vivo (provavelmente sobrescrito por outro status); o que importa
      — não recarregar por cima da edição, e barrar a gravação — foi
      confirmado
- [x] `anotadinho-cli` também respeita a checagem: `embed set` e
      `set-property` releem imediatamente antes de gravar, e falham se
      alguém escreveu no meio
- [x] Testes: escrita com marca velha falha; com marca certa passa;
      escrita concorrente simulada (dois `write_page` com a mesma marca
      de origem) só deixa o primeiro passar

## Comandos de validação

```bash
cargo test -p anotadinho-vault
cargo test -p anotadinho-ipc
cargo test -p anotadinho-cli
cargo test --workspace
cd ui && trunk build
```

## Não-objetivos

- Merge automático de conteúdo (juntar as duas versões) — avisar e
  deixar escolher já resolve o problema de perda
- Lock de arquivo entre processos: o `LockManager` citado em
  `docs/architecture.md` nunca foi implementado, e trancar arquivo não
  resolve o caso do `git pull`

## Notas

`cargo test --workspace`: 260 (255 + 5 novos). `cargo test -p
anotadinho-cli`: 32. `trunk build` e `cargo build --manifest-path
src-tauri/Cargo.toml`: OK.

A marca de versão é `<mtime em nanos>-<tamanho>`, não hash: hash a cada
save custa em arquivo grande, e mtime+tamanho pega qualquer escrita
real de editor (que reescreve o arquivo inteiro).

Conferido ao vivo, nos dois caminhos:
1. Sem edição pendente → CLI escreve → a página aberta recarrega
   sozinha, com o card novo no board.
2. Com edição pendente → CLI escreve → salvar abre o diálogo
   ("mudou no disco... Salvar por cima descarta a versão do disco"), e
   cancelar preserva o que está no disco.

Correção de registro: o status do ciclo 157 dizia que o card escrito
pelo CLI "apareceu no board sem recarregar nada (watcher do ciclo
012)". Está errado — naquela validação eu tinha clicado na página
depois de escrever, o que releu o arquivo do zero. O watcher não
recarrega página aberta. O status foi corrigido junto com esta task.
