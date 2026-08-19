---
id: "156"
titulo: "Embed actions: botões de ação na página"
status: pending
criado: 2026-08-19
autor: humano
prioridade: alta
depende_de: ["148"]
estima_min: 120
agente_alvo: claude-sonnet
---

# Embed actions: botões de ação na página

## Objetivo

Todo embed até aqui MOSTRA coisas. Este FAZ coisas: transforma uma
página comum (em especial uma `type: landing`) num painel operável.
No fluxo do agent-os, criar uma spec hoje é: abrir a sidebar, escolher
a pasta certa, escolher o template certo, digitar o título. Com um
botão `new-from-template` isso vira um clique — e o mesmo painel serve
pro humano e pro agente, porque o que o botão faz está declarado em
YAML legível no `.md`.

## Critérios de aceite

- [ ] `EmbedKind::Actions` + `{{ type: "actions" }}`
- [ ] `ActionsEmbedData { layout: ActionsLayout (Row|Grid), buttons:
      Vec<ActionButton { label, icon, variant, action: ActionSpec }> }`
      com `ActionSpec` enum serde-tagged por `action:`:
      - `new-from-template { template, folder, title_prompt }` →
        `api::create_page_from_template` + navega pra página criada
      - `open-page { path }` → `on_page_selected`
      - `set-property { path, field, value }` → grava frontmatter pelo
        mesmo caminho do painel de propriedades (ciclo 099), sem
        duplicar a lógica de escrita
      - `run-search { query }` → abre a paleta de comandos já filtrada
- [ ] Componente `embeds/inline_actions.rs`: botões com ícone
      (`components/icon.rs`) e variante visual (primário/fantasma),
      layout linha ou grade
- [ ] Modo de edição do embed: adicionar/remover botão e configurar a
      ação num modal (padrão `column_settings_modal.rs`), com o
      seletor de template lendo `api::list_templates` e o de página
      lendo `api::list_pages`
- [ ] Ação que falha mostra o erro via `PendingDialog::Alert`, sem
      deixar o embed em estado inconsistente
- [ ] `set-property` numa página aberta noutra aba recarrega a página
      afetada (ou avisa que mudou), pra não sobrescrever com estado
      velho
- [ ] Botão é `<button>` de verdade: foco visível, Enter/Espaço
      ativam, `data-nav-item`/`data-nav-group`
- [ ] Testes de round-trip de cada variante de `ActionSpec`, incluindo
      uma ação desconhecida (arquivo escrito por versão futura ou por
      um agente) que degrada pra botão desabilitado com aviso, sem
      perder o YAML original

## Comandos de validação

```bash
cargo build --workspace
cargo test --workspace
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Rodar comando de shell / invocar agente externo a partir do botão —
  é a superfície de ataque óbvia (um `.md` de terceiro executando
  código ao abrir a página); as ações são um conjunto FECHADO de
  operações do próprio app
- Confirmação/undo de `set-property` além do histórico via git (117)
- Encadear várias ações num botão só

## Notas

A lista fechada de ações é decisão de segurança, não de escopo:
registrar isso na task pra não ser "melhorado" depois sem querer.
