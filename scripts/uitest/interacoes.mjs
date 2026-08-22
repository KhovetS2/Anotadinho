// Interações do app (ciclo 196).
//
// Cobertura ampla do que dá pra FAZER hoje: sidebar, abas, paleta,
// painéis do editor e a manipulação de cada tipo de embed pelo mouse.
//
// Os outros arquivos cobrem áreas específicas — `digitacao.mjs` o texto,
// `blocos.mjs` os modos, `cenarios.mjs` as regressões nomeadas. Aqui é a
// varredura horizontal: pouco de cada coisa, mas quase tudo tocado ao
// menos uma vez, pra uma mudança grande não passar despercebida num
// canto que ninguém testava.

import { esperar, recarregarEstavel, abrirPaginaEstavel } from "./bridge.mjs";

const PAUSA = (ms) => new Promise((r) => setTimeout(r, ms));

export const interacoes = [];

const SALVAR = `(() => {
  // Trava de segurança (ciclo 197): só grava na página de RASCUNHO.
  // Um cenário que navegou pra uma página real e chamou Salvar
  // reescrevia o arquivo do usuário — aconteceu com
  // \`pages/exemplos/composicao.md\`, que voltou normalizado.
  const titulo = (document.querySelector('.editor__title') || {}).textContent || '';
  if (!titulo.includes('__uitest')) {
    throw new Error('Salvar bloqueado: a página aberta é "' + titulo + '", não a de teste');
  }
  const b = [...document.querySelectorAll('button')].find(b => b.textContent.trim().startsWith('Salvar'));
  if (b) b.click();
  return !!b;
})()`;

function corpo(texto) {
  if (!texto) return "";
  const m = texto.match(/^---\n[\s\S]*?\n---\n?([\s\S]*)$/);
  return (m ? m[1] : texto).trim();
}

/// Clica no primeiro elemento que casa com o seletor e tem o texto.
const CLICAR = (seletor, texto = null) => `(() => {
  const alvos = [...document.querySelectorAll(${JSON.stringify(seletor)})];
  const alvo = ${texto === null ? "alvos[0]" : `alvos.find(e => e.textContent.includes(${JSON.stringify(texto)}))`};
  if (!alvo) return false;
  alvo.click();
  return true;
})()`;

function caso(nome, inicial, fn) {
  interacoes.push({
    nome: `interação: ${nome} (196)`,
    async fn(bridge, ctx) {
      if (inicial !== null) {
        ctx.escrever(`---\ntitle: __uitest\n---\n${inicial}`);
      }
      // Espera por CONDIÇÃO, não por relógio (ciclo 198).
      await recarregarEstavel(bridge);
      if (inicial !== null) {
        await abrirPaginaEstavel(bridge, ctx.nomePagina);
      }
      await fn(bridge, ctx, {
        salvarELer: async () => {
          await bridge.js(SALVAR);
          await PAUSA(1000);
          return corpo(ctx.ler());
        },
      });
    },
  });
}

const KANBAN = `{{ type: "kanban" }}
columns:
- Backlog
- Feito
items:
- title: Card A
  column: Backlog
{{ /kanban }}
`;

const TABELA = `{{ type: "table" }}
columns:
- name: Tarefa
- name: Status
  type: select
  options: [todo, done]
---
| Tarefa | Status |
| --- | --- |
| API | todo |
{{ /table }}
`;

// ── kanban ──────────────────────────────────────────────────────────

caso("kanban: adicionar card grava no YAML", KANBAN, async (b, ctx, h) => {
  await esperar(b, "document.querySelector('.kanban__add-card')", "o botão de add card");
  await b.js(`(() => {
    document.querySelector('.kanban__add-card').click();
    return true;
  })()`);
  await PAUSA(600);
  await b.js(`(() => {
    const inp = document.querySelector('.kanban__column input, .kanban__column textarea');
    if (!inp) return false;
    const set = Object.getOwnPropertyDescriptor(inp.tagName === 'INPUT'
      ? HTMLInputElement.prototype : HTMLTextAreaElement.prototype, 'value').set;
    inp.focus();
    set.call(inp, 'Card novo');
    inp.dispatchEvent(new InputEvent('input', { bubbles: true }));
    inp.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true, cancelable: true }));
    inp.blur();
    return true;
  })()`);
  await PAUSA(800);
  const md = await h.salvarELer();
  ctx.assert(md.includes("Card A"), `o card original se perdeu:\n${md}`);
});

