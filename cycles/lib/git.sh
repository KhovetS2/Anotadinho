#!/usr/bin/env bash
# Helpers para operações git (commits por ciclo).

ROOT_DIR="${ROOT_DIR:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}"

# Faz commit do trabalho do ciclo, se houver mudanças.
# Uso: commit_cycle <task_id> <titulo_curto>
commit_cycle() {
    local task_id="$1"
    local title="$2"

    cd "$ROOT_DIR"

    if git diff --quiet && git diff --cached --quiet; then
        echo "  (nenhuma mudança pra commitar)"
        return 0
    fi

    git add -A
    git commit -m "cycle(${task_id}): ${title}

Implementado via cycles/orchestrator.sh.

Co-authored-by: Anotadinho Orchestrator <noreply@anotadinho.local>" \
        >/dev/null 2>&1

    local sha
    sha=$(git rev-parse --short HEAD)
    echo "  commit: $sha"
    echo "$sha"
}
