# Paridade janela × TUI

Comparação lado a lado do que a janela (Yew/Tauri, `ui/`) faz e do que a
TUI (`crates/tui`) faz. Atualizado a cada ciclo da série 342+. Legenda:
✅ igual · 🟡 parcial · ❌ falta · — não se aplica ao terminal.

## Navegação e estrutura

| Janela | TUI | Ciclo / nota |
|---|---|---|
| Sidebar em árvore de pastas, filtro | ✅ | 299, 292 |
| Seção Journals + "Journal de hoje" | 🟡 | "Ir pra Hoje" na barra (339); sem seção própria |
| Nova página / por tipo | ✅ | 339 |
| Nova pasta, nova página na pasta | ❌ | |
| Mover página pra pasta | ❌ | |
| Excluir página | ✅ | 339 |
| Exportar pasta / vault | ❌ | |
| Abas de documentos (Ctrl+1..9) | ❌ | |
| Paleta de comandos (Ctrl+K) | ✅ | 339 |
| Busca no CONTEÚDO na paleta (FTS) | ✅ | 342 — `Ctrl+F` (ou Enter sem resultado) |
| Atalhos (`?`) | ✅ | 339 |
| Wikilink `[[Título]]` abre a página | ✅ | 342 — Enter; oferece criar a que não existe |
| Autocompletar wikilink ao digitar | ❌ | |
| Recarregar quando o arquivo muda por fora | ❌ | a TUI só recusa gravar |
| Escolher / criar vault | — | a TUI recebe `--vault` |
| Controles da janela | — | |

## Páginas de tipo

| Janela | TUI | Ciclo / nota |
|---|---|---|
| Markdown com embeds | ✅ | |
| `type: conversa` | ✅ | 340, 341 |
| `type: propostas` (revisar diff, aplicar/recusar) | ❌ | |
| `type: tags` (tags do vault e onde aparecem) | ❌ | |
| `type: assets` (arquivos, uso, excluir) | ❌ | |
| `type: kanban` / `calendar` / `table` de página inteira | ❌ | |
| `type: graph` | — | grafo 3D; talvez lista de conexões |
| Cabeçalho de página tipada + propriedades | 🟡 | 344 — propriedades pela barra |

## Editor

| Janela | TUI | Ciclo / nota |
|---|---|---|
| Editar texto dos blocos | ✅ | 334 (modo vim) |
| Menu `/` (inserir bloco e embed) | ❌ | |
| Barra de seleção (negrito, itálico, link, cor) | 🟡 | marcas digitadas à mão |
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
| Galeria: escolher arquivo de assets | 🟡 | caminho digitado |
| Fluxo: transições, nota | ✅ | 335 |
| Fluxo: planejar/executar/pedir alteração (abre conversa) | ❌ | |

## Agente e vault

| Janela | TUI | Ciclo / nota |
|---|---|---|
| Conversa, anexos, prompt padrão, virar spec/proposta/execução | ✅ | 340, 341 |
| Configurar agente (binário, args, pasta, pastas extras) | 🟡 | só presets |
| Propostas do agente (diff, aplicar, recusar) | ❌ | |
| Git: status, pull, commit & push | ❌ | |
| Nova página a partir de template | 🟡 | 342 — pelo botão de ação; sem comando próprio |

## Aparência e teclado

| Janela | TUI | Ciclo / nota |
|---|---|---|
| Tema | ✅ | 339 |
| Cor de destaque, botões | ❌ | |
| Sidebar escondível | ✅ | 339 |
| Remapear atalhos globais e do vim | ❌ | |
