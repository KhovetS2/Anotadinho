---
id: "218"
titulo: "Conversa não afoga o renderizador"
status: done
criado: 2026-08-23
autor: humano
prioridade: alta
depende_de: ["216"]
estima_min: 90
agente_alvo: claude-opus-5
---

# Conversa não afoga o renderizador

## Objetivo

A janela travava durante uma execução do agente. Não era o app parado:
era o processo de renderização a 85–88% de CPU, e não voltava nem
depois do trabalho terminar.

## Critérios de aceite

- [x] Markdown das mensagens é renderizado uma vez por mudança da lista
- [x] O acompanhamento só pergunta enquanto há trabalho
- [x] A primeira volta sempre pergunta, pra recuperar trabalho que
      terminou com a tela fechada
- [x] Medido antes e depois, com a mesma conversa

## Comandos de validação

```bash
cargo test --workspace
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
node scripts/uitest/run.mjs
```

## Notas

### O diagnóstico veio de medição, não de palpite

| processo | CPU |
|---|---|
| `anotadinho` (backend) | 3,6% |
| `WebKitWebProcess` | **88,2%** |

O acompanhamento do ciclo 213 atualiza progresso e tempo decorrido de
segundo em segundo. Qualquer mudança de estado re-renderiza o
componente inteiro, e o componente chamava
`markdown_render::render` pra CADA mensagem. Numa conversa de 26 KB,
isso é 26 KB de markdown reparseados por segundo.

Vira espiral: as chamadas se acumulam mais rápido do que são atendidas,
e o renderizador não volta nem com o agente já parado — foi por isso
que continuou a 88% depois do fim, e a janela não respondia nem a JS.

As três perguntas repetidas no arquivo da conversa (12:03, 12:06, 14:48)
são consequência disso: a pessoa reenviava porque a tela não respondia.

### Medição depois

Mesma conversa (31 KB, 16 mensagens), agente de mentira:

- em repouso, conversa aberta: **1–7%**
- com trabalho rodando: **16–18%**, voltando a 0% no fim

### O `use_state` congelado, de novo

A primeira versão da trava lia `*ocupado` de dentro do closure do
intervalo. O handle de `use_state` capturado num closure fica CONGELADO
no valor de quando o efeito rodou: o laço lia `false` pra sempre e nunca
voltava a perguntar — a UI ficava sem mostrar "pensando" com o backend
rodando.

Agora é `use_mut_ref`. É o mesmo defeito dos ciclos 155, 157, 201 e 213,
e a regra está no `AGENTS.md`.

### AGENTS.md

Reescrito pra ser o que codex e opencode leem antes de trabalhar: como
falar com o vault pelo `anotadinho-cli` em vez de `cat`/`>`, o ciclo,
a validação (incluindo quando NÃO dá pra validar), as regras que não se
negociam, e a diferença entre spec, proposta e execução.

Cada comando e cada caminho citado no arquivo foi conferido contra o
binário e o disco.
