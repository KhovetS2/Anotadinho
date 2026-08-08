#!/usr/bin/env bash
# Build release do Anotadinho: GUI (Tauri) + CLI headless
# (anotadinho-cli, ciclo 110) lado a lado — sem o CLI, quem builda a
# aplicação fica sem a peça usada por um agente pra operar o vault
# sem a janela aberta (ciclo 114).

set -uo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "==> Buildando anotadinho-cli (release)..."
cargo build --release --manifest-path "$ROOT_DIR/Cargo.toml" -p anotadinho-cli || exit 1

echo "==> Buildando a GUI (cargo tauri build)..."
( cd "$ROOT_DIR/src-tauri" && cargo tauri build )
GUI_BUILD_STATUS=$?

GUI_DIR="$ROOT_DIR/src-tauri/target/release"
CLI_BIN="$ROOT_DIR/target/release/anotadinho-cli"

# `cargo tauri build` pode falhar só no empacotamento de um formato
# específico (ex: AppImage precisa de FUSE + downloads externos —
# comum não estar disponível num sandbox/CI) mesmo com o binário da
# GUI compilado com sucesso. Não aborta o script nesse caso — só avisa
# — mas aborta de verdade se o binário nem existe.
if [[ ! -x "$GUI_DIR/anotadinho" ]]; then
    echo "ERRO: binário da GUI não foi gerado em $GUI_DIR/anotadinho" >&2
    exit 1
fi

cp "$CLI_BIN" "$GUI_DIR/anotadinho-cli"

if [[ $GUI_BUILD_STATUS -ne 0 ]]; then
    echo ""
    echo "==> Aviso: 'cargo tauri build' terminou com erro ao empacotar algum"
    echo "    instalador específico (ex: AppImage sem FUSE disponível) — o"
    echo "    binário da GUI e os pacotes que deram certo continuam em"
    echo "    $GUI_DIR/bundle/."
fi

echo ""
echo "==> Build completo."
echo "    GUI: $GUI_DIR/anotadinho (+ instaladores em $GUI_DIR/bundle/)"
echo "    CLI: $GUI_DIR/anotadinho-cli"
