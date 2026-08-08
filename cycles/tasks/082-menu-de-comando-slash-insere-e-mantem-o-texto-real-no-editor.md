---
id: "082"
titulo: "Menu de comando slash insere e mantem o texto real no editor"
status: done
criado: 2026-08-07
autor: humano
prioridade: alta
depende_de: ["073", "079"]
estima_min: 150
agente_alvo: claude-sonnet
---

# Menu de comando slash insere e mantém o texto real no editor

## Objetivo

Antes deste ciclo, digitar `/` era interceptado no `keydown` e NUNCA
tocava o documento — o filtro (`/consulta`) só existia como estado
interno do Rust, mostrado num cabeçalho flutuante do menu. Usuário pediu
pra `/` (e o texto digitado depois) aparecer de verdade no editor
enquanto filtra, e que Esc/Espaço fechem o menu sem apagar o que foi
digitado.

## Critérios de aceite

- [x] Digitar `/` insere o caractere de verdade no documento (não é mais
      interceptado) e abre o menu
- [x] Continuar digitando filtra o menu ao vivo, com o texto aparecendo
      no documento normalmente
- [x] Selecionar um item (clique ou Enter) apaga só o `/consulta` digitado
      e insere o conteúdo escolhido no lugar
- [x] Esc fecha o menu sem apagar nada — `/consulta` continua como texto
      normal
- [x] Espaço fecha o menu naturalmente (não precisa de tratamento
      especial pra essa tecla — digitar espaço já invalida o casamento
      "/sem-espaço" no próximo `oninput`)
- [x] Setas continuam navegando os itens, Enter continua selecionando o
      destacado

## Comandos de validação

```bash
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Mudar o mecanismo de inserção dos itens NÃO-embed (heading, lista,
  etc) — continuam usando `execCommand` como antes; só o GATILHO/detecção
  do comando slash mudou, não como cada item se insere depois de apagar
  o `/consulta` (isso é o escopo da task #70/próximo ciclo)

## Notas

**Redesenho**: `find_slash_context()` (novo) olha o texto de verdade
imediatamente antes do cursor a cada `oninput` — se casar com
`/consulta` (sem espaço entre o `/` e o cursor, e o `/` no início da
linha ou depois de um espaço, pra não disparar em "3/4"), abre/atualiza
o menu; senão fecha. O `keydown` não intercepta mais `/` nem os
caracteres do filtro nem Backspace — só sobra Escape (fecha sem tocar no
texto)/ArrowUp/ArrowDown (navegam)/Enter (seleciona), todos com
`e.prevent_default()`.

Na hora de aplicar o item escolhido, `select_slash` reconsulta
`find_slash_context()` fresco (mesma função, chamada de novo) pra achar
o nó de texto + posição exata do `/`, e `delete_slash_context_and_collapse`
apaga esses caracteres do nó de texto (`CharacterData::delete_data`) e
recoloca o cursor colapsado exatamente onde o `/` estava — a partir daí
a inserção (embed via `Range`, ciclo 079, ou os outros itens via
`execCommand`) roda normalmente, como se o `/consulta` nunca tivesse
existido.

**Bug relacionado encontrado e corrigido durante a validação**: o
handler de Escape do menu slash não tinha `e.stop_propagation()` — o
evento borbulhava pro atalho GLOBAL do app (`app.rs`) que desseleciona a
página inteira quando Escape é pressionado. Fechar o menu de comando
estava fechando a página junto. Adicionado `stop_propagation()` nos 4
casos tratados (Escape/ArrowUp/ArrowDown/Enter) do bloco `if *slash_open`.

Validado ao vivo via MCP `tauri` (digitação simulada via
`execCommand('insertText', ...)`, que dispara `oninput` de verdade,
igual ao navegador faz de verdade ao digitar):
- `/kan` digitado de verdade no documento → menu abre, filtra pra 1
  resultado (Kanban) → clicar aplica → board de verdade aparece, texto
  `/kan` sumiu
- `/foo` + Escape → menu fecha, `/foo` continua como texto normal, PÁGINA
  continua selecionada (confirma o fix do bug relacionado)
- `/bar hello` (com espaço no meio) → menu fecha sozinho ao digitar o
  espaço, texto completo preservado
- `/lista` → clicar "Lista" aplica corretamente via `execCommand` (item
  não-embed), confirma que o novo gatilho não quebrou o caminho antigo

Nenhuma edição de teste vazou pro vault (revertida com `git checkout`
depois de cada teste).
