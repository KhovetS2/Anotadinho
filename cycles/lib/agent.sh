#!/usr/bin/env bash
# Invoca o AI agent com a task como contexto.

ROOT_DIR="${ROOT_DIR:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}"

# Comando do agent. Default: claude code.
# Pode ser customizado via AGENT_CMD env var.
AGENT_CMD="${AGENT_CMD:-claude code}"

# Monta o contexto e invoca o agent.
# Uso: invoke_agent <task_file> [failure_log]
invoke_agent() {
    local task_file="$1"
    local failure_log="${2:-}"

    echo "== Invocando AI agent =="
    echo "  Agent: $AGENT_CMD"
    echo "  Task:  $task_file"
    [[ -n "$failure_log" ]] && echo "  Failure anterior: $failure_log"
    echo

    # Por enquanto, este é um STUB. A invocação real do agent virá
    # no ciclo 002 ou quando integrarmos com Claude Code CLI.
    #
    # O agent recebe a task como contexto, implementa, e o orchestrator
    # valida depois. Quando o agent não estiver disponível, este script
    # apenas imprime o que deveria ser feito.

    if [[ "$AGENT_CMD" == "stub" ]] || ! command -v claude >/dev/null 2>&1; then
        echo "(modo stub: agent não disponível, instruções abaixo)"
        echo
        echo "Para implementar este ciclo manualmente:"
        echo
        echo "1. Leia a task:"
        echo "   cat '$task_file'"
        echo
        echo "2. Implemente as mudanças"
        echo
        echo "3. Rode a validação:"
        echo "   cd '$ROOT_DIR' && cargo build && cargo test && cargo clippy"
        echo
        if [[ -n "$failure_log" ]]; then
            echo "4. Veja o failure anterior:"
            echo "   cat '$failure_log'"
            echo
        fi
        echo
        echo "5. Quando terminar, rode novamente:"
        echo "   $ROOT_DIR/cycles/orchestrator.sh retry <task_id>"
        return 0
    fi

    # Monta o prompt: task + (se houver) log de falha anterior.
    local prompt
    prompt="Você está executando UM ciclo de desenvolvimento autônomo do Anotadinho.
Leia a task abaixo, implemente exatamente o que ela pede, e pare. Não peça
confirmação — a validação (cargo build/test/clippy + trunk build) roda
automaticamente depois que você terminar; se falhar, o ciclo é revertido pra
'pending' e você será chamado de novo com o log da falha.

## Task ($task_file)

$(cat "$task_file")
"
    if [[ -n "$failure_log" ]]; then
        prompt+="

## Falha na tentativa anterior deste ciclo

$(cat "$failure_log")

Não repita a mesma abordagem que causou essa falha."
    fi

    # Tools liberadas por padrão: leitura livre + escrita de código + os
    # comandos de validação/git que o próprio orchestrator já roda depois.
    # Não usamos --dangerously-skip-permissions por padrão (rodar sem
    # supervisão e com permissão irrestrita é uma decisão que quem chama
    # o orchestrator deve tomar explicitamente via AGENT_UNATTENDED=1).
    local allowed_tools="${AGENT_ALLOWED_TOOLS:-Read Edit Write Grep Glob Bash(cargo *) Bash(trunk *) Bash(git *)}"

    local -a claude_args=(-p "$prompt" --add-dir "$ROOT_DIR")
    if [[ -n "$allowed_tools" ]]; then
        # shellcheck disable=SC2206
        claude_args+=(--allowedTools $allowed_tools)
    fi
    if [[ "${AGENT_UNATTENDED:-0}" == "1" ]]; then
        claude_args+=(--dangerously-skip-permissions)
    fi

    claude "${claude_args[@]}"
}