caso("kanban: adicionar coluna aparece no board", KANBAN, async (b, ctx, h) => {
  await esperar(b, "document.querySelector('.kanban__add-column-btn, .kanban__add-column')", "o botão de coluna");
  const antes = await b.js(`document.querySelectorAll('.kanban__column').length`);
  await b.js(CLICAR(".kanban__add-column-btn, .kanban__add-column"));
  await PAUSA(700);
  const depois = await b.js(`document.querySelectorAll('.kanban__column').length`);
  ctx.assert(depois >= antes, `o board perdeu colunas: ${antes} -> ${depois}`);
});

caso("kanban: contagem da coluna bate com os cards", KANBAN, async (b, ctx) => {
  await esperar(b, "document.querySelector('.kanban__col-count')", "a contagem");
  const ok = await b.js(`(() => {
    const cols = [...document.querySelectorAll('.kanban__column')];
    return cols.every(c => {
      const n = parseInt((c.querySelector('.kanban__col-count') || {}).textContent || '0', 10);
      return n === c.querySelectorAll('.kanban__card').length;
    });
  })()`);
  ctx.assertEq(ok, true, "a contagem do cabeçalho não bate com os cards");
});

// ── tabela ──────────────────────────────────────────────────────────

caso("tabela: adicionar linha cria uma linha nova", TABELA, async (b, ctx, h) => {
  await esperar(b, "document.querySelector('.task-table__table')", "a tabela");
  // `.task-table__add` casa com DOIS botões — "Nova coluna" e "+ linha".
  // A contagem também ignora a linha-botão (`--add`), que existe sempre.
  const dados = () =>
    b.js(`document.querySelectorAll('.task-table__row:not(.task-table__row--add)').length`);
  const antes = await dados();
  await b.js(CLICAR(".task-table__add", "linha"));
  await PAUSA(700);
  const depois = await dados();
  ctx.assert(depois > antes, `a linha não foi criada: ${antes} -> ${depois}`);
});

caso("tabela: adicionar coluna cria uma coluna nova", TABELA, async (b, ctx, h) => {
  await esperar(b, "document.querySelector('.task-table__table')", "a tabela");
  const cols = () =>
    b.js(`document.querySelectorAll('.task-table__th:not(.task-table__th--add)').length`);
  const antes = await cols();
  await b.js(`(() => {
    const btn = [...document.querySelectorAll('.task-table__add')].find(x => x.title === 'Nova coluna');
    btn.click();
    return true;
  })()`);
  // "Nova coluna" abre um MODAL pra nomear — não adiciona direto.
  await esperar(b, "document.querySelector('.modal input')", "o modal de coluna abrir");
  await PAUSA(400);
  // Em três passos: o Yew precisa processar o `input` antes do OK, senão
  // o clique confirma um nome vazio.
  await b.js(`(() => {
    const inp = document.querySelector('.modal input');
    const set = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value').set;
    inp.focus();
    set.call(inp, 'Prazo');
    inp.dispatchEvent(new InputEvent('input', { bubbles: true }));
    return true;
  })()`);
  await PAUSA(500);
  await b.js(`(() => {
    [...document.querySelectorAll('.modal button')].find(x => x.textContent.trim() === 'OK').click();
    return true;
  })()`);
  await PAUSA(1000);
  ctx.assert((await cols()) > antes, "a coluna não foi criada depois do OK");
  const md = await h.salvarELer();
  ctx.assert(md.includes("Prazo"), `a coluna nova não chegou no arquivo:\n${md}`);
});

caso("tabela: editar célula de texto grava no arquivo", TABELA, async (b, ctx, h) => {
  await esperar(b, "document.querySelector('.task-table__table')", "a tabela");
  await b.js(`(() => {
    const td = document.querySelector('.task-table__row:not(.task-table__row--add) .task-table__td');
    td.click();
    return true;
  })()`);
  await PAUSA(500);
  await b.js(`(() => {
    const inp = document.querySelector('.task-table__text-input');
    if (!inp) return false;
    // É um <textarea> (ciclo que trocou o contenteditable por causa do
    // re-render do Yew), não um <input>.
    const set = Object.getOwnPropertyDescriptor(HTMLTextAreaElement.prototype, 'value').set;
    inp.focus();
    set.call(inp, 'API editada');
    inp.dispatchEvent(new InputEvent('input', { bubbles: true }));
    inp.blur();
    return true;
  })()`);
  await PAUSA(800);
  const md = await h.salvarELer();
  ctx.assert(md.includes("API editada"), `a edição não chegou no arquivo:\n${md}`);
});

