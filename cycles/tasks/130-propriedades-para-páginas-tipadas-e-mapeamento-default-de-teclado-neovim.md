---
id: "130"
titulo: "Propriedades para páginas tipadas e mapeamento default de teclado neovim"
status: done
criado: 2026-08-09
autor: humano
prioridade: media
depende_de: ["129"]
estima_min: 60
agente_alvo: claude-sonnet
---

# Propriedades para páginas tipadas e mapeamento default de teclado neovim

## Objetivo

Resolve os dois gaps CONSCIENTES documentados no relatório final do
ciclo 129: (1) páginas de tipo específico (kanban/calendário/tabela/
tags/assets/grafo) não tinham nenhuma forma de acessar o painel de
Propriedades; (2) vários atalhos do `GlobalKeymap` (incluindo "Focar
sidebar") nasciam sem tecla padrão. Além disso, define um mapeamento
default pra TODAS as ações ainda sem tecla, pensado pra quem já usa
neovim no dia a dia (pedido explícito do usuário, usando a config real
dele como referência).

## Critérios de aceite

- [x] Novo componente `TypedPageHeader` (`ui/src/components/
      typed_page_header.rs`) — cabeçalho compartilhado com botão
      "⚙ Propriedades" que abre o mesmo `PropertiesPanel`/`Modal` já
      usado pelo `Editor`, carregando e gravando o frontmatter direto
      (sem a sessão de edição contínua do editor — não precisa, é só
      abrir/editar/fechar)
- [x] `page_view.rs` monta esse cabeçalho acima dos 6 tipos que nunca
      renderizam `Editor` (kanban/calendar/table/tags/assets/graph);
      corpo da página preservado byte-a-byte ao gravar (confirmado ao
      vivo com `kanban-projeto.md`)
- [x] Mudar o campo `type` pelo painel reroteia a página NA HORA (sem
      precisar trocar de página e voltar) — `reload_nonce` em
      `page_view.rs` incrementado via `on_properties_changed`
- [x] `GlobalKeymap::default()` (`ui/src/state.rs`) preenche as 10
      ações que nasciam com `String::new()`: new_folder=f,
      toggle_theme=t, today=d, view_tags=g, view_assets=u,
      close_tab=q, prev_tab=h, toggle_vim_mode=m, focus_sidebar=e,
      focus_editor=l — mnemônicos documentados inline, referenciando a
      config real de neovim do usuário onde fez sentido (focus_editor
      = "l", mesma tecla do `<C-l>` dele pra mover foco pra janela da
      direita; close_tab = "q", mesmo "q" do `:q` do vim)
- [x] Nenhuma tecla nova colide com Ctrl+X/C/V/A (cut/copy/paste/
      select-all nativos do `contenteditable` — únicos atalhos que o
      WebView do Tauri herda de verdade do motor de edição de texto,
      diferente de atalhos tipo Ctrl+N/F que são só convenção de
      CHROME de navegador, ausente num WebView embutido sem chrome)
- [x] Testes atualizados: `global_keymap_default_leaves_new_actions_unbound`
      substituído por `global_keymap_default_binds_neovim_inspired_shortcuts`
      + novo `global_keymap_default_has_no_duplicate_keys`
- [x] `cargo test --workspace`, `cd ui && cargo test --lib`, `trunk
      build`, `cargo build --manifest-path src-tauri/Cargo.toml`
      passam
- [x] Validação ao vivo via MCP `tauri`: botão Propriedades aparece e
      funciona em página kanban real; troca de tipo reroteia na hora
      (testado em página descartável); Ctrl+T/Ctrl+E/Ctrl+D
      confirmados funcionando com keymap limpo (sem uma customização
      antiga salva no localStorage por engano)

## Comandos de validação

```bash
cargo test --workspace
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Mudar as teclas JÁ vinculadas antes deste ciclo (n/k/b/w/s/z/y) —
  pedido do usuário foi preencher o que faltava e pensar em neovim,
  não redesenhar do zero o que já existia e já está documentado/
  memorizado
- Editor rico por tipo de campo no `PropertiesPanel` (número/bool/
  data) — já era um não-objetivo do ciclo 099, continua fora daqui
- Migrar/avisar usuários com uma customização antiga salva no
  `localStorage` — o default novo só vale pra quem NUNCA customizou
  (`load_global_keymap` só cai no `default()` se não houver nada
  salvo, comportamento correto e intencional do sistema de persistência
  já existente)

## Notas

### Painel de Propriedades pra páginas tipadas

Reaproveita o `PropertiesPanel` existente (ciclo 099) sem modificá-lo
— só um novo consumidor, com seu próprio fluxo de leitura/gravação
direto (`api::read_page`/`api::write_page`), mais simples que o do
`Editor` porque não precisa de autosave/dirty-tracking: abre, edita,
grava na hora. Reusa as classes CSS `.editor__header`/`.editor__title`/
`.editor__actions` já existentes — nenhum CSS novo.

### Mapeamento de teclado — processo de decisão

Lido a config real de neovim do usuário (`~/.config/nvim/lua/khovet/`)
antes de escolher as teclas: leader=Space (não aplicável ao esquema
"uma tecla + Ctrl implícito" do Anotadinho), `<C-n>` abre o Neo-tree
(inspirou `focus_sidebar`, mas "n" já estava ocupado por "Nova
página" — usei "e" de Explorer em vez da tecla literal), `<leader>x`
= `:bdelete!` (mesma AÇÃO de `close_tab`, mas leader+x não é "x" sozinho
— usei "q" do `:q` do vim como mnemônico mais direto pra fechar
buffer/aba), `<C-l>` = mover foco pra janela da direita (correspondência
EXATA com `focus_editor`, já que a sidebar fica à esquerda e o editor à
direita no Anotadinho também).

Considerei e REJEITEI usar a/c/v/x pra qualquer ação nova: são
select-all/copy/paste/cut nativos que o WebKitGTK implementa de
verdade dentro de qualquer `contenteditable` (parte do motor de
edição, não do chrome do navegador) — vincular uma ação do app a
essas teclas quebraria cut/copy/paste normal enquanto o usuário edita
texto. Reconsiderei um medo inicial sobre Ctrl+Q ("quit" de app GTK) e
Ctrl+F ("find" de navegador): confirmei que este Tauri app não registra
menu nativo nenhum (`grep -rn "Menu" src-tauri/src/`, vazio) e que
essas são convenções de CHROME de navegador ausentes num WebView puro
sem chrome — não achei motivo real pra evitá-las.

**Achado de teste, não bug**: ao validar Ctrl+T ao vivo, o toggle não
tinha efeito nenhum — intrigante até eu inspecionar
`localStorage.getItem('anotadinho.global_keymap')` e achar um keymap
customizado salvo de teste anterior (`new_page: "t"` etc). Confirma que
`load_global_keymap()` (comportamento pré-existente, correto) só cai no
novo `default()` quando NADA foi salvo — limpei o localStorage da
janela de teste pra validar o default de verdade, o que é exatamente o
que um usuário com instalação nova veria.

Cheatsheet (`?`, ciclo 108/129) não precisou de nenhuma mudança — lê
`global_keymap.labeled_fields()` ao vivo, então os 10 atalhos novos já
aparecem lá automaticamente.

### Build Windows

Investigado em paralelo (pedido do usuário) — máquina sem mingw-w64
nem alvo Rust `x86_64-pc-windows-gnu`. Alvo Rust instalado (não precisa
sudo). Toolchain mingw-w64 precisa de `sudo dnf install mingw64-gcc
mingw64-gcc-c++ mingw64-winpthreads-static` — usuário está rodando.
Build em si é o próximo passo depois que o toolchain estiver instalado
(não coberto por este arquivo de task — ver próximo status/commit).
