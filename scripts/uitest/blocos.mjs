// Matriz completa de BLOCOS (ciclo 195).
//
// Por que um arquivo só pra isto: os bugs desta área nunca estiveram
// dentro de um modo — estiveram na TRANSIÇÃO entre eles. `d` apagando
// bloco enquanto se digitava, Enter editando sem sair da navegação,
// foco perdido depois de apagar. Testar cada modo isolado não pega
// nenhum desses.
//
// Por isso quase todo cenário aqui termina checando DUAS coisas:
// o efeito pedido, e que o MODO na barra bate com o comportamento.
//
// Organização:
//   1. navegação   — entrar, andar, sair
//   2. movimentação — subir, descer, limites
//   3. adição      — n, Shift+Enter, duplicar
//   4. edição      — digitar, quebrar linha, fundir
//   5. misto       — as transições, que é onde dói

import { esperar, recarregarEstavel, abrirPaginaEstavel } from "./bridge.mjs";

const PAUSA = (ms) => new Promise((r) => setTimeout(r, ms));

export const blocos = [];

/// Estado observável: o que a barra diz e como estão os blocos.
const ESTADO = `(() => ({
  modo: (document.querySelector('.editor__modo') || {}).textContent || null,
  blocos: [...document.querySelectorAll('.editor__bloco')].map(b => b.textContent.trim()),
  focado: document.activeElement ? document.activeElement.textContent.trim().slice(0, 30) : null,
  destacado: (document.querySelector('.nav-mode__item-active') || {}).textContent || null,
  menuAberto: !!document.querySelector('.slash-menu'),
}))()`;

/// Escape a partir do texto — o caminho real pro modo de navegação.
const IR_PRA_NAVEGACAO = (texto) => `(() => {
  const alvo = [...document.querySelectorAll('.editor__bloco')]
    .find(b => b.textContent.includes(${JSON.stringify("§ALVO§")}));
  if (!alvo) return false;
  alvo.focus();
  const r = document.createRange();
  r.selectNodeContents(alvo); r.collapse(false);
  const s = getSelection(); s.removeAllRanges(); s.addRange(r);
  alvo.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true }));
  return true;
})()`.replace("§ALVO§", texto);

/// Tecla no alvo do momento (o destacado pelo nav-mode, ou o focado).
const TECLA = (key, extra = {}) => `(() => {
  const alvo = document.querySelector('.nav-mode__item-active')
    || document.activeElement.closest('.editor__bloco')
    || document.activeElement;
  alvo.dispatchEvent(new KeyboardEvent('keydown', Object.assign(
    { key: ${JSON.stringify(key)}, bubbles: true, cancelable: true }, ${JSON.stringify(extra)})));
  return true;
})()`;

const ESCREVER = (t) => `(() => { document.execCommand('insertText', false, ${JSON.stringify(t)}); return true; })()`;

const CURSOR_NO_FIM_DO_BLOCO = (texto) => `(() => {
  const alvo = [...document.querySelectorAll('.editor__bloco')]
    .find(b => b.textContent.includes(${JSON.stringify("§ALVO§")}));
  alvo.focus();
  const r = document.createRange();
  r.selectNodeContents(alvo); r.collapse(false);
  const s = getSelection(); s.removeAllRanges(); s.addRange(r);
  return true;
})()`.replace("§ALVO§", texto);

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

/// Monta um cenário com a página `inicial` já aberta.
function bloco(nome, inicial, fn, ciclo = 195) {
  blocos.push({
    // O ciclo vem do parâmetro: cenário novo escrito num ciclo posterior
    // não pode ficar rotulado com o número de quem criou o arquivo.
    nome: `blocos: ${nome} (${ciclo})`,
    async fn(bridge, ctx) {
      ctx.escrever(`---\ntitle: __uitest\n---\n${inicial}`);
      // Espera por CONDIÇÃO, não por relógio (ciclo 198).
      await recarregarEstavel(bridge);
      await abrirPaginaEstavel(bridge, ctx.nomePagina);
      await fn(bridge, ctx, {
        estado: () => bridge.js(ESTADO),
        salvarELer: async () => {
          await bridge.js(SALVAR);
          await PAUSA(1000);
          return corpo(ctx.ler());
        },
      });
    },
  });
}

