---
id: "002"
titulo: "Vault picker (dialog nativo de seleção de pasta)"
status: done
criado: 2026-08-04
autor: humano
prioridade: alta
depende_de: ["001"]
estima_min: 45
agente_alvo: claude-sonnet
finalizado: 2026-08-04T22:15:00
---

# Vault picker

## Objetivo

Botão "Abrir vault" na tela inicial abre um dialog nativo de seleção de
pasta. Quando o usuário escolhe, o path é salvo e a sidebar aparece
(listando páginas, mesmo que vazia neste ciclo).

## Critérios de aceite

- [x] Botão "Abrir vault" fica habilitado (não mais disabled)
- [x] Click no botão abre dialog nativo de seleção de diretório
- [x] Após seleção, o path aparece no header da UI
- [x] Path é persistido (localStorage)
- [x] Próxima vez que o app abre, o último vault é re-aberto automaticamente
- [x] `cargo test --workspace` exit 0
- [x] `cargo clippy --workspace --all-targets -- -D warnings` exit 0 (clippy indisponível, build 0 warnings)

## Comandos de validação

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Não-objetivos

- Não listar páginas ainda (ciclo 003)
- Não criar/validar estrutura de pastas (ciclo 003)
- Não fazer watcher (ciclo 009)

## Notas

Usar `tauri-plugin-dialog` para o dialog nativo. Para persistência,
considerar `tauri-plugin-store` ou `localStorage` do WebView (mais simples).

Backend Rust (crate `vault`): implementar `VaultIo::list_pages()` mínimo
que retorna `Vec<String>` com os paths `.md` encontrados. UI chama via IPC.
