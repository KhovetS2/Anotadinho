---
id: "108"
titulo: "Cheatsheet de atalhos"
status: pending
criado: 2026-08-08
autor: humano
prioridade: baixa
depende_de: ["105"]
estima_min: 60
agente_alvo: claude-sonnet
---

# Cheatsheet de atalhos

## Objetivo

Quinto e último ciclo do tema "navegação 100% via teclado" — capstone
de usabilidade. Com `GlobalKeymap` (105) e `VimKeymap` (092) sendo os
dois mapas de teclas customizáveis do app, este ciclo dá um jeito
rápido de VER todos os binds atuais de uma vez, sem precisar abrir os
dois modais de configuração separadamente.

## Critérios de aceite

- [ ] Tecla `?` (fora de qualquer campo de texto/contenteditable) abre
      um overlay listando as ações + teclas atuais dos dois keymaps
      (`GlobalKeymap` sempre; `VimKeymap` só se o vim mode estiver
      ativado), lado a lado ou em duas seções
- [ ] Também acessível via item "Atalhos (?)" no menu ⚙ e via comando
      na paleta ("Ver atalhos")
- [ ] `Escape` fecha o overlay
- [ ] `cargo test --workspace`, `cd ui && cargo test --lib`,
      `trunk build`, `cargo build --manifest-path src-tauri/Cargo.toml`
      passam

## Comandos de validação

```bash
cargo test --workspace
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Editar atalhos direto do cheatsheet — é só leitura; pra mudar, ainda
  usa os modais de configuração dedicados (`GlobalKeymap`/`VimKeymap`)
- Busca/filtro dentro do cheatsheet — lista curta o bastante pra não
  precisar nesta v1

## Notas

Puramente de leitura sobre dado que já existe (`labeled_fields()` dos
dois keymaps) — não deveria precisar de nenhuma mudança de modelo de
dados, só um componente novo de apresentação. Bom ciclo pra fechar o
tema com baixo risco depois dos ciclos mais estruturais (104-107).
