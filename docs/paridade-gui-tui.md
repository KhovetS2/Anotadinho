# Paridade janela × TUI

Comparação lado a lado do que a janela (Yew/Tauri, `ui/`) faz e do que a
TUI (`crates/tui`) faz. Atualizado a cada ciclo da série 342+. Legenda:
✅ igual · 🟡 parcial · ❌ falta · — não se aplica ao terminal.

## Navegação e estrutura

| Janela | TUI | Ciclo / nota |
|---|---|---|
| Sidebar em árvore de pastas, filtro | ✅ | 299, 292 |
| Busca da sidebar também no conteúdo ("Resultados") | ✅ | 370 — a partir de 3 letras, com o trecho |
| Abrir resultado de busca já no trecho | ✅ | 371, 397 — pela âncora vai ao embed e ao registro certos; dobras abertas |
| Seção Journals + "Journal de hoje" | ✅ | 339, 355 — seção no fim da sidebar, o mais novo em cima |
| Nova página / por tipo | ✅ | 339, 384 — inclui "Nova página inicial (landing)" |
| Nova pasta, nova página na pasta | ✅ | 345 — `O`/`o` na sidebar, "Nova pasta…" |
| Mover página pra pasta | ✅ | 345 — `m` na sidebar |
| Excluir página | ✅ | 339, 345 — `dd` na sidebar |
| Exportar pasta / vault | ✅ | 345 — grava `anotadinho-<pasta>.md` na pasta atual |
| Abas de documentos (Ctrl+1..9) | ✅ | 354 — `Alt+1…9`, `Ctrl+W`/`Alt+H/L`, `Alt+Q` fecha; barra na borda de cima |
| Paleta de comandos (Ctrl+K) | ✅ | 339 |
| Cabeçalho: propostas pendentes e status do git | ✅ | 355 — no rodapé da sidebar |
| Busca no CONTEÚDO na paleta (FTS) | ✅ | 342, 379 — aparece enquanto digita (3 letras); `Ctrl+F` lista só o conteúdo |
| Atalhos (`?`) | ✅ | 339 |
| Embed seguinte/anterior, foco na sidebar/editor | ✅ | 381 — `Alt+.`/`Alt+,`, `Ctrl+E`/`Ctrl+L` |
| Atalhos globais padrão da janela (Ctrl+N/F/T/B/D/G/U) | ✅ | 382 — remapeáveis |
| Wikilink `[[Título]]` abre a página | ✅ | 342 — Enter; oferece criar a que não existe |
| Links de fora (texto, célula de URL, imagem da galeria) abrem no sistema | ✅ | 374 — Enter; `xdg-open`/`open` |
| Autocompletar wikilink ao digitar | ✅ | 352 — `[[` na inserção; ↑↓, Enter/Tab completa, Esc fecha a lista |
| Recarregar quando o arquivo muda por fora | ✅ | 351, 398 — pelo watcher do vault (na hora), sem perder o lugar; consulta periódica de reserva |
| Conflito: mudou no disco durante a edição (ver diferença, manter o meu, recarregar) | ✅ | 363 |
| Escolher / criar vault | ✅ | 372 — sem `--vault` reabre o último; pasta vazia pergunta (ou `--criar`) e prepara o vault novo, abrindo o guia |
| Trocar de vault sem sair (abrir outro, criar novo) | ✅ | 373 — "Abrir outro vault…", "Criar vault novo…" |
| Controles da janela | — | |

## Páginas de tipo

| Janela | TUI | Ciclo / nota |
|---|---|---|
| Markdown com embeds | ✅ | |
| `type: conversa` | ✅ | 340, 341 |
| `type: propostas` (revisar diff, aplicar/recusar) | ✅ | 346, 399 — `a`/`r`, `v` Diff/Visualização (desenhada como página, com embeds) |
| `type: tags` (tags do vault e onde aparecem) | ✅ | 346 — "Ver tags"; `h`/`l` + Enter abre |
| `type: assets` (arquivos, uso, excluir) | ✅ | 346 — "Ver assets"; `x`/`dd` exclui |
| `type: kanban` / `calendar` / `table` de página inteira | ✅ | 353 — quadro com `column::`, calendário do vault, tarefas com `status::` |
| `type: graph` | ✅ | 361 — grafo 3D vira lista de conexões por wikilink; Enter abre |
| Cabeçalho de página tipada + propriedades | ✅ | 344, 360 — título da página e "Propriedades" (`=`) nas telas de tipo |

## Editor

