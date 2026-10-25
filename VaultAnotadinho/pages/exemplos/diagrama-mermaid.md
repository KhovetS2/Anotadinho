---
title: Diagrama mermaid
tags:
- exemplo
---
# Diagrama mermaid

Um bloco ````mermaid` vira desenho ao abrir a página. Clique no
diagrama para ver e editar o código; clique fora para ele redesenhar.

## Fluxo de uma tarefa

```mermaid
graph TD
  A[Ideia]
  B[Detalhando]
  C[Pronta]
  D{Executar?}
  E[Executando]
  F[Revisão]
  G[Feita]
  H[Bloqueada]
  N1(Nó 9)
  A --> B
  B --> C
  C --> D
  D -->|sim| E
  D -->|não| C
  E --> F
  F -->|aprovada| G
  F -->|voltou| B
  E -->|parou| H
  G -.-> N1
```

## Quem fala com quem

```mermaid
sequenceDiagram
  participant P as Pessoa
  participant A as Agente
  participant V as Vault
  P->>A: pede uma mudança
  A->>V: propor
  V-->>P: proposta na fila
  P->>V: aprova
```