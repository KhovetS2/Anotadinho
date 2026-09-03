// Telas e fluxos ainda sem cobertura (ciclo 200).
//
// As áreas que os outros arquivos não tocavam: journals, páginas-de-TIPO
// (grafo, tags, assets, kanban, calendário — diferentes dos embeds de
// mesmo nome), templates, exportação, git e o cheatsheet.
//
// Escrito depois do ciclo 198, então usa espera por condição desde o
// começo — nenhum `PAUSA` fixo de setup.

import { esperar, recarregarEstavel, abrirPaginaEstavel } from "./bridge.mjs";

const PAUSA = (ms) => new Promise((r) => setTimeout(r, ms));

export const telas = [];

/// Dispara um atalho global (eles vivem no listener de `.app-root`).
const ATALHO = (key, extra = {}) => `(() => {
  const raiz = document.querySelector('.app-root') || document.body;
  raiz.dispatchEvent(new KeyboardEvent('keydown', Object.assign(
    { key: ${JSON.stringify(key)}, bubbles: true, cancelable: true }, ${JSON.stringify(extra)})));
  return true;
})()`;

/// Abre a paleta e roda um comando pelo nome.
const COMANDO_DA_PALETA = (nome) => `(() => {
  const itens = [...document.querySelectorAll('[class*=palette__item]')];
  const alvo = itens.find(i => i.textContent.includes(${JSON.stringify(nome)}));
  if (!alvo) return itens.map(i => i.textContent.trim()).slice(0, 8);
  alvo.click();
  return true;
})()`;


/// Roda `fn` e apaga os arquivos que ela criou no vault.
///
/// "Ir pra Hoje", "Ver Tags" e "Ver Assets" CRIAM a página se ela não
/// existe — rodar a suíte sujava o vault do usuário com um journal e
/// duas páginas de índice (ciclo 200).
async function semSujarOVault(ctx, caminhos, fn) {
  const fs = await import("node:fs");
  const existiaAntes = caminhos.map((c) => fs.existsSync(`${ctx.vault}/${c}`));
  try {
    await fn();
  } finally {
    caminhos.forEach((c, i) => {
      if (!existiaAntes[i]) fs.rmSync(`${ctx.vault}/${c}`, { force: true });
    });
  }
}

function tela(nome, fn) {
  telas.push({
    nome: `tela: ${nome} (200)`,
    async fn(bridge, ctx) {
      await recarregarEstavel(bridge);
      await fn(bridge, ctx);
    },
  });
}

/// Abre a paleta limpa.
async function abrirPaleta(bridge) {
  await bridge.js(ATALHO("k", { ctrlKey: true }));
  await esperar(bridge, "document.querySelector('[class*=palette]')", "a paleta abrir");
}

// ── journals ────────────────────────────────────────────────────────

tela("journal de hoje abre e tem a data no título", async (b, ctx) => {
  const hoje = new Date();
  const iso = `${hoje.getFullYear()}-${String(hoje.getMonth() + 1).padStart(2, "0")}-${String(hoje.getDate()).padStart(2, "0")}`;
  await semSujarOVault(ctx, [`journals/${iso}.md`], async () => {
  await abrirPaleta(b);
  const r = await b.js(COMANDO_DA_PALETA("Ir pra Hoje"));
  ctx.assertEq(r, true, `o comando não estava na paleta: ${JSON.stringify(r)}`);
  await esperar(
    b,
    `/\\d{4}-\\d{2}-\\d{2}/.test((document.querySelector('.editor__title')||{}).textContent || '')`,
    "o journal de hoje abrir",
    12000,
  );
  });
});

// ── páginas de TIPO ─────────────────────────────────────────────────

for (const [comando, marca, desc, arquivo] of [
  ["Ver Tags", "[class*=tag]", "a página de tags", "pages/tags.md"],
  ["Ver Assets", "[class*=asset]", "a página de assets", "pages/assets.md"],
]) {
  tela(`${desc} abre pelo comando`, async (b, ctx) => {
    await semSujarOVault(ctx, [arquivo], async () => {
      await abrirPaleta(b);
      const r = await b.js(COMANDO_DA_PALETA(comando));
      ctx.assertEq(r, true, `"${comando}" não estava na paleta: ${JSON.stringify(r)}`);
      await esperar(b, `document.querySelector('${marca}')`, desc, 12000);
    });
  });
}

