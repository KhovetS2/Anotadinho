---
id: "488"
titulo: "Construtor mermaid: o modal, a aba Código e a prévia"
status: done
criado: 2026-10-23
autor: agente
prioridade: media
depende_de: ["487"]
estima_min: 300
---

# 488 — Modal, código e prévia

Segunda etapa da proposta `pages/propostas/construtor-mermaid.md`.

## Critérios de aceite

- [x] O `/mermaid` abre o modal no lugar do campo de uma linha. "Inserir"
      põe um bloco ` ```mermaid ` onde o `/` estava, e ele aparece
      desenhado (RF1).
- [x] Um bloco existente tem "Editar no construtor", por botão e por
      `Ctrl+Enter`. "Aplicar" troca só aquele bloco (RF2).
- [x] O modal tem as abas Construtor e Código, sobre o mesmo texto, e a
      prévia ao vivo (RF3).
- [x] Fora do modelo, o modal abre no código com o motivo, e a aba
      Construtor fica indisponível (RF7).
- [x] Abrir e aplicar sem mexer não muda nada, nem marca "não salvo"
      (RF8).
- [x] Um cenário passa o que o construtor escreve pelo `mermaid.parse` de
      verdade.

## Comandos de validação

```bash
cargo test -p anotadinho-core --lib diagrama
cargo test --workspace
cd ui && cargo test --lib && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
node scripts/uitest/run.mjs "(488)"
node scripts/uitest/run.mjs
node scripts/uitest/run.mjs --estresse
```

## Não-objetivos

- A edição visual de cada tipo. A sequência e o gantt são do 489, e o
  fluxograma e os estados, do 490. Até lá, a aba Construtor mostra o que
  leu e manda para o código.
