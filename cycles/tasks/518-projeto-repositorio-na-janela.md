---
id: "518"
titulo: "Janela: configurar o repositório do projeto e ver o que a tarefa mudou"
status: done
criado: 2026-10-25
autor: agente
prioridade: alta
depende_de: ["517"]
estima_min: 180
---

# 518 — O repositório na tela do projeto

A janela do que o 517 pôs no núcleo.

## Critérios de aceite

- [x] **"Configurar"** no cabeçalho abre um painel com o comando de
      verificação, o prazo em minutos, o branch por tarefa e o commit ao
      aceitar. Grava só no botão, e mantém o repositório e a pausa.
- [x] **Regras à vista no cabeçalho:** "verificação `cmd`" ou "sem
      verificação — vale o que o agente disser" (na cor de atenção),
      "branch por tarefa" e "commit ao aceitar".
- [x] **Selo no cartão:** ✓ teste ou ✗ teste, pelo `verificacao`.
- [x] **"Mudanças"** (ícone de branch) no cartão que já rodou pelo app
      (tem `base_git`) e nos detalhes, que ganham a seção "Repositório"
      (branch e verificação). O modal mostra a base, o branch, os
      arquivos com o estado e o diff colorido.
- [x] O painel abre com a config de AGORA. Na primeira versão, o
      rascunho copiado na abertura pegava a padrão.
- [x] Cenário (518): com um repositório git de verdade, confere regras,
      selos, o botão só com base, o diff, o painel e a gravação no
      `projetos.json`, e a config da pessoa volta inteira no fim.

## Comandos de validação

```bash
node scripts/uitest/run.mjs "(518)"
node scripts/uitest/run.mjs "(513)"
node scripts/uitest/run.mjs "(477)"
cargo build --manifest-path src-tauri/Cargo.toml
```
