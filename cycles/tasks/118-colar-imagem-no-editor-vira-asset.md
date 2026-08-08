---
id: "118"
titulo: "Colar imagem no editor vira asset"
status: pending
criado: 2026-08-08
autor: humano
prioridade: alta
depende_de: []
estima_min: 90
agente_alvo: claude-sonnet
---

# Colar imagem no editor vira asset

## Objetivo

`assets/` hoje só é gerenciado (listar/excluir, ciclo 096) — não
existe paste de imagem no editor que já copie pro vault e insira o
markdown. Colar (Ctrl+V) uma imagem da área de transferência dentro
do editor deve salvar o arquivo em `assets/` e inserir
`![](assets/nome-gerado.ext)` no cursor.

## Critérios de aceite

- [ ] Handler de `paste` no contenteditable do editor: detecta imagem
      em `event.clipboardData.items` (`image/*`)
- [ ] Novo comando IPC `save_pasted_asset(vault_path, filename_hint,
      base64_data) -> Result<String, String>` — decodifica base64,
      grava em `assets/`, gera nome único (mesmo padrão de slug único
      já usado em `create_page_in`/`find_unique_relative_path`),
      devolve o path relativo
- [ ] Ao colar, imagem é convertida pra base64 no frontend
      (`FileReader`), enviada pro backend, e o markdown
      `![](assets/arquivo.png)` é inserido na posição do cursor
      (reaproveita `insert_element_at_cursor`, generalizado no
      ciclo 084)
- [ ] Colar texto normal continua funcionando sem regressão (handler
      só intercepta quando há imagem no clipboard)
- [ ] Teste novo em `crates/vault` cobrindo a função de gravar bytes
      com nome único em `assets/`
- [ ] `cargo test --workspace`, `cd ui && cargo test --lib`,
      `trunk build`, `cargo build --manifest-path src-tauri/Cargo.toml`
      passam
- [ ] Validação ao vivo via MCP `tauri`: simular paste de imagem
      (via `webview_execute_js` disparando um evento `paste` sintético
      com `DataTransfer`) e confirmar arquivo criado em `assets/` +
      markdown inserido

## Comandos de validação

```bash
cargo test --workspace
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Drag-and-drop de arquivo de imagem (só paste via clipboard nesta
  versão — drag-and-drop é um evento diferente, fica pra depois se
  pedirem)
- Compressão/redimensionamento de imagem antes de salvar — grava o
  arquivo como veio do clipboard
- Colar outros tipos de arquivo (PDF, etc) — só imagem; ver
  ciclo 121 pra embed de PDF (fluxo diferente, via asset já existente)

## Notas

Tamanho de imagem colada pode ser grande — base64 infla ~33%; usar o
comando IPC existente como referência de payload (`write_page` já
manda strings grandes sem problema, mesmo princípio).
