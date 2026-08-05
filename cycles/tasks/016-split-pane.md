---
id: "016"
titulo: "Editor split-pane: edição + preview ao vivo lado a lado"
status: done
criado: 2026-08-05
autor: humano
prioridade: alta
depende_de: ["015"]
estima_min: 45
agente_alvo: claude-sonnet
---

# Editor split-pane com preview ao vivo

## Objetivo

O editor passa a mostrar dois painéis lado a lado:
- Esquerda: textarea de edição (Markdown)
- Direita: preview renderizado em tempo real (a cada digitação)

Remove o botão Visualizar/Editar (agora sempre visível).

## Critérios de aceite

- [x] Split pane horizontal (50/50) com arraste de redimensionamento
- [x] Preview atualiza em tempo real (oninput)
- [x] Botão Visualizar/Editar removido
- [x] App continua compilando e abrindo

## Comandos de validação

```bash
cargo test --workspace
cargo build --manifest-path src-tauri/Cargo.toml
cd ui && trunk build
```
