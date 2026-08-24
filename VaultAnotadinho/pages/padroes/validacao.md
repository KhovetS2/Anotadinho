---
title: Validação
date: 2026-08-24
dominio: processo
tags:
- padrao
---
# Validação

## Quando se aplica

No fim de todo ciclo, antes do status e do commit.

## A sequência

```bash
cargo test --workspace
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
node scripts/uitest/run.mjs        # se mexeu em UI
```

## As regras

1. **`cargo test` não pega bug de DOM.** Mexeu em UI, o harness é
   obrigatório.
2. **Não tente subir o app.** `dev.sh` abre janela e não retorna. Quem
   deixa o app de pé é a pessoa.
3. **Não conseguiu validar? Diga.** Um ciclo de UI sem harness é meio
   ciclo, e dizer o contrário é pior do que não ter rodado.
4. **Regressão nova vira cenário**, com o número do ciclo onde
   apareceu.
5. **Baseline de snapshot só se regrava depois de OLHAR a tela.**
   Regravar pra calar o teste desfaz o motivo de ele existir.
6. **Meça antes de afirmar.** "Ficou mais rápido" sem número é opinião.
7. **Amostre a pilha antes de culpar o próprio código.** Travamento com
   backend em 0% e renderizador em 100% pode não ser seu.

## Armadilhas conhecidas

- **Fixture com data fixa envelhece.** Um cenário com `2026-08-10`
  cravado ficou vermelho sozinho quando a data passou. Use datas
  relativas a hoje.
- **Snapshot guarda pixel absoluto.** Janela mais estreita faz
  `grid-template-columns` diferir em tudo sem mudança de estilo;
  proporção constante denuncia o falso positivo.
- **Dois apps no mesmo vault** escrevem nos mesmos arquivos. Dev usa a
  porta 9223, o instalado 9323.
- **Clique e leitura no mesmo tick** pegam a tela de antes: o Yew só
  re-renderiza no tique seguinte.
