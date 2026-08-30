---
id: "245"
titulo: "Arrastar imagem de fora: não derrubar o app, e inserir de verdade"
status: done
criado: 2026-08-30
autor: agente
prioridade: alta
depende_de: ["242"]
estima_min: 120
---

# 245 — Arrasto vindo do sistema

## Objetivo

Dois defeitos encontrados usando: soltar uma imagem um pouco fora do
editor **derrubava o app**, e soltar dentro não inseria nada.

## Critérios de aceite

- [x] Um drop em qualquer lugar da janela nunca navega o webview
- [x] O arrasto do sistema, que chega como `text/uri-list`, insere
- [x] Caminho com espaço (`%20`) é decodificado
- [x] Arquivo sem tipo MIME é aceito pela extensão
- [x] Cenário cobre o caminho de URI, não só o de `File`
- [x] O snapshot deixa de depender da largura da janela

## Comandos de validação

```bash
cargo test --workspace
cargo build --manifest-path src-tauri/Cargo.toml
node scripts/uitest/run.mjs
```