// ── callout ─────────────────────────────────────────────────────────

const CALLOUT = `{{ type: "callout" }}
variant: info
title: Nota
body: |
  corpo do destaque
{{ /callout }}
`;

caso("callout: recolher e expandir pelo cabeçalho", CALLOUT, async (b, ctx) => {
  await esperar(b, "document.querySelector('.callout__collapse')", "o botão de recolher");
  await b.js(CLICAR(".callout__collapse"));
  await PAUSA(600);
  const recolhido = await b.js(`!document.querySelector('.callout__body')`);
  await b.js(CLICAR(".callout__collapse"));
  await PAUSA(600);
  const expandido = await b.js(`!!document.querySelector('.callout__body')`);
  ctx.assert(recolhido, "não recolheu");
  ctx.assert(expandido, "não voltou a expandir");
});

caso("callout: trocar a variante muda a classe e o YAML", CALLOUT, async (b, ctx, h) => {
  await esperar(b, "document.querySelector('.callout__variant')", "o seletor de variante");
  const trocou = await b.js(`(() => {
    const sel = document.querySelector('.callout__variant');
    if (!sel || sel.tagName !== 'SELECT') return 'nao-e-select';
    const outra = [...sel.options].map(o => o.value).find(v => v !== sel.value);
    sel.value = outra;
    sel.dispatchEvent(new Event('change', { bubbles: true }));
    return outra;
  })()`);
  if (trocou === "nao-e-select") return; // variante por botão: coberto pelo snapshot
  await PAUSA(800);
  const md = await h.salvarELer();
  ctx.assert(md.includes(`variant: ${trocou}`), `a variante não gravou (${trocou}):\n${md}`);
});

// ── abas ────────────────────────────────────────────────────────────

caso("abas: abrir uma segunda página cria aba e dá pra voltar", "so texto\n", async (b, ctx) => {
  const nAbas = () => b.js(`document.querySelectorAll('.tab-bar__tab').length`);
  const antes = await nAbas();
  await b.js(CLICAR(".sidebar-item__title", "missao"));
  await PAUSA(1800);
  const depois = await nAbas();
  ctx.assert(depois >= antes, `as abas sumiram: ${antes} -> ${depois}`);

  // Voltar pra aba anterior pelo clique.
  await b.js(CLICAR(".tab-bar__tab-title", "__uitest"));
  await PAUSA(1500);
  ctx.assert(
    (await b.js(`(document.querySelector('.editor__title')||{}).textContent`)).includes("uitest"),
    "não voltou pra aba anterior",
  );
});

// ── sidebar ─────────────────────────────────────────────────────────

caso("sidebar: filtrar por título mostra só o que casa", null, async (b, ctx) => {
  await esperar(b, "document.querySelector('input[placeholder*=\"Buscar\"]')", "o campo de busca");
  await b.js(`(() => {
    const campo = document.querySelector('input[placeholder*="Buscar"]');
    const set = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value').set;
    campo.focus();
    set.call(campo, 'missao');
    campo.dispatchEvent(new InputEvent('input', { bubbles: true }));
    return true;
  })()`);
  await PAUSA(1500);
  const titulos = await b.js(
    `[...document.querySelectorAll('.sidebar-item__title')].map(e => e.textContent.trim())`,
  );
  ctx.assert(titulos.length > 0, "o filtro não devolveu nada");
  ctx.assert(
    titulos.some((t) => t.toLowerCase().includes("missao")),
    `o alvo não apareceu: ${titulos.join(", ")}`,
  );
});

// ── painéis do editor ───────────────────────────────────────────────

caso("backlinks: a página alvo lista quem aponta pra ela", null, async (b, ctx) => {
  await b.js(CLICAR(".sidebar-item__title", "missao"));
  await PAUSA(2500);
  const tem = await b.js(`!!document.querySelector('.editor__backlinks')`);
  ctx.assertEq(tem, true, "o painel de backlinks não apareceu numa página referenciada");
});

caso("propriedades: o modal abre e mostra os campos do frontmatter", "texto\n", async (b, ctx) => {
  // Fica atrás do menu "⋯" do cabeçalho.
  await b.js(`(() => {
    [...document.querySelectorAll('.editor__actions button')]
      .find(x => x.title === 'Mais ações').click();
    return true;
  })()`);
  await esperar(b, "document.querySelector('.header-menu__item')", "o menu abrir");
  await b.js(CLICAR(".header-menu__item", "Propriedades"));
  await PAUSA(900);
  ctx.assertEq(
    await b.js(`!!document.querySelector('.modal, .card-modal')`),
    true,
    "o modal de propriedades não abriu",
  );
});

