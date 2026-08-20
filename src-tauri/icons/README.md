# Ícones do Anotadinho

Gerados de `anotadinho-icon-zoom.png` (2048×2048, fundo transparente) com:

```bash
cargo tauri icon anotadinho-icon-zoom.png -o src-tauri/icons
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

`anotadinho-icon-zoom.png` é o `anotadinho-icon.png` reenquadrado: o
recorte é a caixa do conteúdo (x 368–1676, y 340–1830) com 70px de folga,
reescalado de volta pra 2048 — 1,26× maior, perdendo só gradiente sólido
da borda. O alfa do original é reaplicado depois do recorte, senão o
arredondamento sumiria junto com os cantos cortados.

O limite do zoom é a base: o subtítulo "Eu gosto assim." termina em
y=1830, com só 218px de margem. Um recorte centrado no meio geométrico
corta esse texto antes dos 1,2× — por isso o enquadramento segue o centro
do CONTEÚDO, não o da imagem.
