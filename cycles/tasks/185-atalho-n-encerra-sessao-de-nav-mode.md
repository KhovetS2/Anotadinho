---
id: 185
titulo: "Atalho `n`: entrar em digitação encerra a sessão de nav-mode"
status: done
depende_de: [181, 184]
---

## Objetivo

Depois do `n`, o bloco de ORIGEM continuava com o retângulo azul do
nav-mode aceso e o indicador `-- NAV: editor-blocos --` continuava na
tela — mesmo com o cursor já no bloco novo, digitando. As setas também
seguiam navegando entre blocos em vez de andar no texto.

## Critérios de aceite

- [x] `n` num bloco de texto apaga o destaque do nav-mode e derruba a
      sessão (indicador some, setas voltam a andar no texto).
- [x] `n` sobre um embed faz o mesmo.
- [x] Os dois cenários de harness (181 e 184) conferem
      `.nav-mode__item-active` zerado depois do atalho.

## Validação

- `cargo build --workspace`, `cargo test --workspace`
- `cargo build --manifest-path src-tauri/Cargo.toml`
- `cd ui && trunk build`
- `node scripts/uitest/run.mjs`

## Não-objetivos

- Mudar quando o nav-mode COMEÇA (isso é `on_enter_block_nav`, ciclo 174).
- Edição estruturada por bloco (ciclo 175, adiado).

## Notas

São dois estados independentes e os dois precisavam cair: a classe
`nav-mode__item-active` (vive no DOM, gerenciada por
`nav_mode::focus_item`) e o `nav_mode_active`/`nav_stack` do `app.rs`
(que decide para onde vão as setas). O `Enter` num bloco já fazia as
duas coisas desde o ciclo 174; o `n` do 181 nasceu sem elas.
