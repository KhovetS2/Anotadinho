# Empacota o Anotadinho NO Windows, com o anotadinho-cli dentro do
# instalador (ciclo 450). É o equivalente do `build.sh` pra quem tem
# NSIS/WiX — o build cruzado do Linux não gera instalador.
#
# O app procura o CLI ao lado do próprio executável pra ligar o MCP do
# agente; o `externalBin` de `tauri.bundle.json` é o que o põe lá. Ele
# fica fora do `tauri.conf.json` porque o `tauri-build` exige o arquivo
# em todo build.
#
#   powershell -ExecutionPolicy Bypass -File scripts\build.ps1
#   powershell -ExecutionPolicy Bypass -File scripts\build.ps1 --bundles nsis

$ErrorActionPreference = 'Stop'
$raiz = Split-Path -Parent $PSScriptRoot

$alvo = (rustc -vV | Select-String '^host: (.+)$').Matches[0].Groups[1].Value

Write-Host "==> Buildando anotadinho-cli (release)..."
cargo build --release --manifest-path "$raiz\Cargo.toml" -p anotadinho-cli
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

# O nome leva o alvo como sufixo: é a convenção do Tauri, que o tira ao
# copiar pra pasta do app.
New-Item -ItemType Directory -Force "$raiz\src-tauri\binaries" | Out-Null
Copy-Item "$raiz\target\release\anotadinho-cli.exe" "$raiz\src-tauri\binaries\anotadinho-cli-$alvo.exe" -Force

Write-Host "==> Buildando a GUI e os instaladores (cargo tauri build)..."
Push-Location "$raiz\src-tauri"
try {
    cargo tauri build --config tauri.bundle.json @args
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
} finally {
    Pop-Location
}

Write-Host ""
Write-Host "==> Instaladores em $raiz\src-tauri\target\release\bundle\"
