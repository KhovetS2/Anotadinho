---
id: "248"
titulo: "O autosave não pode gravar vazio sozinho"
status: done
criado: 2026-09-02
autor: humano
prioridade: alta
depende_de: []
estima_min: 60
agente_alvo: claude-sonnet
---

# O autosave não pode gravar vazio sozinho

## Objetivo

Reportado ao vivo (instância WSL): abrir uma página (qualquer uma —
`journals/2026-09-02.md`, depois `pages/ciclos.md`) mostrava, sem
nenhuma edição do usuário, `JsValue("gravação recusada: isso apagaria
as N letras de \"...\". Pra esvaziar de propósito, apague a
página.")`. A trava de `recusar_esvaziamento` (ciclo anterior, comentário
em `crates/ipc/src/lib.rs:150-157`) barrou a escrita — nada se perdeu —,
mas duas coisas estavam erradas: o autosave automático tentou gravar
vazio por cima de conteúdo real sem o usuário ter feito nada, e a
mensagem chegou na tela como o `Debug` cru do `JsValue` em vez do texto.

## Critérios de aceite

- [x] O erro do backend aparece como texto limpo, não `JsValue("...")`
- [x] O autosave automático (debounce de 3s) nunca grava markdown vazio
      por cima de uma página com conteúdo — só o "Salvar" manual, que
      continua passando pela trava do backend (esvaziar de propósito
      continua possível, só não pelo automático)
- [x] Cenário no harness reproduzindo: abrir página com conteúdo, não
      tocar em nada, esperar passar da janela do autosave, conferir que
      nem o overlay de erro aparece nem o disco muda
- [x] **Achado durante a validação, não previsto no objetivo original:**
      `recusar_esvaziamento` só comparava a string INTEIRA — uma
      gravação que zera o CORPO mas mantém o bloco de frontmatter
      (`---\ntitle: x\n---`) não é vazia depois do `trim()`, e passava
      direto pela trava, apagando o conteúdo de verdade sem erro
      nenhum. Corrigido pra comparar o corpo (pós-frontmatter), com
      teste novo provando o caso.

## Comandos de validação

```bash
cargo build --workspace
cargo test --workspace
cd ui && PATH="$HOME/.cargo/bin:$PATH" cargo check --target wasm32-unknown-unknown
node scripts/uitest/run.mjs 248
```

## Não-objetivos

- Achar com certeza QUAL evento dispara o `oninput` vazio no meio do
  carregamento de página (below, em "Notas" — é hipótese, não
  confirmado ao vivo por falta de instrumentação no momento do
  incidente)
- Limpar todo `format!("{:?}", jsvalue_err)` de `ui/src/api.rs` — o
  mesmo padrão (`JsValue("...")` cru na tela) se repete em ~40 outras
  chamadas de `tauri_invoke` no arquivo; só a de `write_page_checked`
  foi corrigida aqui, por ser a implicada neste incidente

## Notas

### O que aconteceu de verdade

Print 1: `journals/2026-09-02.md` (66 caracteres) — overlay vermelho
com `JsValue("gravação recusada: isso apagaria as 66 letras de
\"journals/2026-09-02.md\"...")`. Print 2, minutos depois, depois de
recusar uma proposta do agente: `pages/ciclos.md` (3275 caracteres) —
mesmo overlay, agora citando `ciclos.md`. Perguntado diretamente, o
nome do arquivo na mensagem SEMPRE batia com a página que estava aberta
na hora — ou seja: não é UMA página específica travada num estado
ruim, é a gravação vazia acontecendo de novo a cada página nova
aberta.

### O que foi descartado

- **`recusar_proposta` escrevendo em página.** Não escreve: só apaga o
  `.json` da fila (`handle_recusar_proposta`, `crates/ipc/src/lib.rs`).
  O reload que ela dispara (`vault_version` → efeito em
  `editor.rs:874-908`) só RELÊ a página aberta do disco, e só age se a
  versão bateu diferente — o que não é o caso de uma página não
  tocada.