const TRES = "alfa\n\nbeta\n\ngama\n";

// ── 1. navegação ────────────────────────────────────────────────────

bloco("Escape entra em navegação e Enter volta pra edição", TRES, async (b, ctx, h) => {
  ctx.assertEq((await h.estado()).modo, "EDIÇÃO", "começa em edição");

  await b.js(IR_PRA_NAVEGACAO("beta"));
  await PAUSA(500);
  ctx.assertEq((await h.estado()).modo, "NAVEGAÇÃO", "Escape devia entrar em navegação");

  await b.js(TECLA("Enter"));
  await PAUSA(600);
  const dep = await h.estado();
  ctx.assertEq(dep.modo, "EDIÇÃO", "Enter devia VOLTAR pra edição");
  ctx.assertEq(dep.destacado, null, "o destaque de navegação não pode sobrar");
});

bloco("setas andam entre blocos em navegação", TRES, async (b, ctx, h) => {
  await b.js(IR_PRA_NAVEGACAO("alfa"));
  await PAUSA(500);
  await b.js(TECLA("ArrowDown"));
  await PAUSA(400);
  ctx.assert(
    ((await h.estado()).destacado || "").includes("beta"),
    "a seta devia ter movido pro bloco de baixo",
  );
});

// ── 2. movimentação ─────────────────────────────────────────────────

for (const [nome, tecla, extra, esperado] of [
  ["Alt+↓ desce o bloco", "ArrowDown", { altKey: true }, "beta,alfa,gama"],
  ["J desce o bloco", "J", {}, "beta,alfa,gama"],
]) {
  bloco(nome, TRES, async (b, ctx, h) => {
    await b.js(IR_PRA_NAVEGACAO("alfa"));
    await PAUSA(500);
    await b.js(TECLA(tecla, extra));
    await PAUSA(600);
    ctx.assertEq((await h.estado()).blocos.join(","), esperado, nome);
  });
}

bloco("K sobe o bloco e a ordem chega ao arquivo", TRES, async (b, ctx, h) => {
  await b.js(IR_PRA_NAVEGACAO("gama"));
  await PAUSA(500);
  await b.js(TECLA("K"));
  await PAUSA(600);
  ctx.assertEq((await h.estado()).blocos.join(","), "alfa,gama,beta", "K devia subir");
  const md = await h.salvarELer();
  ctx.assert(/alfa[\s\S]*gama[\s\S]*beta/.test(md), `a ordem não chegou no disco:\n${md}`);
});

bloco("mover no limite não faz nada e não perde o foco", TRES, async (b, ctx, h) => {
  await b.js(IR_PRA_NAVEGACAO("alfa"));
  await PAUSA(500);
  await b.js(TECLA("K")); // já é o primeiro
  await PAUSA(500);
  const est = await h.estado();
  ctx.assertEq(est.blocos.join(","), "alfa,beta,gama", "nada devia mudar");
  ctx.assertEq(est.modo, "NAVEGAÇÃO", "continua navegando");
  ctx.assert(est.destacado !== null, "o foco não pode se perder no limite");
});

// ── 3. adição ───────────────────────────────────────────────────────

bloco("n em navegação abre bloco novo com o menu /", TRES, async (b, ctx, h) => {
  await b.js(IR_PRA_NAVEGACAO("beta"));
  await PAUSA(500);
  await b.js(TECLA("n"));
  await esperar(b, "document.querySelector('.slash-menu')", "o menu / abrir");
  ctx.assertEq((await h.estado()).modo, "EDIÇÃO", "criar bloco entra em edição");
});

