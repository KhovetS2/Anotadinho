---
title: Memória do Anotadinho
type: docs
tags:
- produto
- desempenho
date: 2026-10-23
---
# Memória do Anotadinho

Quanto o app ocupa num dia de uso, e como medir de novo.

## Como medir

```bash
node scripts/memoria/janela.mjs --porta 9323 --rodadas 5   # o app instalado
node scripts/memoria/janela.mjs --rodadas 5                # o app de dev (9223)
cargo build --release -p anotadinho-tui && python3 scripts/memoria/tui.py --rodadas 5
```

Cada rodada é um "dia":
- abrir 20 notas;
- digitar 2000 caracteres;
- buscar;
- abrir um projeto de 300 tarefas, trocando filtros, abas e o painel
  de detalhes;
- abrir uma conversa de 30 mensagens e o grafo;
- fechar as abas.

O script da janela cria as páginas no vault aberto, em
`pages/uitest-memoria/` e `pages/projetos/uitest-memoria/`, e as apaga no
fim, mesmo se falhar ou levar Ctrl+C. O da TUI usa um vault temporário.

O número a comparar é o **PSS**. O RSS (o que o monitor do sistema
mostra) conta as bibliotecas compartilhadas em cada processo, e somar o
do app com o do WebKit conta a GTK duas vezes.

## Primeira medida (2026-10-23, Fedora)

### TUI, release

| repouso | pico | fim | por rodada |
|---|---|---|---|
| 7,1 MiB | 10,6 MiB | 10,0 MiB | +0,1 MiB |

Estável a partir da 2ª rodada: 9,5 → 10,1 → 10,4 → 10,0 → 10,0.

### Janela, build de DESENVOLVIMENTO

Rust e WASM sem otimização: é um teto pessimista, e não a estimativa do
app instalado.

| processo | repouso (app recém-aberto) | pico em 6 rodadas |
|---|---|---|
| app (Rust) | 143 MiB | 145 MiB |
| página web (WebKit) | 255 MiB | 1364 MiB |
| rede (WebKit) | 38 MiB | 38 MiB |
| **total** | **436 MiB** | **1546 MiB** |

- **Quase tudo é a página web**, o processo do WebKit que roda a
  interface. O app em Rust fica parado em ~145 MiB.
- **O pico vem com todas as abas fechadas.** A janela cai na página
  inicial, que tem consultas sobre o vault inteiro: depois de fechar as
  abas, a página web sobe 300 a 500 MiB.
- **Não é um vazamento que só cresce.** Depois de abrir as notas, a
  memória ficou em ~600 MiB por três rodadas (+15 MiB por rodada) e
  saltou pra ~1,2 GB na 5ª. Num app já cheio, outra medida caiu de
  ~1,5 GB pra ~717 MiB na rodada seguinte: o WebKit devolve a memória
  quando o coletor de lixo resolve recolher, e não logo depois de cada
  ação.

### Janela, app instalado (release, `/usr/bin/anotadinho`)

Esta é a estimativa de verdade: 5 rodadas, com o app recém-aberto e o
de dev fechado.

| processo | repouso | durante o uso | pico |
|---|---|---|---|
| app (Rust) | 142 MiB | 141 MiB | 142 MiB |
| página web (WebKit) | 257 MiB | — | 1176 MiB |
| rede (WebKit) | 35 MiB | 35 MiB | 35 MiB |
| **total** | **433 MiB** | **480 a 750 MiB** | **1351 MiB** |

- **No repouso, release e dev dão quase o mesmo** (433 e 436 MiB). O
  peso é do WebKit, que é o mesmo nas duas builds, e não do nosso
  código.
- **Durante as ações** (notas, digitação, busca, projeto de 300
  tarefas, conversa, grafo), o total fica entre ~480 e ~750 MiB e sobe
  devagar: uns 50 MiB por rodada.
- **O pico vem da página inicial desenhada sem nenhuma aba**, como no
  dev. E ele CRESCE a cada rodada: 652 → 824 → 1048 → 1194 → 1341 MiB.
  É o principal suspeito de memória presa, porque cresce de forma
  monotônica nas cinco rodadas, ao contrário das outras ações.

**Pra um dia normal**, conte com ~0,5 a 0,8 GB pra janela e ~10 MiB pra
TUI. Voltar muitas vezes pra página inicial com consultas sobre o vault
inteiro é o que leva a janela acima de 1 GB.

## Depois do ciclo 482: o vazamento era o grafo

A suspeita do pico era a página inicial. A sonda
(`node scripts/memoria/sonda-pagina.mjs --porta 9323`) alterna duas
páginas 12 vezes e mede a memória da página web depois de cada ida,
contra um controle que alterna duas notas leves.

| visita | antes | depois |
|---|---|---|
| controle (nota ↔ nota) | ~0 MiB | ~0 MiB |
| página inicial | ~0,5 MiB | ~0 MiB |
| quadro de 300 tarefas | ~1 MiB | ~0,4 MiB |
| **grafo** | **+24,3 MiB** | **+1,8 MiB** |

A página inicial, as consultas, o embed de ações e abrir e fechar abas
ficaram no ruído, inclusive com 340 páginas a mais. O grafo 3D não era
destruído ao sair da página: o renderizador WebGL e o laço de animação
seguiam vivos em segundo plano. O pico aparecia depois, na página
inicial, só porque o "dia" visitava o grafo antes de fechar as abas.

O dia inteiro, no app instalado, depois da correção:

| | antes | depois |
|---|---|---|
| pico | 1351 MiB | 758 MiB |
| crescimento por rodada | +172 MiB | +12,6 MiB |
| fim de cada rodada | 652 → 1341 | 638 → 688 |

## Metas

Ordens de grandeza, não medidas de outros apps. Uma janela Tauri vazia
no WebKitGTK costuma ficar entre 150 e 250 MiB.

- **Repouso**, app recém-aberto: até ~300 MiB. Hoje: ~433.
- **Uso do dia a dia**: até ~500 a 600 MiB. Hoje: ~630 a 700.
- **Por visita** a qualquer página: no ruído. É a meta que mais importa,
  porque é ela que separa "pesado" de "vazando".
- **TUI**: até ~30 MiB. Hoje: ~10.

O próximo corte é o processo do app em Rust: ~145 MiB parado, e parte
disso é biblioteca compartilhada contada no PSS. Separar o que é nosso
pede olhar a memória que é só dele (USS).

## O que olhar se o número incomodar

- Abrir a página inicial com consultas sobre o vault inteiro é a ação
  mais cara.
- Se o crescimento por rodada for grande no instalado, a próxima
  pergunta é qual ação deixa memória presa. O relatório traz o fim de
  cada ação por rodada pra isso.
