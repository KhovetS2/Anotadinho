---
title: Ciclo 118 — Colar imagem no editor vira asset
type: ciclo
ciclo: "118"
status: concluida
date: 2026-08-08
prioridade: alta
depende_de: []
tags:
- ciclo
---

# Ciclo 118 — Colar imagem no editor vira asset

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Colar imagem no editor vira asset

## Objetivo

`assets/` hoje só é gerenciado (listar/excluir, ciclo 096) — não
existe paste de imagem no editor que já copie pro vault e insira o
markdown. Colar (Ctrl+V) uma imagem da área de transferência dentro
do editor deve salvar o arquivo em `assets/` e inserir
`![](assets/nome-gerado.ext)` no cursor.

## Critérios de aceite

- [x] Handler de `paste` no contenteditable do editor: detecta imagem
      via `clipboardData.files` (`image/*`)
- [x] Novo comando IPC `save_pasted_asset(vault_path, extension,
      base64_data) -> Result<String, String>` — decodifica base64,
      grava em `assets/`, gera nome único (`colado-N.ext`), devolve o
      path relativo
- [x] Ao colar, imagem é lida como bytes no frontend (`gloo_file`),
      codificada em base64, enviada pro backend, e um `<img
      src="assets/colado-N.ext">` é inserido na posição do cursor
      (reaproveita `insert_element_at_cursor`, ciclo 084) — mesmo
      padrão HTML já usado pelo item "__ASSET__" do menu `/`
- [x] Colar texto normal continua funcionando sem regressão (handler
      só chama `prevent_default` quando acha uma imagem)
- [x] Teste novo em `crates/vault` cobrindo `save_asset_bytes` — 3
      testes
- [x] `cargo test --workspace`, `cd ui && cargo test --lib`,
      `trunk build`, `cargo build --manifest-path src-tauri/Cargo.toml`
      passam
- [x] Validação ao vivo via MCP `tauri`: evento `paste` sintético
      (`DataTransfer` + `dt.items.add(file)`) com um PNG fake de 12
      bytes — arquivo criado em `assets/colado-1.png` com os bytes
      exatos, `<img>` inserido no DOM, e `![imagem colada](assets/
      colado-1.png)` persistido no `.md` ao salvar

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

`gloo_file::futures::read_as_bytes` (crate `gloo-file`, já dependência
do `ui`) cobre a leitura de `File`→`Vec<u8>` sem precisar de
`web_sys::FileReader` manual com closures.

**Descoberta durante a implementação**: drag-and-drop de imagem já
existe (`on_drop`, provavelmente de um ciclo anterior), mas usa uma
URL `blob:` (só válida durante a sessão do navegador) em vez de
persistir em `assets/` — a imagem droppada "funciona" visualmente na
hora mas quebra ao recarregar o app, porque a blob URL não sobrevive.
Diferente do paste (que agora persiste de verdade), esse é um bug
pré-existente, fora do escopo deste ciclo — o `on_drop` teria o mesmo
tipo de correção que este ciclo aplicou (chamar `save_pasted_asset`
em vez de `createObjectURL`), candidato a um ciclo futuro.

## Resultado

# Ciclo 118 - done

## Resumo

Colar (Ctrl+V) uma imagem no editor agora grava de verdade em
`assets/` (via novo comando `save_pasted_asset`) e insere
`![](assets/colado-N.ext)` no cursor — antes, `assets/` só era
gerenciado (listar/excluir), sem fluxo de captura direto no editor.

## Arquivos criados/modificados

- `crates/vault/src/io.rs` — `save_asset_bytes`, 3 testes
- `crates/ipc/src/lib.rs` — `handle_save_pasted_asset` (+ dep `base64`)
- `crates/ipc/Cargo.toml` — dependência `base64`
- `src-tauri/src/main.rs` — comando Tauri `save_pasted_asset`
- `ui/src/api.rs` — `save_pasted_asset`
- `ui/src/components/editor.rs` — handler `on_paste`, `onpaste` nos
  dois contenteditable
- `ui/Cargo.toml` — dependência `base64`

## Testes

`cargo test --workspace`: 107. `cd ui && cargo test --lib`: 75. Total 182.
`trunk build` + `cargo build --manifest-path src-tauri/Cargo.toml`: OK.

Validação ao vivo via MCP `tauri` (processo de dev reiniciado): evento
`paste` sintético com um PNG fake de 12 bytes — arquivo criado em
`assets/colado-1.png` com bytes exatos, `<img>` inserido no DOM,
markdown persistido corretamente ao salvar. Artefatos de teste
revertidos/removidos depois.

## Notas

Descoberta relevante: drag-and-drop de imagem já existia (`on_drop`)
mas usa uma `blob:` URL de sessão em vez de persistir — quebra ao
recarregar. Bug pré-existente, não corrigido aqui (fora do escopo,
documentado no arquivo de task como candidato a ciclo futuro).

Próximo: sincronização via git (119).
