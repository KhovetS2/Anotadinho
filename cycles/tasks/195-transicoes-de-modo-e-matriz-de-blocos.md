---
id: "195"
titulo: "Transições de modo e matriz de blocos no harness"
status: done
criado: 2026-08-21
autor: humano
prioridade: alta
depende_de: [194]
estima_min: 180
agente_alvo: claude-opus
---

# Transições de modo e matriz de blocos

## Objetivo

Três defeitos relatados pelo usuário, todos na TRANSIÇÃO entre modos, e
o harness completo que faltava pra essa área.

1. **Editar e navegar ao mesmo tempo.** Enter em navegação não saía do
   modo: a barra seguia dizendo NAVEGAÇÃO, as setas seguiam pulando de
   bloco, e a digitação entrava no bloco. Parecia "dois editores".
2. **`d` apagava o bloco e travava a navegação.** Depois de apagar não
   dava pra andar com as setas nem sair com Escape ou Backspace.
3. **A dica de bloco vazio aparecia no meio da escrita.**

## Critérios de aceite

- [x] Enter em navegação entra no bloco E sai do modo.
- [x] Depois de apagar/mover, o foco volta pra um bloco válido e a
      navegação continua respondendo.
- [x] Mover e depois editar escreve no bloco CERTO.
- [x] Dica de bloco vazio: só no bloco único da página, ou no hover do
      MOUSE — nunca por foco.
- [x] Balão flutuante de nav-mode removido (a barra de modo já informa).
- [x] `scripts/uitest/blocos.mjs`: matriz de navegação, movimentação,
      adição, edição e transições.

## Comandos de validação

```bash
cargo test --workspace
cd ui && cargo test && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
node scripts/uitest/run.mjs
```

## Não-objetivos

- Seleção de múltiplos blocos.

## Notas

**Causa do (1):** o handler de Enter do editor rodava também em
navegação e dava `stop_propagation`, então o Enter nunca chegava no
`app.rs`, que é quem encerra a sessão de navegação. Passou a ser gateado
por `!em_navegacao`.

**Causa do (2):** o re-render depois da mutação troca os nós do DOM, e o
`focus_item` feito durante a ação morria junto — o foco caía no
`<body>`, onde nem seta nem Escape do nav-mode têm em quê se ancorar.
Agora o foco é reancorado por ÍNDICE depois do re-render.

**Causa do (3):** o CSS usava `:only-child`, que conta filhos do
SEGMENTO. Numa página com embeds há vários segmentos, então um parágrafo
vazio no meio satisfazia a regra. A decisão passou pro Rust
(`marcar_convite`), que olha a página inteira.

**Bug achado pela matriz nova:** o refoco depois de mover usava o índice
capturado ANTES da ação. Ao subir um bloco, o foco pousava no vizinho
que tomou a posição antiga — e a digitação seguinte ia pro bloco errado.
`aplicar_acao_de_bloco` passou a devolver o índice pós-mutação.
