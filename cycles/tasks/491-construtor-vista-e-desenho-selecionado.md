---
id: "491"
titulo: "Construtor mermaid: canvas sem borda, com vista arrastável; o desenho selecionado não vira código"
status: done
criado: 2026-10-23
autor: agente
prioridade: media
depende_de: ["490"]
estima_min: 120
---

# 491 — A vista do canvas e o desenho selecionado

Dois defeitos achados na conferência à mão do construtor (487–490), no app
instalado:

1. **O canvas cortava o diagrama.** O plano era o próprio retângulo de
   640×400, e a arrumação em camadas prendia na borda tudo que passava
   dele. Num fluxograma de oito passos, as três últimas camadas se
   empilharam na linha de baixo ("Executando" sumiu atrás de "Revisão",
   "Feita" e "Bloqueada"). A prévia ao lado também cortava o fim.
2. **Clicar no diagrama mostrava o código.** Desde o 485, o foco no bloco
   trocava o desenho por desenho + código. Quem clica num diagrama quer
   selecioná-lo, não ler a sintaxe.

## Critérios de aceite

- [x] O canvas é uma vista sobre um plano sem borda: a arrumação não prende
      nó na borda, e a vista enquadra todos os nós (afastando até 3×).
- [x] Arrastar o fundo anda a vista; Ctrl+roda (e a pinça) aproxima no
      ponteiro; a roda sozinha anda; os botões −, + e Ajustar ficam no
      canto do canvas.
- [x] A vista para de se enquadrar sozinha quando a pessoa começa um gesto
      (o canvas não foge debaixo do ponteiro) e volta a enquadrar com
      Ajustar ou com um nó novo.
- [x] A prévia rola em vez de cortar o topo ou o fim.
- [x] Bloco desenhado: o foco mantém o desenho, marcado como selecionado,
      com o botão "Editar no construtor"; Enter abre o construtor; nada que
      se digite, cole ou apague muda o código escondido.
- [x] Bloco com erro de sintaxe continua mostrando e editando o código ali
      mesmo (é o único jeito de consertar sem abrir o modal).

## Comandos de validação

```bash
cargo test --workspace
cd ui && cargo test --lib && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
node scripts/uitest/run.mjs "(485)"
node scripts/uitest/run.mjs "(49[01])"
node scripts/uitest/run.mjs
node scripts/uitest/run.mjs --estresse
```

## Não-objetivos

- Apagar o bloco do diagrama por Backspace/Delete com ele selecionado. Hoje
  o `beforeinput` é recusado; apagar segue pela seleção que atravessa
  blocos.
- O pontilhado do fundo acompanhar a vista ao arrastar.
