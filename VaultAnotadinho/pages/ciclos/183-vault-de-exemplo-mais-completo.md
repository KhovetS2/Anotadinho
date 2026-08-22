---
title: Ciclo 183 — Vault de exemplo mais completo
type: ciclo
ciclo: "183"
status: concluida
date: 2026-08-21
prioridade: media
depende_de: []
tags:
- ciclo
---

# Ciclo 183 — Vault de exemplo mais completo

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Vault de exemplo mais completo

## Objetivo

Pedido do usuário: o `VaultAnotadinho` mostrava só três dos nove
embeds, e nada dos recursos que entraram depois (consulta agrupada,
transclusão, id de bloco). Quem abre o app pela primeira vez não tem
como descobrir o que ele faz sem ler o código.

## Critérios de aceite

- [x] Os NOVE tipos de embed aparecem em alguma página do vault, com
      conteúdo real (não "lorem ipsum")
- [x] Página inicial reconstruída como índice navegável dos exemplos,
      usando os próprios embeds (callout + botões + consulta viva)
- [x] Uma página por tema, em vez de uma página gigante:
      composição, consultas, referências
- [x] Transclusão demonstrada nas três formas (página inteira, seção,
      bloco) com alvos que existem de verdade no vault
- [x] Galeria com imagens de verdade em `assets/` (geradas, pequenas)
- [x] Cada exemplo mostra também o comando de terminal equivalente,
      quando existe
- [x] Tudo conferido renderizando no app, não só no arquivo

## Comandos de validação

```bash
anotadinho-cli --vault VaultAnotadinho embed list pages/exemplos-embeds.md
anotadinho-cli --vault VaultAnotadinho query --from-embed pages/exemplos/consultas.md:0
cargo test --workspace
```

## Não-objetivos

- Tutorial passo a passo (o guia do Agent OS já é o texto longo; estas
  páginas são vitrine)
- Traduzir o vault (segue em português, como o resto)

## Notas

Três imagens de exemplo foram GERADAS (gradientes de 480×300, ~1,8 KB
cada) em vez de trazer arquivo de fora — a galeria precisava de
conteúdo real, e imagem de terceiro num repositório de exemplo é
dívida de licença esperando acontecer.

Dois achados durante a escrita:

1. `![[X]]` entre crases virava transclusão — virou o ciclo 182.
2. O teste `exemplos_embeds_vault_file_parses` (que pina a estrutura da
   página de demo contra o parser) falhou assim que a página cresceu.
   Fez exatamente o que devia; a expectativa foi atualizada pros dois
   embeds novos.

Transclusão de bloco da PRÓPRIA página é barrada junto com a
auto-transclusão, então o exemplo aponta pra um bloco de outra página
(`consultas.md^neq`) — e a página de referências documenta esse limite
em vez de escondê-lo.

## Resultado

# Ciclo 183 - done

## Resumo

O vault de exemplo passou a mostrar os nove embeds e os recursos novos
(consulta agrupada, transclusão nas três formas, id de bloco), com
conteúdo real e uma página inicial que serve de índice.

## Arquivos criados/modificados

- `VaultAnotadinho/pages/incio.md` — reconstruída como índice, com
  callout, botões de ação, consulta viva e agenda
- `VaultAnotadinho/pages/exemplos/composicao.md` (novo) — callout,
  colunas, galeria
- `VaultAnotadinho/pages/exemplos/consultas.md` (novo) — lista, tabela,
  cartões, agrupada com contagem, tabela de operadores
- `VaultAnotadinho/pages/exemplos/referencias.md` (novo) — wikilink,
  transclusão de página/seção/bloco
- `VaultAnotadinho/pages/exemplos-embeds.md` — ganhou cronograma e
  ações, e virou índice dos outros
- `VaultAnotadinho/assets/exemplo-{grafo,fluxo,quadro}.png` (novos)
- `crates/core/src/embed.rs` — expectativa do teste de sincronia

## Testes adicionados

- Nenhum novo: o conteúdo é conferido pelo teste de sincronia que já
  existia (`exemplos_embeds_vault_file_parses`) e pela renderização ao
  vivo.

## Problemas encontrados

- `![[X]]` em código inline virava transclusão → ciclo 182.
- O teste de sincronia falhou ao a página crescer — funcionou como
  esperado; expectativa atualizada.
- Bloco da própria página não pode ser transcluído (regra do 170), então
  o exemplo aponta pra outra página e documenta o limite.

## Notas para próximos ciclos

- As imagens da galeria são geradas por script, não copiadas: nada de
  licença de terceiro no repositório.