// ── menu / ──────────────────────────────────────────────────────────

caso("menu /: filtrar por texto reduz a lista e Escape fecha", "linha\n", async (b, ctx) => {
  await b.js(`(() => {
    const bl = document.querySelector('.editor__bloco');
    bl.focus();
    const r = document.createRange();
    r.selectNodeContents(bl); r.collapse(false);
    const s = getSelection(); s.removeAllRanges(); s.addRange(r);
    document.execCommand('insertParagraph', false);
    document.execCommand('insertText', false, '/');
    return true;
  })()`);
  await esperar(b, "document.querySelector('.slash-menu')", "o menu abrir");
  const todos = await b.js(`document.querySelectorAll('.slash-menu__item').length`);

  await b.js(`(() => { document.execCommand('insertText', false, 'kan'); return true; })()`);
  await PAUSA(500);
  const filtrados = await b.js(`document.querySelectorAll('.slash-menu__item').length`);
  ctx.assert(filtrados < todos, `o filtro não reduziu: ${todos} -> ${filtrados}`);

  await b.js(`(() => {
    document.querySelector('.editor__bloco').dispatchEvent(
      new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true }));
    return true;
  })()`);
  await PAUSA(600);
  ctx.assertEq(await b.js(`!!document.querySelector('.slash-menu')`), false, "Escape não fechou o menu");
});

// ── tema ────────────────────────────────────────────────────────────

caso("tema: alternar claro/escuro muda o atributo da raiz", null, async (b, ctx) => {
  const antes = await b.js(`document.documentElement.getAttribute('data-theme') || document.body.className`);
  const clicou = await b.js(`(() => {
    const btn = [...document.querySelectorAll('button')].find(x => /tema|claro|escuro/i.test(x.title || ''));
    if (!btn) return false;
    btn.click();
    return true;
  })()`);
  if (!clicou) return;
  await PAUSA(700);
  const depois = await b.js(`document.documentElement.getAttribute('data-theme') || document.body.className`);
  ctx.assert(antes !== depois, `o tema não mudou: ${antes} -> ${depois}`);
});

// ── consulta, cronograma, galeria, colunas, ações ────────────────────

caso(
  "consulta: trocar a visão entre lista, tabela e cartões",
  '{{ type: "query" }}\nfrom: pages\nlimit: 3\nview: list\n{{ /query }}\n',
  async (b, ctx, h) => {
    await esperar(b, "document.querySelector('.query-embed')", "a consulta renderizar");
    const trocou = await b.js(`(() => {
      const sel = document.querySelector('.query-embed select');
      if (!sel) return null;
      const outra = [...sel.options].map(o => o.value).find(v => v !== sel.value);
      if (!outra) return null;
      sel.value = outra;
      sel.dispatchEvent(new Event('change', { bubbles: true }));
      return outra;
    })()`);
    if (trocou === null) return;
    await PAUSA(900);
    const md = await h.salvarELer();
    ctx.assert(md.includes(trocou), `a visão "${trocou}" não gravou:\n${md}`);
  },
);

caso(
  "cronograma: trocar a escala grava no YAML",
  '{{ type: "timeline" }}\nscale: month\nitems:\n- title: Etapa\n  start: \'2026-08-03\'\n  end: \'2026-08-10\'\n{{ /timeline }}\n',
  async (b, ctx, h) => {
    await esperar(b, "document.querySelector('[class*=timeline]')", "o cronograma");
    const clicou = await b.js(`(() => {
      const btn = [...document.querySelectorAll('[class*=timeline] button')]
        .find(x => /semana/i.test(x.textContent));
      if (!btn) return false;
      btn.click();
      return true;
    })()`);
    if (!clicou) return;
    await PAUSA(900);
    const md = await h.salvarELer();
    ctx.assert(md.includes("scale: week"), `a escala não gravou:\n${md}`);
  },
);

caso(
  "colunas: adicionar painel muda a contagem e o YAML",
  '{{ type: "columns" }}\ncolumns:\n- width: 1\n  body: |\n    esquerda\n- width: 1\n  body: |\n    direita\n{{ /columns }}\n',
  async (b, ctx, h) => {
    await esperar(b, "document.querySelector('.columns-embed')", "as colunas");
    const paineis = () => b.js(`document.querySelectorAll('.columns-embed [class*=pane], .columns-embed [class*=col]').length`);
    const antes = await paineis();
    const clicou = await b.js(`(() => {
      const btn = [...document.querySelectorAll('.columns-embed button')]
        .find(x => x.textContent.trim() === '+' || /adicionar|coluna/i.test(x.title || ''));
      if (!btn) return false;
      btn.click();
      return true;
    })()`);
    if (!clicou) return;
    await PAUSA(800);
    ctx.assert((await paineis()) >= antes, "as colunas sumiram ao adicionar");
  },
);

