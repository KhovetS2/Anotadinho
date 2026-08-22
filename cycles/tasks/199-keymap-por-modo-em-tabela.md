---
id: "199"
titulo: "Keymap por modo em tabela"
status: done
criado: 2026-08-21
autor: humano
prioridade: alta
depende_de: [197]
estima_min: 120
agente_alvo: claude-opus
---

# Keymap por modo em tabela

## Objetivo

Os bugs dos ciclos 194, 195 e 197 foram o MESMO defeito estrutural: uma
tecla tratada no modo errado. Cada atalho carregava sua própria condição
solta dentro de um `on_keydown` de centenas de linhas, e nada obrigava a
responder "isto vale em qual modo?".

O harness pega a ocorrência; a tabela mata a espécie.

## Critérios de aceite

- [x] `ATALHOS`: tabela de `Atalho { tecla, alt, modo, descricao }`.
- [x] `comando_vale()` — um lugar só respondendo "é comando aqui?".
- [x] Os handlers consultam a tabela em vez de repetir a condição.
- [x] Testes DERIVADOS da tabela: sem duplicata, todo atalho de bloco é
      de navegação, letra comum não é atalho, atalho só vale no próprio
      modo, seta só move bloco com Alt, precedência de modo, e toda
      entrada com descrição.
- [x] Comportamento inalterado — suíte inteira verde.

## Comandos de validação

```bash
cargo test --workspace
cd ui && cargo test && trunk build
node scripts/uitest/run.mjs
```

## Não-objetivos

- Mover os atalhos GLOBAIS (`app.rs`/`state.rs`) pra esta tabela: eles
  já têm um keymap configurável próprio, e misturar os dois seria outro
  ciclo.

## Notas

O teste `todo_atalho_de_bloco_e_de_navegacao` é o que fecha o buraco do
194: se alguém marcar um atalho de bloco como `Edicao`, ele volta a
disparar durante a digitação — e agora o `cargo test` reprova antes de
chegar no app.
