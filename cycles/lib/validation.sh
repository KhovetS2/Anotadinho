#!/usr/bin/env bash
# Helpers para validação (testes, build, clippy).

ROOT_DIR="${ROOT_DIR:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}"

# Roda todos os checks padrão do Anotadinho.
# Retorna 0 se todos passam, 1 se algum falha.
# Uso: run_validation
run_validation() {
    local failed=0

    echo ""
    echo "== Validando =="
    echo ""

    echo "[1/3] cargo build --workspace"
    if ! (cd "$ROOT_DIR" && cargo build --workspace 2>&1 | tail -10); then
        echo "  FALHOU: cargo build"
        failed=1
    else
        echo "  OK"
    fi
    echo

    echo "[2/3] cargo test --workspace"
    if ! (cd "$ROOT_DIR" && cargo test --workspace 2>&1 | tail -15); then
        echo "  FALHOU: cargo test"
        failed=1
    else
        echo "  OK"
    fi
    echo

    echo "[3/3] cargo clippy --workspace"
    if ! (cd "$ROOT_DIR" && cargo clippy --workspace --all-targets 2>&1 | tail -10); then
        echo "  FALHOU: cargo clippy"
        failed=1
    else
        echo "  OK"
    fi
    echo

    return $failed
}
