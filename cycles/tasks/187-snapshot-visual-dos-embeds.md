---
id: "187"
titulo: "Snapshot visual dos embeds no harness"
status: pending
criado: 2026-08-20
autor: humano
prioridade: media
depende_de: [177]
estima_min: 90
agente_alvo: claude-opus
---

# Snapshot visual dos embeds no harness

## Objetivo

Os 19 cenários do harness testam COMPORTAMENTO. Nada pega regressão
visual — CSS de um embed vazando pro outro, badge sem contraste, grade
quebrada — que é justamente o que só aparece abrindo a página. Este
ciclo acrescenta um passo de captura por tipo de embed, comparado com
uma baseline versionada.

## Critérios de aceite

- [ ] `scripts/uitest/snapshot.mjs`: monta uma página com um embed de
      cada tipo, captura a região de cada um e compara com
      `scripts/uitest/baseline/<tipo>.png`.
- [ ] Comparação por diferença de pixel com tolerância configurável —
      antialiasing não pode reprovar.
- [ ] `--atualizar` regrava a baseline; sem a flag, diferença acima da
      tolerância falha e grava o diff em `scripts/uitest/diff/`.
- [ ] Roda dentro de `run.mjs` como um cenário a mais, e pode rodar
      sozinho.
- [ ] Baseline dos 9 tipos versionada no repositório.

## Comandos de validação

```bash
node scripts/uitest/run.mjs
node scripts/uitest/snapshot.mjs --atualizar
```

## Não-objetivos

- Snapshot do app inteiro (chrome, sidebar) — o alvo é o embed, que é o
  que muda com frequência.
- Testar temas claro/escuro em separado neste ciclo.

## Notas

A captura sai por `webview_screenshot` do bridge MCP, recortada pelo
`getBoundingClientRect` do embed. Fixar largura de janela antes de
capturar, senão a baseline vira loteria.

PNG na baseline pesa; manter as capturas pequenas (só a região do
embed, não a tela) pra não repetir o problema do ciclo 148.
