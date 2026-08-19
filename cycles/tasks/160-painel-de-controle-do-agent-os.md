---
id: "160"
titulo: "Painel de controle do agent-os"
status: pending
criado: 2026-08-19
autor: humano
prioridade: alta
depende_de: ["151", "154", "155", "156"]
estima_min: 90
agente_alvo: claude-sonnet
---

# Painel de controle do agent-os

## Objetivo

Último ciclo da série. Monta com os embeds novos a interface que o
esquema de agent-os nunca teve: uma página inicial que mostra o estado
real do vault e opera o fluxo do guia sem ninguém tocar em markdown
cru. É também o teste de integração de verdade dos ciclos 151-156 —
com conteúdo real, não fixture.

## Critérios de aceite

- [ ] `VaultAnotadinho/pages/produto/painel.md` (novo, `type: landing`,
      definida como página de início) contendo:
      - callout de orientação, linkando o [[Guia do Agent OS]]
      - `actions` com "Nova spec", "Nova decisão", "Novo padrão",
        "Sessão de hoje" (cada um apontando pro template e pasta
        corretos do esquema)
      - `columns` com duas queries lado a lado: specs em
        `in-progress` e specs em `backlog` ordenadas por prioridade
      - `query` de decisões recentes, view `cards`
      - `timeline` em `source: vault` mostrando as specs com data
- [ ] `guia-agent-os.md` ganha uma seção "Painel" descrevendo a página
      e o que cada bloco faz, e o fluxo recomendado passa a começar
      pelo painel
- [ ] `docs/design-system.md` documenta os componentes criados em
      151-156 (classes BEM, variantes, tokens usados)
- [ ] `README.md` do repo menciona os tipos de embed disponíveis
- [ ] Validação de ponta a ponta ao vivo (MCP `tauri`): clicar "Nova
      spec" cria a página pelo template → ela aparece sozinha na query
      de backlog → mudar `status` pra `in-progress` pelo painel de
      propriedades a move pra outra coluna → a spec com data aparece
      na timeline → nada disso exigiu editar markdown na mão
- [ ] O mesmo recorte visto no painel sai igual no terminal via
      `anotadinho-cli query --from-embed pages/produto/painel.md <idx>`

## Comandos de validação

```bash
cargo build --workspace
cargo test --workspace
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Trocar o [[Roadmap]] (kanban manual) pelo painel — os dois convivem;
  o roadmap é ordenação intencional, a query é estado derivado
- Onboarding/tour da interface
- Criar o painel automaticamente ao abrir um vault vazio — este ciclo
  entrega a página no vault de exemplo, não um gerador

## Notas

Se algum embed da série não sobreviver ao uso real aqui, o conserto
vira task nova (regra de isolamento do `cycles/README.md`), não um
remendo dentro deste ciclo.
