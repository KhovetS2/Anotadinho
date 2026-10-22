---
id: "466"
titulo: "Sidebar: pastas aninhadas abrem vazias — trocar o <details> nativo por estado"
status: done
criado: 2026-10-21
autor: agente
prioridade: alta
depende_de: ["465"]
estima_min: 90
---

# 466 — Pastas da sidebar sem `<details>` nativo

Relato da pessoa, com imagem: "Está com esse bug na hora de abrir as
pastas". Em `projetos > anotadinho`, a pasta aparece aberta, mas embaixo
dela sobra um vazio de ~330 px, com as pastas seguintes empurradas pra
baixo e nada desenhado no lugar.

## O que foi medido (e o que não deu pra medir)

- **O vazio tem o tamanho exato do conteúdo de `anotadinho`.** Na
  imagem, `anotadinho` está em y≈420 e `prompts-default` em y≈776. Com
  conversas, tarefas (aberta) e as duas páginas, o corpo mede ~290–330
  px.
- **Por script, layout e DOM ficam certos em todas as combinações de
  abrir e fechar.** Abri e fechei de fora pra dentro e de dentro pra
  fora. Os itens existem, estão `visible`, com opacidade 1, e respondem
  ao `elementFromPoint`, sem buraco nenhum.
- **A pintura NÃO deu pra medir:** o screenshot da janela não funciona no
  Linux (ciclo 460). Sobra a hipótese que bate com o sintoma: o
  conteúdo está no layout e não foi pintado. No WebKit recente, o
  conteúdo de um `<details>` fechado é escondido por
  `content-visibility`, e a pasta da sidebar é um `<details>` CONTROLADO
  pelo Yew, que mexe no atributo `open` por script a cada clique. É o
  caminho menos exercitado do navegador.
- **Por que só agora:** até o módulo Projeto, nenhuma pasta tinha pasta
  dentro de pasta. `pages/projetos/<slug>/tarefas` é o primeiro
  aninhamento de três níveis, e ele só passou a existir na sidebar
  depois que o 458 subiu a profundidade da varredura.

## Critérios de aceite

- [x] A pasta é um cabeçalho comum (`role="button"`,
      `aria-expanded`) que alterna o estado. Os filhos só existem no DOM
      com a pasta aberta, sem depender de `<details>` nem de `ontoggle`.
- [x] Os botões do cabeçalho ("+" e exportar) continuam funcionando sem
      abrir nem fechar a pasta.
- [x] As pastas continuam nascendo recolhidas (ciclo 207), e a navegação
      por teclado (setas) continua abrindo e fechando.
- [x] Harness: o cenário das pastas recolhidas passa sem `details` e
      ganha o caso aninhado (abrir 3 níveis, fechar o do meio, reabrir).
- [ ] A pessoa confirma na janela de verdade, porque a pintura não dá
      pra medir daqui.

## Comandos de validação

```bash
cd ui && trunk build
node scripts/uitest/run.mjs "pasta"
node scripts/uitest/run.mjs
```
