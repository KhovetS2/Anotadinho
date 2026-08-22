---
title: Ciclo 166 — Toolbar do embed sobrepõe os controles do próprio embed
type: ciclo
ciclo: "166"
status: concluida
date: 2026-08-20
prioridade: alta
depende_de: ["159"]
tags:
- ciclo
---

# Ciclo 166 — Toolbar do embed sobrepõe os controles do próprio embed

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Toolbar do embed sobrepõe os controles do próprio embed

## Objetivo

Pedido do usuário, com print: a toolbar de segmento criada no ciclo 159
(mover ↑↓, duplicar, remover) fica em `position: absolute; top: sp-2;
right: sp-2` — DENTRO da caixa do embed, exatamente em cima da barra de
controles que quase todo embed tem no próprio canto superior direito.
No cronograma ela cobre o botão "Trimestre"; na galeria cobre
"+ imagem"; na consulta, o botão de configurar.

## Critérios de aceite

- [x] A toolbar sai de cima do conteúdo do embed: fica INTEIRA acima
      da caixa (`bottom: 100%`), alinhada à direita. Centrar na borda
      (`top: -14px`, a primeira tentativa) não bastou — metade dela
      ainda cobria a primeira linha de controles, que começa logo
      depois do padding da caixa
- [x] Nenhum controle dos 9 embeds fica coberto — conferido por
      geometria, comparando o retângulo da toolbar com o de TODO
      controle focável de cada embed (0 sobreposições no `painel.md` e
      no `exemplos-embeds.md`; antes eram 9):
      kanban (+ coluna), calendário (fonte/visões), tabela, callout
      (recolher), colunas, galeria (+ imagem), consulta (configurar),
      cronograma (escala/fonte), ações
- [x] A toolbar não colide com o "+" de adicionar linha (que é
      centralizado) em largura de editor estreita — se colidir, o "+"
      cede
- [x] Continua aparecendo só em `:hover`/`:focus-within` do embed
- [x] Embed que é o PRIMEIRO segmento da página não fica com a toolbar
      cortada fora da área visível
- [x] Validação ao vivo (MCP `tauri`) no `painel.md`, que tem 5 tipos
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

`cd ui && cargo test --lib`: 26. `trunk build` e `cargo build
--manifest-path src-tauri/Cargo.toml`: OK.

A conferência foi por medida, não a olho: um script no console compara
o retângulo da toolbar com o de cada `button`/`input`/`[tabindex]` do
embed. Antes: 9 sobreposições (título do callout, 3 botões de
configurar consulta, "Trimestre" e "Vault" do cronograma, e os 3
controles de coluna). Depois: zero, nas duas páginas de teste.

Embed como PRIMEIRO segmento: a toolbar continua dentro da área
visível (o editor tem padding no topo), conferido com uma página que
começa com callout.

Alternativa considerada e descartada: empurrar a barra interna de cada
embed pra baixo quando o wrapper está em hover. Seria uma regra de CSS
por tipo de embed, e quebraria de novo a cada embed novo — a toolbar
sair da caixa resolve pros nove de uma vez.

## Resultado

# Ciclo 166 - done

## Resumo

A toolbar de segmento (ciclo 159) ficava dentro da caixa do embed, no
canto superior direito — exatamente onde quase todo embed tem a própria
barra de controles. Cobria "Trimestre" e "Vault" no cronograma, o botão
de configurar nas 3 consultas, o título do callout e os controles de
largura das colunas. Agora ela flutua inteira ACIMA do embed.

## Arquivos criados/modificados

- `ui/src/styles/main.css` — `.embed-hover-wrapper__toolbar` com
  `bottom: 100%`; media query pro "+" de cima ceder em editor estreito

## Testes adicionados

- Nenhum automático (é geometria de CSS). A conferência foi medindo os
  retângulos no console: 9 sobreposições antes, 0 depois.

## Problemas encontrados

- Centrar a toolbar na linha da borda (`top: -14px`) não resolveu:
  metade dela ainda cobria a primeira linha de controles. Precisou sair
  inteira da caixa.

## Notas para próximos ciclos

- Em editor estreito (<560px) o "+" de adicionar linha de cima some pra
  não colidir com a toolbar; o de baixo continua lá.
