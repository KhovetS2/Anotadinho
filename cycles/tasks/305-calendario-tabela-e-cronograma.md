---
id: "305"
titulo: "Calendário, tabela e cronograma: da janela pra caixa preenchida no terminal"
status: done
criado: 2026-09-16
autor: agente
prioridade: alta
depende_de: ["304"]
estima_min: 240
---

# 305 — Calendário, tabela e cronograma

## O que motivou

Continuação do 304: os três embeds que faltavam ganhar tratamento de
CONTEÚDO (não só a cor da moldura, que os nove já tinham). O pedido
específico foi: tirar print de cada um na versão GUI e usar como base
pra trazer a mesma representação pro terminal.

## O print antes do código

Montei três fixtures HTML com as classes REAIS da janela
(`.calendar-grid__*`, `.task-table__*`, `.timeline__*`) e o CSS
compilado (`ui/dist/*.css`), com os mesmos dados da página de exemplos,
e tirei print headless (`playwright screenshot`, instalado só pra
isto — não tem MCP `tauri`/`playwright` nesta sessão). Três achados que
mudaram o desenho:

1. **Calendário é GRADE de mês**, com eventos como pílulas neutras (borda
   esquerda azul, fundo `--bg-elevated`) dentro de cada dia.
2. **Tabela é grade alinhada**, células `select`/`multiselect` em badge
   colorido (`badge_class`, 4 cores cíclicas).
3. **Cronograma é barra PROPORCIONAL** à duração — cor quase sempre
   `badge--info` na prática, porque `badge_class(&item.tags, tag)`
   usa as tags do PRÓPRIO item como opções, e um item de uma tag só
   cai sempre no índice 0. (Achado ao montar o fixture, não é bug
   deste ciclo — é como a janela já se comporta.)

Um mês inteiro em grade de 7 colunas não cabe no modelo atual (cada
unidade é uma ou mais LINHAS de terminal, não uma célula 2D) — isso é
trabalho de outro ciclo, do tamanho do 305 sozinho. O que entrou aqui
foi o que dá pra fazer com o vocabulário que já existe (`Arranjo`,
`Realce`, a caixa preenchida do 304): calendário virou LISTA de
eventos em selo colorido — mais perto do modo "agenda" que a própria
janela já tem pra semana/dia — e cronograma e tabela pegaram o
tratamento que o formato deles pede.

## Cronograma: retângulo proporcional

`bar_span` (núcleo, ciclo 167) já calculava posição e largura em
porcentagem — só a janela usava, escolhendo a janela de tempo pela
navegação (mês/semana corrente). TUI e CLI não têm "mês atual", então
a janela agora é o PRÓPRIO conjunto: do primeiro início ao último fim.

Início e duração viajam como duas partes-FILHAS em porcentagem
(`"inicio"`/`"duracao"`) — mas são dado de desenho, não conteúdo: não
aparecem na tela (`tela::e_geometria` as filtra da lista de linhas,
igual `achatar_fileiras` já fazia pros filhos de fileira) e Entrar
numa barra não desce pra elas (guarda em `tela::andar` — sem isso o
Enter pareceria não fazer nada, e só o Backspace devolveria).

`linha_de_caixa` (304) ganhou um terceiro modo, `Proporcional`, ao lado
de `Cheia` (cartão) e `Conteudo` (miniatura/evento) — mesmo contorno de
meio-bloco, só que com deslocamento e largura calculados da
porcentagem em vez de do texto.

## Tabela: grade alinhada, não fileira de botão

Cabeçalho e linha viraram FILEIRA (`Arranjo::Linha`), reusando o
achatamento em segmentos que o botão de ações já tinha — mas uma
tabela não quer caixa de 3 linhas por célula, quer texto alinhado numa
linha só. `linha_de_tabela` é o renderizador novo: calcula a largura
de cada COLUNA olhando cabeçalho e linhas juntos
(`larguras_de_tabela`) e alinha cada célula com `{:largura$}`.

A cor do badge (`select`/`multiselect`) vira o NOME da célula
(`badge_class` do núcleo, o mesmo que a janela usa) — pra
`multiselect` com várias tags, a cor vem da PRIMEIRA e o texto mostra
todas juntas num badge só; mostrar cada tag no seu próprio badge, como
a janela faz, fica pro próximo corte.

## Calendário: evento em selo

Sem grade de mês, cada `entry` vira uma caixa `Conteudo` (like a
miniatura da galeria) — cresce até caber "AAAA-MM-DD Título", não a
largura do painel: um evento é um PONTO, esticar feito cartão sugeriria
duração, que é o cronograma quem tem.

## Critérios de aceite

- [x] Prints da GUI tirados ANTES do código (fixture + `main.css`
      real + `playwright screenshot`)
- [x] Cronograma: barras proporcionais à duração, posicionadas pela
      data de início, sem estourar a largura do painel
- [x] Início/duração não aparecem como linha na tela nem são
      alcançáveis por Enter
- [x] Tabela: cabeçalho e linhas alinham em coluna, célula
      select/multiselect ganha cor de badge
- [x] Calendário: cada evento vira selo preenchido, do tamanho do
      conteúdo
- [x] `cargo test --workspace` sem quebrar nenhum teste existente
- [x] `cargo clippy -p anotadinho-tui --all-targets` e
      `-p anotadinho-core --all-targets` sem aviso novo

## Comandos de validação

```bash
cargo test --workspace
cargo clippy -p anotadinho-tui --all-targets
cargo clippy -p anotadinho-core --all-targets
```
