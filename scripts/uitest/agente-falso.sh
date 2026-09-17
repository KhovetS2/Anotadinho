#!/usr/bin/env bash
# Agente de MENTIRA pros cenários do ciclo 202.
#
# O que se testa com ele é o CONTRATO da execução — prompt chega inteiro,
# saída volta, timeout mata, falha é reportada — e não a qualidade da
# resposta de um modelo. Usar claude/codex de verdade tornaria a suíte
# lenta, cara e não determinística.
#
# Modos: --responder (padrão), --demorar, --falhar, --mudo, --devagar,
# --stream (Claude Code), --codex, --onde, --args, --falha-no-stream.
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

  # Falha reportando o motivo no STREAM e ruído no stderr (ciclo 219).
  --falha-no-stream)
    echo "Reading additional input from stdin..." >&2
    echo '{"type":"error","message":"bateu o limite de uso"}'
    echo '{"type":"turn.failed","error":{"message":"bateu o limite de uso"}}'
    exit 1
    ;;
  # Diz onde está rodando (ciclo 215).
  --onde) pwd; exit 0 ;;

  # PROPÕE de verdade, pela mesma porta de um agente real (ciclo 413):
  # acha o vault na mensagem e chama `anotadinho-cli propor`. É o que faz
  # a suíte de avaliação medir o arranjo — proposta, validação e
  # permissões — sem depender de modelo.
  --propor)
    PROMPT="$2"
    # Fecha no ". Para agir": o diretório temporário pode ter ponto no
    # nome, e um `[^.]*` aqui quebrava justamente na suíte.
    VAULT=$(printf '%s' "$PROMPT" | sed -n 's/.*O vault está em \(.*\)\. Para agir.*/\1/p' | head -1)
    ALVO=$(printf '%s' "$PROMPT" | sed -n 's/.*PROPONHA \([^ ]*\).*/\1/p' | head -1)
    CLI="${ANOTADINHO_CLI:-anotadinho-cli}"
    if [ -z "$VAULT" ] || [ -z "$ALVO" ]; then
      echo "não achei vault ou alvo no prompt" >&2
      exit 4
    fi
    printf -- "---\ntitle: Proposta do falso\n---\n\nAtalho pra duplicar um cartão do kanban.\n" \
      | "$CLI" --vault "$VAULT" propor "$ALVO" --motivo "avaliação" --autor "falso" \
      || { echo "propor recusado" >&2; exit 5; }
    echo "propus $ALVO no vault $VAULT"
    exit 0
    ;;
  # Fala o dialeto JSONL do Codex (ciclo 214).
  --codex)
    echo '{"type":"thread.started","thread_id":"t1"}'
    echo '{"type":"turn.started"}'
    echo '{"type":"item.completed","item":{"id":"i0","type":"agent_message","text":"Vou conferir a pasta."}}'
    sleep 1
    echo '{"type":"item.started","item":{"id":"i1","type":"command_execution","command":"ls -1"}}'
    sleep 1
    ESCAPADO=$(printf '%s' "$2" | sed -e 's/\\/\\\\/g' -e 's/"/\\"/g' | tr '\n\t' '  ')
    printf '{"type":"item.completed","item":{"id":"i2","type":"agent_message","text":"RESPOSTA para: %s"}}\n' "$ESCAPADO"
    echo '{"type":"turn.completed","usage":{}}'
    exit 0
    ;;
  # Fala o dialeto `stream-json` do Claude Code (ciclo 213).
  --stream)
    # O prompt entra em JSON, então precisa ser ESCAPADO: um prompt com
    # aspas ou quebra de linha — que é o normal quando há contexto —
    # produzia JSON inválido, a linha era ignorada e a resposta virava o
    # "pensando alto" do evento anterior (achado no ciclo 422).
    ESCAPADO=$(printf '%s' "$2" | sed -e 's/\\/\\\\/g' -e 's/"/\\"/g' | tr '\n\t' '  ')
    echo '{"type":"system","subtype":"init"}'
    echo '{"type":"assistant","message":{"content":[{"type":"tool_use","name":"Read"}]}}'
    sleep 1
    echo '{"type":"assistant","message":{"content":[{"type":"text","text":"pensando alto"}]}}'
    sleep 1
    # Com `usage` e custo (ciclo 422): é o que o Claude Code manda, e é
    # o que o registro de execuções guarda.
    printf '{"type":"result","is_error":false,"result":"RESPOSTA para: %s","usage":{"input_tokens":12000,"output_tokens":800},"total_cost_usd":0.0432}\n' "$ESCAPADO"
    exit 0
    ;;
esac
printf 'RESPOSTA para: %s' "$2"
