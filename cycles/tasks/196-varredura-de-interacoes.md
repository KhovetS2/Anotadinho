---
id: "196"
titulo: "Varredura horizontal de interações no harness"
status: done
criado: 2026-08-21
autor: humano
prioridade: alta
depende_de: [195]
estima_min: 180
agente_alvo: claude-opus
---

# Varredura horizontal de interações

## Objetivo

Cobrir o máximo de interações que o app tem hoje, para uma mudança
grande não passar despercebida num canto que ninguém testava.

Os arquivos existentes cobrem em PROFUNDIDADE áreas específicas —
`digitacao.mjs` o texto, `blocos.mjs` os modos, `cenarios.mjs` as
regressões nomeadas. Faltava a varredura em LARGURA.

## Critérios de aceite

- [x] `scripts/uitest/interacoes.mjs` com 21 cenários.
- [x] Cobre: kanban (card, coluna, contagem), tabela (linha, coluna,
      célula), callout (recolher, variante), consulta (visão),
      cronograma (escala), colunas (painel), ações (abrir página),
      abas, sidebar (filtro), backlinks, propriedades, menu `/`
      (filtro e Escape), tema, desfazer de embed, paleta (Ctrl+K) e
      persistência de edição ao trocar de página.
- [x] Suíte inteira verde.

## Comandos de validação

```bash
node scripts/uitest/run.mjs interação
node scripts/uitest/run.mjs
```

## Não-objetivos

- Profundidade nas áreas que já têm arquivo próprio.

## Bug corrigido no caminho

**Título da página dependia do caminho por onde se chegava nela.** O
botão de ação `open-page` montava o `PageMeta` com o nome do ARQUIVO,
enquanto o clique em wikilink usava o título do frontmatter (corrigido
no 191). `missao.md` aparecia como "missao" pelo botão e "Missão" pelo
link. Agora os dois resolvem por `scan_vault`.

## Notas de método

Três "falhas" da primeira execução eram cenário mal escrito, não bug:
`.task-table__add` casa com "Nova coluna" E "+ linha"; o editor de
célula é `<textarea>` e não `<input>`; e "Nova coluna" abre um modal em
vez de adicionar direto. Registrado porque a lição vale: a primeira
suspeita de um cenário novo que falha deve ser o próprio cenário.

Ctrl+K e Ctrl+Z globais precisam ser disparados em `.app-root`, não em
`body` — o listener vive lá, e um evento no `body` sobe pro documento
sem passar por ele.
