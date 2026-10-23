---
id: "487"
titulo: "Construtor mermaid: o modelo dos quatro diagramas no núcleo"
status: done
criado: 2026-10-23
autor: agente
prioridade: media
depende_de: ["485"]
estima_min: 300
---

# 487 — O modelo dos diagramas

Primeira etapa da proposta `pages/propostas/construtor-mermaid.md`.

## Critérios de aceite

- [x] `crates/core/src/diagrama/`: `ler(codigo) -> Result<Diagrama,
      ForaDoModelo>` e `escrever(&Diagrama) -> String` para fluxograma,
      sequência, estados e gantt.
- [x] O leitor é estrito. O que ele não reconhece vira `ForaDoModelo`,
      com a linha e o motivo, e nunca é ignorado em silêncio.
- [x] Round-trip:
  - `ler(escrever(d)) == d` para o que o construtor gera;
  - `escrever(ler(x)) == x` para o código canônico;
  - código escrito à mão no modelo (`graph TD; A-->B`, rótulo `-- t -->`,
    participante implícito) dá o mesmo modelo depois de reescrito.
- [x] As posições do canvas vão no comentário `%% anotadinho-posicoes:`.
      Ele é aceito em fluxograma e estados, e o leitor funciona sem ele.
- [x] O texto com `"`, `;` e `:` sai escapado como o mermaid espera
      (`#quot;`, `#59;`, `#58;`) e volta igual.
- [x] RF9: o código mermaid aparece sem ligaduras no bloco.

## Comandos de validação

```bash
cargo test -p anotadinho-core --lib diagrama
cargo test --workspace
cd ui && trunk build
node scripts/uitest/run.mjs mermaid
```

## Não-objetivos

- A UI do construtor. É do 488 em diante.
- Subgrafos, estilos, `classDef`, `click`, blocos `loop`/`alt` da
  sequência e estados compostos. Todos viram `ForaDoModelo`.
