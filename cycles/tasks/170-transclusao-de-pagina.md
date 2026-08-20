---
id: "170"
titulo: "Transclusão de página"
status: pending
criado: 2026-08-20
autor: humano
prioridade: media
depende_de: []
estima_min: 120
agente_alvo: claude-opus-5
---

# Transclusão de página

## Objetivo

O painel (160) consegue LISTAR páginas, nunca MOSTRAR o conteúdo delas.
Pra montar um dashboard de verdade — "a missão do produto aqui em cima,
o padrão de nomenclatura ali embaixo" — hoje só copiando texto, que
duplica a fonte da verdade.

Transclusão resolve: `![[Missão]]` renderiza o conteúdo daquela página
no lugar, sempre atualizado.

## Critérios de aceite

- [ ] `![[Página]]` no markdown renderiza o corpo da página alvo
      (sem o frontmatter), com um cabeçalho discreto que leva pra ela
- [ ] `![[Página#Seção]]` traz só a seção daquele heading até o próximo
      do mesmo nível
- [ ] Ciclo de transclusão (A inclui B que inclui A) para no primeiro
      nível repetido, com aviso no lugar — nunca laço infinito
- [ ] Embed dentro de página transcluída renderiza como embed de
      verdade, em modo somente leitura (editar continua sendo na página
      de origem)
- [ ] Alvo inexistente mostra o nome pedido e um jeito de criar a
      página, não um buraco
- [ ] Conta como backlink (painel de backlinks e grafo enxergam)
- [ ] Testes do parser no core: `![[x]]` vs `[[x]]`, com âncora, dentro
      de fence de código (não transclui), aninhado

## Comandos de validação

```bash
cargo test -p anotadinho-core
cargo test --workspace
cd ui && trunk build
```

## Não-objetivos

- Editar a página de origem pelo bloco transcluído
- Transcluir bloco específico por id (`![[x^bloco]]`) — precisa de id
  de bloco, que o projeto não tem ainda

## Notas

`crates/core/src/links.rs` já separa alvo/alias/âncora — o parser de
transclusão deve reusar isso em vez de reimplementar.
