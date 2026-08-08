---
id: "113"
titulo: "Sidebar Nova pagina reaproveita templates"
status: done
criado: 2026-08-08
autor: humano
prioridade: media
depende_de: ["100"]
estima_min: 45
agente_alvo: claude-sonnet
---

# Sidebar "Nova página" reaproveita templates

## Objetivo

Bug encontrado ao validar ao vivo o esquema de agent-os (templates
novos de spec/decisão/padrão/sessão): o botão "+" da sidebar (o
principal ponto de entrada pra criar página) tinha sua PRÓPRIA
implementação de criação de página, separada de `new_page_action` em
`app.rs` — nunca foi atualizado no ciclo 100, que só ligou o fluxo de
templates ao atalho Ctrl+N e à paleta de comandos. Resultado: clicar
"+" na sidebar pulava direto pro prompt de título, sem NUNCA oferecer
templates, mesmo com templates existindo no vault.

## Critérios de aceite

- [x] `ui/src/components/sidebar.rs`: `on_new_page` (botão "+" da
      sidebar) ganha a mesma lógica de `new_page_action`
      (`app.rs`) — checa `api::list_templates`, mostra
      `PendingDialog::Select` se houver templates, senão vai direto
      pro prompt de título (comportamento antigo, inalterado quando
      não há templates)
- [x] Lógica duplicada intencionalmente (não delegada a um callback de
      `app.rs`) — a Sidebar já gerencia seu próprio refresh
      (`refresh_tick`) e destaque do item criado (`selected_path`)
      independente do `list_version` de `app.rs`, mesmo padrão que as
      outras ações da Sidebar (criar pasta, mover página) já seguem
- [x] `cargo test --workspace`, `cd ui && cargo test --lib`,
      `trunk build`, `cargo build --manifest-path src-tauri/Cargo.toml`
      passam

## Comandos de validação

```bash
cargo test --workspace
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Unificar a lógica de criação de página da Sidebar com a de `app.rs`
  numa função/hook compartilhado — a duplicação já existia antes deste
  ciclo pras outras ações da Sidebar (criar pasta, mover, criar em
  pasta); consolidar é uma refatoração maior, fora do escopo de um fix
  pontual
- Suporte a template nos botões "+" por pasta (`make_on_new_page_in`,
  criar página DENTRO de uma pasta específica) — só o botão de
  nível superior "Nova página" foi corrigido; o de pasta é um fluxo
  hoje mais simples e não foi o ponto reportado

## Notas

Descoberto ao validar ao vivo (MCP `tauri`) o esquema de agent-os:
criei uma spec de teste clicando "+" na sidebar e o modal de escolha
de template nunca apareceu. Comparado com `app.rs:372`
(`new_page_action`, usado por Ctrl+N e pela paleta Ctrl+K), que já
tinha a lógica certa desde o ciclo 100 — só a Sidebar ficou pra trás.

Validado ao vivo depois da correção: botão "+" da sidebar agora abre
"Escolher template" com as 5 opções (`decisao`, `nota-de-reuniao`,
`padrao-codigo`, `sessao-de-trabalho`, `spec` + "Página em branco"),
título é pedido em seguida, página é criada com `{{title}}`/`{{date}}`
resolvidos, e o item novo aparece destacado na árvore da sidebar.

Também descoberto durante a mesma sessão de validação: o processo
Tauri de dev estava rodando um binário compilado ANTES da correção do
ciclo 112 (`{{date}}` em templates) — mudanças em `crates/*` exigem
reiniciar `./scripts/dev.sh` por completo, `trunk serve` sozinho só
cobre o frontend WASM. Reiniciado antes de validar; não é um bug, é o
comportamento já documentado em ciclos anteriores.
