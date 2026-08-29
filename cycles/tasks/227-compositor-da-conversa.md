---
id: "227"
titulo: "Compositor da conversa: espaço pra escrever, prompt que não se reinjeta"
status: done
criado: 2026-08-29
autor: agente
prioridade: alta
depende_de: ["224"]
estima_min: 120
---

# 227 — Compositor da conversa

## Objetivo

Devolver ao campo de escrever o espaço que a faixa de prompt padrão
tomava, e parar de escrever texto no campo sozinho ao trocar de conversa.

## Critérios de aceite

- [x] A pergunta inicial sabe a qual conversa pertence e é esquecida
      depois de usada — abrir outra conversa não reescreve o rascunho
- [x] A faixa fixa do prompt padrão sai; no lugar, um botão na linha de
      ações que abre a lista por cima do campo
- [x] Sem prompt no vault, o botão não aparece
- [x] Escolher um molde com variáveis mantém a lista aberta, porque é
      onde os campos vivem; sem variáveis, fecha
- [x] Abrir a lista não empurra o campo de escrever
- [x] `pages/prompts-default/` deixa de estar vazia
- [x] A validação de contexto do prompt reaproveita a varredura da
      abertura em vez de varrer o vault de novo
- [x] O cenário do ciclo 224 apaga só o que ele criou

## Comandos de validação

```bash
cargo test --workspace
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
node scripts/uitest/run.mjs
```

## Não-objetivos

- Mover o botão de anexos para a linha de ações (ele mora no cabeçalho,
  junto dos chips que ele controla)
- Menu por `/` dentro da textarea
- Escrever o conjunto completo de prompts padrão (fica para o 236)
