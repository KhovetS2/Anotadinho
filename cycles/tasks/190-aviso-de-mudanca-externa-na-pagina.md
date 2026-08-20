---
id: "190"
titulo: "Aviso de mudança externa na página aberta"
status: done
criado: 2026-08-20
autor: humano
prioridade: alta
depende_de: [150]
estima_min: 90
agente_alvo: claude-opus
---

# Aviso de mudança externa na página aberta

## Objetivo

`write_page_checked` já detecta que o arquivo mudou no disco desde a
leitura e grava um bloco de conflito — mas você só descobre DEPOIS, no
arquivo. Como o agente escreve pelo CLI enquanto a janela está aberta,
isso acontece de verdade. Este ciclo avisa antes, com o watcher que já
existe.

## Critérios de aceite

- [x] O editor compara a `page_version` da leitura com a do disco (já
      existia desde o ciclo 173; este ciclo troca o aviso sem saída pela
      barra de decisão).
- [x] Mudança externa SEM edição local pendente: recarrega sozinho e
      mostra "Recarregado do disco".
- [x] Mudança externa COM edição local pendente: barra fixa com
      "Ver a diferença" / "Manter o meu" /
      "Recarregar (perde o que você escreveu)".
- [x] "Ver a diferença" abre o comparativo linha a linha embutido na
      própria barra (não em modal — ver notas).
- [x] Salvar por cima continua passando por `write_page_checked`.
- [x] Dois cenários de harness: com e sem edição pendente.

## Comandos de validação

```bash
cargo build --workspace
cargo test --workspace
cargo build --manifest-path src-tauri/Cargo.toml
cd ui && trunk build
node scripts/uitest/run.mjs
```

## Não-objetivos

- Merge automático de três vias.
- Estado de sincronia do vault INTEIRO (badge de git já cobre parte).

## Notas

Diff linha a linha no core (`crates/core/src/diff.rs`, LCS simples), pra
ser testável fora do WASM e reaproveitável pelo CLI depois.

**Ficou embutido na barra, não em modal.** Um modal precisa ser fechado
antes de decidir, e a decisão é justamente sobre o que o comparativo
mostra — ter os dois na tela ao mesmo tempo é o ponto.

**Bug achado na validação ao vivo:** o comparativo lia `content_md`, e
com isso NÃO mostrava o texto que a pessoa tinha acabado de digitar. A
fonte de verdade do texto local é o DOM (`content_md` só é atualizado em
algumas transições), então o lado "meu" passou a ser recalculado por
`recompute_markdown_from_dom` no momento em que a diferença é aberta. O
cenário de harness trava exatamente isso.

Duas decisões sobre estado:
- O conteúdo de fora é GUARDADO no aviso, não relido do disco na hora do
  clique: entre uma coisa e outra o arquivo pode mudar de novo, e trazer
  algo diferente do que foi mostrado seria pior que não mostrar nada.
- "Manter o meu" adota a `file_version` de fora, e é isso que faz o
  `write_page_checked` do próximo salvamento aceitar gravar por cima —
  sem bloco de conflito no arquivo.

Só o diff mais uma linha de contexto em volta é desenhado: uma página
grande com uma linha alterada não pode virar parede de texto idêntico.
