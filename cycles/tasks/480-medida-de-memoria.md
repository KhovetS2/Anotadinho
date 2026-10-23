---
id: "480"
titulo: "Medida de memória: um dia de uso na janela e na TUI"
status: done
criado: 2026-10-23
autor: agente
prioridade: media
depende_de: ["479"]
estima_min: 150
---

# 480 — Medida de memória

Quinta e última etapa da proposta
`pages/propostas/tarefa-rica-home-e-memoria.md`.

## Critérios de aceite

- [x] `scripts/memoria/janela.mjs`:
  - acha o app pela porta da ponte (9223 no dev, `--porta 9323` no
    instalado) e soma o app e os processos do WebKit, por papel;
  - amostra PSS, RSS e USS a cada 500 ms;
  - dirige um "dia" em rodadas: 20 notas, digitar 2000 caracteres,
    buscar, projeto de 300 tarefas (filtros, painel, abas, 3 voltas da
    releitura), conversa de 30 mensagens, grafo e fechar as abas.
- [x] `scripts/memoria/tui.py`: o mesmo dia num pty, num vault
      temporário, só com a biblioteca padrão. Recusa binário mais velho
      que o código.
- [x] O relatório traz repouso, pico, fim e crescimento por rodada, por
      processo, mais a build medida (RF17, RF18).
- [x] Nada fica no vault: `finally` e Ctrl+C removem as páginas `uitest-*`
      (RNF4).
- [x] A primeira medida vai no status e numa página proposta,
      `pages/produto/memoria.md`.

## Comandos de validação

```bash
node scripts/memoria/janela.mjs --rodadas 5
cargo build --release -p anotadinho-tui && python3 scripts/memoria/tui.py --rodadas 5
```
