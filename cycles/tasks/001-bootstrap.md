---
id: "001"
titulo: "Bootstrap do projeto Tauri + Yew com tema dark"
status: done
criado: 2026-08-04
autor: humano
prioridade: alta
depende_de: []
estima_min: 30
agente_alvo: claude-sonnet
---

# Bootstrap do projeto Tauri + Yew com tema dark

## Objetivo

Projeto compila, abre uma janela nativa (Tauri) renderizando o frontend Yew
(WASM) com o tema dark + acentos azul/roxo já configurados. Mostra uma tela
de boas-vindas "Anotadinho" com o subtítulo "Selecione um vault para começar".

## Critérios de aceite

- [ ] `cargo build --workspace` exit 0
- [ ] `cargo test --workspace` exit 0 (todos os testes existentes passam)
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` exit 0
- [ ] Tauri config valida (`tauri.conf.json` sem erros)
- [ ] Frontend Yew builda pra WASM sem erros
- [ ] Tema dark com acentos azul (`#3B82F6`) e roxo (`#8B5CF6`) está aplicado
- [ ] Título "Anotadinho" aparece na janela
- [ ] Subtítulo "Selecione um vault para começar" centralizado
- [ ] Botão "Abrir vault" existe (disabled por enquanto, ciclo 002 implementa)

## Comandos de validação

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Não-objetivos

- Não implementar picker de vault (fica pro ciclo 002)
- Não criar lógica de I/O
- Não integrar com filesystem
- Não adicionar testes de UI automatizados (Playwright/WebDriver é overkill no MVP)

## Notas

Estrutura já está pronta (criada no commit inicial):

- `crates/core/`, `crates/vault/`, `crates/search/`, `crates/ipc/` (stubs compiláveis)
- `ui/` (Yew com tema dark, componente `EmptyState` mostra a tela de boas-vindas)
- `src-tauri/` (Tauri shell, comando `ping` de exemplo)

O trabalho deste ciclo é:
1. Verificar que tudo compila junto (`cargo build --workspace`)
2. Garantir que os testes passam
3. Garantir que clippy está limpo
4. Documentar como rodar (`cargo tauri dev`)
5. Se algo falhar, corrigir
