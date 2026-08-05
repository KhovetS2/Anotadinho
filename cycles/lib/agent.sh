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

    # Quando agent estiver disponível, algo como:
    # $AGENT_CMD --task-file "$task_file" --failure-log "$failure_log" --cd "$ROOT_DIR"
    echo "ERRO: invocação real do agent ainda não implementada"
    return 1
}
