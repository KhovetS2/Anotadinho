---
id: "214"
titulo: "Codex configurado, progresso inteiro e troca de agente"
status: done
criado: 2026-08-23
autor: humano
prioridade: alta
depende_de: ["213"]
estima_min: 120
agente_alvo: claude-opus-5
---

# Codex configurado, progresso inteiro e troca de agente

## Objetivo

Três arestas do ciclo 213:

1. Só o Claude estava configurado pra falar em tempo real; o Codex
   ficava no modo texto, sem progresso.
2. O painel de progresso guardava só a PRIMEIRA linha de cada trecho —
   justamente o miolo do raciocínio, que é o que diz se o agente
   entendeu o pedido, ficava de fora.
3. Não havia como trocar de agente: a configuração só se mudava
   editando o `localStorage` na mão.

## Critérios de aceite

- [x] Preset do Codex com `exec --json`, verificado contra o binário
      de verdade
- [x] O leitor de stream entende os dois dialetos sem saber de antemão
      qual agente falou
- [x] O progresso mostra o texto inteiro, com a caixa de altura fixa e
      rolagem própria
- [x] Seletor de agente na conversa, com os três
- [x] Trocar de agente e voltar preserva o binário ajustado à mão
- [x] Configuração antiga do Codex é migrada
- [x] Cenários de harness pra cada item

## Comandos de validação

```bash
cargo test --workspace
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
node scripts/uitest/run.mjs
```

## Notas

### Um leitor, dois dialetos

Os nomes de evento são disjuntos, então não há ambiguidade nem
necessidade de dizer ao leitor qual agente está falando:

- Claude Code: `system`/`assistant`/`result`; a resposta é o `result`.
- Codex: `thread.started`/`item.completed`/`turn.completed`; a resposta
  é o ÚLTIMO `agent_message`, não a soma deles — o Codex narra o que vai
  fazer antes de fazer, e essa narração não é resposta.

O opencode tem `--format json`, mas não deu pra conferir o formato dos
eventos contra um binário rodando (não havia modelo configurado aqui).
Ficou em `Texto`, que funciona com qualquer agente, até alguém checar.

### O bug que o harness pegou

O cenário de troca de agente falhou de primeira: ir pro Codex e voltar
devolvia `binario: "claude"` em vez do caminho ajustado. Causa: só o
agente recém-escolhido era guardado na lista de conhecidos; o que SAÍA
não era. Agora a troca guarda o que sai antes de gravar o que entra.

### Erro do agente agora é diagnosticável

O stderr passou a ser lido SEMPRE, em thread própria. Dois motivos: um
agente que escreve muito nele encheria o buffer do pipe e ficaria
bloqueado esperando alguém ler; e quando a saída vem vazia, o que ele
disse no stderr é a única pista do motivo.

Antes, a primeira tentativa com o Codex falhou com "o agente terminou
sem escrever nada na saída" e não havia como saber por quê. A causa
daquela falha não foi estabelecida — depois desta mudança o mesmo caso
passou a funcionar, e uma recorrência agora vem com o motivo junto.
