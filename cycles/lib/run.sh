#!/usr/bin/env bash
# Orquestra UM ciclo completo:
# 1. Encontra a task (próxima pending OU especificada)
# 2. Marca como in_progress
# 3. Invoca o AI agent
# 4. Roda validação
# 5. Salva status (done ou failed)
# 6. Se done, faz commit
# 7. Volta a task pra pending se falhou (pra próximo retry)

set -euo pipefail

# Importa helpers
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=state.sh
source "$SCRIPT_DIR/state.sh"
# shellcheck source=validation.sh
source "$SCRIPT_DIR/validation.sh"
# shellcheck source=git.sh
source "$SCRIPT_DIR/git.sh"
# shellcheck source=agent.sh
source "$SCRIPT_DIR/agent.sh"

ROOT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"

run_cycle() {
    local task_id="${1:-}"
    local task_file=""

    # 1. Encontrar a task
    if [[ -n "$task_id" ]]; then
        task_file=$(find_task "$task_id")
        if [[ -z "$task_file" ]]; then
            echo "ERRO: task $task_id não encontrada" >&2
            exit 1
        fi
    else
        task_file=$(find_next_pending) || {
            echo "Nenhuma task pendente com dependências satisfeitas"
            exit 0
        }
    fi

    task_id=$(get_field "$task_file" "id")
    local titulo
    titulo=$(get_field "$task_file" "titulo")
    local now
    now=$(date +%Y-%m-%dT%H:%M:%S)
    local ts_safe
    ts_safe=$(date +%Y%m%dT%H%M%S)

    echo "== Ciclo $task_id: $titulo =="
    echo "  Início: $now"
    echo

    # Verifica se tem failures anteriores
    local failure_log=""
    local latest_failure
    latest_failure=$(ls -t "$FAILURES_DIR/${task_id}-"*.md 2>/dev/null | head -1 || true)
    if [[ -n "$latest_failure" ]]; then
        failure_log="$latest_failure"
        echo "  AVISO: failure anterior em $latest_failure"
        echo
    fi

    # 2. Marca como in_progress
    mark_in_progress "$task_id"

    # 3. Invoca agent
    if ! invoke_agent "$task_file" "$failure_log"; then
        echo "  Agent falhou ao executar"
        mark_pending "$task_id"
        save_failure "$task_id" "$now" "agent inválido ou falhou"
        exit 1
    fi

    # 4. Validação
    if run_validation; then
        echo "  ✓ Validação passou"

        # 5. Salva status
        save_status "$task_id" "$now" "done" "0" "0"
        mark_done "$task_id"

        # 6. Commit
        echo
        echo "== Commit =="
        commit_cycle "$task_id" "$titulo" || echo "  (commit falhou, ciclo continua marcado como done)"

        echo
        echo "✓ Ciclo $task_id concluído com sucesso"
    else
        echo "  ✗ Validação falhou"

        # 5. Salva status de falha
        save_status "$task_id" "$now" "failed" "?" "?"
        save_failure "$task_id" "$now" "validação falhou - ver log"
        mark_pending "$task_id"

        echo
        echo "✗ Ciclo $task_id falhou"
        echo "  Failure log salvo em cycles/failures/"
        echo "  Para retry: ./cycles/orchestrator.sh retry $task_id"
        exit 1
    fi
}

# Salva arquivo de status.
save_status() {
    local task_id="$1"
    local now="$2"
    local status="$3"
    local pass="$4"
    local fail="$5"
    local ts_safe
    ts_safe=$(echo "$now" | tr -d ':' | tr 'T' '_' | cut -c1-15)
    local status_file="$STATUS_DIR/${task_id}-${ts_safe}-${status}.md"

    cat > "$status_file" <<EOF
---
id: "$task_id"
executado_em: $now
status: $status
testes_passaram: $pass
testes_falharam: $fail
agente: ${AGENT_CMD:-manual}
---

## Resumo
Ciclo $task_id: $status
EOF
}

# Salva log de falha.
save_failure() {
    local task_id="$1"
    local now="$2"
    local reason="$3"
    local ts_safe
    ts_safe=$(echo "$now" | tr -d ':' | tr 'T' '_' | cut -c1-15)
    local failure_file="$FAILURES_DIR/${task_id}-${ts_safe}-failure.md"

    cat > "$failure_file" <<EOF
---
id: "$task_id"
executado_em: $now
motivo: $reason
---

## Contexto para retry

A task $task_id falhou. Ao re-executar, considere:

1. Verifique o que mudou desde a última tentativa
2. Olhe os logs acima
3. Tente uma abordagem diferente
4. Se necessário, divida a task em tasks menores

## Log da execução
- $now: $reason
EOF
}
