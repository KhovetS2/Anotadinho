---
id: "482"
titulo: "Investigar a memória que a página inicial deixa presa"
status: done
criado: 2026-10-23
autor: agente
prioridade: alta
depende_de: ["480"]
estima_min: 240
---

# 482 — A memória da página inicial

O ciclo 480 mediu o app instalado. O pico vinha da página inicial
desenhada sem nenhuma aba, e crescia a cada rodada: 652 → 824 → 1048 →
1194 → 1341 MiB. É o único ponto com crescimento monotônico.

## Critérios de aceite

- [x] Uma sonda que alterna entre duas páginas N vezes e mede o PSS da
      página web depois de cada ida, com controle (duas páginas leves).
- [x] Separar a causa: as consultas, o embed de ações, o tamanho do vault,
      ou o WebKit (que retém memória mesmo sem vazamento nosso).
- [x] Se a causa for nossa, corrigir. O ciclo só fecha com a sonda
      mostrando o crescimento por visita perto do controle.
- [x] Se for do WebKit, dizer com a medida que separa as duas coisas, e
      não corrigir às cegas.

## Comandos de validação

```bash
node scripts/memoria/sonda-pagina.mjs --porta 9323
```