- **`edited_ref` preso em `true` entre páginas.** É resetado no início
  do efeito de carregamento de CADA página (`editor.rs:520`), não
  vazava de uma pra outra.
- **O `oninput` nativo disparando sozinho por injeção programática de
  HTML.** Não deveria: `set_inner_html`/manipulação direta do DOM não
  dispara evento `input` em navegador nenhum. Só `execCommand` ou
  digitação real disparam.

### Hipótese líder (não confirmada ao vivo)

Alguma populatação inicial de segmento usa `execCommand` (o helper
`doc_exec` existe no arquivo) em vez de manipulação direta do DOM — e
isso DISPARA `input` de verdade. Se isso acontece em mais de um passo
(um segmento de cada vez), o primeiro evento pode ler o
`contenteditable` ainda vazio/parcial via `recompute_markdown_from_dom`
antes do resto ser injetado. `on_edit` chama `mark_edited(md_vazio)`,
que arma o autosave de 3s com esse valor **capturado por valor** — não
recalculado na hora de gravar (`editor.rs:1049`, `1071-1076`). 3
segundos depois, mesmo com o DOM já certo, o autosave grava a
STRING VAZIA que ficou presa na closure.

Não reproduzi isso ao vivo por conta própria (não há instrumentação de
qual evento disparou o `mark_edited` inicial). Mas simulei DIRETO pelo
harness — `el.textContent = ""` seguido de um `InputEvent("input")`
real no bloco do editor, contra a instância dev de verdade — o que deu
o mesmo formato de sintoma. Isso confirmou dois comportamentos, um
esperado e um NÃO:

1. Com o fix do autosave: o `input` vazio simulado NÃO disparou
   gravação nenhuma 4.5s depois — disco intacto, sem overlay. Autosave
   corrigido.
2. Clicar "Salvar" manualmente (de propósito, simulando o usuário
   confirmando) NA MESMA página com o corpo vazio **passou pela trava
   sem erro nenhum e apagou o corpo de verdade**, deixando só
   `---\ntitle: __uitest2\n---\n` no disco. Esse é o buraco descrito no
   critério de aceite novo — achado só por testar de verdade contra o
   app rodando, não por leitura de código.

### Correções

- `ui/src/api.rs::write_page_checked`: erro do backend vira
  `e.as_string().unwrap_or_else(...)` em vez de `format!("{:?}", e)` —
  sem isso, TODA mensagem de erro do `write_page_checked` (não só
  esta) aparecia com o wrapper `JsValue(...)` por cima.
- `ui/src/components/editor.rs::mark_edited_com`: o `persist(md)`
  agendado pro autosave de 3s só dispara se `!md.trim().is_empty()`.
- `crates/ipc/src/lib.rs::recusar_esvaziamento`: compara o corpo via
  `MarkdownCodec::split_frontmatter_text`, não a string inteira — o
  fix que realmente fecha o buraco de perda de dado.
- Cenário novo no harness (`scripts/uitest/cenarios.mjs`): abre página
  com conteúdo, espera 4.5s sem tocar em nada, confere que não aparece
  overlay de erro nem o disco muda.
- Teste novo em Rust (`crates/ipc/src/lib.rs`,
  `esvaziar_so_o_corpo_e_deixar_o_frontmatter_tambem_e_recusado`)
  prova o caso do frontmatter sobrevivendo sozinho.

### Pra quem pegar isso depois

Se acontecer de novo, a pista que falta é: abrir o DevTools (ou plugar
um `console.log` temporário em `on_edit`) bem na hora de abrir uma
página grande, e ver se `oninput` dispara ANTES do conteúdo estar
completo no DOM. Se confirmar, o fix de verdade é não ligar (ou
ignorar) o handler de `oninput` enquanto a página ainda está sendo
populada — não só engolir o resultado vazio depois.
