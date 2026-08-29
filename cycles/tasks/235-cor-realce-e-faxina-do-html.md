---
id: "235"
titulo: "Cor, realce e faxina do HTML"
status: done
criado: 2026-08-29
autor: agente
prioridade: alta
depende_de: ["234"]
estima_min: 180
---

# 235 — Cor, realce e faxina do HTML

## Objetivo

O modelo escrevia texto colorido no `.md` e a pessoa não tinha como
produzir a mesma coisa — nem manter: digitar um caractere numa página
colorida apagava a cor no autosave seguinte, porque `<span>` caía no braço
genérico do `html_to_md`, que devolve só o texto.

Como cor entra por HTML inline, e HTML inline vai direto pro DOM sem
filtro nenhum, a mesma porta aceitava `<script>` e `onerror=`.

## Critérios de aceite

- [x] Paleta nomeada de 7 tons, para texto e para realce, em tokens do
      tema — a cor escolhida no escuro continua legível no claro
- [x] Cor personalizada por seletor do sistema
- [x] Texto e realce são independentes
- [x] Pintar de novo reaproveita o span em vez de aninhar outro
- [x] "Tirar a cor"
- [x] A cor sobrevive a salvar, reabrir e salvar de novo
- [x] `<script>` e `on*=` não chegam ao DOM
- [x] A faxina não come conteúdo legítimo: transclusão, âncora de bloco,
      imagem inserida e a caixinha da checklist continuam

## Comandos de validação

```bash
cargo test --workspace
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
node scripts/uitest/run.mjs
```

## Não-objetivos

- Allowlist completa de HTML
- Cor em embed