bloco("y duplica o bloco logo abaixo", TRES, async (b, ctx, h) => {
  await b.js(IR_PRA_NAVEGACAO("beta"));
  await PAUSA(500);
  await b.js(TECLA("y"));
  await PAUSA(600);
  ctx.assertEq((await h.estado()).blocos.join(","), "alfa,beta,beta,gama", "y devia duplicar");
});

bloco("Shift+Enter em edição cria bloco", TRES, async (b, ctx, h) => {
  await b.js(CURSOR_NO_FIM_DO_BLOCO("gama"));
  await b.js(TECLA("Enter", { shiftKey: true }));
  await PAUSA(500);
  await b.js(ESCREVER("delta"));
  await PAUSA(300);
  const est = await h.estado();
  ctx.assertEq(est.blocos.length, 4, "devia haver 4 blocos");
  ctx.assertEq(est.modo, "EDIÇÃO", "continua editando");
});

// ── 4. edição ───────────────────────────────────────────────────────

bloco("Enter quebra linha sem criar bloco", TRES, async (b, ctx, h) => {
  await b.js(CURSOR_NO_FIM_DO_BLOCO("beta"));
  await b.js(TECLA("Enter"));
  await PAUSA(400);
  await b.js(ESCREVER("mesma caixa"));
  await PAUSA(300);
  const est = await h.estado();
  ctx.assertEq(est.blocos.length, 3, "Enter não pode criar bloco");
  ctx.assert(
    est.blocos.some((t) => t.includes("beta") && t.includes("mesma caixa")),
    `as duas linhas deviam estar no MESMO bloco: ${JSON.stringify(est.blocos)}`,
  );
});

bloco("Backspace no início funde com o anterior", TRES, async (b, ctx, h) => {
  await b.js(`(() => {
    const alvo = [...document.querySelectorAll('.editor__bloco')].find(x => x.textContent.includes('beta'));
    alvo.focus();
    const r = document.createRange();
    r.selectNodeContents(alvo); r.collapse(true);
    const s = getSelection(); s.removeAllRanges(); s.addRange(r);
    return true;
  })()`);
  await b.js(TECLA("Backspace"));
  await PAUSA(500);
  ctx.assertEq((await h.estado()).blocos.join(","), "alfabeta,gama", "deviam ter fundido");
});

// ── 5. misto: as transições ─────────────────────────────────────────

bloco("apagar bloco em navegação e CONTINUAR navegando", TRES, async (b, ctx, h) => {
  // O bug: depois do `d` o re-render trocava os nós do DOM, o foco caía
  // no <body> e nem seta nem Escape respondiam mais.
  await b.js(IR_PRA_NAVEGACAO("beta"));
  await PAUSA(500);
  await b.js(TECLA("d"));
  await PAUSA(800);

  const dep = await h.estado();
  ctx.assertEq(dep.blocos.join(","), "alfa,gama", "beta devia ter sumido");
  ctx.assertEq(dep.modo, "NAVEGAÇÃO", "continua em navegação");
  ctx.assert(dep.destacado !== null, "tem que sobrar um bloco destacado");

  // E a navegação ainda responde.
  await b.js(TECLA("ArrowUp"));
  await PAUSA(400);
  ctx.assert((await h.estado()).destacado !== null, "as setas pararam de responder depois do d");

  // E dá pra sair.
  await b.js(TECLA("Enter"));
  await PAUSA(600);
  ctx.assertEq((await h.estado()).modo, "EDIÇÃO", "não consegui sair do modo depois de apagar");
});

bloco("apagar o ÚLTIMO bloco não deixa o foco no vazio", "único\n", async (b, ctx, h) => {
  await b.js(IR_PRA_NAVEGACAO("único"));
  await PAUSA(500);
  await b.js(TECLA("d"));
  await PAUSA(800);
  const dep = await h.estado();
  ctx.assertEq(dep.modo, "NAVEGAÇÃO", "o modo não pode se perder");
  ctx.assert(dep.blocos.length >= 1, "tem que sobrar um bloco vazio pra digitar");
});

