---
id: "159"
titulo: "Toolbar do embed: mover e duplicar"
status: pending
criado: 2026-08-19
autor: humano
prioridade: baixa
depende_de: []
estima_min: 60
agente_alvo: claude-sonnet
---

# Toolbar do embed: mover e duplicar

## Objetivo

O `.embed-hover-wrapper` (ciclos 075 e 083) só sabe inserir uma linha
acima/abaixo e remover o embed. Com 9 tipos de embed depois desta
série, montar uma página vira um exercício de ordenação — e hoje
reordenar significa recortar e colar YAML na mão. Como o wrapper age
no nível do `DocSegment`, mover e duplicar valem pra todos os tipos de
uma vez, com o mesmo código.

## Critérios de aceite

- [ ] Botões novos no wrapper: mover pra cima, mover pra baixo,
      duplicar
- [ ] Mover troca o embed de posição com o segmento vizinho do mesmo
      nível no `Vec<DocSegment>` e re-`join`; o markdown entre os dois
      é preservado (o embed passa pro outro lado do trecho, o texto
      não some nem duplica)
- [ ] Botão desabilitado quando não há pra onde mover (primeiro/último
      segmento)
- [ ] Duplicar insere uma cópia idêntica logo abaixo, com uma linha em
      branco entre as duas (mesma regra do ciclo 075: `join` não
      escreve segmento vazio, então o separador precisa ser explícito)
- [ ] Ações refletem no editor ao vivo, sem precisar salvar (mesma
      regra do ciclo 079)
- [ ] Undo (ciclo 095) desfaz mover e duplicar
- [ ] `data-nav-item` nos botões novos, na mesma ordem visual
- [ ] Testes puros sobre `Vec<DocSegment>`: mover primeiro pra cima é
      no-op, mover último pra baixo é no-op, mover no meio preserva
      todo o texto ao redor, duplicar não altera o original

## Comandos de validação

```bash
cargo build --workspace
cargo test --workspace
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Arrastar o embed pra reordenar (drag-and-drop dentro do
  contenteditable conflita com seleção de texto — ver ciclo 068)
- Recortar/colar embed entre páginas diferentes

## Notas

Ícones novos em `icon.rs`: `arrow-up`, `arrow-down`, `copy`.
