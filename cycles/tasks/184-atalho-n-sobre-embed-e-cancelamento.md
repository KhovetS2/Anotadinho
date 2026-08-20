---
id: 184
titulo: "Atalho `n`: funciona sobre embed e Escape cancela sem deixar lixo"
status: done
depende_de: [181]
---

## Objetivo

Fechar os dois furos que apareceram no uso real do atalho `n` (ciclo 181):

1. Cancelar o menu `/` com `Escape` deixava a barra `/` digitada no texto,
   virando uma linha solta no `.md` na próxima gravação.
2. O atalho não respondia quando o foco estava num controle de **embed** —
   só funcionava com um bloco de texto focado.

## Critérios de aceite

- [x] `Escape` no menu `/` aberto pelo `n` apaga o `/` digitado e não deixa
      linha solta no arquivo salvo.
- [x] `n` com um controle de embed focado insere um segmento de markdown
      logo DEPOIS do embed e abre o menu `/` nele.
- [x] O embed e o texto ao redor sobrevivem ao round-trip de gravação.
- [x] Cenário novo no harness de UI cobrindo os dois casos.

## Validação

- `cargo build --workspace`, `cargo test --workspace`
- `cargo build --manifest-path src-tauri/Cargo.toml`
- `cd ui && trunk build`
- `node scripts/uitest/run.mjs`

## Não-objetivos

- Edição estruturada por bloco (ciclo 175, adiado).
- Mudar a lista de itens do menu `/`.

## Notas

O motivo do (2) não é o `segmento_do_embed_focado()` e sim ONDE o handler
estava: os controles de um embed ficam FORA de qualquer `contenteditable`,
então a tecla nunca subia até o `onkeydown` do editor. O tratamento desse
caso desceu pro contêiner `.editor__wysiwyg-segments`, que é ancestral tanto
dos blocos de texto quanto dos embeds.

Também ficou de fora do escopo, mas corrigido junto: o cenário 180 do harness
conferia `window_is_maximized` no mesmo tick do `window_toggle_maximize` e
falhava de forma intermitente — o comando volta quando PEDE ao gerenciador de
janelas, não quando o estado muda. Virou polling curto.
