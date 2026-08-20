---
id: "175"
titulo: "Edição estruturada por bloco"
status: pending
criado: 2026-08-20
autor: humano
prioridade: media
depende_de: ["174"]
estima_min: 240
agente_alvo: claude-opus-5
---

# Edição estruturada por bloco

## Objetivo

Segunda fatia do editor de blocos. Com o ciclo 174 dá pra NAVEGAR por
blocos; aqui o bloco passa a ser a unidade de EDIÇÃO: um
`contenteditable` por bloco, em vez de um por segmento de markdown.

É o que destrava reordenar bloco, dobrar/desdobrar e (depois) comentar
por bloco. É também a fatia arriscada: mexe em digitação, que é o
caminho mais usado do app inteiro e a origem histórica de quase todo
bug do editor (076, 078, 079, 082, 111, 141-143).

## Critérios de aceite

- [ ] Um `contenteditable` por bloco, com o markdown do bloco
- [ ] Enter no fim de um bloco cria um bloco novo depois; Enter no meio
      divide o bloco em dois
- [ ] Backspace no início de um bloco funde com o anterior (e no
      primeiro bloco não faz nada)
- [ ] Seleção e cópia atravessando blocos preservam o markdown (colar
      em outro editor tem que sair legível)
- [ ] Colar vários parágrafos cria vários blocos
- [ ] Mover bloco pra cima/baixo por atalho (a mesma ação que a toolbar
      de embed já tem, agora pra qualquer bloco)
- [ ] Desfazer/refazer (ciclo 095) continua funcionando bloco a bloco
- [ ] Atalhos de formatação por prefixo (`#`, `-`, `>`, `[]` — ciclos
      142/143) continuam disparando dentro do bloco
- [ ] O `.md` gerado é byte-idêntico ao de hoje pra uma página que não
      foi editada (teste de regressão sobre `VaultAnotadinho/`)
- [ ] Vim mode continua operando dentro do bloco

## Comandos de validação

```bash
cargo test --workspace
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Arrastar bloco com o mouse (entra depois; o atalho já resolve o caso)
- Blocos aninhados/outliner com indentação (é outro modelo, e o projeto
  é markdown-first, não outliner)

## Notas

Fazer só depois do 174 estar em uso por alguns dias: se a navegação por
blocos já resolver o que o usuário precisa, esta fatia pode nem ser
necessária — e ela é a que mais pode quebrar coisa que funciona.

Recomendação forte: entrar aqui só com o harness de teste de UI de pé
(ver proposta de roadmap), porque os modos de falha desta mudança são
todos de comportamento de DOM.
