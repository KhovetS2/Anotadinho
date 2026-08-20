---
id: "166"
titulo: "Toolbar do embed sobrepõe os controles do próprio embed"
status: done
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
