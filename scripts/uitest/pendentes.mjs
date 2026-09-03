// Bateria PENDENTE: cenários escritos a partir de specs ainda não
// implementadas.
//
// Por que fica separada de `todos`: ela é VERMELHA de propósito. A
// suíte principal é o sinal de "está tudo certo?" e precisa ficar verde
// — misturar aqui destruiria esse sinal. Roda só com:
//
//   node scripts/uitest/run.mjs --pendentes
//
// À medida que cada spec for implementada, o cenário correspondente
// migra pra bateria permanente (`interacoes.mjs`, `telas.mjs`, etc) e
// sai daqui. Quando este arquivo esvaziar, o backlog de UI acabou.
//
// Cada cenário nomeia a spec que o originou, pra a ligação sobreviver.

import { recarregarEstavel, abrirPaginaEstavel, esperar } from "./bridge.mjs";

const PAUSA = (ms) => new Promise((r) => setTimeout(r, ms));

export const pendentes = [];

/// Markdown mínimo da página de rascunho.
const RASCUNHO = "---\ntitle: __uitest\n---\ntexto\n";

/// `setup` pode ser o markdown da página de rascunho, ou
/// `{ md, vim }`.
///
/// O markdown é escrito e o vim é ligado ANTES do reload: a sidebar é
/// montada na carga (escrever depois deixa a página invisível pro
/// `abrirPaginaEstavel`) e o vim só entra em vigor no mount.
function pendente(spec, nome, setup, fn) {
  if (typeof setup === "function") {
    fn = setup;
    setup = null;
  }
  const { md = null, vim = false } =
    typeof setup === "string" ? { md: setup } : setup || {};
  pendentes.push({
    nome: `[${spec}] ${nome}`,
    async fn(bridge, ctx) {
      if (md) ctx.escrever(md);
      if (vim) {
        await bridge.js(`localStorage.setItem('anotadinho.vim_mode_enabled', 'true'); true`);
      }
      try {
        await recarregarEstavel(bridge);
        if (md) await abrirPaginaEstavel(bridge, ctx.nomePagina);
        await fn(bridge, ctx);
      } finally {
        // `run.mjs` normaliza o vim pra desligado só no INÍCIO da
        // suíte — sem desligar aqui, um cenário de vim contamina todos
        // os seguintes.
        if (vim) {
          await bridge.js(
            `localStorage.setItem('anotadinho.vim_mode_enabled', 'false'); true`,
          );
        }
      }
    },
  });
}

/// Dispara um atalho global (o listener vive em `.app-root`).
const ATALHO = (key, extra = {}) => `(() => {
  const raiz = document.querySelector('.app-root') || document.body;
  raiz.dispatchEvent(new KeyboardEvent('keydown', Object.assign(
    { key: ${JSON.stringify(key)}, bubbles: true, cancelable: true }, ${JSON.stringify(extra)})));
  return true;
})()`;

/// Manda uma tecla pro elemento que está com o foco de navegação.
const TECLA = (key, extra = {}) => `(() => {
  const alvo = document.querySelector('.nav-mode__item-active')
    || document.activeElement
    || document.body;
  alvo.dispatchEvent(new KeyboardEvent('keydown', Object.assign(
    { key: ${JSON.stringify(key)}, bubbles: true, cancelable: true }, ${JSON.stringify(extra)})));
  return true;
})()`;

/// Clica em Salvar. A trava é a mesma das outras baterias (ciclo 197):
/// um cenário que navegou pra uma página real e mandou salvar reescreve
/// o arquivo do usuário.
const SALVAR = `(() => {
  const titulo = (document.querySelector('.editor__title') || {}).textContent || '';
  if (!titulo.includes('__uitest')) {
    throw new Error('Salvar bloqueado: a página aberta é "' + titulo + '"');
  }
  const b = [...document.querySelectorAll('button')].find(b => b.textContent.trim().startsWith('Salvar'));
  if (b) b.click();
  return !!b;
})()`;

const ITEM_ATIVO = `(document.querySelector('.nav-mode__item-active')
  ?.getAttribute('data-nav-item') ?? null)`;

