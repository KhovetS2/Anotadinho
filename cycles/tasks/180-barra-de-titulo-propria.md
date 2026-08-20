---
id: "180"
titulo: "Barra de título própria"
status: done
criado: 2026-08-21
autor: humano
prioridade: media
depende_de: []
estima_min: 90
agente_alvo: claude-opus-5
---

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
- [x] Arrastar o header move a janela; duplo clique maximiza
      (`data-tauri-drag-region`)
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
