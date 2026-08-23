---
title: Abordagem para o uso agêntico
tags:
- proposta
type: proposta
date: 2026-08-22
prioridade: alta
status: aprovada
---
# Abordagem para o uso agêntico

{{ type: "fluxo" }}
artefato: proposta
etapa: aprovada
origem: pages/specs/uso-agentico-do-anotadinho.md

{{ /fluxo }}

## Spec que atende

[[Uso agêntico do Anotadinho no dia a dia]] — cobre os cinco requisitos
funcionais dela.

## Abordagem

Reaproveitar o que já existe em vez de criar mecanismo novo: a conversa
já é uma página, o fluxo já é uma máquina de estados, a proposta já tem
diff e revisão. O que falta são os atalhos entre eles.

## Etapas

1. **Conversa em um passo** (ciclo 208, feito) — comando na paleta, com
   a página aberta anexada e o vínculo no frontmatter.
2. **Contexto anexável** (ciclo 208, feito) — lista de páginas que o
   modelo deve consultar, com seletor filtrável.
3. **Aviso de proposta pendente** — indicador no cabeçalho, alimentado
   pelo `listar_propostas` que já existe.
4. **Executar a partir da proposta** — botão que chama o adaptador com a
   proposta como prompt e registra uma página `type: execucao`.

## Padrões seguidos

- [[Nomenclatura]] — nomes em português no domínio, `handle_` nos
  handlers de IPC.
- [[Capacidades de agente]] — a regra de que nenhuma escrita do agente
  chega ao vault sem revisão.

## Alternativas consideradas

- **Terminal embutido no app.** Descartado: daria um terminal dentro de
  um app de notas, sem criar estrutura nenhuma, e abriria uma superfície
  de risco grande. O protocolo com adaptador plugável cobre o mesmo uso.
- **Guardar a conversa num banco interno.** Descartado: perderia o
  wikilink, o git e a leitura pelo próprio agente.

## Riscos

- **Execução longa travando a UI.** Mitigado com timeout que mata o
  processo (ciclo 202).
- **Injeção pelo conteúdo anexado.** Mitigado por blindagem do contexto
  no prompt e, principalmente, por nada da resposta executar sozinho.
