---
id: "222"
titulo: "Travamento do compositor com driver NVIDIA"
status: done
criado: 2026-08-23
autor: humano
prioridade: alta
depende_de: ["221"]
estima_min: 60
agente_alvo: claude-opus-5
---

# Travamento do compositor com driver NVIDIA

## Objetivo

A janela congelou e ficou congelada. Descobrir por quê, sem adivinhar.

## Critérios de aceite

- [x] Causa identificada por amostragem de pilha, não por hipótese
- [x] Mitigação aplicada antes do WebView subir, valendo pro RPM e pro dev
- [x] Só age quando há NVIDIA proprietária carregada
- [x] Nunca por cima de escolha explícita da pessoa
- [x] Confirmado que a variável chega ao processo de renderização

## Comandos de validação

```bash
cargo test --workspace
cargo build --manifest-path src-tauri/Cargo.toml
node scripts/uitest/run.mjs
```

## Notas

### O que a pilha disse

Três amostras do processo travado com `gdb`, todas no mesmo lugar:

```text
WebCore::BitmapTexturePool::releaseUnusedTexturesTimerFired()
  → WebCore::BitmapTexture::~BitmapTexture()
    → libnvidia-eglcore.so.580.173.02
```

O compositor liberando textura de GPU e ficando preso dentro do driver.
Três evidências de que não é laço do nosso código: o backend fica em 0%
o tempo todo, não há chamada de IPC nenhuma, e a pilha não passa por
JavaScript nem por Yew em momento algum.

Máquina: NVIDIA proprietária (GTX 1060), `__GLX_VENDOR_LIBRARY_NAME=nvidia`.
O app não tinha mitigação nenhuma — nem no `dev.sh`, nem no `.desktop`,
nem no código.

### A hipótese que estava errada

Antes de olhar a pilha, o palpite era um laço de rolagem na
virtualização do ciclo 217: `onscroll` grava `scroll_top`, e um efeito
sobre `scroll_top` escreve `scrollTop` de volta, o que dispara `scroll`
outra vez.

O mecanismo existe mesmo no código, mas NÃO é o que travou: reproduzir
com a consulta de 168 ciclos, rolando de propósito, deu 0% de CPU. A
pilha desmentiu o palpite. Fica registrado porque o padrão é frágil e
pode morder depois.

### O limite honesto desta correção

Não deu pra reproduzir o travamento. Então isto não é um conserto
provado: é a saída conhecida pra uma interação conhecida, escolhida a
partir de onde a pilha mostrou que o processo está preso.

Se voltar a travar com a variável ativa, o próximo passo é
`WEBKIT_DISABLE_COMPOSITING_MODE=1`, que é mais pesado mas tira o
compositor do caminho de vez.

### Uma fragilidade do snapshot, de passagem

A suíte acusou `grid-template-columns` diferente em `columns` e
`calendar`. Não era regressão: a janela do dev abriu com 841px de
largura, e a baseline foi gravada num tamanho bem maior. As PROPORÇÕES
continuavam idênticas (1:2 nos dois casos), que é o que denuncia o
falso positivo. Anotado no cabeçalho do `snapshot.mjs`.
