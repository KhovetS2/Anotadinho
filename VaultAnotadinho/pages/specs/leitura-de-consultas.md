---
title: "Leitura de consultas: alinhamento, cor e altura"
type: spec
date: 2026-08-23
status: em-revisao
prioridade: alta
tags:
- spec
- ui
---
# Leitura de consultas: alinhamento, cor e altura

{{ type: "fluxo" }}
artefato: spec
etapa: em-revisao
{{ /fluxo }}

## Contexto

As consultas viraram a principal forma de ler o vault — a página de
[[Ciclos]] e o [[Início]] são feitos delas. Com o vault em 200+ páginas,
três problemas de leitura ficaram evidentes.

**Valores desalinhados da coluna.** Numa consulta em tabela, os valores
de `type` e `prioridade` não ficam alinhados com o título da coluna, o
que obriga a conferir posição em vez de bater o olho.

**Tudo em cinza.** Os valores de propriedade são texto puro. Distinguir
"alta" de "baixa" ou "spec" de "ciclo" exige ler, quando deveria ser
percebido pela cor — que é o que o embed de tabela e o de kanban já
fazem com badge.

**Bloco sem fim.** Uma consulta que devolve 168 ciclos ocupa uma página
inteira de rolagem, e passar por ela pra chegar na consulta seguinte é
um exercício de paciência.

## Requisitos funcionais

- **RF1.** Numa consulta em tabela, cada valor fica alinhado com o
  título da sua coluna.
- **RF2.** Valores de propriedade aparecem com cor consistente: o mesmo
  valor tem sempre a mesma cor, em qualquer consulta da mesma página.
- **RF3.** O bloco de consulta tem altura máxima e rola internamente,
  sem empurrar o resto da página.
- **RF4.** A altura máxima é ajustável por consulta, pra um recorte
  curto não ganhar barra de rolagem desnecessária.
- **RF5.** É visível que há mais conteúdo abaixo do que a área mostra.

## Requisitos não funcionais

- **RNF1.** Cor não pode ser a ÚNICA forma de distinguir um valor —
  o texto continua lá, legível.
- **RNF2.** A cor sai do valor de forma determinística, sem estado
  guardado: a mesma propriedade não pode mudar de cor entre sessões.
- **RNF3.** O `.md` não muda: isto é apresentação.
- **RNF4.** O contraste atende leitura em tema claro e escuro.

## Critérios de aceite

- [ ] Numa consulta com quatro colunas, todos os valores ficam sob o
      título correto.
- [ ] Dois valores iguais em consultas diferentes da mesma página têm a
      mesma cor.
- [ ] Uma consulta com 168 resultados ocupa altura limitada e rola
      dentro de si.
- [ ] O snapshot visual dos embeds continua passando (ou é atualizado de
      propósito, com a mudança registrada na task).

## Fora de escopo

- Escolher cor manualmente por valor.
- Ordenar ou agrupar clicando no cabeçalho.

## Relacionado

- [[Ciclos]] — o caso que motivou
- [[Consultas — listas que se mantêm sozinhas]]
