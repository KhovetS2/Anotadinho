# Ícones do Anotadinho

Gerados de `anotinho-icon-A.png` (2048×2048, fundo transparente) com:

```bash
cargo tauri icon anotinho-icon-A.png -o src-tauri/icons
```

O comando também cria ícones de iOS, Android e Windows Store. Este
projeto é só desktop e `tauri.conf.json` lista apenas os seis abaixo, então
os outros são apagados depois de gerar — são ~5MB de arquivo que ninguém
lê.

| Arquivo | Uso |
|---|---|
| `32x32.png`, `64x64.png`, `128x128.png`, `128x128@2x.png` | Linux |
| `icon.ico` | Windows |
| `icon.icns` | macOS |
| `icon.png` | fonte 512×512 do bundle |

A arte tem cantos arredondados com alfa de verdade — nada de fundo branco
recortado, que era o que fazia aparecer "pontinhas" claras nas bordas
quando o sistema desenhava o ícone sobre um fundo escuro.