bloco("navegar, entrar, digitar e voltar preserva tudo", TRES, async (b, ctx, h) => {
  await b.js(IR_PRA_NAVEGACAO("alfa"));
  await PAUSA(500);
  await b.js(TECLA("ArrowDown"));
  await PAUSA(400);
  await b.js(TECLA("Enter"));
  await PAUSA(600);
  ctx.assertEq((await h.estado()).modo, "EDIÇÃO", "Enter devia entrar em edição");

  await b.js(ESCREVER(" editado"));
  await PAUSA(300);

  await b.js(TECLA("Escape"));
  await PAUSA(600);
  ctx.assertEq((await h.estado()).modo, "NAVEGAÇÃO", "Escape devia voltar pra navegação");

  const md = await h.salvarELer();
  ctx.assert(md.includes("editado"), `a edição se perdeu:\n${md}`);
  for (const t of ["alfa", "gama"]) {
    ctx.assert(md.includes(t), `"${t}" se perdeu:\n${md}`);
  }
});

bloco("mover em navegação, entrar e editar mantém a ordem nova", TRES, async (b, ctx, h) => {
  await b.js(IR_PRA_NAVEGACAO("gama"));
  await PAUSA(500);
  await b.js(TECLA("K"));
  await PAUSA(600);
  await b.js(TECLA("Enter"));
  await PAUSA(600);
  await b.js(ESCREVER("!"));
  await PAUSA(300);

  const md = await h.salvarELer();
  ctx.assert(/alfa[\s\S]*gama[\s\S]*beta/.test(md), `a ordem se perdeu ao editar:\n${md}`);
  ctx.assert(md.includes("gama!"), `a edição foi pro bloco errado:\n${md}`);
});

bloco("bloco vazio no meio da página não recebe o convite", TRES, async (b, ctx, h) => {
  // Sem hover do mouse, um bloco vazio no meio de uma página escrita
  // não pode mostrar instrução nenhuma.
  await b.js(CURSOR_NO_FIM_DO_BLOCO("beta"));
  await b.js(TECLA("Enter", { shiftKey: true }));
  await PAUSA(600);
  const temConvite = await b.js(`(() => {
    const vazio = [...document.querySelectorAll('.editor__bloco')].find(x => !x.textContent.trim());
    return vazio ? vazio.classList.contains('editor__bloco--convite') : 'sem bloco vazio';
  })()`);
  ctx.assertEq(temConvite, false, "bloco vazio no meio da página não é convite");
});

bloco("página vazia recebe o convite no único bloco", "\n", async (b, ctx, h) => {
  const temConvite = await b.js(`(() => {
    const bs = [...document.querySelectorAll('.editor__bloco')];
    return bs.length === 1 && bs[0].classList.contains('editor__bloco--convite');
  })()`);
  ctx.assertEq(temConvite, true, "página em branco devia convidar a escrever");
});

// ── ciclo 197: tecla comum em navegação, e âncora perdida ────────────

bloco("tecla comum em navegação NÃO vira texto", TRES, async (b, ctx, h) => {
  // O espelho do bug do 194: lá comando virava ação durante a digitação,
  // aqui letra vira texto durante a navegação.
  await b.js(IR_PRA_NAVEGACAO("beta"));
  await PAUSA(500);
  ctx.assertEq((await h.estado()).modo, "NAVEGAÇÃO", "precisa estar navegando");

  for (const ch of "xpto") {
    await b.js(TECLA(ch));
  }
  await PAUSA(600);

  const est = await h.estado();
  ctx.assertEq(est.blocos.join(","), "alfa,beta,gama", "nada podia ter mudado");
  ctx.assertEq(est.modo, "NAVEGAÇÃO", "continua navegando");

  const md = await h.salvarELer();
  ctx.assert(!md.includes("xpto"), `as letras entraram como texto:\n${md}`);
}, 197);

