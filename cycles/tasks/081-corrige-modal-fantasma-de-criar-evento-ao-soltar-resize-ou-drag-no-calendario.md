---
id: "081"
titulo: "Corrige modal fantasma de criar evento ao soltar resize ou drag no calendario"
status: done
criado: 2026-08-07
autor: humano
prioridade: alta
depende_de: ["071"]
estima_min: 90
agente_alvo: claude-sonnet
---

# Corrige modal fantasma de criar evento ao soltar resize/drag no calendário

## Objetivo

Usuário reportou: soltar o mouse depois de redimensionar (arrastar a
borda) um evento do calendário abria o diálogo de "novo evento", como se
tivesse clicado numa área vazia — e a atualização do resize que estava
em andamento não era aplicada. Raiz: o `onclick` do dia/coluna (usado
pra criação rápida de evento) não tinha nenhuma proteção contra um
"clique fantasma" que o navegador pode gerar logo depois do mouseup de
um resize/arraste.

## Critérios de aceite

- [x] Soltar um resize não abre mais o diálogo de novo evento
- [x] A atualização do resize (novo horário) é aplicada normalmente
- [x] Mesma proteção aplicada em todos os pontos de início de
      arraste/resize (mês e semana/dia: barra, bloco com horário, item
      da gaveta, as duas alças de resize)
- [x] Reset adiado da proteção (não fica "travada" pra sempre se por
      algum motivo nenhum clique disparar depois de um arraste)

## Comandos de validação

```bash
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Investigar/mudar o mecanismo de clique-fantasma em si (é comportamento
  do navegador) — só proteger contra o EFEITO dele

## Notas

**Causa raiz confirmada**: o `onclick` do dia/coluna (`render_month_grid`
e `render_day_columns`) abre o diálogo de novo evento incondicionalmente,
sem checar se um resize/arraste tinha acabado de acontecer. Quando o
usuário solta o mouse depois de arrastar a borda de um bloco, o
navegador pode gerar um evento `click` sintético no elemento sob o
cursor (o dia/coluna, não o bloco) — sem proteção, esse `onclick` sempre
abria o diálogo por cima da atualização que tinha acabado de ser
commitada no próprio `mouseup` do resize.

**Fix**: `suppress_click_ref` (`use_mut_ref`, não `use_state` — precisa
refletir o valor mais atual mesmo lido de dentro de um handler montado
ANTES do mousedown que o setou, mesma razão do `edited_ref` do ciclo
074). Setado `true` em TODOS os pontos que iniciam um arraste/resize
(barra do mês, barra/bloco com horário da semana/dia, item da gaveta, as
duas alças de redimensionar); os `onclick` de criar evento (mês e
semana/dia) checam e resetam o flag no início — se `true`, ignoram o
clique. Os dois listeners globais de `mouseup` (o "zera o arraste"
sempre-ativo e o do efeito de resize) também resetam o flag, mas de
forma ADIADA (`spawn_local` + sleep de 0ms) — precisa continuar `true`
durante o "click" sintético que dispara LOGO depois do mouseup (mesmo
task síncrona), mas não pode ficar `true` pra sempre, senão um clique de
verdade bem mais tarde (sem nenhum resize/arraste por perto) seria
ignorado por engano.

Validado ao vivo via MCP `tauri`: resize de "Deploy produção" (14:30–
15:15 → 14:30–16:00), soltando o mouse e IMEDIATAMENTE disparando um
`click` sintético no mesmo `mousedown+mouseup` (mesma execução síncrona,
pra simular fielmente o timing real do navegador) — sem diálogo de novo
evento, e o modal do evento confirmou o novo horário aplicado (16:00).
Testado também sem regressão no arrastar-mover normal (mês).

**Nota de ambiente**: durante a validação, a janela do Tauri ficou
travada (até leitura de log via MCP dava timeout) depois de alguns ciclos
de hot-reload consecutivos — reiniciado `cargo tauri dev`/`trunk serve`
limpo resolveu (a nova instância sobe o bridge MCP numa porta diferente,
9224 em vez de 9223 — precisa reconectar o driver na porta nova). Não
parece relacionado ao código deste ciclo especificamente, mas registrado
caso se repita.