| Janela | TUI | Ciclo / nota |
|---|---|---|
| Editar texto dos blocos | ✅ | 334 (modo vim) |
| Tabela markdown comum (`\| a \| b \|`): ver em grade e editar | ✅ | 389 — Enter ou `A` abre colunas e linhas |
| Editar bloco de código (e Mermaid) | ✅ | 390 — Enter ou `i`/`A` abre o editor de várias linhas; Esc grava |
| Lista dentro de lista | ✅ | 393, 402, 403 — nível de verdade na árvore (TUI e janela): Enter entra, Esc sai, `Ctrl+A`/`Ctrl+X` aninham |
| Vim mode liga/desliga (sem vim: digitar edita direto) | ✅ | 376 — Enter cria o seguinte, Backspace apaga, Delete apaga o bloco, Ctrl+Z/Y |
| Menu `/` (inserir bloco e embed) | ✅ | 347 — `/` num bloco novo vazio, ou "Inserir bloco ou embed…" |
| Barra de seleção (negrito, itálico, link, cor) | ✅ | 357 — na inserção: `Ctrl+B` negrito, `Ctrl+T` menu Formatar (itálico, riscado, código, link, cor, fundo); 394 — cor personalizada `#hex` e tirar a cor; 395 — `Shift+setas` seleciona o trecho a formatar |
| Painel de propriedades (frontmatter) | ✅ | 344 — "Propriedades da página…" |
| Inserir imagem (texto alternativo, legenda, tamanho, alinhamento) | ✅ | 388 — menu `/` → Imagem; a figura aparece como ▨ legenda e Enter abre o arquivo |
| Imagem colada/arrastada, Mermaid, PDF | — | terminal |
| Desfazer/refazer | ✅ | 322 |
| Salvar (Ctrl+S) e salvamento automático liga/desliga | ✅ | 375 — desligado, a página ganha ● e grava no Ctrl+S (ou ao sair dela) |
| Definir como início (abre primeiro, aba fixa) | ✅ | 362 |
| Exportar HTML da página | ✅ | 362 — grava `<página>.html` na pasta atual |

## Embeds

| Janela | TUI | Ciclo / nota |
|---|---|---|
| Desenho dos 10 tipos | ✅ | 325–336 |
| Kanban: cartão/coluna criar, mover, reordenar | ✅ | 319, 337, 338 |
| Kanban: modal do cartão (descrição, tags, prazo, checklist, comentários, anexos) | ✅ | 343, 391 — Enter no cartão; descrição de várias linhas |
| Calendário: modal do evento (datas, horário, tags) | ✅ | 343 — Enter no evento |
| Calendário: criar evento com horário (grade de horas) | ✅ | 378 — `10:00 título` ou `10:00-11:30 título` no `o` |
| Seletor de data nos formulários (prazo, início, fim, criado) | ✅ | 364 — Enter abre o mês; `c` digita |
| Seletor de horário (15 em 15 min) | ✅ | 365 |
| Cronograma: agendar item "Sem data" | ✅ | 366 — Enter na gaveta |
| Cronograma: mudar início/fim, tags, excluir etapa | ✅ | 385 — Enter ou `=` na barra abre a etapa |
| Calendário: gaveta "Sem data" sempre à vista, "+ evento sem data", abrir o evento sem data | ✅ | 368 — `o` na gaveta; Enter abre o detalhe (dá a data) |
| Tabela: células, linhas, colunas, tipo, opções | ✅ | 321, 337, 339 |
| Tabela: número ▲▼, data, coluna de página (escolher e abrir) | ✅ | 367 — Ctrl+A/X; Enter na página |
| Consulta: configuração completa | ✅ | 335, 338, 344 — `=` na consulta |
| Consulta: recolher grupo (guardado no arquivo) | ✅ | 377 — `z` no grupo |
| Consulta: altura máxima | ✅ | 401 — no `=`; no terminal a página rola, o valor vale na janela |
| Ações: EXECUTAR o botão | ✅ | 342 — abrir, template, propriedade, busca |
| Ações: configurar o botão (ação, destino, ícone) | ✅ | 344 — `=` no botão |
| Ações: layout em grade | ✅ | 380 — botões da mesma largura; `~` fora de um botão troca |
| Galeria: escolher arquivo de assets | ✅ | 356 — `o` lista as imagens de assets/ (ou digita o caminho) |
| Fluxo: transições, nota | ✅ | 335 |
| Fluxo: planejar/executar/pedir alteração (abre conversa) | ✅ | 348 — Enter na ação; executar continua na conversa de origem; botão "origem" |

## Agente e vault