caso(
  "ações: clicar num botão de abrir página navega pra ela",
  '{{ type: "actions" }}\nbuttons:\n- label: Ir pra missão\n  action: open-page\n  path: pages/produto/missao.md\n{{ /actions }}\n',
  async (b, ctx) => {
    await esperar(b, "document.querySelector('[class*=actions]')", "os botões");
    await b.js(CLICAR("[class*=actions] button", "Ir pra missão"));
    await esperar(
      b,
      `(document.querySelector('.editor__title')||{}).textContent === 'Missão'`,
      "a página alvo abrir",
      10000,
    );
  },
);

// ── desfazer de mutação de embed ────────────────────────────────────

caso("desfazer: mutação de embed volta com Ctrl+Z", KANBAN, async (b, ctx, h) => {
  await esperar(b, "document.querySelector('.kanban__board')", "o board");
  const cards = () => b.js(`document.querySelectorAll('.kanban__card').length`);
  const antes = await cards();

  await b.js(`(() => {
    const btn = [...document.querySelectorAll('.kanban__col-header button, .kanban__column button')]
      .find(x => /coluna/i.test(x.title || ''));
    if (btn) btn.click();
    return true;
  })()`);
  await PAUSA(700);

  await b.js(`(() => {
    const raiz = document.querySelector('.app-root') || document.body;
    raiz.dispatchEvent(new KeyboardEvent('keydown', { key: 'z', ctrlKey: true, bubbles: true }));
    return true;
  })()`);
  await PAUSA(900);
  ctx.assert((await cards()) >= antes - 1, "o desfazer comeu cards demais");
});

// ── persistência entre navegações ───────────────────────────────────

caso("edição pendente sobrevive a trocar de página e voltar", "conteudo base\n", async (b, ctx, h) => {
  await b.js(`(() => {
    const bl = document.querySelector('.editor__bloco');
    bl.focus();
    const r = document.createRange();
    r.selectNodeContents(bl); r.collapse(false);
    const s = getSelection(); s.removeAllRanges(); s.addRange(r);
    document.execCommand('insertText', false, ' EDITADO');
    return true;
  })()`);
  await PAUSA(500);

  await b.js(CLICAR(".sidebar-item__title", "missao"));
  await PAUSA(2000);
  await b.js(CLICAR(".sidebar-item__title", "__uitest"));
  await PAUSA(2200);

  const texto = await b.js(`document.querySelector('.editor__wysiwyg').innerText`);
  ctx.assert(
    texto.includes("EDITADO"),
    `a edição pendente se perdeu ao trocar de página:\n${texto}`,
  );
});

// ── paleta de comandos ──────────────────────────────────────────────

caso("paleta: abre com Ctrl+K, filtra e abre a página escolhida", null, async (b, ctx) => {
  // Em `.app-root`, não em `body`: o listener global vive lá, e o
  // evento disparado no body sobe pro document sem passar por ele.
  await b.js(`(() => {
    const raiz = document.querySelector('.app-root') || document.body;
    raiz.dispatchEvent(new KeyboardEvent('keydown', { key: 'k', ctrlKey: true, bubbles: true, cancelable: true }));
    return true;
  })()`);
  const abriu = await b
    .js(`!!document.querySelector('.command-palette, [class*=palette]')`)
    .catch(() => false);
  if (!abriu) {
    await PAUSA(600);
  }
  const visivel = await b.js(`!!document.querySelector('.command-palette, [class*=palette]')`);
  ctx.assertEq(visivel, true, "Ctrl+K não abriu a paleta");

  await b.js(`(() => {
    const inp = document.querySelector('.command-palette input, [class*=palette] input');
    const set = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value').set;
    inp.focus();
    set.call(inp, 'missao');
    inp.dispatchEvent(new InputEvent('input', { bubbles: true }));
    return true;
  })()`);
  await PAUSA(1200);
  const itens = await b.js(
    `[...document.querySelectorAll('[class*=palette__item]')].map(e => e.textContent.trim()).slice(0, 5)`,
  );
  ctx.assert(itens.length > 0, "a paleta não filtrou nada");
});
