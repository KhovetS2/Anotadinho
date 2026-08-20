---
id: "168"
titulo: "Editar propriedade direto na consulta"
status: pending
criado: 2026-08-20
autor: humano
prioridade: alta
depende_de: ["154"]
estima_min: 90
agente_alvo: claude-opus-5
---

# Editar propriedade direto na consulta

## Objetivo

O embed de consulta (154) é somente leitura por decisão daquele ciclo.
Na prática isso quebra o fluxo do painel: você vê a spec em `backlog`,
decide começar, e precisa abrir a página, achar o painel de
propriedades e voltar. "Ver" e "agir" ficam em lugares diferentes.

Este ciclo deixa editar, NA PRÓPRIA LINHA, os campos que a consulta já
mostra em `columns`.

## Critérios de aceite

- [ ] Célula de um campo listado em `columns` vira editável no clique
      (e no Enter, pelo teclado), com o mesmo visual de célula da
      tabela embedada
- [ ] A escrita passa por `MarkdownCodec::set_frontmatter_field` — o
      mesmo caminho do `anotadinho-cli set-property` e do embed de
      ações, sem um terceiro jeito de gravar frontmatter
- [ ] Depois de gravar, a consulta reavalia: se a página deixou de bater
      com o filtro, ela sai da lista na hora (é o feedback de que a
      ação funcionou)
- [ ] Campo com poucos valores conhecidos no vault (ex: `status`) sai
      como lista de opções, não campo de texto livre — as opções vêm do
      que já existe nas páginas do recorte
- [ ] Página aberta noutra aba não é sobrescrita com estado velho
- [ ] Erro de escrita aparece pro usuário, não em silêncio
- [ ] Testes do motor: reavaliação depois da edição, e edição de campo
      que não está em `columns` não é oferecida

## Comandos de validação

```bash
cargo test --workspace
cd ui && cargo test --lib
cd ui && trunk build
```

## Não-objetivos

- Editar título/corpo da página pela consulta (só frontmatter)
- Criar página pela consulta (é o embed de ações)

## Notas

Fecha o par com a task 163 (modal de configuração de botão): as duas
juntas tiram o painel do estágio "mostra e manda abrir" pra "resolve
ali".
