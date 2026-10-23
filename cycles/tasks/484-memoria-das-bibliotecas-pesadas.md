---
id: "484"
titulo: "Quanto pesam as bibliotecas JS: carregar e usar"
status: done
criado: 2026-10-23
autor: agente
prioridade: media
depende_de: ["482", "483"]
estima_min: 180
---

# 484 — A memória das bibliotecas pesadas

O 483 mostrou que o processo do app não tem o que cortar. Faltam ~250 MiB
no processo da página web: 253 MiB recém-aberto e ~440 MiB depois de um
dia, com 18 MiB de WASM e ~2 mil nós de DOM. A hipótese é que esse peso
vem das bibliotecas JS. O `index.html` carrega as três na abertura, mesmo
para quem nunca as usa:
- `mermaid` (3,2 MB);
- `3d-force-graph`, com o three (1,3 MB);
- `highlight` (126 KB).

## Critérios de aceite

- [x] Custo de CARREGAR: cada biblioteca sozinha no MiniBrowser do
      WebKitGTK, comparada a uma página em branco. Mede o PSS do
      processo da página.
- [x] Custo de USAR, no app: recarregar a página web, medir, usar a
      biblioteca uma vez (um diagrama, o grafo, um bloco de código) e
      medir de novo, com as abas fechadas. *Medido, e a medida mostrou o
      próprio limite: recarregar não devolve memória, então o primeiro uso
      foi medido isolado no MiniBrowser, com um processo novo por medida.*
- [x] Controle: a mesma sequência abrindo uma nota sem nada pesado.
- [x] Um script em `scripts/memoria/` que repete a medida.
- [x] Dizer, com os números, quanto um carregamento sob demanda
      economizaria. A troca, se valer, é outro ciclo.

## Comandos de validação

```bash
node scripts/memoria/bibliotecas.mjs --porta 9323
```

## Não-objetivos

- Mudar o carregamento das bibliotecas neste ciclo.
- Trocar de biblioteca.