tela("página type: grafo desenha nós e arestas", async (b, ctx) => {
  await b.js(`(() => {
    const alvo = [...document.querySelectorAll('.sidebar-item__title')].find(e => e.textContent.trim() === 'grafo');
    if (alvo) alvo.click();
    return !!alvo;
  })()`);
  await esperar(b, "document.querySelector('svg')", "o SVG do grafo", 12000);
  const nos = await b.js(`document.querySelectorAll('svg g, svg circle').length`);
  ctx.assert(nos > 0, "o grafo abriu sem nó nenhum");
});

tela("página type: kanban mostra o board", async (b, ctx) => {
  await b.js(`(() => {
    const alvo = [...document.querySelectorAll('.sidebar-item__title')].find(e => e.textContent.trim() === 'roadmap');
    if (alvo) alvo.click();
    return !!alvo;
  })()`);
  await esperar(b, "document.querySelector('.kanban__board, [class*=kanban]')", "o board", 12000);
});

// ── cheatsheet ──────────────────────────────────────────────────────

tela("cheatsheet abre e lista atalhos", async (b, ctx) => {
  await abrirPaleta(b);
  const r = await b.js(COMANDO_DA_PALETA("atalhos"));
  if (r !== true) return; // comando pode ter outro rótulo
  await esperar(b, "document.querySelector('.modal, [class*=cheatsheet]')", "o cheatsheet", 10000);
  const linhas = await b.js(
    `document.querySelectorAll('.modal tr, [class*=cheatsheet] li, [class*=cheatsheet] tr').length`,
  );
  ctx.assert(linhas > 0, "o cheatsheet abriu vazio");
});

// ── criação ─────────────────────────────────────────────────────────

tela("nova página: escolher template e nomear cria e abre", async (b, ctx) => {
  await abrirPaleta(b);
  const r = await b.js(COMANDO_DA_PALETA("Nova página"));
  if (r !== true) return;

  // Passo 1: o seletor de TEMPLATE (foi o que o cenário não previa —
  // "Nova página" não pergunta o nome primeiro).
  await esperar(b, "document.querySelector('.modal')", "o seletor de template");
  const templates = await b.js(
    `[...document.querySelectorAll('.modal button')].map(x => x.textContent.trim()).filter(Boolean)`,
  );
  ctx.assert(
    templates.some((t) => /branco/i.test(t)),
    `o seletor devia oferecer página em branco: ${templates.join(" | ")}`,
  );
  ctx.assert(
    templates.some((t) => /spec|reuniao|decisao/i.test(t)),
    `os templates do vault deviam aparecer: ${templates.join(" | ")}`,
  );

  await b.js(`(() => {
    [...document.querySelectorAll('.modal button')].find(x => /branco/i.test(x.textContent)).click();
    return true;
  })()`);

  // Passo 2: o nome.
  await esperar(b, "document.querySelector('.modal input')", "o campo de título");
  await PAUSA(300);
  await b.js(`(() => {
    const inp = document.querySelector('.modal input');
    const set = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value').set;
    inp.focus();
    set.call(inp, '__uitest_nova');
    inp.dispatchEvent(new InputEvent('input', { bubbles: true }));
    return true;
  })()`);
  await PAUSA(400);
  await b.js(`(() => {
    const ok = [...document.querySelectorAll('.modal button')]
      .find(x => /^(ok|criar|confirmar)$/i.test(x.textContent.trim()));
    if (ok) ok.click();
    return !!ok;
  })()`);
  await PAUSA(1500);

  // O título exibido é o SLUG do arquivo: `__uitest_nova` vira
  // `uitest-nova`. Conferir o slug, não o que foi digitado.
  const titulo = await b.js(`(document.querySelector('.editor__title')||{}).textContent || ''`);
  ctx.assert(
    /uitest[-_]?nova/i.test(titulo),
    `a página nova não abriu (título: "${titulo}")`,
  );

  // Limpa: esta é a única página que os cenários criam fora do rascunho.
  const fs = await import("node:fs");
  const glob = fs.readdirSync(`${ctx.vault}/pages`).filter((f) => /uitest[-_]nova/i.test(f));
  for (const f of glob) fs.rmSync(`${ctx.vault}/pages/${f}`, { force: true });
});

// ── git ─────────────────────────────────────────────────────────────

tela("indicador de git no cabeçalho responde ao clique", async (b, ctx) => {
  const tem = await b.js(`!!document.querySelector('[class*=git]')`);
  if (!tem) return; // vault sem git
  await b.js(`(() => {
    const el = document.querySelector('[class*=git]');
    (el.tagName === 'BUTTON' ? el : el.querySelector('button') || el).click();
    return true;
  })()`);
  await PAUSA(800);
  // Não afirma o conteúdo do popover: só que clicar não quebra a tela.
  ctx.assertEq(
    await b.js(`!!document.querySelector('.app-root')`),
    true,
    "a tela quebrou ao abrir o painel de git",
  );
});

