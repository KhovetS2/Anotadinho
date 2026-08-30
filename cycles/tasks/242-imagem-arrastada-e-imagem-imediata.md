---
id: "242"
titulo: "Imagem arrastada de fora, e imagem que aparece na hora"
status: done
criado: 2026-08-30
autor: agente
prioridade: alta
depende_de: ["226"]
estima_min: 60
---

# 242 — Imagem arrastada de fora, e imagem que aparece na hora

## Objetivo

Dois relatos do uso real: arrastar uma imagem de fora não inseria nada, e
a inserida por `/imagem` só aparecia depois de trocar de página.

## Critérios de aceite

- [x] O webview volta a receber o arrasto do sistema
- [x] A imagem inserida é resolvida na hora, sem recarregar
- [x] O caminho relativo continua guardado, e o markdown não leva data URL

## Comandos de validação

```bash
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
node scripts/uitest/run.mjs
```

## Não-objetivos

- Arrastar vídeo, áudio ou PDF
