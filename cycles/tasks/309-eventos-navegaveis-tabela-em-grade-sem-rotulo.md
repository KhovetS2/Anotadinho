---
id: "309"
titulo: "Eventos navegáveis no dia, a tabela em grade, e o embed sem rótulo"
status: done
criado: 2026-09-16
autor: agente
prioridade: alta
depende_de: ["308"]
estima_min: 180
---

# 309 — Eventos navegáveis, tabela em grade, embed sem rótulo

## O que motivou

Três pedidos juntos: chegar com o cursor nos eventos de um dia do
calendário (o 306 anotou que só o dia era destino), dar à tabela a cara
de grade, e tirar os rótulos `[kanban]`, `[table]`… de cima das caixas.

## Eventos navegáveis

O evento continua sem linha própria — quem o desenha é a semana, na
coluna do dia —, mas agora é destino. `tela::fica_fora_da_tela` segue
dizendo o que não vira linha; `tela::cursor_passa_por_cima` diz o que o
cursor pula, e deixa o evento de fora. Enter no dia pousa no primeiro
evento (pulando faixa vazia e continuação de barra), `j`/`k` andam
entre os eventos do dia, Backspace volta pro dia. Dia sem evento não
deixa entrar.

`Linha::mostra` passa a valer também pro que está DENTRO de um
segmento: sem isso o cursor num evento não teria linha, e a rolagem e a
lateral ficariam sem alvo — o defeito do ciclo 298, por outra porta. O
evento sob o cursor acende na grade.

Limite: o cursor chega no evento pelo dia em que ele COMEÇA na semana.
No meio de uma sprint, o dia só tem a continuação da barra, e Enter não
entra.

## A tabela em grade

A janela desenha a borda de baixo de cada `<tr>`. A tabela ganhou borda
nos quatro lados, `│` entre as colunas com um espaço de respiro, régua
dupla (`╞═╪═╡`) depois do cabeçalho e régua simples entre as linhas.
Cada linha rende sua régua junto (o grupo é indivisível), o cabeçalho
abre a grade e a última linha a fecha.

Num painel estreito a coluna mais larga cede até caber, e o texto sai
cortado com `…` — a borda direita não some. A pílula que não cabe
inteira sai cortada, e as seguintes ficam de fora.

## Sem rótulo

Com toda caixa pintada na cor do tipo (308), o rótulo dizia duas vezes
a mesma coisa, e a janela não tem rótulo. O embed aberto não desenha
mais a linha dele. Fechado, o rótulo volta (é a única identidade que
sobra); e um embed sem nenhuma linha de conteúdo o mantém, senão não
haveria caixa.

A linha do rótulo era onde o cursor pousava no embed. Agora, com o
cursor no embed, a lateral de TODAS as linhas da caixa acende.

## Critérios de aceite

- [x] Enter num dia pousa no primeiro evento; `j`/`k` entre eventos
- [x] Faixa vazia e continuação de barra são puladas
- [x] O evento sob o cursor acende e é considerado visível pela rolagem
- [x] A tabela é grade fechada, com réguas alinhadas às bordas
- [x] Tabela larga cabe no painel, cortando texto
- [x] Embed aberto sem rótulo; fechado ou vazio com rótulo
- [x] Cursor no embed acende a caixa inteira

## Comandos de validação

```bash
cargo test --workspace
cargo clippy -p anotadinho-tui --all-targets
```
