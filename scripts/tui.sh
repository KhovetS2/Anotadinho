#!/usr/bin/env bash
# Roda o Anotadinho num terminal, em modo dev.
#
# Por padrão fica de olho no código: mudou um `.rs` do workspace ou o
# `main.css` (de onde a paleta é lida em tempo de compilação), ele
# recompila e RELIGA a TUI. Não é hot reload de verdade — binário Rust
# não troca de código em voo —, mas é o mesmo gesto: salvar e ver.
#
# Requer: cargo, inotifywait (pacote inotify-tools).

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

if [ -f "$HOME/.cargo/env" ]; then
    source "$HOME/.cargo/env"
fi
export PATH="$HOME/.cargo/bin:$PATH"

VAULT="$ROOT_DIR/VaultAnotadinho"
TEMA="escuro"
VIGIAR=1

while [ $# -gt 0 ]; do
    case "$1" in
        --vault) VAULT="$2"; shift 2 ;;
        --tema) TEMA="$2"; shift 2 ;;
        --uma-vez) VIGIAR=0; shift ;;
        -h|--help)
            sed -n '2,9p' "$0" | sed 's/^# \?//'
            echo
            echo "uso: $0 [--vault CAMINHO] [--tema escuro|papel|contraste|claro] [--uma-vez]"
            exit 0 ;;
        *) echo "opção desconhecida: $1" >&2; exit 2 ;;
    esac
done

BIN="$ROOT_DIR/target/debug/anotadinho-tui"

# O terminal fica em modo cru e em tela alternativa enquanto a TUI roda.
# Ela desliga os dois na saída normal, mas aqui a gente MATA o processo
# a cada recarga — e aí a limpeza dela não roda. Sem isto, a primeira
# recompilação deixaria o shell da pessoa quebrado.
restaura_terminal() {
    printf '\033[?1049l\033[?25h'
    stty sane 2>/dev/null || true
}

if [ "$VIGIAR" -eq 0 ]; then
    cargo build -p anotadinho-tui
    exec "$BIN" --vault "$VAULT" --tema "$TEMA"
fi

if ! command -v inotifywait >/dev/null 2>&1; then
    echo "ERRO: falta o inotifywait (pacote inotify-tools)." >&2
    echo "Sem ele não dá pra vigiar o código. Rode com --uma-vez, ou:" >&2
    echo "  sudo dnf install inotify-tools" >&2
    exit 1
fi

TUI_PID=""
limpar() {
    [ -n "$TUI_PID" ] && kill "$TUI_PID" 2>/dev/null || true
    restaura_terminal
}
trap limpar EXIT INT TERM

echo "TUI em modo dev — salvar recompila e religa. Ctrl-C aqui encerra."

while true; do
    # Compila ANTES de limpar a tela: erro de compilação tem que ficar
    # legível, e a TUI que já está de pé continua no ar até dar certo.
    if ! cargo build -p anotadinho-tui 2>&1; then
        echo
        echo "--- não compilou; esperando a próxima mudança ---"
    else
        "$BIN" --vault "$VAULT" --tema "$TEMA" &
        TUI_PID=$!
    fi

    # Espera o que vier primeiro: a pessoa sair da TUI, ou o código
    # mudar. `-e close_write` é o evento do editor que ACABOU de gravar;
    # `modify` dispara no meio da escrita e recompilaria arquivo pela
    # metade.
    inotifywait -q -r -e close_write,move,create,delete \
        --include '\.(rs|css|toml)$' \
        crates ui/src/styles Cargo.toml >/dev/null &
    OLHO=$!

    if [ -n "$TUI_PID" ]; then
        wait -n "$TUI_PID" "$OLHO" 2>/dev/null || true
    else
        wait "$OLHO" 2>/dev/null || true
    fi

    # Quem sobreviveu morre: ou a TUI saiu e o olho não serve mais, ou o
    # código mudou e a TUI vai ser trocada.
    if [ -n "$TUI_PID" ] && ! kill -0 "$TUI_PID" 2>/dev/null; then
        kill "$OLHO" 2>/dev/null || true
        wait "$OLHO" 2>/dev/null || true
        echo "TUI encerrada."
        exit 0
    fi
    kill "$OLHO" 2>/dev/null || true
    [ -n "$TUI_PID" ] && kill "$TUI_PID" 2>/dev/null || true
    wait 2>/dev/null || true
    TUI_PID=""
    restaura_terminal
    echo "--- mudou; recompilando ---"
done
