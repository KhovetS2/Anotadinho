---
id: "019"
titulo: "Blocos embed: PDF, imagem, Mermaid (diagramas)"
status: pending
criado: 2026-08-05
autor: humano
prioridade: media
depende_de: ["018"]
estima_min: 60
agente_alvo: claude-sonnet
---

# Blocos Embed

## Objetivo

Suporte a blocos especiais inseridos via comando `/`:
- `/pdf` → input file que mostra o PDF
- `/img` → input file que mostra a imagem
- `/mermaid` → bloco de código com preview de diagrama

Assets salvos em `vault/assets/` e referenciados no Markdown.

## Critérios de aceite

- [ ] `/pdf` seleciona PDF e mostra embed
- [ ] `/img` seleciona imagem e mostra preview
- [ ] Arquivos copiados para `vault/assets/`
- [ ] Markdown gerado com referência `![alt](assets/file)`
- [ ] `/mermaid` renderiza diagrama (Mermaid.js via CDN)
- [ ] App continua compilando