// ─────────────────────────────────────────────────────────────────────
// Spec: Leitura de consultas
// ─────────────────────────────────────────────────────────────────────

/// Página com uma consulta em tabela de 3 colunas — usada pelos três
/// cenários abaixo.
const PAGINA_CONSULTA = `---
title: __uitest
---
# consulta

{{ type: "query" }}
from: pages
where:
- field: type
  op: exists
view: table
columns:
- type
- tags
- status
{{ /query }}
`;

pendente("consultas", "valores ficam alinhados com o cabeçalho", PAGINA_CONSULTA, async (bridge, ctx) => {
  await esperar(bridge, `!!document.querySelector('.query-embed__table td')`,
    "a tabela da consulta não renderizou");

  const desalinhadas = await bridge.js(`(() => {
    const tab = document.querySelector('.query-embed__table');
    const ths = [...tab.querySelectorAll('th')].map(e => Math.round(e.getBoundingClientRect().left));
    const linha = tab.querySelector('tbody tr');
    const tds = [...linha.querySelectorAll('td')].map(e => Math.round(e.getBoundingClientRect().left));
    return ths.map((x, i) => (tds[i] === undefined || Math.abs(tds[i] - x) > 2)
      ? { coluna: i, th: x, td: tds[i] } : null).filter(Boolean);
  })()`);
  ctx.assertEq(desalinhadas.length, 0,
    `colunas desalinhadas: ${JSON.stringify(desalinhadas)}`);
});

pendente("consultas", "o mesmo valor tem sempre a mesma cor", PAGINA_CONSULTA, async (bridge, ctx) => {
  await esperar(bridge, `!!document.querySelector('.query-embed__table td')`,
    "a tabela da consulta não renderizou");

  const r = await bridge.js(`(() => {
    const cores = new Map();
    const conflitos = [];
    for (const td of document.querySelectorAll('.query-embed__table tbody td')) {
      const txt = td.textContent.trim();
      if (!txt) continue;
      const alvo = td.firstElementChild || td;
      const cor = getComputedStyle(alvo).color + '|' + getComputedStyle(alvo).backgroundColor;
      if (cores.has(txt) && cores.get(txt) !== cor) conflitos.push(txt);
      cores.set(txt, cor);
    }
    const distintas = new Set(cores.values());
    return { conflitos, valores: cores.size, distintas: distintas.size };
  })()`);
  ctx.assertEq(r.conflitos.length, 0,
    `mesmo valor com cores diferentes: ${JSON.stringify(r.conflitos)}`);
  ctx.assert(r.distintas > 1,
    `todos os ${r.valores} valores saíram com a mesma cor — não há cor por valor`);
});

pendente("consultas", "o bloco de consulta tem altura limitada e rola", PAGINA_CONSULTA, async (bridge, ctx) => {
  await esperar(bridge, `!!document.querySelector('.query-embed__table td')`,
    "a tabela da consulta não renderizou");

  const r = await bridge.js(`(() => {
    const el = document.querySelector('.query-embed');
    const rola = [el, ...el.querySelectorAll('*')].find(
      n => n.scrollHeight > n.clientHeight + 4 && /auto|scroll/.test(getComputedStyle(n).overflowY));
    return { altura: Math.round(el.getBoundingClientRect().height),
             janela: Math.round(window.innerHeight),
             rolaDentro: !!rola };
  })()`);
  ctx.assert(r.altura <= r.janela,
    `a consulta ocupa ${r.altura}px, mais que a janela (${r.janela}px)`);
  ctx.assertEq(r.rolaDentro, true, "a consulta não rola internamente");
});

