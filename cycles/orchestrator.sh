#!/usr/bin/env bash
# Anotadinho orchestrator: roda UM ciclo.
#
# Uso:
#   cycles/orchestrator.sh run [task_id]
#   cycles/orchestrator.sh list
#   cycles/orchestrator.sh status [task_id]
#   cycles/orchestrator.sh history
#   cycles/orchestrator.sh retry <task_id>
#   cycles/orchestrator.sh help

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

# shellcheck source=lib/state.sh
source "$SCRIPT_DIR/lib/state.sh"
# shellcheck source=lib/validation.sh
source "$SCRIPT_DIR/lib/validation.sh"
# shellcheck source=lib/git.sh
source "$SCRIPT_DIR/lib/git.sh"
# shellcheck source=lib/agent.sh
source "$SCRIPT_DIR/lib/agent.sh"
# shellcheck source=lib/run.sh
source "$SCRIPT_DIR/lib/run.sh"

CMD="${1:-help}"
shift || true

case "$CMD" in
    run)
        run_cycle "${1:-}"
        ;;
    list)
        list_tasks
        ;;
    status)
        show_status "${1:-}"
        ;;
    history)
        show_history
        ;;
    retry)
        retry_cycle "$1"
        ;;
    help|--help|-h|"")
        cat <<EOF
Anotadinho orchestrator

Comandos:
  run [task_id]      Roda próxima task pendente (ou a especificada)
  list               Lista todas as tasks
  status [task_id]   Status geral ou de uma task específica
  history            Histórico completo de execuções
  retry <task_id>    Re-executa task que falhou, com contexto da falha

Variáveis:
  AGENT_CMD          Comando do AI agent (default: claude code)

Exemplos:
  ./cycles/orchestrator.sh run
  ./cycles/orchestrator.sh run 005
  AGENT_CMD="aider" ./cycles/orchestrator.sh run
EOF
        ;;
    *)
        echo "Comando desconhecido: $CMD" >&2
        echo "Rode '$0 help' para ajuda." >&2
        exit 1
        ;;
esac
