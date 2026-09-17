# Paridade janela × TUI

Comparação lado a lado do que a janela (Yew/Tauri, `ui/`) faz e do que a
TUI (`crates/tui`) faz. Atualizado a cada ciclo da série 342+. Legenda:
✅ igual · 🟡 parcial · ❌ falta · — não se aplica ao terminal.

## Navegação e estrutura

| Janela | TUI | Ciclo / nota |
|---|---|---|
| Sidebar em árvore de pastas, filtro | ✅ | 299, 292 |
| Seção Journals + "Journal de hoje" | ✅ | 339, 355 — seção no fim da sidebar, o mais novo em cima |
| Nova página / por tipo | ✅ | 339 |
| Nova pasta, nova página na pasta | ✅ | 345 — `O`/`o` na sidebar, "Nova pasta…" |
| Mover página pra pasta | ✅ | 345 — `m` na sidebar |
| Excluir página | ✅ | 339, 345 — `dd` na sidebar |
| Exportar pasta / vault | ✅ | 345 — grava `anotadinho-<pasta>.md` na pasta atual |
| Abas de documentos (Ctrl+1..9) | ✅ | 354 — `Alt+1…9`, `Ctrl+W`/`Alt+H/L`, `Alt+Q` fecha; barra na borda de cima |
| Paleta de comandos (Ctrl+K) | ✅ | 339 |
| Cabeçalho: propostas pendentes e status do git | ✅ | 355 — no rodapé da sidebar |
| Busca no CONTEÚDO na paleta (FTS) | ✅ | 342 — `Ctrl+F` (ou Enter sem resultado) |
| Atalhos (`?`) | ✅ | 339 |
| Wikilink `[[Título]]` abre a página | ✅ | 342 — Enter; oferece criar a que não existe |
| Autocompletar wikilink ao digitar | ✅ | 352 — `[[` na inserção; ↑↓, Enter/Tab completa, Esc fecha a lista |
| Recarregar quando o arquivo muda por fora | ✅ | 351 — relê a cada segundo parado, sem perder o lugar; lista de páginas também |
| Escolher / criar vault | — | a TUI recebe `--vault` |
| Controles da janela | — | |

## Páginas de tipo

| Janela | TUI | Ciclo / nota |
|---|---|---|
| Markdown com embeds | ✅ | |
| `type: conversa` | ✅ | 340, 341 |
| `type: propostas` (revisar diff, aplicar/recusar) | ✅ | 346 — `a`/`r`, `v` Diff/Visualização |
| `type: tags` (tags do vault e onde aparecem) | ✅ | 346 — "Ver tags"; `h`/`l` + Enter abre |
| `type: assets` (arquivos, uso, excluir) | ✅ | 346 — "Ver assets"; `x`/`dd` exclui |
| `type: kanban` / `calendar` / `table` de página inteira | ✅ | 353 — quadro com `column::`, calendário do vault, tarefas com `status::` |
| `type: graph` | ✅ | 361 — grafo 3D vira lista de conexões por wikilink; Enter abre |
| Cabeçalho de página tipada + propriedades | ✅ | 344, 360 — título da página e "Propriedades" (`=`) nas telas de tipo |

## Editor

| Janela | TUI | Ciclo / nota |
|---|---|---|
| Editar texto dos blocos | ✅ | 334 (modo vim) |
| Menu `/` (inserir bloco e embed) | ✅ | 347 — `/` num bloco novo vazio, ou "Inserir bloco ou embed…" |
| Barra de seleção (negrito, itálico, link, cor) | ✅ | 357 — na inserção: `Ctrl+B` negrito, `Ctrl+T` menu Formatar (itálico, riscado, código, link, cor, fundo) |
| Painel de propriedades (frontmatter) | ✅ | 344 — "Propriedades da página…" |
| Imagem colada/arrastada, Mermaid, PDF | — | terminal |
| Desfazer/refazer | ✅ | 322 |
| Salvar (Ctrl+S) | ✅ | grava a cada edição |

## Embeds

| Janela | TUI | Ciclo / nota |
|---|---|---|
| Desenho dos 10 tipos | ✅ | 325–336 |
| Kanban: cartão/coluna criar, mover, reordenar | ✅ | 319, 337, 338 |
| Kanban: modal do cartão (descrição, tags, prazo, checklist, comentários, anexos) | ✅ | 343 — Enter no cartão |
| Calendário: modal do evento (datas, horário, tags) | ✅ | 343 — Enter no evento |
| Tabela: células, linhas, colunas, tipo, opções | ✅ | 321, 337, 339 |
| Consulta: configuração completa | ✅ | 335, 338, 344 — `=` na consulta |
| Ações: EXECUTAR o botão | ✅ | 342 — abrir, template, propriedade, busca |
| Ações: configurar o botão (ação, destino, ícone) | ✅ | 344 — `=` no botão |
| Galeria: escolher arquivo de assets | ✅ | 356 — `o` lista as imagens de assets/ (ou digita o caminho) |
| Fluxo: transições, nota | ✅ | 335 |
| Fluxo: planejar/executar/pedir alteração (abre conversa) | ✅ | 348 — Enter na ação; executar continua na conversa de origem; botão "origem" |

## Agente e vault

| Janela | TUI | Ciclo / nota |
|---|---|---|
| Conversa, anexos, prompt padrão, virar spec/proposta/execução | ✅ | 340, 341 |
| Configurar agente (binário, args, pasta, pastas extras) | ✅ | 350 — "Configurar agente…" (formulário, valida `{prompt}`) |
| Propostas do agente (diff, aplicar, recusar) | ✅ | 346 — "Propostas do agente" na barra |
| Git: status, pull, commit & push | ✅ | 349 — "Git: status e sincronizar…" na barra; histórico da página |
| Nova página a partir de template | ✅ | 350 — "Nova página" pergunta o template quando há |

## Aparência e teclado

| Janela | TUI | Ciclo / nota |
|---|---|---|
| Tema | ✅ | 339 |
| Cor de destaque, botões | ✅ | 358 — "Cor de destaque…" e "Estilo dos botões…" (também em Personalizar), gravados |
| Sidebar escondível | ✅ | 339 |
| Remapear atalhos globais e do vim | ✅ | 359 — "Remapear teclas…": ações do vim de uma tecla e comandos globais, com checagem de repetição |
