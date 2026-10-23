#!/usr/bin/env bash
# Build release do Anotadinho: GUI (Tauri) + CLI headless
# (anotadinho-cli, ciclo 110) lado a lado — sem o CLI, quem builda a
# aplicação fica sem a peça usada por um agente pra operar o vault
# sem a janela aberta (ciclo 114).

set -uo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# `--instalar`: depois do build, reinstala o pacote do sistema — um
# comando só pra atualizar a janela, o CLI e a TUI. Sem `--bundles`, gera
# só o pacote da distro (rpm no Fedora, deb no Debian/Ubuntu): o AppImage
# é o mais lento e não é o que se instala. O resto dos argumentos vai pro
# `cargo tauri build`, como antes.
INSTALAR=0
ARGS=()
for a in "$@"; do
    if [[ "$a" == "--instalar" ]]; then INSTALAR=1; else ARGS+=("$a"); fi
done
PACOTE=""
if [[ $INSTALAR -eq 1 ]]; then
    if command -v dnf >/dev/null 2>&1; then PACOTE=rpm
    elif command -v apt-get >/dev/null 2>&1; then PACOTE=deb
    else echo "ERRO: --instalar sabe instalar com dnf ou apt; nenhum dos dois achado." >&2; exit 1
    fi
    if [[ ! " ${ARGS[*]-} " =~ " --bundles " ]]; then ARGS+=(--bundles "$PACOTE"); fi
fi

echo "==> Buildando anotadinho-cli e anotadinho-tui (release)..."
cargo build --release --manifest-path "$ROOT_DIR/Cargo.toml" -p anotadinho-cli -p anotadinho-tui || exit 1

# O instalador leva o CLI junto (ciclo 450): é ele que atende o MCP do
# agente, e o app o procura AO LADO do próprio executável. Sem isto, o app
# instalado rodava o agente sem as ferramentas do vault.
#
# O `externalBin` mora em `tauri.bundle.json`, aplicado só aqui pelo
# `--config`, e não no `tauri.conf.json`: o `tauri-build` exige o arquivo
# em TODO build, e um `cargo build` de checkout limpo quebraria. O nome
# leva o alvo como sufixo — é a convenção do Tauri, que o tira ao copiar.
HOST="$(rustc -vV | sed -n 's/^host: //p')"
mkdir -p "$ROOT_DIR/src-tauri/binaries"
cp "$ROOT_DIR/target/release/anotadinho-cli" "$ROOT_DIR/src-tauri/binaries/anotadinho-cli-$HOST"
# A TUI vai no mesmo pacote (`/usr/bin/anotadinho-tui`): instalar o app
# instala os três, e atualizar um atualiza todos.
cp "$ROOT_DIR/target/release/anotadinho-tui" "$ROOT_DIR/src-tauri/binaries/anotadinho-tui-$HOST"

echo "==> Buildando a GUI (cargo tauri build)..."
( cd "$ROOT_DIR/src-tauri" && cargo tauri build --config tauri.bundle.json "${ARGS[@]}" )
GUI_BUILD_STATUS=$?

GUI_DIR="$ROOT_DIR/src-tauri/target/release"
CLI_BIN="$ROOT_DIR/target/release/anotadinho-cli"
TUI_BIN="$ROOT_DIR/target/release/anotadinho-tui"

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
cp "$TUI_BIN" "$GUI_DIR/anotadinho-tui"

# Cross-compile pro Windows (raw `cargo build`, sem passar pelo bundler do
# tauri — que precisa de NSIS/WiX rodando em Windows de verdade). O
# `ui/dist` já está fresco pelo `cargo tauri build` acima, que roda o
# `beforeBuildCommand` (trunk); o build.rs do tauri embute esses assets
# no binário na hora da compilação, então o `cargo build` abaixo pega o
# mesmo frontend sem precisar rodar o trunk de novo.
WIN_TARGET="x86_64-pc-windows-gnu"
WIN_DIR="$ROOT_DIR/src-tauri/target/$WIN_TARGET/release"
WIN_BUILD_OK=0
echo ""
echo "==> Buildando para Windows (cross-compile, target $WIN_TARGET)..."
if ! rustup target list --installed 2>/dev/null | grep -qx "$WIN_TARGET"; then
    echo "    Aviso: target $WIN_TARGET não instalado (rustup target add $WIN_TARGET) — pulando build Windows."