pendente("imagens", "arrastar imagem grava no acervo, não uma URL de sessão",
  RASCUNHO, async (bridge, ctx) => {
    const fs = await import("node:fs");
    const antes = fs.existsSync(`${ctx.vault}/assets`)
      ? fs.readdirSync(`${ctx.vault}/assets`) : [];

    await bridge.js(`(() => {
      const bin = atob(${JSON.stringify(PNG_1X1)});
      const buf = new Uint8Array(bin.length);
      for (let i = 0; i < bin.length; i++) buf[i] = bin.charCodeAt(i);
      const file = new File([buf], 'arrastada.png', { type: 'image/png' });
      const dt = new DataTransfer();
      dt.items.add(file);
      const alvo = document.querySelector('.editor__bloco');
      alvo.focus();
      const r = document.createRange();
      r.selectNodeContents(alvo); r.collapse(false);
      const s = getSelection(); s.removeAllRanges(); s.addRange(r);
      for (const tipo of ['dragenter', 'dragover', 'drop']) {
        alvo.dispatchEvent(new DragEvent(tipo, {
          bubbles: true, cancelable: true, dataTransfer: dt }));
      }
      return true;
    })()`);
    await PAUSA(1500);
    await bridge.js(SALVAR);
    await PAUSA(1000);

    // O sintoma que a pessoa vê: a imagem aparece na tela apontando
    // pra uma URL `blob:`, que morre com a sessão.
    const blob = await bridge.js(`(() => {
      const img = document.querySelector('.editor__bloco img, .editor img');
      return img ? img.getAttribute('src') : null;
    })()`);
    ctx.assert(!blob || !blob.startsWith("blob:"),
      `a imagem entrou como URL de sessão (${blob}) — some ao recarregar`);

    const depois = fs.existsSync(`${ctx.vault}/assets`)
      ? fs.readdirSync(`${ctx.vault}/assets`) : [];
    const novos = depois.filter((f) => !antes.includes(f));
    try {
      ctx.assertEq(novos.length, 1,
        `esperava 1 arquivo novo no acervo, vieram ${novos.length}`);
      const md = ctx.ler() || "";
      ctx.assert(/!\[[^\]]*\]\([^)]+\)/.test(md),
        `a nota não ganhou referência de imagem:\n${md}`);
    } finally {
      novos.forEach((f) => fs.rmSync(`${ctx.vault}/assets/${f}`, { force: true }));
    }
  });

pendente("imagens", "a imagem arrastada sobrevive ao recarregar",
  RASCUNHO, async (bridge, ctx) => {
    const fs = await import("node:fs");
    const antes = fs.existsSync(`${ctx.vault}/assets`)
      ? fs.readdirSync(`${ctx.vault}/assets`) : [];

    await bridge.js(`(() => {
      const bin = atob(${JSON.stringify(PNG_1X1)});
      const buf = new Uint8Array(bin.length);
      for (let i = 0; i < bin.length; i++) buf[i] = bin.charCodeAt(i);
      const file = new File([buf], 'persiste.png', { type: 'image/png' });
      const dt = new DataTransfer();
      dt.items.add(file);
      const alvo = document.querySelector('.editor__bloco');
      alvo.focus();
      alvo.dispatchEvent(new DragEvent('drop', {
        bubbles: true, cancelable: true, dataTransfer: dt }));
      return true;
    })()`);
    await PAUSA(2000);
    await bridge.js(SALVAR);
    await PAUSA(1000);
    await recarregarEstavel(bridge);
    await abrirPaginaEstavel(bridge, ctx.nomePagina);

    const depois = fs.existsSync(`${ctx.vault}/assets`)
      ? fs.readdirSync(`${ctx.vault}/assets`) : [];
    const novos = depois.filter((f) => !antes.includes(f));
    try {
      const visivel = await bridge.js(`(() => {
        const img = document.querySelector('.editor img');
        return img ? { src: img.getAttribute('src'), largura: img.naturalWidth } : null;
      })()`);
      ctx.assert(visivel, "depois de recarregar não há imagem nenhuma na página");
      ctx.assert(!visivel.src.startsWith("blob:"),
        `a referência gravada no .md é uma URL de sessão morta: ${visivel.src}`);
      ctx.assert(visivel.largura > 0,
        "a imagem está na página mas não carrega — referência quebrada");
    } finally {
      novos.forEach((f) => fs.rmSync(`${ctx.vault}/assets/${f}`, { force: true }));
    }
  });

