---
id: "233"
titulo: "Vault novo nasce completo"
status: done
criado: 2026-08-29
autor: agente
prioridade: alta
depende_de: ["223", "227"]
estima_min: 150
---

# 233 — Vault novo nasce completo

## Objetivo

Não existia "criar vault": só apontar para uma pasta. Apontar para uma
vazia funcionava — e abria um app sem sidebar, sem modelo, sem prompt e
sem nenhum sinal do que fazer. Pior: as pastas com significado são
esperadas pelo código e nunca criadas, então o fluxo inteiro ficava mudo.

## Critérios de aceite

- [x] `crates/core/src/semente.rs` guarda a estrutura e o conteúdo
- [x] `handle_criar_vault` cria pastas e arquivos e **nunca sobrescreve**
- [x] Botão "Criar vault novo" na tela inicial
- [x] Abrir uma pasta sem página nenhuma **oferece** preparar — pergunta,
      não faz
- [x] `anotadinho-cli init` semeia igual
- [x] `pages/inicio.md` é o guia: explica os tipos de página e o que dá
      pra pôr dentro delas
- [x] Um vault recém-semeado já tem prompt padrão descobrível e as quatro
      pastas do fluxo

## Comandos de validação

```bash
cargo test --workspace
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
node scripts/uitest/run.mjs
```

## Não-objetivos

- Assistente de várias etapas para criar vault
- Escolher quais partes semear
