---
id: "344"
titulo: "Paridade: propriedades da página, configurar botão e consulta"
status: done
criado: 2026-09-17
autor: agente
prioridade: alta
depende_de: ["343"]
---

# 344 — Propriedades, configurar botão e consulta

- `Formulario` ganhou `Valor::Opcoes` (girar com `~`/Enter/`l`, voltar com
  `h`).
- "Propriedades da página…" (barra de comandos) — o `PropertiesPanel`:
  título, tipo (os tipos da janela), tags, criado, atualizado e as
  propriedades livres como `chave: valor` (valor que não mudou mantém o
  tipo YAML). Núcleo: `MarkdownCodec::substituir_frontmatter`.
- `=` configura o item — o "configurar" da TUI:
  - num botão, o `ActionButtonModal`: rótulo, ícone, destaque, ação, e só os
    campos da ação escolhida (página; template e pasta; campo e valor;
    busca); "Excluir botão".
  - numa consulta, o `QuerySettingsModal`: em, tags, condições
    (`campo=valor`…, validadas), ordenar e decrescente, limite, visão,
    colunas, agrupar por, agregados (`count`, `sum:campo`…, validados).
- Núcleo: `Aggregate::parse`/`como_texto` e `Condition::como_texto`; o CLI
  usa o parser do núcleo.