// Guarda de regressão do RNF2: o caminho mais usado do editor não pode
// quebrar quando o arraste for consertado.
//
// Um `ClipboardEvent` sintético não é confiável, então o navegador não
// executa a inserção nativa — não dá pra conferir o TEXTO. O que dá, e
// é o que importa aqui, é conferir que o app não CANCELA o paste de
// texto: se a implementação passar a `preventDefault()` em todo paste,
// o de texto morre junto.
pendente("imagens", "o app não cancela o paste de texto",
  "---\ntitle: __uitest\n---\ninicio\n", async (bridge, ctx) => {
  const cancelado = await bridge.js(`(() => {
    const dt = new DataTransfer();
    dt.setData('text/plain', 'colado');
    const alvo = document.querySelector('.editor__bloco');
    alvo.focus();
    const ev = new ClipboardEvent('paste', {
      bubbles: true, cancelable: true, clipboardData: dt });
    alvo.dispatchEvent(ev);
    return ev.defaultPrevented;
  })()`);
  ctx.assertEq(cancelado, false,
    "o app cancelou um paste que só tinha texto — o caminho de texto quebrou");
});

pendente("imagens", "colar imagem grava no acervo (guarda do ciclo 118)",
  RASCUNHO, async (bridge, ctx) => {
    const fs = await import("node:fs");
    const antes = fs.existsSync(`${ctx.vault}/assets`)
      ? fs.readdirSync(`${ctx.vault}/assets`) : [];
    await bridge.js(COLAR_IMAGEM);
    await PAUSA(2000);
    // Sem salvar, a nota no disco ainda é a de antes — o autosave está
    // desligado nos testes.
    await bridge.js(SALVAR);
    await PAUSA(1000);
    const depois = fs.existsSync(`${ctx.vault}/assets`)
      ? fs.readdirSync(`${ctx.vault}/assets`) : [];
    const novos = depois.filter((f) => !antes.includes(f));
    try {
      ctx.assertEq(novos.length, 1,
        `colar imagem devia gravar 1 arquivo, gravou ${novos.length}`);
      const md = ctx.ler() || "";
      ctx.assert(/!\[[^\]]*\]\([^)]+\)/.test(md),
        `a nota não ganhou a referência da imagem colada:\n${md}`);
    } finally {
      novos.forEach((f) => fs.rmSync(`${ctx.vault}/assets/${f}`, { force: true }));
    }
  });

// ─────────────────────────────────────────────────────────────────────
// Spec: Tema configurável
// ─────────────────────────────────────────────────────────────────────

pendente("tema", "há configuração de tema, além do alternador claro/escuro",
  async (bridge, ctx) => {
    const abriu = await bridge.js(`(() => {
      const b = document.querySelector('[data-nav-item="header-menu"]');
      if (!b) return 'sem botão de configurações no header';
      b.click();
      return true;
    })()`);
    ctx.assertEq(abriu, true, String(abriu));
    await PAUSA(500);

    const itens = await bridge.js(`(() => [...document.querySelectorAll('button, [role=menuitem]')]
      .map(e => e.textContent.trim()).filter(t => /tema|aparência|aparencia/i.test(t)))()`);
    // "Tema escuro"/"Tema claro" é o alternador que já existe — não
    // conta como configuração.
    const config = itens.filter((i) => !/^tema (escuro|claro)$/i.test(i.replace(/\s+/g, " ")));
    ctx.assert(config.length > 0,
      `o menu só tem o alternador claro/escuro: ${JSON.stringify(itens)}`);
  });

pendente("tema", "a escolha de tema sobrevive ao recarregar", async (bridge, ctx) => {
  const temas = await bridge.js(`(() => {
    const b = document.querySelector('[data-nav-item="header-menu"]');
    if (b) b.click();
    return [...document.querySelectorAll('[data-tema]')].map(e => e.getAttribute('data-tema'));
  })()`);
  ctx.assert(temas.length > 1,
    `esperava vários temas pra escolher, achei ${JSON.stringify(temas)}`);

  await bridge.js(`(() => {
    document.querySelector('[data-tema="' + ${JSON.stringify(temas)}[1] + '"]').click();
    return true;
  })()`);
  await PAUSA(600);
  await recarregarEstavel(bridge);
  const atual = await bridge.js(`document.documentElement.getAttribute('data-theme')`);
  ctx.assertEq(atual, temas[1], "o tema escolhido não sobreviveu ao recarregar");
});
