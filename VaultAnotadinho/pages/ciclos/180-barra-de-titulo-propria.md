---
title: Ciclo 180 — Barra de título própria
type: ciclo
ciclo: "180"
status: concluida
date: 2026-08-21
prioridade: media
depende_de: []
tags:
- ciclo
---

# Ciclo 180 — Barra de título própria

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Barra de título própria

## Objetivo

Pedido do usuário: a barra de título do sistema (com o nome "Anotadinho"
centralizado e os três botões do WM) destoa da identidade visual do
app — é a única faixa da janela que não segue o tema. Este ciclo tira a
decoração do sistema e traz minimizar/maximizar/fechar pro header do
próprio Anotadinho.

## Critérios de aceite

- [x] `"decorations": false` no `tauri.conf.json`
- [x] Minimizar, maximizar/restaurar e fechar como botões do header,
      alinhados à direita, com o visual dos demais controles
- [x] O botão de maximizar troca de ícone e de rótulo conforme o
      estado, inclusive quando a janela abre já maximizada (consulta o
      estado na montagem)
- [x] Arrastar o header move a janela; duplo clique maximiza. Além do
      atributo `data-tauri-drag-region`, precisou de DUAS coisas que não
      são óbvias — ver Notas: a permissão
      `core:window:allow-start-dragging` na capability, e marcar também
      os contêineres e os textos do header (o atributo só age quando o
      alvo do clique É o elemento marcado)
- [x] Redimensionar continua possível: 8 faixas invisíveis nas bordas e
      cantos entregam o arraste pro compositor
      (`start_resize_dragging`)
- [x] Os três botões são alcançáveis pelo nav-mode e por Tab, com foco
      visível
- [x] Cenário no harness: os 3 controles existem, as 8 faixas existem,
      o header é área de arraste, e maximizar/restaurar de verdade
      volta ao estado inicial

## Comandos de validação

```bash
cargo build --manifest-path src-tauri/Cargo.toml
cd ui && trunk build
node scripts/uitest/run.mjs janela
```

## Não-objetivos

- Convenção do macOS (botões à esquerda, em formato de semáforo): o
  layout atual é o de Windows/Linux. Quando alguém empacotar pra mac,
  vira uma variação por plataforma
- Barra de título arrastável em OUTRAS janelas (o app só tem uma)
- Menu de sistema no clique direito da barra

## Notas

**O arraste não funcionou de primeira** (reportado pelo usuário), por
dois motivos somados:

1. O conjunto `core:default` do Tauri 2 **não** inclui
   `allow-start-dragging` — ele traz as consultas de janela e o
   `internal-toggle-maximize` (por isso o duplo clique já funcionava),
   mas não o arraste. Sem a permissão, o pedido era negado em silêncio:
   `window.start_dragging not allowed`. Entrou explícita na capability.
2. `data-tauri-drag-region` só age quando o alvo do clique É o elemento
   marcado. Com o atributo só no `<header>`, a área de arraste era
   apenas o vão entre os dois lados — os contêineres `__left`/`__right`
   e os textos capturavam o resto. Agora eles também têm o atributo.
   Medido: 18 de 25 pontos ao longo do header arrastam; os 7 que não
   são exatamente os botões e seus ícones.

O cenário do harness passou a conferir os dois: que
`plugin:window|start_dragging` não volta com erro de permissão, e que
mais da metade da largura do header arrasta.

`ResizeDirection` não é reexportado pelo crate `tauri` (2.11.5) apesar
de aparecer na assinatura de `start_resize_dragging`, então entrou
`tauri-runtime = "2"` como dependência direta — anotado no `Cargo.toml`
pra ninguém achar que é dependência acidental.

O que só o usuário consegue julgar, porque depende do gerenciador de
janelas dele: se o arraste, o snap (arrastar pro topo/lateral) e o
redimensionar pelas bordas ficaram bons na prática. O que dava pra
verificar por aqui — os controles existirem, maximizar/restaurar
funcionar de verdade e a decoração ter sumido — está no cenário e foi
conferido na janela real.

## Resultado

# Ciclo 180 - done

## Resumo

A barra de título do sistema saiu e os controles de janela viraram
parte do header do Anotadinho. A faixa que destoava do tema não existe
mais.

## Arquivos criados/modificados

- `src-tauri/tauri.conf.json` — `decorations: false`
- `src-tauri/Cargo.toml` — `tauri-runtime` (só pelo `ResizeDirection`)
- `src-tauri/src/main.rs` — `window_minimize`, `window_toggle_maximize`,
  `window_close`, `window_is_maximized`, `window_start_resize`
- `ui/src/api.rs` — as cinco chamadas
- `ui/src/components/header_bar.rs` — botões + `data-tauri-drag-region`
- `ui/src/app.rs` — 8 faixas de redimensionar
- `ui/src/components/icon.rs` — `window-minimize`, `window-maximize`,
  `window-restore`
- `ui/src/styles/main.css` — `.window-controls*` e `.window-resize*`
- `scripts/uitest/cenarios.mjs` — cenário novo

## Testes adicionados

- Cenário de harness: 3 controles, 8 faixas, header como área de
  arraste, e um ciclo maximizar → conferir → restaurar

## Problemas encontrados

- `ResizeDirection` não é reexportado pelo crate `tauri`, apesar de
  estar na assinatura pública de `start_resize_dragging`. Entrou
  `tauri-runtime` como dependência direta, comentada no `Cargo.toml`.
- Durante o ciclo sobraram três instâncias do app (portas 9223/9224/
  9225) por causa de reinícios com erro de compilação no meio —
  derrubadas antes de validar, senão o harness falaria com a janela
  errada.

## Correção (mesmo ciclo, depois do teste do usuário)

O arraste da janela não funcionava. Duas causas somadas:

1. Faltava `core:window:allow-start-dragging` na capability — o
   `core:default` não inclui essa permissão (inclui o
   `internal-toggle-maximize`, e por isso o duplo clique já funcionava).
   O pedido era negado em silêncio.
2. `data-tauri-drag-region` só vale quando o alvo do clique É o
   elemento marcado. Só o `<header>` tinha o atributo, então apenas o
   vão entre os dois lados arrastava.

Ambas corrigidas e cobertas pelo cenário do harness.

## Notas para próximos ciclos

- Arraste, snap e resize pelas bordas dependem do gerenciador de
  janelas: precisam do julgamento do usuário na máquina dele.
- Layout dos botões é o de Windows/Linux; macOS pediria variação.
