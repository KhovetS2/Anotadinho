---
id: "186"
titulo: "Desfazer/refazer que entende de blocos"
status: done
criado: 2026-08-20
autor: humano
prioridade: alta
depende_de: [149, 159]
estima_min: 120
agente_alvo: claude-opus
---

# Desfazer/refazer que entende de blocos

## Objetivo

**Correção da premissa original:** a pilha própria já existia desde o
ciclo 074 — o `Ctrl+Z` do editor NÃO é o nativo do `contenteditable`, é
um histórico de snapshots do markdown inteiro. O bug real, achado ao ler
o código, é outro: a decisão de agrupar snapshots era só temporal (janela
de 800ms), então uma mutação ESTRUTURAL disparada logo depois de digitar
caía dentro da janela, não virava ponto de desfazer, e o estado
pré-mutação sumia do histórico. Desfazer pulava para um estado bem mais
antigo, comendo a digitação junto.

Este ciclo separa as duas coisas: digitação agrupa, mutação estrutural
nunca agrupa. E o histórico, que era duas `Vec<String>` soltas dentro do
componente, vira um tipo testado no core.

## Critérios de aceite

- [x] `crates/core/src/history.rs` com `History` (`registrar`,
      `desfazer`, `refazer`, `pode_desfazer`, `pode_refazer`,
      `reiniciar`, limite de profundidade), testado fora do WASM.
- [x] Toda mutação estrutural do editor (inserir pelo menu `/`,
      inserir/remover/mover/duplicar segmento, mudança de dados de embed,
      gravar `^id` de bloco) cria um ponto de desfazer próprio,
      independente do relógio.
- [x] Digitação continua agrupando numa janela de 800ms — uma rajada é
      um passo só, não um passo por tecla.
- [x] `Ctrl+Z` depois de inserir um embed logo após digitar tira só o
      embed e mantém o texto.
- [x] Trocar de página e recarga externa reiniciam o histórico.
- [x] Cenário de harness cobrindo exatamente o caso do bug.

## Comandos de validação

```bash
cargo build --workspace
cargo test --workspace
cargo build --manifest-path src-tauri/Cargo.toml
cd ui && trunk build
node scripts/uitest/run.mjs
```

## Não-objetivos

- Desfazer com granularidade de caractere sobre a pilha própria — isso é
  o desfazer nativo e continua sendo dele.
- Persistir histórico entre sessões.

## Notas

Guardar o markdown COMPLETO por entrada, não um diff: o documento cabe
folgado em memória, e um snapshot elimina a classe inteira de bug de
"patch aplicado fora de ordem". O limite de profundidade existe só pra
página gigante não crescer sem fim.

É pré-requisito de conforto pro ciclo 175: mover bloco pelo teclado sem
poder desfazer assusta.
