---
id: "166"
titulo: "Toolbar do embed sobrepõe os controles do próprio embed"
status: pending
criado: 2026-08-20
autor: humano
prioridade: alta
depende_de: ["159"]
estima_min: 30
agente_alvo: claude-opus-5
---

# Toolbar do embed sobrepõe os controles do próprio embed

## Objetivo

Pedido do usuário, com print: a toolbar de segmento criada no ciclo 159
(mover ↑↓, duplicar, remover) fica em `position: absolute; top: sp-2;
right: sp-2` — DENTRO da caixa do embed, exatamente em cima da barra de
controles que quase todo embed tem no próprio canto superior direito.
No cronograma ela cobre o botão "Trimestre"; na galeria cobre
"+ imagem"; na consulta, o botão de configurar.

## Critérios de aceite

- [ ] A toolbar sai de cima do conteúdo do embed: passa a flutuar na
      LINHA DA BORDA superior (mesma faixa onde já vive o "+" de
      adicionar linha, que usa `top: -11px`), alinhada à direita
- [ ] Nenhum controle dos 9 embeds fica coberto — conferir um a um:
      kanban (+ coluna), calendário (fonte/visões), tabela, callout
      (recolher), colunas, galeria (+ imagem), consulta (configurar),
      cronograma (escala/fonte), ações
- [ ] A toolbar não colide com o "+" de adicionar linha (que é
      centralizado) em largura de editor estreita — se colidir, o "+"
      cede
- [ ] Continua aparecendo só em `:hover`/`:focus-within` do embed
- [ ] Embed que é o PRIMEIRO segmento da página não fica com a toolbar
      cortada fora da área visível
- [ ] Validação ao vivo (MCP `tauri`) no `painel.md`, que tem 5 tipos
      de embed numa página só

## Comandos de validação

```bash
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Redesenhar as barras de controle internas dos embeds
- Mudar o que a toolbar faz (159 já definiu as 4 ações)

## Notas

Alternativa considerada e descartada: empurrar a barra interna de cada
embed pra baixo quando o wrapper está em hover. Seria uma regra de CSS
por tipo de embed, e quebraria de novo a cada embed novo — a toolbar
sair da caixa resolve pros nove de uma vez.
