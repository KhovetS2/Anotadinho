---
id: "201"
titulo: "Protocolo de fluxo: spec, proposta e execução"
status: done
criado: 2026-08-21
autor: humano
prioridade: alta
depende_de: [200]
estima_min: 180
agente_alvo: claude-opus
---

# Protocolo de fluxo

## Objetivo

Primeiro passo do agent-os conversacional: transformar em DADO o fluxo
que este repositório já pratica à mão (`cycles/tasks/` e
`cycles/status/`). Nada executa nada aqui — é só o protocolo.

## Critérios de aceite

- [x] `crates/core/src/fluxo.rs`: `Etapa`, `Artefato`, transições.
- [x] Não existe caminho de rascunho pra execução sem passar por revisão.
- [x] Embed `{{ type: "fluxo" }}` desenha a trilha e oferece SÓ as
      transições que o core permite.
- [x] Avançar espelha o valor em `status:` do frontmatter, que é o campo
      que as consultas filtram.
- [x] Etapa depois da aprovação avisa que é fechada pra edição
      automática (`agente_pode_preparar`).
- [x] 6 cenários de harness.

## Comandos de validação

```bash
cargo test --workspace
cd ui && cargo test && trunk build
node scripts/uitest/run.mjs
```

## Não-objetivos

- Executar qualquer coisa (ciclo 202).

## Três bugs meus, no caminho

1. **Tipo novo sem registro no parser.** `EmbedKind::Fluxo` entrou em
   `all()` e `type_name()` mas não em `from_type_name`: o embed
   aparecia no menu, gravava no arquivo, e sumia ao reabrir. Não havia
   teste de round-trip sobre `all()` — agora há, mais dois derivados da
   mesma lista.
2. **Dois escritores no mesmo arquivo.** O embed gravava o frontmatter
   direto no disco (com o conteúdo VELHO) enquanto o editor tinha a
   versão nova em memória; quem escrevesse por último apagava o outro.
   O embed passou a PEDIR ao editor.
3. **Estado congelado no mesmo tick.** `on_change` e `on_set_property`
   liam o mesmo `content_md` capturado, e o segundo `set` apagava o
   primeiro — a etapa não avançava. Resolvido lendo do
   `pending_flush_ref`, que o editor mantém fresco. É o mesmo padrão de
   bug dos ciclos 155 e 157.
