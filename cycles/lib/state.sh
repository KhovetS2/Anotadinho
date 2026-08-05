#!/usr/bin/env bash
# Helpers para gerenciar tasks e status.

# Diretórios
TASKS_DIR="${TASKS_DIR:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/tasks}"
STATUS_DIR="${STATUS_DIR:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/status}"
FAILURES_DIR="${FAILURES_DIR:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/failures}"

# Lê o frontmatter de um arquivo .md e retorna o campo pedido.
# Strippa aspas simples/duplas e colchetes de listas.
# Uso: get_field <arquivo> <campo>
get_field() {
    local file="$1"
    local field="$2"
    awk -v f="$field" '
        /^---$/ { in_fm = !in_fm; next }
        in_fm && $0 ~ "^"f":" {
            sub("^"f":[[:space:]]*", "")
            # Strip leading/trailing quotes and brackets
            gsub(/^["'\''[]+/, "")
            gsub(/["'\''\]]+$/, "")
            print
            exit
        }
    ' "$file"
}

# Lê a lista de IDs de dependências (formato: ['001', '002']).
# Retorna uma linha por ID.
get_depends() {
    local file="$1"
    local deps
    deps=$(get_field "$file" "depende_de")
    # Remove [ ] ' " e vírgulas, depois quebra por espaço
    echo "$deps" | tr -d "[]'\"\," | tr -s ' ' '\n' | grep -E '^[0-9]+$' || true
}

# Verifica se todas as dependências estão done.
# Retorna 0 se OK, 1 se falta dependência.
check_dependencies() {
    local file="$1"
    local dep
    while IFS= read -r dep; do
        [[ -z "$dep" ]] && continue
        local dep_status
        dep_status=$(get_latest_status "$dep" 2>/dev/null || echo "missing")
        if [[ "$dep_status" != "done" ]]; then
            echo "Dependência não satisfeita: $dep (status: $dep_status)" >&2
            return 1
        fi
    done < <(get_depends "$file")
    return 0
}

# Pega o status mais recente de uma task (lê todos os arquivos de status).
# Uso: get_latest_status <task_id>
get_latest_status() {
    local task_id="$1"
    local status_file
    status_file=$(ls -t "$STATUS_DIR/${task_id}-"*.md 2>/dev/null | head -1 || true)
    if [[ -z "$status_file" ]]; then
        echo "missing"
        return
    fi
    get_field "$status_file" "status"
}

# Marca uma task como in_progress.
# Uso: mark_in_progress <task_id>
mark_in_progress() {
    local task_id="$1"
    local task_file="$TASKS_DIR/${task_id}-"*.md
    # shellcheck disable=SC2086
    sed -i 's/^status: pending$/status: in_progress/' $task_file
}

# Marca uma task como pending (rollback de in_progress).
mark_pending() {
    local task_id="$1"
    local task_file="$TASKS_DIR/${task_id}-"*.md
    # shellcheck disable=SC2086
    sed -i 's/^status: in_progress$/status: pending/' $task_file
}

# Marca uma task como done.
mark_done() {
    local task_id="$1"
    local task_file="$TASKS_DIR/${task_id}-"*.md
    # shellcheck disable=SC2086
    sed -i 's/^status: in_progress$/status: done/' $task_file
}

# Marca uma task como failed.
mark_failed() {
    local task_id="$1"
    local task_file="$TASKS_DIR/${task_id}-"*.md
    # shellcheck disable=SC2086
    sed -i 's/^status: in_progress$/status: failed/' $task_file
}

# Encontra a próxima task pendente.
# Uso: find_next_pending
find_next_pending() {
    for f in "$TASKS_DIR"/*.md; do
        [[ -f "$f" ]] || continue
        local status
        status=$(get_field "$f" "status")
        if [[ "$status" == "pending" ]]; then
            # Verifica dependências
            if check_dependencies "$f"; then
                echo "$f"
                return 0
            fi
        fi
    done
    return 1
}

# Encontra uma task por ID.
# Uso: find_task <task_id>
find_task() {
    local task_id="$1"
    ls "$TASKS_DIR/${task_id}-"*.md 2>/dev/null | head -1
}

# Lista todas as tasks.
list_tasks() {
    echo "Tasks:"
    for f in "$TASKS_DIR"/*.md; do
        [[ -f "$f" ]] || continue
        local id
        id=$(get_field "$f" "id")
        local title
        title=$(get_field "$f" "titulo")
        local status
        status=$(get_field "$f" "status")
        printf "  %s [%s] %s\n" "$id" "$status" "$title"
    done
}

# Mostra status geral ou de uma task.
show_status() {
    if [[ -n "${1:-}" ]]; then
        local task_id="$1"
        echo "Histórico da task $task_id:"
        for f in "$STATUS_DIR/${task_id}-"*.md; do
            [[ -f "$f" ]] || continue
            echo "---"
            cat "$f"
        done
    else
        echo "Status geral:"
        list_tasks
        echo
        echo "Últimas 5 execuções:"
        ls -t "$STATUS_DIR"/*.md 2>/dev/null | head -5 | while read -r f; do
            local id status ts
            id=$(get_field "$f" "id")
            status=$(get_field "$f" "status")
            ts=$(get_field "$f" "executado_em")
            printf "  %s | %s | %s\n" "$ts" "$id" "$status"
        done
    fi
}

# Mostra histórico completo.
show_history() {
    echo "Histórico completo:"
    ls -t "$STATUS_DIR"/*.md 2>/dev/null | while read -r f; do
        local id status ts
        id=$(get_field "$f" "id")
        status=$(get_field "$f" "status")
        ts=$(get_field "$f" "executado_em")
        printf "  %s | %s | %s\n" "$ts" "$id" "$status"
    done
}

# Re-executa uma task que falhou.
retry_cycle() {
    local task_id="$1"
    if [[ -z "$task_id" ]]; then
        echo "Uso: retry <task_id>" >&2
        exit 1
    fi
    local latest_status
    latest_status=$(get_latest_status "$task_id")
    if [[ "$latest_status" != "failed" ]]; then
        echo "Task $task_id não está como failed (status: $latest_status)" >&2
        exit 1
    fi
    echo "Re-executando task $task_id (última falhou)..."
    # Volta status pra pending e roda
    mark_pending "$task_id"
    run_cycle "$task_id"
}
