---
id: "163"
titulo: "Modal de configuração de botão do embed de ações"
status: pending
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

- [ ] Botão "+ ação" no embed abre um modal (padrão
      `embeds/query_settings_modal.rs`)
- [ ] Seletor de ação (`new-from-template`, `open-page`,
      `set-property`, `run-search`) que mostra só os campos daquela
      ação — hoje são 6 campos possíveis, e mostrar todos sempre é o
      que tornou a interface inviável no ciclo 156
- [ ] `new-from-template`: seletor de template lendo
      `api::list_templates` e de pasta lendo `api::list_folders`
- [ ] `open-page`/`set-property`: seletor de página lendo
      `api::list_pages`
- [ ] Clicar num botão existente com Alt (ou um lápis no hover) reabre
      o modal pra editar aquele botão
- [ ] Label, ícone (lista dos nomes de `icon.rs`) e variante
      (primário/fantasma) configuráveis
- [ ] Botão com ação desconhecida (escrita por versão futura) continua
      desabilitado e NÃO perde o YAML ao ser editado pelo modal
- [ ] Round-trip: configurar pelo modal e reabrir a página traz a
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

Separado do ciclo 156 porque lá o embed já ficou utilizável e o ciclo
160 (painel) consome o YAML direto. Escopo declarado explicitamente
como não entregue no status do 156.