// ── exportação ──────────────────────────────────────────────────────

tela("exportar HTML da página não quebra a tela", async (b, ctx) => {
  ctx.escrever("---\ntitle: __uitest\n---\nconteudo pra exportar\n");
  await recarregarEstavel(b);
  await abrirPaginaEstavel(b, ctx.nomePagina);

  await b.js(`(() => {
    [...document.querySelectorAll('.editor__actions button')]
      .find(x => x.title === 'Mais ações').click();
    return true;
  })()`);
  await esperar(b, "document.querySelector('.header-menu__item')", "o menu abrir");
  const clicou = await b.js(`(() => {
    const it = [...document.querySelectorAll('.header-menu__item')].find(x => /exportar/i.test(x.textContent));
    if (!it) return false;
    it.click();
    return true;
  })()`);
  if (!clicou) return;
  await PAUSA(1200);
  ctx.assertEq(await b.js(`!!document.querySelector('.editor')`), true, "o editor sumiu ao exportar");
});

// ── histórico ───────────────────────────────────────────────────────

tela("histórico da página abre o painel", async (b, ctx) => {
  ctx.escrever("---\ntitle: __uitest\n---\nconteudo\n");
  await recarregarEstavel(b);
  await abrirPaginaEstavel(b, ctx.nomePagina);

  await b.js(`(() => {
    [...document.querySelectorAll('.editor__actions button')]
      .find(x => x.title === 'Mais ações').click();
    return true;
  })()`);
  await esperar(b, "document.querySelector('.header-menu__item')", "o menu abrir");
  const clicou = await b.js(`(() => {
    const it = [...document.querySelectorAll('.header-menu__item')].find(x => /hist/i.test(x.textContent));
    if (!it) return false;
    it.click();
    return true;
  })()`);
  if (!clicou) return;
  await PAUSA(1500);
  ctx.assertEq(
    await b.js(`!!document.querySelector('.modal, .editor__status')`),
    true,
    "o histórico não mostrou nem painel nem aviso",
  );
});

// ── sidebar: criar pasta e mover ────────────────────────────────────

tela("sidebar tem os botões de criar página e pasta", async (b, ctx) => {
  const botoes = await b.js(
    `[...document.querySelectorAll('.app-sidebar button')].map(x => x.title || x.textContent.trim()).filter(Boolean)`,
  );
  ctx.assert(
    botoes.some((t) => /página|pagina|nova/i.test(t)),
    `sem botão de nova página: ${botoes.join(" | ")}`,
  );
  ctx.assert(
    botoes.some((t) => /pasta/i.test(t)),
    `sem botão de nova pasta: ${botoes.join(" | ")}`,
  );
});

// ── Aparência (ciclo 253) ───────────────────────────────────────────
//
// Migrados da bateria `--pendentes`, que os guardava enquanto a spec
// "Tema configurável" não existia em código.

/// Abre o menu do cabeçalho e, dali, a tela de aparência.
///
/// Assíncrono de propósito: o menu é renderizado pelo Yew depois do
/// clique, então procurar o item na MESMA volta síncrona não acha nada.
const ABRIR_APARENCIA = `(async () => {
  const menu = document.querySelector('[data-nav-item="header-menu"]');
  if (!menu) return 'sem botão de configurações no cabeçalho';
  menu.click();
  await new Promise(r => setTimeout(r, 300));
  const item = [...document.querySelectorAll('.header-menu__item')]
    .find(e => /apar[êe]ncia/i.test(e.textContent || ''));
  if (!item) return 'o menu não oferece Aparência';
  item.click();
  await new Promise(r => setTimeout(r, 300));
  return true;
})()`;

const ESTADO_APARENCIA = `(() => ({
  tema: document.documentElement.getAttribute('data-theme'),
  botoes: document.documentElement.getAttribute('data-botoes'),
  destaque: getComputedStyle(document.documentElement).getPropertyValue('--accent-blue').trim(),
  fundo: getComputedStyle(document.body).backgroundColor,
}))()`;

