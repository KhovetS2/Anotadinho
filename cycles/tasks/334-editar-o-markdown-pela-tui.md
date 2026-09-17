---
id: "334"
titulo: "Editar o markdown pela TUI"
status: done
criado: 2026-09-17
autor: agente
prioridade: alta
depende_de: ["333"]
---

# 334 — Editar o markdown pela TUI

Módulo `app/markdown.rs`: a mesma gramática vale nos blocos da página, do
corpo de um callout e dos painéis de colunas.

- `i`/`a`/`I`/`A`/`cc` editam o TEXTO do bloco (com `**`, `[[ ]]` à vista);
  a marca do bloco (`## `, `- [ ] `, `> `) fica fora e volta sozinha. A
  inserção aparece no lugar da linha; num painel, no rodapé.
- `Enter` confirma e abre o bloco seguinte (item depois de item, com a
  caixa vazia e o número seguinte); `Esc` só confirma.
- `o`/`O` criam depois/antes; `dd` apaga; `yy`/`p`/`P` copiam e colam.
- `>>`/`<<` trocam o bloco de lugar com o vizinho (itens dentro da lista,
  blocos no nível da página) sem mexer na separação.
- `Ctrl+A`/`Ctrl+X` mudam o nível do título; `~` marca a caixa do item.
- `x` avisa em vez de apagar (não há cursor de caractere no modo normal).
- No PRÓPRIO embed, `o`/`O`/`dd`/`yy`/`p`/`>>` tratam o embed como bloco da
  página.
- Parágrafo de várias linhas vira uma linha ao ser reescrito.
- Página com frontmatter: ele fica intacto.
