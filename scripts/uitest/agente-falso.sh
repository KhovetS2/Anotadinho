#!/usr/bin/env bash
# Agente de MENTIRA pros cenários do ciclo 202.
#
# O que se testa com ele é o CONTRATO da execução — prompt chega inteiro,
# saída volta, timeout mata, falha é reportada — e não a qualidade da
# resposta de um modelo. Usar claude/codex de verdade tornaria a suíte
# lenta, cara e não determinística.
#
# Modos: --responder (padrão), --demorar, --falhar, --mudo, --devagar,
# --stream (Claude Code), --codex, --onde, --args.
# Guarda a linha inteira antes de consumir as pastas extras — é ela que
# o modo --args devolve.
TODOS="$*"

# Pastas extras chegam ANTES do resto (ciclo 216), igual nos agentes de
# verdade: consome os pares `--add-dir <pasta>` pra o modo cair no
# `case` certo.
while [ "$1" = "--add-dir" ]; do
  shift 2
done

case "$1" in
  # Ecoa a linha de argumentos inteira, pastas extras incluídas.
  --args) echo "$TODOS"; exit 0 ;;
  --demorar) sleep 60 ;;
  --falhar) echo "erro proposital" >&2; exit 3 ;;
  --mudo) exit 0 ;;
  # Escreve aos poucos: é o que permite afirmar sobre a saída PARCIAL
  # e sobre cancelar no meio (ciclo 213).
  --devagar)
    for i in 1 2 3 4 5 6 7 8 9 10; do
      echo "linha $i"
      sleep 1
    done
    exit 0
    ;;

  # Diz onde está rodando (ciclo 215).
  --onde) pwd; exit 0 ;;
  # Fala o dialeto JSONL do Codex (ciclo 214).
  --codex)
    echo '{"type":"thread.started","thread_id":"t1"}'
    echo '{"type":"turn.started"}'
    echo '{"type":"item.completed","item":{"id":"i0","type":"agent_message","text":"Vou conferir a pasta."}}'
    sleep 1
    echo '{"type":"item.started","item":{"id":"i1","type":"command_execution","command":"ls -1"}}'
    sleep 1
    printf '{"type":"item.completed","item":{"id":"i2","type":"agent_message","text":"RESPOSTA para: %s"}}\n' "$2"
    echo '{"type":"turn.completed","usage":{}}'
    exit 0
    ;;
  # Fala o dialeto `stream-json` do Claude Code (ciclo 213).
  --stream)
    echo '{"type":"system","subtype":"init"}'
    echo '{"type":"assistant","message":{"content":[{"type":"tool_use","name":"Read"}]}}'
    sleep 1
    echo '{"type":"assistant","message":{"content":[{"type":"text","text":"pensando alto"}]}}'
    sleep 1
    printf '{"type":"result","is_error":false,"result":"RESPOSTA para: %s"}\n' "$2"
    exit 0
    ;;
esac
printf 'RESPOSTA para: %s' "$2"
