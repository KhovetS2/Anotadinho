---
id: "226"
titulo: "Imagens persistidas e personalizáveis"
status: done
criado: 2026-08-28
autor: agente
prioridade: alta
depende_de: ["118"]
estima_min: 240
agente_alvo: codex
---

# Imagens persistidas e personalizáveis

## Objetivo

Executar a proposta aprovada [[Proposta de implementação — imagens persistidas e personalizáveis]], que substitui a abordagem anterior: `/imagem` e drop abrem um modal único antes de gravar; paste mantém o gesto direto; toda inserção cria assets novos e referências duráveis.

## Critérios de aceite

- [x] Backend valida e grava lotes de imagens sem sobrescrita e sem publicação parcial
- [x] Modelo no core serializa HTML semântico com alt, título, legenda, dimensões, alinhamento e proporção
- [x] O HTML de imagem faz round-trip sem perda ao salvar e reabrir
- [x] `/imagem` usa seletor nativo e abre modal preenchido; cancelamentos não produzem efeito
- [x] Drop simples e múltiplo abre o mesmo modal, preserva ordem e ignora não-imagens
- [x] Modal permite revisar, personalizar e remover cada imagem, com navegação por teclado
- [x] Paste de imagem grava asset novo e insere com valores padrão, inclusive no editor por bloco
- [x] Paste de texto continua no caminho nativo
- [x] Inserção usa Range, marca edição e uma confirmação múltipla forma uma alteração de undo
- [x] Nenhum `blob:` é persistido e assets relativos voltam a renderizar depois da reabertura
- [x] Harness cobre seletor, cancelamentos, drop, múltiplos, personalização, paste, undo, falha e reabertura

## Comandos de validação

```bash
cargo test --workspace
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
node scripts/uitest/run.mjs
```

## Não-objetivos

- Editar bytes da imagem (recorte, compressão ou redimensionamento físico)
- Suportar drop de vídeo, áudio ou PDF
- Baixar imagens informadas por URL
- Avançar a etapa de fluxo da spec ou da proposta em nome da pessoa

## Nota de coerência

A proposta aprovada substitui expressamente a abordagem anterior e registra ajustes ainda necessários na redação da spec. A execução segue as decisões aprovadas sem alterar a etapa de fluxo dos artefatos.
