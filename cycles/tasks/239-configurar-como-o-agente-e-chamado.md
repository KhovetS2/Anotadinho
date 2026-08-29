---
id: "239"
titulo: "Configurar como o agente é chamado"
status: done
criado: 2026-08-29
autor: agente
prioridade: alta
depende_de: ["214", "237"]
estima_min: 150
---

# 239 — Configurar como o agente é chamado

## Objetivo

Não havia campo nenhum: dava pra trocar de preset e escolher pastas, e
só. Quem precisasse apontar outro executável — ou chamar um modelo que
não fosse claude, codex ou opencode — tinha que editar o `localStorage`
na mão. É o item B3 do diagnóstico de portabilidade, e o único que também
melhora o Linux de hoje.

## Critérios de aceite

- [x] Tela de agentes com nome, executável, argumentos, formato da saída,
      tempo limite e argumento de pasta extra
- [x] Criar agente novo, que aparece na lista depois de gravado
- [x] Remover agente criado; preset não se apaga
- [x] Renomear substitui, não duplica
- [x] Caminho com espaço é aceito
- [x] Linha de comando colada continua recusada, com o botão desabilitado
- [x] A configuração continua nas preferências, nunca no vault

## Comandos de validação

```bash
cargo test --workspace
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
node scripts/uitest/run.mjs
```

## Não-objetivos

- Descobrir o executável no PATH automaticamente
- Presets novos além dos três