bloco("setas voltam a andar depois do foco cair no genérico", TRES, async (b, ctx, h) => {
  // Reproduz o efeito de fechar um overlay: o foco vai pro `.app-root` e
  // o nav-mode fica sem item. Antes disso travar as setas de vez, ele
  // reancora no grupo atual (ciclo 197).
  await b.js(IR_PRA_NAVEGACAO("beta"));
  await PAUSA(500);

  await b.js(`(() => {
    const raiz = document.querySelector('.app-root');
    raiz.focus();
    return true;
  })()`);
  await PAUSA(400);
  ctx.assertEq(
    await b.js(`!!document.querySelector('[data-nav-item]:focus')`),
    false,
    "o teste precisa começar com o foco perdido",
  );

  await b.js(`(() => {
    document.querySelector('.app-root').dispatchEvent(
      new KeyboardEvent('keydown', { key: 'ArrowDown', bubbles: true, cancelable: true }));
    return true;
  })()`);
  await PAUSA(600);
  ctx.assert(
    (await h.estado()).destacado !== null,
    "a seta devia ter reancorado a navegação em vez de não fazer nada",
  );
}, 197);

// ── 6. o que o menu / INSERE ────────────────────────────────────────
//
// Existia cenário provando que o menu `/` ABRE, e nenhum provando o que
// acontece depois de escolher um item. Foi nesse vão que o ciclo 249
// morou: `insert_element_at_cursor` punha o elemento no DOM cru — sem
// `contenteditable`, sem `data-nav-block`, sem `editor__bloco` — e
// mandava o cursor pra DEPOIS dele, ou seja, pro contêiner do segmento,
// que é `contenteditable="false"`. O bloco novo não aceitava tecla e o
// editor inteiro parecia travado, até sair da página e voltar (aí o
// efeito de render remarcava tudo).
//
// Por isso estes cenários não param em "inseriu": eles DIGITAM depois.

/// Abre o menu `/` num bloco novo no fim da página e escolhe um item.
const ESCOLHER_NO_MENU = (rotulo) => `(() => {
  const item = [...document.querySelectorAll('.slash-menu__item')]
    .find(i => (i.querySelector('.slash-menu__item-label') || {}).textContent === ${JSON.stringify(rotulo)});
  if (!item) throw new Error('item "' + ${JSON.stringify(rotulo)} + '" não está no menu');
  item.click();
  return true;
})()`;

/// Onde o cursor está DE VERDADE: num bloco editável, ou em lugar nenhum.
const ONDE_ESTA_O_CURSOR = `(() => {
  const s = getSelection();
  if (!s.anchorNode) return null;
  const el = s.anchorNode.nodeType === 1 ? s.anchorNode : s.anchorNode.parentElement;
  const editavel = el && el.closest('[contenteditable="true"]');
  if (!editavel) return null;
  return { tag: editavel.tagName, bloco: editavel.classList.contains('editor__bloco') };
})()`;

