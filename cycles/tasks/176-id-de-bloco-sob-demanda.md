---
id: "176"
titulo: "Id de bloco sob demanda"
status: pending
criado: 2026-08-20
autor: humano
prioridade: baixa
depende_de: ["174", "170"]
estima_min: 120
agente_alvo: claude-opus-5
---

# Id de bloco sob demanda

## Objetivo

Terceira fatia. Referenciar um bloco ESPECÍFICO ao longo do tempo
(transcluir `![[página^id]]`, backlink que aponta pro parágrafo, e não
só pra página) exige um identificador estável no arquivo.

A preocupação do usuário — "não quero um arquivo poluído" — é o
requisito central deste ciclo: **id só é escrito no bloco que alguém
de fato referenciou**, nunca em todos.

## Critérios de aceite

- [ ] Copiar referência de um bloco (ação no bloco focado) grava um
      `^id` curto no fim daquela linha, e SÓ nela
- [ ] Bloco nunca referenciado continua sem marca nenhuma no `.md`
- [ ] `^id` é renderizado como marca discreta (ou escondido), não como
      texto solto no meio da nota
- [ ] Id sobrevive a editar o texto do bloco, a mover o bloco de lugar
      e a reordenar a página
- [ ] Id colidindo (arquivo copiado, dois blocos com o mesmo) é
      resolvido na leitura, sem quebrar as referências existentes
- [ ] `anotadinho-cli` sabe resolver `página^id` (ler o bloco)
- [ ] Testes no core: extrair id, escrever id só onde pedido,
      round-trip, colisão

## Comandos de validação

```bash
cargo test -p anotadinho-core
cargo test -p anotadinho-cli
cargo test --workspace
```

## Não-objetivos

- Migrar o vault pra ter id em todo bloco (é exatamente o que NÃO se
  quer)
- Id em bloco dentro de embed (o embed já tem estrutura própria)

## Notas

Mesma convenção do Obsidian (`^id` no fim da linha), que é a mais
compatível com vault existente e a menos intrusiva — e mantém o `.md`
legível fora do app, que é premissa do projeto.