elif ! command -v x86_64-w64-mingw32-gcc >/dev/null 2>&1; then
    echo "    Aviso: mingw-w64 (pacote gcc-mingw-w64-x86-64) não encontrado — pulando build Windows."
else
    # `--features tauri/custom-protocol` é o que o `cargo tauri build`
    # liga por baixo dos panos: sem ela o binário compila em modo dev e
    # tenta abrir http://localhost:1420 (o servidor do `trunk serve`) em
    # vez de carregar os assets embutidos — tela em branco com
    # ERR_CONNECTION_REFUSED, já que não tem dev server rodando.
    ( cd "$ROOT_DIR/src-tauri" && cargo build --release --target "$WIN_TARGET" --features tauri/custom-protocol )
    # O CLI vai junto pelo mesmo motivo do instalador (ciclo 450).
    cargo build --release --manifest-path "$ROOT_DIR/Cargo.toml" -p anotadinho-cli -p anotadinho-tui --target "$WIN_TARGET"
    if [[ -f "$WIN_DIR/anotadinho.exe" ]]; then
        WIN_BUILD_OK=1
    else
        echo "    Aviso: build Windows terminou mas $WIN_DIR/anotadinho.exe não foi gerado."
    fi
fi

WIN_DIST="$ROOT_DIR/dist-windows"
if [[ $WIN_BUILD_OK -eq 1 ]]; then
    rm -rf "$WIN_DIST"
    mkdir -p "$WIN_DIST"
    cp "$WIN_DIR/anotadinho.exe" "$WIN_DIST/"
    cp "$ROOT_DIR/target/$WIN_TARGET/release/anotadinho-cli.exe" "$WIN_DIST/"
    cp "$ROOT_DIR/target/$WIN_TARGET/release/anotadinho-tui.exe" "$WIN_DIST/"
    DLL="$(find "$WIN_DIR" -maxdepth 1 -iname 'WebView2Loader.dll' | head -1)"
    if [[ -n "$DLL" ]]; then
        cp "$DLL" "$WIN_DIST/"
    else
        echo "    Aviso: WebView2Loader.dll não achado em $WIN_DIR — copie na mão antes de testar no Windows."
    fi
fi

# Linka o anotadinho-cli em ~/.local/bin pra ficar disponível no PATH sem
# caminho completo — é o binário que um agente (ex: Claude Code no WSL)
# chama pra operar o vault sem abrir a janela (ciclo 114).
LOCAL_BIN="$HOME/.local/bin"
CLI_LINKED=0
if [[ -d "$LOCAL_BIN" ]]; then
    ln -sf "$CLI_BIN" "$LOCAL_BIN/anotadinho-cli"
    ln -sf "$TUI_BIN" "$LOCAL_BIN/anotadinho-tui"
    CLI_LINKED=1
fi

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
echo "    TUI: $GUI_DIR/anotadinho-tui"
if [[ $CLI_LINKED -eq 1 ]]; then
    echo "    CLI e TUI também linkados em $LOCAL_BIN (PATH) — \`anotadinho-cli --vault <path> ...\`, \`anotadinho-tui --vault <path>\`"
else
    echo "    Aviso: $LOCAL_BIN não existe — CLI e TUI não foram linkados no PATH."
fi
if [[ $WIN_BUILD_OK -eq 1 ]]; then
    echo "    Windows: $WIN_DIST/ (anotadinho.exe + anotadinho-cli.exe + anotadinho-tui.exe + WebView2Loader.dll — sem instalador, só os binários crus)"
fi

if [[ $INSTALAR -eq 1 ]]; then
    PKG="$(ls -t "$GUI_DIR/bundle/$PACOTE/"*."$PACOTE" 2>/dev/null | head -1)"
    if [[ -z "$PKG" ]]; then
        echo "ERRO: --instalar pediu o pacote $PACOTE, e ele não está em $GUI_DIR/bundle/$PACOTE/" >&2
        exit 1
    fi
    echo ""
    echo "==> Instalando $PKG (feche o Anotadinho instalado antes)..."
    # `reinstall` quando já está instalado: a versão não muda entre um
    # build e outro, e o `install` diria que não há nada a fazer.
    if [[ $PACOTE == rpm ]]; then
        if rpm -q anotadinho >/dev/null 2>&1; then sudo dnf reinstall -y "$PKG"; else sudo dnf install -y "$PKG"; fi
    else
        sudo apt-get install -y --reinstall "$PKG"
    fi || exit 1
    echo "    Instalado: anotadinho, anotadinho-cli e anotadinho-tui em /usr/bin."
fi