bloco("bloco escolhido no menu / aceita digitação na hora", TRES, async (b, ctx, h) => {
  await b.js(CURSOR_NO_FIM_DO_BLOCO("gama"));
  await b.js(TECLA("Enter", { shiftKey: true }));
  await PAUSA(400);
  await b.js(ESCREVER("/"));
  await esperar(b, "document.querySelector('.slash-menu')", "o menu / abrir");
  await b.js(ESCOLHER_NO_MENU("Título 1"));
  await PAUSA(500);

  // 1. O cursor tem que ter sobrado DENTRO de um bloco editável. Sem
  //    isto, nenhuma tecla chega em lugar nenhum.
  const cursor = await b.js(ONDE_ESTA_O_CURSOR);
  ctx.assert(cursor !== null, "o cursor ficou fora de qualquer bloco editável");
  ctx.assertEq(cursor.tag, "H1", "o cursor devia estar dentro do título inserido");
  ctx.assert(cursor.bloco, "o bloco inserido não recebeu a classe editor__bloco");

  // 2. E digitar tem que entrar no bloco novo, não sumir. O item do
  //    menu já vem com o texto de exemplo ("Título"), então o que se
  //    prova aqui é que o digitado se junta a ELE — não que o bloco
  //    fique só com o texto novo.
  await b.js(ESCREVER("meu título"));
  await PAUSA(400);
  const est = await h.estado();
  ctx.assert(
    est.blocos.some((t) => t.endsWith("meu título")),
    `o texto digitado não entrou no bloco novo: ${JSON.stringify(est.blocos)}`,
  );

  // 3. E chega no disco como título de verdade.
  const md = await h.salvarELer();
  ctx.assert(/^# .*meu título$/m.test(md), `o título não chegou no disco:\n${md}`);
}, 249);

bloco("bloco do menu / entra marcado pra navegação", TRES, async (b, ctx, h) => {
  await b.js(CURSOR_NO_FIM_DO_BLOCO("gama"));
  await b.js(TECLA("Enter", { shiftKey: true }));
  await PAUSA(400);
  await b.js(ESCREVER("/"));
  await esperar(b, "document.querySelector('.slash-menu')", "o menu / abrir");
  await b.js(ESCOLHER_NO_MENU("Citação"));
  await PAUSA(500);

  // O nav-mode só enxerga o que tem `data-nav-block`: um bloco inserido
  // sem a marca é invisível pras setas, mesmo estando na tela.
  const marcados = await b.js(
    `[...document.querySelectorAll('[data-nav-block]')].map(e => e.tagName)`,
  );
  ctx.assert(
    marcados.includes("BLOCKQUOTE"),
    `a citação inserida não entrou na navegação: ${JSON.stringify(marcados)}`,
  );
}, 249);

bloco("embed do menu / não deixa o foco no body", TRES, async (b, ctx, h) => {
  await b.js(CURSOR_NO_FIM_DO_BLOCO("gama"));
  await b.js(TECLA("Enter", { shiftKey: true }));
  await PAUSA(400);
  await b.js(ESCREVER("/"));
  await esperar(b, "document.querySelector('.slash-menu')", "o menu / abrir");
  await b.js(ESCOLHER_NO_MENU("Tabela de Tarefas"));
  // O marcador vira componente Yew num render seguinte, que refaz o DOM
  // do segmento — o pouso do foco é DEPOIS disso (ciclo 195).
  await PAUSA(900);

  const foco = await b.js(
    `(() => { const a = document.activeElement; return a ? a.tagName + '/' + a.className : null; })()`,
  );
  ctx.assert(
    !/^BODY/.test(foco || "BODY"),
    `o foco não pode terminar no body depois de inserir um embed (ficou em ${foco})`,
  );
}, 249);

bloco("a dica de bloco vazio não aparece por cima de texto", TRES, async (b, ctx, h) => {
  // A classe `--convite` só é revista quando `marcar_blocos` roda. Se o
  // CSS pintar a dica sem exigir vazio, basta o bloco marcado ganhar
  // texto por outro caminho pra mensagem ficar impressa por cima da
  // frase do usuário (ciclo 249).
  const dicaEmBlocoComTexto = await b.js(`(() => {
    const alvo = [...document.querySelectorAll('.editor__bloco')]
      .find(x => x.textContent.includes('beta'));
    alvo.classList.add('editor__bloco--convite');
    const antes = getComputedStyle(alvo, '::before').content;
    alvo.classList.remove('editor__bloco--convite');
    return antes;
  })()`);
  ctx.assert(
    !/Digite ou use/.test(dicaEmBlocoComTexto || ""),
    `a dica apareceu num bloco COM texto: ${dicaEmBlocoComTexto}`,
  );
}, 249);
