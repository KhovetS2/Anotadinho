---
id: "486"
titulo: "As bibliotecas JS carregam só quando a página precisa"
status: done
criado: 2026-10-23
autor: agente
prioridade: media
depende_de: ["484", "485"]
estima_min: 180
---

# 486 — Bibliotecas sob demanda

O 484 mediu: o `index.html` carrega `mermaid`, `3d-force-graph` e
`highlight` na abertura, e só ter as três carregadas custa ~79 MiB no
processo da página. Só o mermaid são 60 MiB. Quem abre o app e não usa
nenhuma delas paga as três.

## Critérios de aceite

- [x] `ui/src/biblioteca.rs`: `carregar(Biblioteca)` injeta o
      `<script>` uma vez só e devolve se o global existe. Duas chamadas
      ao mesmo tempo esperam o mesmo carregamento.
- [x] O mermaid carrega só quando o contêiner tem um bloco mermaid, o
      highlight só quando há `pre code`, e o grafo só ao abrir o grafo.
- [x] O app recém-aberto, numa nota sem nada disso, não tem nenhum dos
      três globais. Isso é um cenário de UI.
- [x] Os cenários 485 (mermaid), 482 (grafo) e os de realce continuam
      passando. O 243 continua provando que nada vem de fora.
- [x] A medida de antes e depois: o processo da página com o app
      recém-aberto.

## Comandos de validação

```bash
cd ui && cargo test --lib
cargo test --workspace
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
node scripts/uitest/run.mjs
node scripts/memoria/app.mjs --porta 9323
node scripts/memoria/janela.mjs --porta 9323 --rodadas 3
```

## Não-objetivos

- Descarregar uma biblioteca depois de usada. Um script carregado não
  sai do processo.
- Trocar de biblioteca.
