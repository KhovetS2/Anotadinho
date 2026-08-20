---
id: "190"
titulo: "Aviso de mudança externa na página aberta"
status: pending
criado: 2026-08-20
autor: humano
prioridade: alta
depende_de: [150]
estima_min: 90
agente_alvo: claude-opus
---

# Aviso de mudança externa na página aberta

## Objetivo

`write_page_checked` já detecta que o arquivo mudou no disco desde a
leitura e grava um bloco de conflito — mas você só descobre DEPOIS, no
arquivo. Como o agente escreve pelo CLI enquanto a janela está aberta,
isso acontece de verdade. Este ciclo avisa antes, com o watcher que já
existe.

## Critérios de aceite

- [ ] O editor observa `VaultEvent` e compara com a `page_version` da
      leitura da página aberta.
- [ ] Mudança externa SEM edição local pendente: recarrega sozinho e
      mostra um aviso discreto e temporário.
- [ ] Mudança externa COM edição local pendente: barra fixa com
      "Recarregar (perde o que você escreveu)" / "Manter o meu" /
      "Ver a diferença".
- [ ] "Ver a diferença" abre um modal com o comparativo linha a linha.
- [ ] Salvar por cima continua passando por `write_page_checked` — a
      barra é aviso, não substitui a rede de segurança.
- [ ] Cenário de harness: escrever no arquivo por fora com edição
      pendente faz a barra aparecer; sem edição pendente, recarrega.

## Comandos de validação

```bash
cargo build --workspace
cargo test --workspace
cargo build --manifest-path src-tauri/Cargo.toml
cd ui && trunk build
node scripts/uitest/run.mjs
```

## Não-objetivos

- Merge automático de três vias.
- Estado de sincronia do vault INTEIRO (badge de git já cobre parte).

## Notas

Diff linha a linha no core (`crates/core/src/diff.rs`, LCS simples), pra
ser testável fora do WASM e reaproveitável pelo CLI depois.

O watcher já emite eventos desde o ciclo 157; o que falta é o editor
escutar. Atenção ao bug recorrente: handle de `use_state` capturado em
efeito/timer congela no valor de criação — usar `use_mut_ref`.
