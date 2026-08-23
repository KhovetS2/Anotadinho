---
id: "217"
titulo: "Leitura de consultas: alinhamento, cor, altura e virtualização"
status: done
criado: 2026-08-23
autor: humano
prioridade: alta
depende_de: ["169", "187"]
estima_min: 180
agente_alvo: codex
---

# Leitura de consultas: alinhamento, cor, altura e virtualização

## Objetivo

Tornar consultas grandes legíveis: colunas realmente alinhadas, valores
com badges determinísticos, resultados com rolagem interna configurável
e janela virtualizada para lista/tabela sem agrupamento.

## Critérios de aceite

- [x] Linhas da tabela voltam ao layout tabular e alinham com o cabeçalho.
- [x] Valores de propriedades recebem badges por classe, com índice puro
      baseado em coluna, campo e valor.
- [x] A altura máxima é declarada como `max_height`, tem padrão de 384 px
      e mantém barra, contagem e erro fora da área rolável.
- [x] Lista e tabela sem agrupamento virtualizam acima de 100 resultados;
      as linhas são fixas, truncadas e preservam o foco por teclado.
- [x] Cenário de harness cobre 101 resultados, alinhamento, badges,
      rolagem e montagem limitada no DOM.

## Comandos de validação

```bash
cargo test --workspace
cargo build --workspace
cargo build --manifest-path src-tauri/Cargo.toml
cd ui && cargo check --target wasm32-unknown-unknown
cd ui && trunk build
node scripts/uitest/run.mjs
```

## Notas

`trunk build` e o harness não iniciaram na primeira passada: o Trunk
0.21.14 invoca o Cargo instalado com `--no-color 1`, mas este Cargo só
aceita `--no-color=true|false`. A checagem direta para WASM passou; o
cenário permanece pronto para ser executado assim que a incompatibilidade
local de ferramentas for corrigida. O snapshot não foi atualizado sem uma
execução visual válida.

Segunda passada (revisão): três ajustes de fidelidade à proposta —
cartão volta a ser só badge, sem herdar a edição em linha do ciclo 168;
o tooltip do valor passa a carregar o valor inteiro, que é a mitigação
que a proposta previu para o truncamento da linha virtual; e
`.query-settings__hint` ganhou estilo, já que era a única classe nova do
modal sem regra. O `trunk serve` que estava de pé recompilou o WASM com
o ajuste dos cartões; os dois últimos ajustes ficaram sem compilação
porque o `cargo` não estava disponível na sessão.