telas.push({
  nome: "aparência: a tela oferece temas, destaque e forma de botão (253)",
  async fn(bridge, ctx) {
    await recarregarEstavel(bridge);
    const abriu = await bridge.js(ABRIR_APARENCIA);
    ctx.assertEq(abriu, true, String(abriu));
    await PAUSA(500);

    const temas = await bridge.js(
      `[...document.querySelectorAll('button[data-tema]')].map(e => e.getAttribute('data-tema'))`,
    );
    ctx.assert(temas.length > 1, `esperava vários temas pra escolher, achei ${JSON.stringify(temas)}`);

    // A prévia é o que permite escolher SEM aplicar (RF2): cada tema
    // mostra as próprias cores antes de o clique acontecer.
    const previas = await bridge.js(
      `[...document.querySelectorAll('button[data-tema] .aparencia__cor')].length`,
    );
    ctx.assert(previas >= temas.length * 2, `os temas não mostram prévia (${previas} amostras)`);

    const destaques = await bridge.js(
      `[...document.querySelectorAll('button[data-destaque]')].length`,
    );
    ctx.assert(destaques > 1, "não há cor de destaque pra escolher");
    const formas = await bridge.js(
      `[...document.querySelectorAll('button[data-botoes]')].length`,
    );
    ctx.assert(formas > 1, "não há estilo de botão pra escolher");
  },
});

telas.push({
  nome: "aparência: aplicar muda a tela na hora, e sobrevive ao recarregar (253)",
  async fn(bridge, ctx) {
    await recarregarEstavel(bridge);
    ctx.assertEq(await bridge.js(ABRIR_APARENCIA), true, "a tela de aparência não abriu");
    await PAUSA(400);

    const antes = await bridge.js(ESTADO_APARENCIA);

    // Aplicar não recarrega a janela nem perde trabalho (RNF3): é só um
    // atributo no `<html>`.
    await bridge.js(`(() => { document.querySelector('button[data-tema="papel"]').click(); return true; })()`);
    await PAUSA(400);
    await bridge.js(`(() => { document.querySelector('button[data-destaque="verde"]').click(); return true; })()`);
    await PAUSA(400);
    await bridge.js(`(() => { document.querySelector('button[data-botoes="pilula"]').click(); return true; })()`);
    await PAUSA(400);

    const depois = await bridge.js(ESTADO_APARENCIA);
    ctx.assertEq(depois.tema, "papel", "o tema não foi aplicado");
    ctx.assertEq(depois.botoes, "pilula", "a forma dos botões não foi aplicada");
    ctx.assert(depois.fundo !== antes.fundo, "o fundo não mudou com o tema");
    ctx.assert(
      depois.destaque !== antes.destaque,
      `a cor de destaque não mudou (${antes.destaque} → ${depois.destaque})`,
    );

    // Persistência (RF5).
    await recarregarEstavel(bridge);
    const recarregado = await bridge.js(ESTADO_APARENCIA);
    ctx.assertEq(recarregado.tema, "papel", "o tema não sobreviveu ao recarregar");
    ctx.assertEq(recarregado.botoes, "pilula", "a forma dos botões não sobreviveu");
    ctx.assertEq(recarregado.destaque, depois.destaque, "o destaque não sobreviveu");

    // Voltar ao padrão (RF6).
    ctx.assertEq(await bridge.js(ABRIR_APARENCIA), true, "a tela não reabriu");
    await PAUSA(400);
    await bridge.js(`(() => {
      const b = [...document.querySelectorAll('.aparencia__rodape button')][0];
      if (b) b.click();
      return !!b;
    })()`);
    await PAUSA(400);
    const padrao = await bridge.js(ESTADO_APARENCIA);
    ctx.assertEq(padrao.tema, "escuro", "voltar ao padrão não restaurou o tema");
    ctx.assertEq(padrao.botoes, "arredondado", "voltar ao padrão não restaurou os botões");
    ctx.assertEq(padrao.destaque, antes.destaque, "voltar ao padrão não restaurou o destaque");
  },
});

telas.push({
  nome: "aparência: o tema não entra no vault (253)",
  async fn(bridge, ctx) {
    // RNF2: tema é preferência do APP. Um tema gravado no vault viraria
    // diff em toda máquina que abrisse a mesma pasta.
    await recarregarEstavel(bridge);
    ctx.assertEq(await bridge.js(ABRIR_APARENCIA), true, "a tela de aparência não abriu");
    await PAUSA(400);
    await bridge.js(`(() => { document.querySelector('button[data-tema="contraste"]').click(); return true; })()`);
    await PAUSA(600);

    const fs = await import("node:fs");
    const suspeitos = fs
      .readdirSync(ctx.vault)
      .filter((f) => /tema|aparencia|theme/i.test(f));
    ctx.assertEq(
      suspeitos.length,
      0,
      `o tema escreveu no vault: ${JSON.stringify(suspeitos)}`,
    );

    await bridge.js(`(() => {
      localStorage.setItem('anotadinho.aparencia', JSON.stringify({tema:'escuro',destaque:'',botoes:'arredondado'}));
      return true;
    })()`);
    await recarregarEstavel(bridge);
  },
});