| Janela | TUI | Ciclo / nota |
|---|---|---|
| Conversa, anexos, prompt padrão, virar spec/proposta/execução | ✅ | 340, 341 |
| Conversa: pasta de trabalho e pastas extras do agente | ✅ | 369 — no topo; trocar/dar alcance/tirar pela barra |
| Agente continua rodando ao trocar de vault | ✅ | 396 — a resposta cai na conversa do vault dela |
| Configurar agente (binário, args, pasta, pastas extras) | ✅ | 350 — "Configurar agente…" (formulário, valida `{prompt}`) |
| Lista de agentes (presets + criados, novo, remover) | ✅ | 392 — em "Trocar agente…" |
| Propostas do agente (diff, aplicar, recusar) | ✅ | 346 — "Propostas do agente" na barra |
| Aprovar trecho a trecho, registro das decisões | ✅ | 404 (TUI), 424 (janela) — caixinha por trecho e "Aplicar N de M" |
| Recusar com motivo, registrado na decisão | ✅ | 404 (TUI), 428 (janela) |
| Permissões de escrita do agente por pasta | ✅ | 405 (TUI), 427 (janela) — no vault, valem pro CLI, TUI e janela |
| Registro de execuções do agente (o que rodou, duração, como terminou) | ✅ | 406 (TUI), 427 (janela) — `execucoes.jsonl` no vault |
| Contrato de ferramentas do agente (conjunto fechado, MCP) | ✅ | 407 (TUI), 427 (janela) — declarado no núcleo; o MCP deriva dele |
| Vários agentes em paralelo com fila e limite | ✅ | 408 (TUI), 429 (janela) — política única em `core::fila` |
| Decidir várias propostas de uma vez (marcar, aplicar/recusar em massa) | ✅ | 409 (TUI), 432 (janela) — um motivo só, gravado em cada decisão |
| Realce palavra a palavra no diff da proposta | ✅ | 410 (TUI), 424 (janela) |
| Editar a proposta antes de aplicar | ✅ | 411 (TUI), 428 (janela) — registro diz "editada" |
| Gatilhos: agente dispara por mudança, consulta ou hora | ✅ | 412 (TUI), 431 (janela) — `gatilhos.json` no vault; a janela avalia de minuto em minuto |
| Suíte de avaliação do agente (tarefas com resultado esperado) | — | 413 — `anotadinho-cli avaliar`; vale pros dois, não é tela |
| Transclusão `![[Página#Seção]]` resolvida no contexto do prompt | ✅ | 414 — anexo chega ao agente com o conteúdo; `read --contexto` no CLI |
| Transclusão DESENHADA na página (conteúdo de outra página à vista) | ✅ | 415 — conteúdo entra como enfeite sob o marcador, com teto de 12 linhas |
| Compor contexto: inserir transclusão pelo menu `/`, anexos viram página de contexto | ✅ | 416 (TUI), 430 (janela) — a página-recorte vira O anexo |
| Peso do contexto no cabeçalho e prévia do prompt montado | ✅ | 417 (TUI), 423 (janela) — montagem única em `core::envio` |
| O prompt diz ao agente onde é o vault e quais ferramentas existem | ✅ | 425 — vale pros dois; o MCP do CLI é o caminho, o CLI é o plano B |
| MCP do vault ligado na execução, sem configurar por fora | ✅ | 426 — vale pros dois; `--mcp-config` gerado por vault, campo no formulário do agente |
| Sessão contínua: a conversa continua sem remontar o histórico | ✅ | 433 — vale pros dois; `sessao:` no frontmatter, `--resume` no preset |
| Guardas do trabalho automático (teto de disparos, de custo, silêncio) | ✅ | 434 — vale pros dois; `guardas.json` no vault, freio só no automático |
| MCP expõe páginas como recursos e prompts padrão como prompts | — | 435 — é do servidor (`anotadinho-cli mcp`), serve qualquer cliente |
| Revisor: segundo agente critica a proposta, veredito no cartão | ✅ | 436 — `anotadinho-cli revisar <id>`; veredito visível nas duas telas |
| Consulta dentro do recorte vira contexto (o vault de hoje, não uma lista velha) | ✅ | 418 — vale pros dois: TUI, janela e CLI passam pelo mesmo handler |
| Poda explícita quando o contexto passa do teto | ✅ | 419 (TUI), 423 (janela) — histórico antigo, depois esqueleto do anexo maior |
| Proposta em lote atômico (mudança que atravessa páginas) | ✅ | 420 (TUI), 428 (janela) — selo ⛓ e decisão do lote inteiro |
| Contar ao agente o que foi aplicado e recusado (com o motivo) | ✅ | 421 (TUI), 430 (janela) — vai pro campo, não é enviado sozinho |
| Tokens e custo por execução, com total do dia | ✅ | 422 (TUI), 427 (janela) — do `usage` do agente |
| Git: status, pull, commit & push | ✅ | 349 — "Git: status e sincronizar…" na barra; histórico da página |
| Nova página a partir de template | ✅ | 350 — "Nova página" pergunta o template quando há |

## Aparência e teclado

| Janela | TUI | Ciclo / nota |
|---|---|---|
| Tema | ✅ | 339, 387 — Catppuccin Mocha é o padrão (janela e TUI) |
| Cor de destaque, botões | ✅ | 358 — "Cor de destaque…" e "Estilo dos botões…" (também em Personalizar), gravados |
| Sidebar escondível | ✅ | 339, 400 — recolhida vira trilha de ícones (≡ ◷ ⌕) |
| Remapear atalhos globais e do vim | ✅ | 359 — "Remapear teclas…": ações do vim de uma tecla e comandos globais, com checagem de repetição |
| Capturar a tecla apertando (KeymapCaptureModal) | ✅ | 383 — Enter no campo e aperta a tecla |
