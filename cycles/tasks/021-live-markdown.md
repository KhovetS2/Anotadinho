---
id: "021"
titulo: "Markdown live formatting (character shortcuts as-you-type)"
status: done
criado: 2026-08-05
autor: humano
prioridade: alta
depende_de: ["020"]
estima_min: 50
agente_alvo: claude-sonnet
---

# Markdown live formatting

## Objetivo

Ao digitar markdown syntax, a formatação é aplicada inline:
- `# ` → heading
- `- ` ou `* ` → lista
- `> ` → citação  
- `**text**` → bold (remove os `**` e aplica bold)
- `*text*` → italic
- `` `code` `` → código inline

## Critérios de aceite

- [ ] `# ` + espaço converte linha para heading
- [ ] `- ` item vira lista
- [ ] `**texto**` detecta na digitação e aplica bold inline
- [ ] App continua compilando e abrindo
