---
id: "163"
titulo: "Modal de configuração de botão do embed de ações"
status: done
criado: 2026-08-19
autor: agente
prioridade: media
depende_de: ["156"]
estima_min: 75
agente_alvo: claude-sonnet
---

# Modal de configuração de botão do embed de ações

## Objetivo

O ciclo 156 entregou o embed `{{ type: "actions" }}` funcionando, mas
adicionar ou reconfigurar um botão só dá escrevendo o YAML na mão. Pra
um agente isso é o caminho certo; pra quem monta o painel pelo app,
não. Este ciclo dá a interface: escolher a ação numa lista e preencher
só os campos que aquela ação usa.

## Critérios de aceite

- [x] Botão "+ ação" no embed abre um modal (padrão
      `embeds/query_settings_modal.rs`)
- [x] Seletor de ação (`new-from-template`, `open-page`,
      `set-property`, `run-search`) que mostra só os campos daquela
      ação — hoje são 6 campos possíveis, e mostrar todos sempre é o
      que tornou a interface inviável no ciclo 156
- [x] `new-from-template`: seletor de template lendo
      `api::list_templates` e de pasta lendo `api::list_folders`
- [x] `open-page`/`set-property`: seletor de página lendo
      `api::list_pages`
- [x] Ícone de engrenagem no hover do botão reabre o modal com aquele
      botão carregado (não Alt+clique: um modificador escondido não se
      descobre sozinho)
- [x] Label, ícone (lista dos nomes de `icon.rs`) e variante
      (primário/fantasma) configuráveis
- [x] Botão com ação desconhecida (escrita por versão futura) continua
      desabilitado e NÃO perde o YAML ao ser editado pelo modal
- [x] Round-trip: configurar pelo modal e reabrir a página traz a
      mesma configuração

## Comandos de validação

```bash
cargo build --workspace
cargo test --workspace
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Ações novas além das 4 existentes (a lista é fechada por segurança —
  ver ciclo 156)
- Reordenar botões por arraste

## Notas

Harness (177): 13/13, com cenário que abre o modal, confere que SÓ os
campos da ação escolhida aparecem, cria o botão e verifica que ele
chega no `.md`.

O modal grava a cada campo alterado (sem "aplicar"), igual ao de
consulta do ciclo 154 — some um passo e mantém o embed sempre
espelhando o que está na tela.

Separado do ciclo 156 porque lá o embed já ficou utilizável e o ciclo
160 (painel) consome o YAML direto. Escopo declarado explicitamente
como não entregue no status do 156.
