// Bateria de TECLADO e VIM (ciclos 250, 251, 252).
//
// Nasceu na bateria `--pendentes`, escrita a partir da spec "Navegação
// por teclado consistente e modo vim completo" antes de existir
// implementação. Com as specs fechadas, os cenários migram pra cá — é a
// regra que `pendentes.mjs` estabelece pra si mesmo, e é o que mantém a
// ligação entre a spec e o que a prova.
//
// O que este arquivo protege, em uma frase por assunto:
//
// - a PILHA de navegação continua descrevendo um lugar que existe,
//   inclusive depois de abrir uma página de dentro de um grupo;
// - Escape sobe um nível e devolve a pessoa ao item de onde saiu;
// - `hjkl` valem onde as setas valem;
// - os modos do vim existem, aparecem na barra, e as teclas de um modo
//   não disparam ações de outro.

import { recarregarEstavel, abrirPaginaEstavel, esperar } from "./bridge.mjs";

const PAUSA = (ms) => new Promise((r) => setTimeout(r, ms));

export const teclados = [];

/// Markdown mínimo da página de rascunho.
const RASCUNHO = "---\ntitle: __uitest\n---\ntexto\n";

/// `setup` pode ser o markdown da página de rascunho, ou `{ md, vim }`.
///
/// O markdown é escrito e o vim é ligado ANTES do reload: a sidebar é
/// montada na carga, e o vim só entra em vigor no mount.
function teclado(nome, setup, fn, ciclo = 252) {
  if (typeof setup === "function") {
    fn = setup;
    setup = null;
  }
  const { md = null, vim = false } =
    typeof setup === "string" ? { md: setup } : setup || {};
  teclados.push({
    nome: `teclado: ${nome} (${ciclo})`,
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
        // `run.mjs` normaliza o vim pra desligado só no INÍCIO da suíte
        // — sem desligar aqui, um cenário de vim contamina todos os
        // seguintes.
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

const ITEM_ATIVO = `(document.querySelector('.nav-mode__item-active')
  ?.getAttribute('data-nav-item') ?? null)`;

/// Página com uma consulta em tabela — precisa de um nível DENTRO do
/// editor pra o cenário dos dois Escapes ter onde descer.
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
{{ /query }}
`;

// ─────────────────────────────────────────────────────────────────────
// Spec: Navegação por teclado e vim
// ─────────────────────────────────────────────────────────────────────

teclado("hjkl move igual às setas", null, async (bridge, ctx) => {
  await bridge.js(TECLA("ArrowDown"));
  await PAUSA(400);
  const primeiro = await bridge.js(ITEM_ATIVO);
  ctx.assert(primeiro, "a seta não entrou no modo de navegação");

  await bridge.js(TECLA("ArrowDown"));
  await PAUSA(300);
  const segundo = await bridge.js(ITEM_ATIVO);
  ctx.assert(segundo !== primeiro, "a segunda seta não moveu — cenário sem base de comparação");

  // `k` tem que desfazer o que a seta pra baixo fez.
  await bridge.js(TECLA("k"));
  await PAUSA(300);
  const depoisDeK = await bridge.js(ITEM_ATIVO);
  ctx.assertEq(depoisDeK, primeiro, "`k` não subiu como a seta pra cima sobe");

  // e `j` tem que refazer.
  await bridge.js(TECLA("j"));
  await PAUSA(300);
  const depoisDeJ = await bridge.js(ITEM_ATIVO);
  ctx.assertEq(depoisDeJ, segundo, "`j` não desceu como a seta pra baixo desce");
}, 250);

teclado("abrir página de dentro de um embed não prende o foco na barra superior", null,
  async (bridge, ctx) => {
    // Caminho relatado: home → grupo de um embed → Enter num card →
    // abre a página → as setas ficam presas na barra de janela.
    await bridge.js(TECLA("ArrowDown"));
    await PAUSA(400);
    await esperar(bridge, `${ITEM_ATIVO} !== null`, "não entrou no modo de navegação");

    const abriu = await bridge.js(`(() => {
      const card = document.querySelector('.query-embed__card, .query-embed__row');
      if (!card) return 'sem cards na home';
      card.click();
      return true;
    })()`);
    ctx.assertEq(abriu, true, String(abriu));
    await PAUSA(1200);

    await bridge.js(TECLA("ArrowDown"));
    await PAUSA(400);
    const ativo = await bridge.js(ITEM_ATIVO);
    ctx.assert(ativo !== "header",
      "depois de abrir a página pelo card, as setas ficaram presas na barra superior");
  }, 250);

teclado("dois Escapes sobem dois níveis", PAGINA_CONSULTA, async (bridge, ctx) => {
  await bridge.js(TECLA("ArrowDown"));
  await PAUSA(400);
  while ((await bridge.js(ITEM_ATIVO)) !== "editor") {
    await bridge.js(TECLA("ArrowDown"));
    await PAUSA(250);
  }
  await bridge.js(TECLA("Enter"));
  await PAUSA(600);
  const dentro = await bridge.js(ITEM_ATIVO);
  ctx.assert(dentro !== "editor", "Enter não desceu pra dentro do editor");

  await bridge.js(TECLA("Escape"));
  await PAUSA(400);
  const umNivel = await bridge.js(ITEM_ATIVO);
  ctx.assertEq(umNivel, "editor", "o primeiro Escape não voltou pro nível do editor");

  await bridge.js(TECLA("Escape"));
  await PAUSA(400);
  const doisNiveis = await bridge.js(`(() => ({
    item: ${ITEM_ATIVO},
    navegando: !!document.querySelector('.nav-mode__item-active'),
  }))()`);
  ctx.assert(!doisNiveis.navegando || doisNiveis.item !== "editor",
    "o segundo Escape não subiu de nível (só Backspace sobe — é o bug relatado)");
}, 250);

teclado("modo visual seleciona por caractere",
  { md: "---\ntitle: __uitest\n---\nabcdef\n", vim: true },
  async (bridge, ctx) => {
    await bridge.js(`(() => {
      const b = document.querySelector('.editor__bloco');
      b.focus();
      const alvo = b.firstChild?.firstChild || b.firstChild;
      if (!alvo) return false;
      const r = document.createRange();
      r.setStart(alvo, 0);
      r.collapse(true);
      const s = getSelection(); s.removeAllRanges(); s.addRange(r);
      return true;
    })()`);
    await bridge.js(TECLA("Escape"));
    await PAUSA(300);
    await bridge.js(TECLA("v"));
    await PAUSA(300);
    const modo = await bridge.js(
      `(document.querySelector('.editor__modo')?.textContent || '').toUpperCase()`);
    ctx.assert(modo.includes("VISUAL"), `a barra de modo não mostra visual: "${modo}"`);

    await bridge.js(TECLA("l"));
    await bridge.js(TECLA("l"));
    await PAUSA(300);
    const sel = await bridge.js(`String(getSelection())`);
    ctx.assert(sel.length >= 2, `visual não estendeu a seleção (veio "${sel}")`);
  });

teclado("modo visual-block seleciona um retângulo",
  { md: "---\ntitle: __uitest\n---\nabc\ndef\nghi\n", vim: true },
  async (bridge, ctx) => {
    await bridge.js(`(() => { document.querySelector('.editor__bloco')?.focus(); return true; })()`);
    await bridge.js(TECLA("Escape"));
    await PAUSA(300);
    await bridge.js(TECLA("v", { ctrlKey: true }));
    await PAUSA(300);
    const modo = await bridge.js(
      `(document.querySelector('.editor__modo')?.textContent || '').toUpperCase()`);
    ctx.assert(modo.includes("BLOCO") || modo.includes("BLOCK"),
      `a barra de modo não mostra visual-block: "${modo}"`);
  });

teclado("`/` abre o modo de comando fora da edição",
  { md: RASCUNHO, vim: true },
  async (bridge, ctx) => {
    await bridge.js(`(() => { document.querySelector('.editor__bloco')?.focus(); return true; })()`);
    await bridge.js(TECLA("Escape"));
    await PAUSA(300);
    await bridge.js(TECLA("/"));
    await PAUSA(400);
    // A paleta de comandos conta (ciclo 252). Este cenário foi escrito
    // antes da implementação e supunha uma linha de comando própria do
    // vim; o app já TEM uma busca de comando, e inventar uma segunda
    // seria um produto pior — duas caixas fazendo a mesma coisa, com
    // vocabulários diferentes. O que a spec pede é que `/` fora da
    // edição abra a busca de comando, e é isso que acontece.
    const abriu = await bridge.js(`(() => {
      const modo = (document.querySelector('.editor__modo')?.textContent || '').toUpperCase();
      return modo.includes('COMANDO')
        || !!document.querySelector('.vim-comando, .editor__comando, .command-palette');
    })()`);
    ctx.assertEq(abriu, true, "`/` não abriu a busca de comando");
  });

teclado("com vim ligado há atalho próprio pro modo de navegação",
  { md: RASCUNHO, vim: true },
  async (bridge, ctx) => {
    // O cheatsheet é a fonte de verdade do keymap (ciclo 199): se o
    // atalho existe, ele está documentado lá.
    await bridge.js(ATALHO("k", { ctrlKey: true }));
    await esperar(bridge, `!!document.querySelector('[class*=palette__item]')`,
      "a paleta não abriu");
    await bridge.js(`(() => {
      const i = [...document.querySelectorAll('[class*=palette__item]')]
        .find(e => e.textContent.includes('Ver atalhos'));
      if (i) i.click();
      return !!i;
    })()`);
    await PAUSA(600);
    const linhas = await bridge.js(`(() => {
      const raiz = document.querySelector('.cheatsheet, .modal');
      return raiz ? [...raiz.querySelectorAll('tr, li')].map(e => e.textContent.trim()) : null;
    })()`);
    ctx.assert(linhas, "o cheatsheet não abriu");
    ctx.assert(
      linhas.some((l) => /navega/i.test(l) && /vim/i.test(l)),
      "o cheatsheet não documenta um atalho de navegação específico do vim",
    );
  });

// ─────────────────────────────────────────────────────────────────────
// Spec: Imagens coladas e arrastadas
// ─────────────────────────────────────────────────────────────────────

/// PNG 1×1 transparente, em base64 — o menor arquivo válido possível.
const PNG_1X1 =
  "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==";

const COLAR_IMAGEM = `(async () => {
  const bin = atob(${JSON.stringify(PNG_1X1)});
  const buf = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) buf[i] = bin.charCodeAt(i);
  const file = new File([buf], 'colada.png', { type: 'image/png' });
  const dt = new DataTransfer();
  dt.items.add(file);
  const alvo = document.querySelector('.editor__bloco');
  alvo.focus();
  // A inserção acontece na SELEÇÃO — sem um range de verdade, o
  // editor não tem onde escrever e o arquivo fica no acervo sem
  // referência na nota.
  const r = document.createRange();
  r.selectNodeContents(alvo); r.collapse(false);
  const s = getSelection(); s.removeAllRanges(); s.addRange(r);
  alvo.dispatchEvent(new ClipboardEvent('paste', {
    bubbles: true, cancelable: true, clipboardData: dt }));
  return true;
})()`;


/// Espia o payload que vai pra área de transferência — ler o clipboard
/// do sistema não é possível no WebView sem permissão, e o que precisa
/// ser provado é o CONTEÚDO copiado.
const ESPIAR_COPIA = `(() => {
  if (!window.__espiaCopia) {
    window.__espiaCopia = true;
    const orig = document.execCommand.bind(document);
    document.execCommand = function (cmd, ...resto) {
      if (cmd === 'copy') {
        const a = document.activeElement;
        window.__copiado = a && 'value' in a ? a.value : null;
      }
      return orig(cmd, ...resto);
    };
  }
  window.__copiado = null;
  return true;
})()`;

/// Cursor no bloco `i`, na coluna `col`.
const CURSOR_EM = (i, col) => `(() => {
  const b = document.querySelectorAll('.editor__bloco')[${i}];
  if (!b || !b.firstChild) return false;
  b.focus();
  const r = document.createRange();
  r.setStart(b.firstChild, ${col});
  r.collapse(true);
  const s = getSelection(); s.removeAllRanges(); s.addRange(r);
  return true;
})()`;

const ONDE_ESTA = `(() => {
  const s = getSelection();
  const n = s.anchorNode;
  const el = n && (n.nodeType === 1 ? n : n.parentElement);
  const bloco = el && el.closest('.editor__bloco');
  return { bloco: bloco ? bloco.textContent : null, coluna: s.anchorOffset };
})()`;

const TRES_LINHAS = "---\ntitle: __uitest\n---\nabcdefghij\n\nklmnopqrst\n\nuvwxyzABCD\n";

teclado("visual-block recorta um retângulo de verdade",
  { md: TRES_LINHAS, vim: true },
  async (bridge, ctx) => {
    // A spec pede retângulo, e "abre o modo" não prova retângulo nenhum.
    // Este cenário checa o CONTEÚDO: as mesmas colunas em três blocos.
    await bridge.js(CURSOR_EM(0, 2));
    await bridge.js(ESPIAR_COPIA);
    await bridge.js(TECLA("v", { ctrlKey: true }));
    await PAUSA(300);

    await bridge.js(TECLA("j"));
    await PAUSA(250);
    await bridge.js(TECLA("j"));
    await PAUSA(250);
    const realcados = await bridge.js(
      `document.querySelectorAll('.editor__bloco--selecionado').length`,
    );
    ctx.assertEq(realcados, 3, "os três blocos do retângulo deviam estar realçados");

    for (let i = 0; i < 3; i++) {
      await bridge.js(TECLA("l"));
      await PAUSA(120);
    }
    await bridge.js(TECLA("y"));
    await PAUSA(400);

    const copiado = await bridge.js(`window.__copiado`);
    ctx.assertEq(
      copiado,
      "cde\nmno\nwxy",
      "o retângulo não saiu por coluna — veio " + JSON.stringify(copiado),
    );
    const limpou = await bridge.js(
      `document.querySelectorAll('.editor__bloco--selecionado').length`,
    );
    ctx.assertEq(limpou, 0, "o realce devia ter sido largado depois do yank");
  });

teclado("j e k atravessam blocos mantendo a coluna",
  { md: TRES_LINHAS, vim: true },
  async (bridge, ctx) => {
    // O que a spec chamou de "o vim ficou pra trás": desde o ciclo 175
    // cada bloco é seu próprio contenteditable, e `Selection.modify` não
    // sai do host — `j` ia pro FIM do parágrafo e parava ali pra sempre.
    await bridge.js(CURSOR_EM(0, 2));
    await bridge.js(TECLA("j"));
    await PAUSA(300);
    const desceu = await bridge.js(ONDE_ESTA);
    ctx.assert(
      (desceu.bloco || "").startsWith("klmno"),
      `j não saiu do primeiro bloco (ficou em ${JSON.stringify(desceu.bloco)})`,
    );
    ctx.assertEq(desceu.coluna, 2, "a coluna devia ter sido mantida");

    await bridge.js(TECLA("k"));
    await PAUSA(300);
    const subiu = await bridge.js(ONDE_ESTA);
    ctx.assert(
      (subiu.bloco || "").startsWith("abcde"),
      `k não voltou pro bloco de cima (ficou em ${JSON.stringify(subiu.bloco)})`,
    );
  });

teclado("com vim ligado, o modo de navegação continua sendo dono das teclas dele",
  { md: TRES_LINHAS, vim: true },
  async (bridge, ctx) => {
    // RNF1: as teclas de um modo não disparam ações de outro. O ramo do
    // vim rodava ANTES dos atalhos de bloco e engolia `hjkl`, `d`, `y` e
    // `v` — o modo de navegação virava letra morta justamente pra quem
    // usa vim.
    await bridge.js(CURSOR_EM(0, 0));
    await bridge.js(`(() => {
      const b = document.querySelector('.editor__bloco');
      b.dispatchEvent(new KeyboardEvent('keydown',
        { key: 'n', code: 'KeyN', altKey: true, bubbles: true, cancelable: true }));
      return true;
    })()`);
    await PAUSA(500);
    const modo = await bridge.js(
      `(document.querySelector('.editor__modo') || {}).textContent || ''`,
    );
    ctx.assertEq(modo, "NAVEGAÇÃO", "Alt+N devia ter entrado na navegação mesmo com vim ligado");

    await bridge.js(TECLA("ArrowDown", { shiftKey: true }));
    await PAUSA(400);
    const selecionados = await bridge.js(
      `document.querySelectorAll('.editor__bloco--selecionado').length`,
    );
    ctx.assertEq(selecionados, 2, "a seleção de blocos devia funcionar com o vim ligado");
  });
