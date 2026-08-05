#!/usr/bin/env bash
# Cria uma nova task a partir do template.
# Uso: scripts/new-cycle.sh [--id NNN] "Título da task"

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
TASKS_DIR="$ROOT_DIR/cycles/tasks"
TEMPLATE="$ROOT_DIR/cycles/templates/task.md"

# Parse args
ID=""
TITLE=""
while [[ $# -gt 0 ]]; do
    case "$1" in
        --id)
            ID="$2"
            shift 2
            ;;
        *)
            TITLE="${TITLE:-$1}"
            shift
            ;;
    esac
done

if [[ -z "$TITLE" ]]; then
    echo "Uso: $0 [--id NNN] \"Título da task\"" >&2
    exit 1
fi

# Auto-detect next ID se não fornecido
if [[ -z "$ID" ]]; then
    LAST=$(ls "$TASKS_DIR"/*.md 2>/dev/null | sed 's/.*\/\([0-9]*\)-.*/\1/' | sort -n | tail -1)
    if [[ -z "$LAST" ]]; then
        ID="001"
    else
        ID=$(printf "%03d" $((10#$LAST + 1)))
    fi
fi

# Slug do título
SLUG=$(echo "$TITLE" | tr '[:upper:]' '[:lower:]' | sed 's/[^a-z0-9]/-/g' | sed 's/--*/-/g' | sed 's/^-//' | sed 's/-$//')
OUT="$TASKS_DIR/${ID}-${SLUG}.md"
DATA=$(date +%Y-%m-%d)

# Cria do template
sed -e "s/{ID}/$ID/g" \
    -e "s/{TITULO}/$TITLE/g" \
    -e "s/{DATA}/$DATA/g" \
    "$TEMPLATE" > "$OUT"

echo "Criada: cycles/tasks/$(basename "$OUT")"
echo "Status: pending (pronto pra rodar com cycles/orchestrator.sh run $ID)"
