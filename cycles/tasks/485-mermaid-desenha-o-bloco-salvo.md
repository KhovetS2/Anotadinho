---
id: "485"
titulo: "O mermaid desenha o bloco salvo"
status: done
criado: 2026-10-23
autor: agente
prioridade: media
depende_de: ["484"]
estima_min: 240
---

# 485 — O mermaid desenha o bloco salvo

O 484 achou um problema: um bloco ` ```mermaid ` salvo vira
`<pre><code class="language-mermaid">` e nunca é desenhado. O mermaid só
era chamado sobre o `div.mermaid` que o `/mermaid` insere, e o
diagrama voltava a ser código ao salvar e reabrir. E `init_mermaid_at`
sobrescrevia `mermaid.initialize` com o objeto de configuração, em vez de
chamá-la. Hoje são 60 MiB de biblioteca carregada sem desenhar nada.

## Desenho

O `<pre>` continua sendo o bloco, e a fonte da verdade. O diagrama vai
como imagem SVG num `::after` do `<pre>`, por variável CSS inline:

- **Por que não um elemento dentro do `<pre>`:** o yank e a seleção do
  vim e várias outras rotinas leem o `text_content` do bloco inteiro.
  Os rótulos do diagrama vazariam pra lá.
- **Por que não um elemento irmão:** mover o bloco, juntar com o
  anterior no Backspace e o vim andam pelos irmãos.
- **Por que o `::after`:** um pseudo-elemento não é nó do DOM. Ele não
  entra no `text_content`, não é irmão de ninguém e não é selecionável.

O comportamento:
- Fora de foco, o bloco mostra só o diagrama. Com foco, mostra o código
  e o diagrama.
- Se o código tiver erro, o bloco mostra o código e a mensagem de erro.
- Ao sair do bloco, o diagrama é redesenhado. Isso vem de um único
  ouvinte de `focusout` no documento, e não de um closure por página.

## Critérios de aceite

- [x] Um bloco ` ```mermaid ` salvo aparece desenhado ao abrir a página.
- [x] Salvar não muda o arquivo: a fence volta igual, sem estilo, classe
      nem texto do SVG.
- [x] O `text_content` do bloco é só o código.
- [x] Editar o código e sair do bloco redesenha o diagrama.
- [x] Um erro de sintaxe aparece no bloco, e o código continua editável.
- [x] O `/mermaid` insere o mesmo `<pre>`, que passa pelo mesmo caminho.
- [x] `mermaid.initialize` é chamada, e não sobrescrita.
- [x] Cenário de UI que confere o desenho, e não só `typeof mermaid`.
      Ele precisa reprovar no código de antes.

## Comandos de validação

```bash
cd ui && cargo test --lib mermaid
cargo test --workspace
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
node scripts/uitest/run.mjs mermaid
node scripts/uitest/run.mjs
node scripts/uitest/run.mjs --estresse
```

## Não-objetivos

- Carregar o mermaid sob demanda. É o ciclo 486.
- Editar o diagrama visualmente.
