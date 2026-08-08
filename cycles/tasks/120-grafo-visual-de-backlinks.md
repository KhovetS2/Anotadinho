---
id: "120"
titulo: "Grafo visual de backlinks"
status: pending
criado: 2026-08-08
autor: humano
prioridade: baixa
depende_de: ["087", "088"]
estima_min: 90
agente_alvo: claude-sonnet
---

# Grafo visual de backlinks

## Objetivo

Backlinks existem (ciclo 088) mas só como lista textual on-demand por
página. Novo `page_type: graph` (página inteira) mostra todas as
páginas do vault como nós e wikilinks como arestas, num SVG simples —
ajuda a navegar um vault de specs/decisões que já está crescendo.

## Critérios de aceite

- [ ] Novo `page_type` "graph", dispatch em `page_view.rs` (mesmo
      padrão de `kanban`/`calendar`/`tags`)
- [ ] Componente novo `ui/src/components/graph_view.rs`: varre todas
      as páginas por `[[wikilink]]` (reaproveita a lógica de scan já
      usada pra backlinks no editor), monta nós (uma página = um nó,
      rótulo = título) e arestas (um wikilink = uma aresta)
- [ ] Layout simples — nós num círculo (`2πi/n` por índice), SEM física
      de force-directed (evita dependência nova); arestas como linhas
      SVG entre os nós
- [ ] Clicar um nó abre a página (mesmo `on_page_selected` usado em
      todo o resto do app)
- [ ] Vault grande (100+ páginas) não trava a UI — cálculo é O(n²) no
      pior caso pro scan de wikilinks; aceitável pro tamanho atual,
      documentar a limitação
- [ ] `cargo test --workspace`, `cd ui && cargo test --lib`,
      `trunk build`, `cargo build --manifest-path src-tauri/Cargo.toml`
      passam
- [ ] Validação ao vivo via MCP `tauri`: criar página `type: graph`,
      confirmar que nós/arestas aparecem coerentes com os wikilinks
      reais do vault de demonstração

## Comandos de validação

```bash
cargo test --workspace
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Layout force-directed/físico de verdade — círculo simples é
  suficiente pra v1; layout melhor é um ciclo futuro se o vault
  crescer a ponto do círculo ficar ilegível
- Zoom/pan interativo — SVG estático que cabe no viewport
- Filtrar o grafo por pasta/tag — mostra o vault inteiro sempre

## Notas

Reaproveita o scan de wikilinks já usado pelo painel de Backlinks
(`editor.rs`, ciclo 088) — extrair essa lógica de scan pra uma função
compartilhada (`ui/src/wikilink.rs` ou novo módulo) em vez de
duplicar, já que agora tem dois consumidores.
