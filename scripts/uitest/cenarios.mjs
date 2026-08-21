// Cenários do harness. Cada um é uma REGRESSÃO que já aconteceu de
// verdade — o número do ciclo entre parênteses é onde ela apareceu.
//
// Convenção: o cenário cria a página de rascunho pelo disco
// (`ctx.escrever`), abre no app, mexe pelo DOM e confere. Nunca toca em
// página real do vault.

import { esperar } from "./bridge.mjs";

const PAUSA = (ms) => new Promise((r) => setTimeout(r, ms));

/// Recarrega o webview e espera a sidebar voltar — usado quando o teste
/// criou um arquivo novo e precisa que a listagem enxergue.
async function recarregar(bridge) {
  await bridge.js("location.reload(); true");
  await PAUSA(1500);
  await esperar(bridge, "document.querySelectorAll('.sidebar, [class*=sidebar]').length > 0", "a UI recarregar");
  // O arquivo que o teste acabou de escrever faz o watcher acusar
  // mudança, e o editor recarrega a página aberta (ciclo 173). Se o
  // cenário mexer no DOM antes disso, a recarga chega no meio e desfaz
  // o que ele fez — foi assim que o cenário de blocos "falhou" na
  // primeira execução, sem bug nenhum no produto.
  await PAUSA(2500);
}

/// Põe o cursor no fim do primeiro contenteditable e digita.
const DIGITAR = (texto) => `(() => {
  const el = document.querySelector('.editor__bloco[contenteditable="true"]');
  if (!el) return false;
  el.focus();
  const r = document.createRange();
  r.selectNodeContents(el); r.collapse(false);
  const s = getSelection(); s.removeAllRanges(); s.addRange(r);
  document.execCommand('insertText', false, ${JSON.stringify(texto)});
  return true;
})()`;


/// Entra de verdade no modo de NAVEGAÇÃO a partir de um bloco.
///
/// Focar o bloco não basta desde o ciclo 194: focar é o que digitar faz,
/// e os atalhos de bloco passaram a exigir o modo — que vive no estado
/// do `app.rs`. O Escape a partir do texto é o caminho real.
const ENTRAR_EM_NAVEGACAO = (textoDoBloco) => `(() => {
  const alvo = [...document.querySelectorAll('[data-nav-block]')]
    .find(b => b.textContent.includes(${JSON.stringify("__ALVO__")}));
  alvo.focus();
  const r = document.createRange();
  r.selectNodeContents(alvo); r.collapse(false);
  const s = getSelection(); s.removeAllRanges(); s.addRange(r);
  alvo.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true }));
  return true;
})()`.replace("__ALVO__", textoDoBloco);

const SALVAR = `(() => {
  const b = [...document.querySelectorAll('button')].find(b => b.textContent.trim().startsWith('Salvar'));
  if (b) b.click();
  return !!b;
})()`;

export const cenarios = [
  {
    nome: "menu / lista os 9 tipos de embed e insere o escolhido (148)",
    async fn(bridge, ctx) {
      ctx.escrever("---\ntitle: __uitest\n---\n\n");
      await recarregar(bridge);
      await ctx.abrirPagina(bridge, ctx.nomePagina);

      await bridge.js(DIGITAR("/"));
      await ctx.esperar(bridge, "document.querySelectorAll('.slash-menu__item').length > 0", "o menu / abrir");

      const itens = await bridge.js(
        `[...document.querySelectorAll('.slash-menu__item-label')].map(e => e.textContent)`,
      );
      for (const tipo of ["Kanban", "Calendário", "Tabela de Tarefas", "Destaque", "Colunas", "Galeria", "Consulta", "Cronograma", "Ações"]) {
        ctx.assert(itens.includes(tipo), `menu / sem o tipo "${tipo}" (veio: ${itens.join(", ")})`);
      }
      const comIcone = await bridge.js(
        `[...document.querySelectorAll('.slash-menu__item')].every(i => !!i.querySelector('svg.slash-menu__item-icon'))`,
      );
      ctx.assert(comIcone, "algum item do menu / ficou sem ícone");

      await bridge.js(
        `(() => { const i=[...document.querySelectorAll('.slash-menu__item')].find(x=>x.textContent.includes('Destaque')); i.click(); return true; })()`,
      );
      await ctx.esperar(bridge, "document.querySelector('.callout')", "o callout aparecer");
    },
  },

  {
    nome: "callout: editar o corpo e recarregar do disco preserva o markdown (151)",
    async fn(bridge, ctx) {
      ctx.escrever(
        '---\ntitle: __uitest\n---\n{{ type: "callout" }}\nvariant: info\ntitle: Nota\nbody: |\n  Original.\n{{ /callout }}\n',
      );
      await recarregar(bridge);
      await ctx.abrirPagina(bridge, ctx.nomePagina);
      await ctx.esperar(bridge, "document.querySelector('.callout')", "o callout renderizar");

      // Entra em edição e escreve markdown com pontuação que já quebrou
      // serialização montada à mão (ciclo 064).
      await bridge.js(`(() => { document.querySelector('.embed-md').click(); return true; })()`);
      await ctx.esperar(bridge, "document.querySelector('.embed-md__input')", "o campo de edição abrir");
      await bridge.js(`(() => {
        const ta = document.querySelector('.embed-md__input');
        const set = Object.getOwnPropertyDescriptor(HTMLTextAreaElement.prototype, 'value').set;
        set.call(ta, "Editado: com dois pontos\\n\\n- item\\n");
        ta.dispatchEvent(new Event('input', { bubbles: true }));
        ta.blur();
        return true;
      })()`);
      await ctx.esperar(bridge, "!document.querySelector('.embed-md__input')", "sair da edição");
      await bridge.js(SALVAR);
      await PAUSA(800);

      const disco = ctx.ler();
      ctx.assert(disco.includes("Editado: com dois pontos"), `o corpo não foi pro disco:\n${disco}`);
      ctx.assert(disco.includes("- item"), "a lista do corpo se perdeu");
      ctx.assert(disco.includes('{{ type: "callout" }}'), "o wrapper do embed se perdeu");
    },
  },

  {
    nome: "cronograma: arrastar a barra grava a data nova (155)",
    async fn(bridge, ctx) {
      ctx.escrever(
        '---\ntitle: __uitest\n---\n{{ type: "timeline" }}\nscale: month\nitems:\n- title: Etapa\n  start: 2026-08-10\n  end: 2026-08-14\n{{ /timeline }}\n',
      );
      await recarregar(bridge);
      await ctx.abrirPagina(bridge, ctx.nomePagina);
      await ctx.esperar(bridge, "document.querySelector('.timeline__bar-item')", "a barra aparecer");

      // Regressão do ciclo 155: o mouseup lia um handle de use_state
      // congelado e commitava sempre 0 dias.
      const antes = await bridge.js(`document.querySelector('.timeline__bar-item').getAttribute('style')`);
      // O mousedown só marca o estado; os listeners de mousemove/mouseup
      // são registrados no efeito da renderização seguinte. Por isso o
      // arraste vai em DUAS chamadas com pausa no meio — num bloco só,
      // os eventos chegariam antes dos listeners existirem (e o teste
      // acusaria uma regressão que não existe).
      await bridge.js(`(() => {
        const bar = document.querySelector('.timeline__bar-item');
        const r = bar.getBoundingClientRect();
        window.__uitestDrag = { x: Math.round(r.left + r.width / 2), y: Math.round(r.top + r.height / 2),
          dia: document.querySelector('.timeline__track').getBoundingClientRect().width / 35 };
        bar.dispatchEvent(new MouseEvent('mousedown', { bubbles: true, clientX: window.__uitestDrag.x, clientY: window.__uitestDrag.y }));
        return true;
      })()`);
      await PAUSA(300);
      await bridge.js(`(() => {
        const d = window.__uitestDrag;
        const destino = Math.round(d.x + d.dia * 7);
        window.dispatchEvent(new MouseEvent('mousemove', { bubbles: true, clientX: d.x, clientY: d.y }));
        window.dispatchEvent(new MouseEvent('mousemove', { bubbles: true, clientX: destino, clientY: d.y }));
        window.dispatchEvent(new MouseEvent('mouseup', { bubbles: true, clientX: destino, clientY: d.y }));
        return true;
      })()`);
      await PAUSA(700);
      const depois = await bridge.js(`document.querySelector('.timeline__bar-item').getAttribute('style')`);
      ctx.assert(antes !== depois, "a barra não se moveu — o arraste não commitou");

      await bridge.js(SALVAR);
      await PAUSA(800);
      const disco = ctx.ler();
      ctx.assert(!disco.includes("start: 2026-08-10"), `a data velha continua no disco:\n${disco}`);
    },
  },

  {
    nome: "Escape fecha só o popup, sem desselecionar a página (161)",
    async fn(bridge, ctx) {
      ctx.escrever('---\ntitle: __uitest\n---\n{{ type: "query" }}\nview: list\n{{ /query }}\n');
      await recarregar(bridge);
      await ctx.abrirPagina(bridge, ctx.nomePagina);
      await ctx.esperar(bridge, "document.querySelector('.query-embed__btn')", "a consulta renderizar");

      await bridge.js(`(() => { document.querySelector('.query-embed__btn').click(); return true; })()`);
      await ctx.esperar(bridge, "document.querySelector('.query-settings')", "o modal abrir");
      await bridge.js(`(() => {
        const alvo = document.querySelector('.query-settings__input');
        alvo.focus();
        alvo.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }));
        return true;
      })()`);
      await PAUSA(600);

      const estado = await bridge.js(`({
        modal: !!document.querySelector('.modal-overlay'),
        pagina: !!document.querySelector('.query-embed'),
        vazio: document.body.textContent.includes('Selecione uma página na sidebar'),
      })`);
      ctx.assertEq(estado.modal, false, "o modal devia ter fechado");
      ctx.assertEq(estado.pagina, true, "a página não podia ter sido desselecionada junto");
      ctx.assertEq(estado.vazio, false, "caiu no estado vazio");
    },
  },

  {
    nome: "toolbar do embed não cobre nenhum controle do embed (166)",
    async fn(bridge, ctx) {
      ctx.escrever(
        '---\ntitle: __uitest\n---\n{{ type: "timeline" }}\nscale: month\nitems: []\n{{ /timeline }}\n\ntexto\n\n{{ type: "gallery" }}\ncolumns: 3\nsize: md\nitems: []\n{{ /gallery }}\n',
      );
      await recarregar(bridge);
      await ctx.abrirPagina(bridge, ctx.nomePagina);
      await ctx.esperar(bridge, "document.querySelectorAll('.embed-hover-wrapper').length >= 2", "os embeds renderizarem");

      const colisoes = await bridge.js(`(() => {
        const out = [];
        for (const w of document.querySelectorAll('.embed-hover-wrapper')) {
          const tb = w.querySelector('.embed-hover-wrapper__toolbar');
          if (!tb) continue;
          const t = tb.getBoundingClientRect();
          for (const b of w.querySelectorAll('button,input,select,[tabindex="0"]')) {
            if (tb.contains(b)) continue;
            const r = b.getBoundingClientRect();
            if (r.width === 0) continue;
            if (!(r.right < t.left || r.left > t.right || r.bottom < t.top || r.top > t.bottom)) {
              out.push((b.textContent || '').trim().slice(0, 20) || b.title || b.className);
            }
          }
        }
        return out;
      })()`);
      ctx.assertEq(colisoes.length, 0, `a toolbar está cobrindo controle(s): ${colisoes.join(", ")}`);
    },
  },

  {
    nome: "teclado: entra no embed, anda pelos controles e volta pro texto (165)",
    async fn(bridge, ctx) {
      ctx.escrever(
        '---\ntitle: __uitest\n---\ntexto antes\n\n{{ type: "callout" }}\nvariant: info\ntitle: Nota\nbody: |\n  Corpo.\n{{ /callout }}\n',
      );
      await recarregar(bridge);
      await ctx.abrirPagina(bridge, ctx.nomePagina);
      await ctx.esperar(bridge, "document.querySelector('.callout')", "o callout renderizar");

      const tecla = (k, ctrl = false) => `(() => {
        document.querySelector('.app-root').dispatchEvent(
          new KeyboardEvent('keydown', { key: ${JSON.stringify(k)}, ctrlKey: ${ctrl}, bubbles: true }));
        return true;
      })()`;

      await bridge.js(`(() => { document.querySelector('.editor__wysiwyg')?.focus(); return true; })()`);
      await bridge.js(tecla(".", true));
      await PAUSA(400);
      const dentro = await bridge.js(`({
        item: document.activeElement?.getAttribute('data-nav-item'),
        grupo: document.activeElement?.getAttribute('data-nav-parent'),
        destaque: document.activeElement?.classList?.contains('nav-mode__item-active'),
      })`);
      ctx.assert(dentro.grupo?.startsWith("embed-"), `Ctrl+. não entrou no embed (veio ${JSON.stringify(dentro)})`);
      ctx.assert(dentro.destaque, "o item focado não recebeu o indicador visual");

      await bridge.js(tecla("ArrowDown"));
      await PAUSA(300);
      const depois = await bridge.js(`document.activeElement?.getAttribute('data-nav-item')`);
      ctx.assert(depois && depois !== dentro.item, "a seta não andou pro próximo controle");

      // Desde o ciclo 174, Escape sai do embed pro nível dos BLOCOS
      // (com o próprio embed destacado), não direto pro texto — quem
      // entrou pelo teclado continua no teclado, sem perder o lugar.
      await bridge.js(tecla("Escape"));
      await PAUSA(400);
      const saiu = await bridge.js(`({
        grupo: document.activeElement?.getAttribute('data-nav-parent'),
        embed: !!document.activeElement?.getAttribute('data-nav-group')?.startsWith('embed-'),
        pagina: !!document.querySelector('.callout'),
      })`);
      ctx.assertEq(saiu.grupo, "editor-blocos", "Escape devia voltar pro nível dos blocos");
      ctx.assertEq(saiu.embed, true, "o embed de onde saímos devia ficar destacado");
      ctx.assertEq(saiu.pagina, true, "a página não podia fechar");
    },
  },

  {
    nome: "página aberta recarrega quando o arquivo muda no disco (173)",
    async fn(bridge, ctx) {
      ctx.escrever("---\ntitle: __uitest\n---\n\nconteudo original\n");
      await recarregar(bridge);
      await ctx.abrirPagina(bridge, ctx.nomePagina);
      await ctx.esperar(bridge, "document.body.textContent.includes('conteudo original')", "o conteúdo aparecer");

      // Simula o que o `anotadinho-cli` (ou um agente) faz por fora.
      ctx.escrever("---\ntitle: __uitest\n---\n\nescrito por fora\n");
      await ctx.esperar(
        bridge,
        "document.body.textContent.includes('escrito por fora')",
        "a página recarregar sozinha",
        15000,
      );
    },
  },
];

// ── ciclo 174: navegação por blocos ──────────────────────────────────

cenarios.push({
  nome: "blocos: setas andam pelo conteúdo, Enter edita texto e entra no embed (174)",
  async fn(bridge, ctx) {
    ctx.escrever(
      '---\ntitle: __uitest\n---\n# Titulo\n\nParagrafo um.\n\n{{ type: "callout" }}\nvariant: info\ntitle: Nota\nbody: |\n  Corpo.\n{{ /callout }}\n\nParagrafo dois.\n',
    );
    await recarregar(bridge);
    await ctx.abrirPagina(bridge, ctx.nomePagina);
    await ctx.esperar(bridge, "document.querySelector('.callout')", "a página renderizar");

    const blocos = await bridge.js(
      `[...document.querySelectorAll('[data-nav-parent="editor-blocos"]')].map(e => e.tagName.toLowerCase())`,
    );
    ctx.assert(blocos.length >= 4, `esperava título, 2 parágrafos e o embed como blocos; veio ${blocos.join(", ")}`);

    const tecla = (k, ctrl = false) => `(() => {
      document.querySelector('.app-root').dispatchEvent(
        new KeyboardEvent('keydown', { key: ${JSON.stringify(k)}, ctrlKey: ${ctrl}, bubbles: true }));
      return true;
    })()`;

    // Entra no nível de blocos pelo Escape a partir do texto.
    await bridge.js(`(() => {
      const el = document.querySelector('.editor__bloco[contenteditable="true"]');
      el.focus();
      const r = document.createRange(); r.selectNodeContents(el); r.collapse(true);
      const s = getSelection(); s.removeAllRanges(); s.addRange(r);
      el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }));
      return true;
    })()`);
    await PAUSA(500);
    const noBloco = await bridge.js(`({
      destacado: !!document.querySelector('.nav-mode__item-active'),
      item: document.activeElement?.getAttribute('data-nav-item'),
      pagina: !!document.querySelector('.callout'),
    })`);
    ctx.assert(noBloco.destacado, "Escape no texto devia destacar o bloco (e não fechar a página)");
    ctx.assertEq(noBloco.pagina, true, "a página não podia ser desselecionada");

    // Anda até o embed e entra nele.
    let achouEmbed = false;
    for (let i = 0; i < 6 && !achouEmbed; i++) {
      await bridge.js(tecla("ArrowDown"));
      await PAUSA(200);
      achouEmbed = await bridge.js(
        `!!document.activeElement?.getAttribute('data-nav-group')?.startsWith('embed-')`,
      );
    }
    ctx.assert(achouEmbed, "as setas não chegaram no bloco do embed");

    await bridge.js(tecla("Enter"));
    await PAUSA(400);
    const dentro = await bridge.js(`document.activeElement?.getAttribute('data-nav-parent')`);
    ctx.assert(dentro?.startsWith("embed-"), `Enter devia descer pros controles do embed (veio ${dentro})`);

    // Escape volta pro nível de blocos, com o embed destacado.
    await bridge.js(tecla("Escape"));
    await PAUSA(400);
    const voltou = await bridge.js(`document.activeElement?.getAttribute('data-nav-parent')`);
    ctx.assertEq(voltou, "editor-blocos", "Escape devia voltar pro nível dos blocos");
  },
});

// ── ciclo 167: operar kanban e cronograma pelo teclado ───────────────

cenarios.push({
  nome: "teclado: Alt+setas movem card do kanban e barra do cronograma (167)",
  async fn(bridge, ctx) {
    ctx.escrever(
      '---\ntitle: __uitest\n---\n{{ type: "kanban" }}\ncolumns:\n- Todo\n- Done\nitems:\n- title: Card A\n  column: Todo\n{{ /kanban }}\n',
    );
    await recarregar(bridge);
    await ctx.abrirPagina(bridge, ctx.nomePagina);
    await ctx.esperar(bridge, "document.querySelector('.kanban__card')", "o board renderizar");

    // Alt+→ leva o card pra coluna seguinte. Sem Alt a seta navega.
    await bridge.js(`(() => {
      const card = document.querySelector('.kanban__card');
      card.focus();
      card.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowRight', altKey: true, bubbles: true }));
      return true;
    })()`);
    await PAUSA(600);

    const colunas = await bridge.js(`[...document.querySelectorAll('.kanban__column')].map(c => ({
      titulo: c.querySelector('.kanban__col-title')?.textContent,
      cards: [...c.querySelectorAll('.kanban__card-title')].map(t => t.textContent),
    }))`);
    const done = colunas.find((c) => c.titulo === "Done");
    ctx.assert(done?.cards.includes("Card A"), `o card não mudou de coluna: ${JSON.stringify(colunas)}`);

    await bridge.js(SALVAR);
    await PAUSA(800);
    ctx.assert(ctx.ler().includes("column: Done"), "a coluna nova não foi pro disco");
  },
});

// ── ciclo 168: editar propriedade direto na consulta ─────────────────

cenarios.push({
  nome: "consulta: editar o campo na linha grava e some do recorte (168)",
  async fn(bridge, ctx) {
    // Uma spec de teste em backlog + uma consulta que só mostra backlog:
    // ao mudar o status pela linha, a página tem que SAIR da lista.
    const spec = "pages/__uitest-spec.md";
    const { writeFileSync, unlinkSync, existsSync } = await import("node:fs");
    const { join } = await import("node:path");
    const specPath = join(ctx.vault, spec);
    writeFileSync(specPath, "---\ntitle: Spec de teste\nstatus: backlog\n---\n# Spec\n");
    try {
      ctx.escrever(
        '---\ntitle: __uitest\n---\n{{ type: "query" }}\nwhere:\n- field: status\n  op: eq\n  value: backlog\nview: list\ncolumns:\n- status\n{{ /query }}\n',
      );
      await recarregar(bridge);
      await ctx.abrirPagina(bridge, ctx.nomePagina);
      await ctx.esperar(
        bridge,
        "document.body.textContent.includes('Spec de teste')",
        "a spec aparecer no recorte",
      );

      await bridge.js(`(() => {
        const linha = [...document.querySelectorAll('.query-embed__row')]
          .find(r => r.textContent.includes('Spec de teste'));
        linha.querySelector('.query-embed__editavel').click();
        return true;
      })()`);
      await ctx.esperar(bridge, "document.querySelector('.query-embed__editar')", "o campo abrir");
      await bridge.js(`(() => {
        const inp = document.querySelector('.query-embed__editar');
        const set = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value').set;
        inp.focus();
        set.call(inp, 'done');
        inp.dispatchEvent(new Event('input', { bubbles: true }));
        inp.blur();
        return true;
      })()`);

      try {
        await ctx.esperar(
          bridge,
          "![...document.querySelectorAll('.query-embed__row')].some(r => r.textContent.includes('Spec de teste'))",
          "a spec sair do recorte depois de mudar o status",
          12000,
        );
      } catch (e) {
        const msg = await bridge.js(`document.querySelector('.query-embed__erro')?.textContent || null`);
        throw new Error(msg ? `${e.message}\n  o embed reportou: ${msg}` : e.message);
      }
      const conteudo = (await import("node:fs")).readFileSync(specPath, "utf8");
      ctx.assert(conteudo.includes("status: done"), `o status novo não foi pro disco:\n${conteudo}`);
    } finally {
      if (existsSync(specPath)) unlinkSync(specPath);
    }
  },
});

// ── ciclo 169: agrupamento e agregados ───────────────────────────────

cenarios.push({
  nome: "consulta agrupada: cabeçalho por valor, contagem e recolher (169)",
  async fn(bridge, ctx) {
    ctx.escrever(
      '---\ntitle: __uitest\n---\n{{ type: "query" }}\nfrom: pages\ngroup_by: type\naggregate:\n- op: count\nview: list\n{{ /query }}\n',
    );
    await recarregar(bridge);
    await ctx.abrirPagina(bridge, ctx.nomePagina);
    await ctx.esperar(bridge, "document.querySelectorAll('.query-embed__grupo').length > 1", "os grupos aparecerem");

    const grupos = await bridge.js(`[...document.querySelectorAll('.query-embed__grupo')].map(g => ({
      nome: g.querySelector('.query-embed__grupo-nome')?.textContent,
      total: g.querySelector('.query-embed__grupo-total')?.textContent,
    }))`);
    ctx.assert(grupos.length >= 2, `esperava mais de um grupo; veio ${JSON.stringify(grupos)}`);
    ctx.assert(
      grupos.every((g) => Number(g.total) > 0),
      `todo grupo devia ter contagem: ${JSON.stringify(grupos)}`,
    );

    const linhasAntes = await bridge.js(`document.querySelectorAll('.query-embed__row').length`);
    await bridge.js(`(() => { document.querySelector('.query-embed__grupo').click(); return true; })()`);
    await PAUSA(600);
    const linhasDepois = await bridge.js(`document.querySelectorAll('.query-embed__row').length`);
    ctx.assert(linhasDepois < linhasAntes, "recolher o grupo devia esconder as linhas dele");

    // O estado de recolhido é persistido no YAML do embed.
    await bridge.js(SALVAR);
    await PAUSA(800);
    ctx.assert(ctx.ler().includes("collapsed:"), `o recolhido não foi pro disco:\n${ctx.ler()}`);
  },
});

// ── ciclo 170: transclusão ───────────────────────────────────────────

cenarios.push({
  nome: "transclusão: embute a página alvo, a seção pedida e barra o ciclo (170)",
  async fn(bridge, ctx) {
    const { writeFileSync, unlinkSync, existsSync } = await import("node:fs");
    const { join } = await import("node:path");
    const alvo = join(ctx.vault, "pages/__uitest-alvo.md");
    writeFileSync(
      alvo,
      "---\ntitle: Alvo de teste\n---\n# Um\ntexto do um\n\n## Dois\ntexto do dois\n\n## Tres\ntexto do tres\n",
    );
    try {
      ctx.escrever(
        "---\ntitle: __uitest\n---\nantes\n\n![[Alvo de teste]]\n\n![[Alvo de teste#Dois]]\n\n![[__uitest]]\n\n![[Nao Existe]]\n",
      );
      await recarregar(bridge);
      await ctx.abrirPagina(bridge, ctx.nomePagina);
      await ctx.esperar(
        bridge,
        "document.querySelectorAll('.transclusao__corpo, .transclusao__vazia').length >= 4",
        "as transclusões resolverem",
      );

      const blocos = await bridge.js(`[...document.querySelectorAll('.transclusao')].map(t => ({
        origem: t.querySelector('.transclusao__origem')?.textContent || null,
        texto: (t.textContent || '').trim().slice(0, 80),
      }))`);

      ctx.assert(blocos[0].texto.includes("texto do um"), `a página inteira não entrou: ${JSON.stringify(blocos[0])}`);
      ctx.assert(blocos[0].texto.includes("texto do dois"), "faltou o resto da página");

      ctx.assert(blocos[1].texto.includes("texto do dois"), `a seção não entrou: ${JSON.stringify(blocos[1])}`);
      ctx.assert(!blocos[1].texto.includes("texto do tres"), "a seção pegou conteúdo além dela");
      ctx.assert(blocos[1].origem?.includes("Dois"), "o cabeçalho devia dizer qual seção");

      ctx.assert(blocos[2].texto.includes("não pode transcluir ela mesma"), "auto-transclusão devia ser barrada");
      ctx.assert(blocos[3].texto.includes("não existe ainda"), "alvo inexistente devia avisar");

      // O `.md` não muda por transcluir.
      ctx.assert(ctx.ler().includes("![[Alvo de teste]]"), "o markdown original foi alterado");
    } finally {
      if (existsSync(alvo)) unlinkSync(alvo);
    }
  },
});

// ── ciclo 163: configurar botão de ação pela interface ───────────────

cenarios.push({
  nome: "ações: configurar botão pelo modal grava no YAML (163)",
  async fn(bridge, ctx) {
    ctx.escrever(
      '---\ntitle: __uitest\n---\n{{ type: "actions" }}\nlayout: row\nbuttons: []\n{{ /actions }}\n',
    );
    await recarregar(bridge);
    await ctx.abrirPagina(bridge, ctx.nomePagina);
    await ctx.esperar(bridge, "document.querySelector('.actions-embed__add')", "o embed de ações renderizar");

    await bridge.js(`(() => { document.querySelector('.actions-embed__add').click(); return true; })()`);
    await ctx.esperar(bridge, "document.querySelector('.query-settings')", "o modal abrir");

    // Só os campos da ação escolhida aparecem: `open-page` não mostra
    // template/pasta/campo/valor.
    const rotulos = await bridge.js(
      `[...document.querySelectorAll('.query-settings__label')].map(l => l.textContent)`,
    );
    ctx.assert(rotulos.includes("Página"), `faltou o campo da ação escolhida: ${rotulos.join(", ")}`);
    ctx.assert(!rotulos.includes("Template"), `campo de outra ação apareceu: ${rotulos.join(", ")}`);

    await bridge.js(`(() => {
      const label = document.querySelector('.query-settings__input');
      const set = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value').set;
      set.call(label, 'Ir pro alvo');
      label.dispatchEvent(new Event('change', { bubbles: true }));
      return true;
    })()`);
    await PAUSA(500);
    await bridge.js(`(() => {
      const sel = [...document.querySelectorAll('.query-settings__select')].pop();
      const set = Object.getOwnPropertyDescriptor(HTMLSelectElement.prototype, 'value').set;
      set.call(sel, sel.options[1].value);
      sel.dispatchEvent(new Event('change', { bubbles: true }));
      return true;
    })()`);
    await PAUSA(600);

    await bridge.js(`(() => {
      document.querySelector('.modal-overlay .modal')
        .dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }));
      return true;
    })()`);
    await PAUSA(500);

    const botoes = await bridge.js(
      `[...document.querySelectorAll('.actions-embed__btn')].map(b => b.textContent.trim())`,
    );
    ctx.assert(botoes.some((b) => b.includes("Ir pro alvo")), `o botão não foi criado: ${botoes.join(", ")}`);

    await bridge.js(SALVAR);
    await PAUSA(800);
    const disco = ctx.ler();
    ctx.assert(disco.includes("Ir pro alvo"), `o botão não foi pro disco:\n${disco}`);
    ctx.assert(disco.includes("action: open-page"), `a ação não foi pro disco:\n${disco}`);
  },
});

// ── ciclo 176: id de bloco sob demanda ───────────────────────────────

cenarios.push({
  nome: "bloco: copiar referência grava ^id só naquela linha (176)",
  async fn(bridge, ctx) {
    ctx.escrever("---\ntitle: __uitest\n---\nprimeira linha\n\nsegunda linha\n\nterceira linha\n");
    await recarregar(bridge);
    await ctx.abrirPagina(bridge, ctx.nomePagina);
    await ctx.esperar(bridge, "document.querySelectorAll('[data-nav-block]').length >= 3", "os blocos aparecerem");

    // Entra em navegação no segundo bloco e pede a referência com "c".
    await bridge.js(ENTRAR_EM_NAVEGACAO("segunda linha"));
    await ctx.esperar(bridge, "document.querySelector('.editor__modo--navegacao')", "o modo virar NAVEGAÇÃO");
    await bridge.js(`(() => {
      const alvo = document.querySelector('.nav-mode__item-active') || document.activeElement;
      alvo.dispatchEvent(new KeyboardEvent('keydown', { key: 'c', bubbles: true, cancelable: true }));
      return true;
    })()`);
    await PAUSA(700);
    await bridge.js(SALVAR);
    await PAUSA(900);

    const disco = ctx.ler();
    ctx.assert(disco && disco.trim().length > 0, `o arquivo ficou vazio depois de salvar: ${JSON.stringify(disco)}`);
    const linhas = disco.split("\n");
    const comId = linhas.filter((l) => /\s\^[a-z0-9-]+$/.test(l));
    ctx.assertEq(comId.length, 1, `só a linha referenciada podia ganhar id:\n${disco}`);
    ctx.assert(comId[0].includes("segunda linha"), `o id foi pra linha errada:\n${disco}`);

    // O id é metadado: fica no DOM (senão sumiria do arquivo no próximo
    // salvamento, que recompõe o markdown a partir dele), mas sempre
    // dentro de `.bloco-id`, que o CSS deixa discreto — nunca solto no
    // meio do texto.
    const marcas = await bridge.js(`(() => {
      const editor = document.querySelector('.editor__wysiwyg');
      const dentroDeSpan = [...editor.querySelectorAll('.bloco-id')].map(s => s.textContent).join('');
      const todoTexto = editor.textContent || '';
      const forasoltos = todoTexto.split('^').length - 1 - (dentroDeSpan.split('^').length - 1);
      return { emSpan: dentroDeSpan, soltos: forasoltos };
    })()`);
    ctx.assert(marcas.emSpan.includes("^"), "o id devia estar num .bloco-id");
    ctx.assertEq(marcas.soltos, 0, "nenhum ^ pode ficar solto no texto");

    // Pedir de novo não gera id novo (idempotente).
    await bridge.js(`(() => {
      const alvo = [...document.querySelectorAll('[data-nav-block]')].find(b => b.textContent.includes('segunda linha'));
      alvo.focus();
      alvo.dispatchEvent(new KeyboardEvent('keydown', { key: 'c', bubbles: true }));
      return true;
    })()`);
    await PAUSA(700);
    await bridge.js(SALVAR);
    await PAUSA(900);
    const depois = ctx.ler();
    ctx.assertEq(
      depois.split("\n").filter((l) => /\s\^[a-z0-9-]+$/.test(l)).length,
      1,
      `pedir a referência duas vezes duplicou id:\n${depois}`,
    );
  },
});

// ── ciclo 178: destaque das regiões no nav-mode ──────────────────────

cenarios.push({
  nome: "nav-mode: as 4 regiões de topo mostram destaque visível (178)",
  async fn(bridge, ctx) {
    ctx.escrever("---\ntitle: __uitest\n---\n\nconteudo\n");
    await recarregar(bridge);
    await ctx.abrirPagina(bridge, ctx.nomePagina);

    // Entra no nav-mode pelo nível das regiões.
    await bridge.js(`(() => {
      const root = document.querySelector('.app-root');
      root.focus();
      root.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowDown', bubbles: true }));
      return true;
    })()`);
    await PAUSA(500);

    // Percorre as regiões e mede o destaque de cada uma. O editor era a
    // que não mostrava nada: os filhos dele têm fundo opaco e cobriam o
    // `background` e o `box-shadow: inset`.
    const vistas = {};
    for (let i = 0; i < 6; i++) {
      const atual = await bridge.js(`(() => {
        const el = document.querySelector('.nav-mode__item-active');
        if (!el) return null;
        const depois = getComputedStyle(el, '::after');
        const propria = getComputedStyle(el);
        return {
          item: el.getAttribute('data-nav-item'),
          raiz: el.getAttribute('data-nav-parent') === 'root',
          overlay: parseFloat(depois.borderTopWidth) || 0,
          sombra: propria.boxShadow !== 'none',
        };
      })()`);
      if (atual?.item) vistas[atual.item] = atual;
      await bridge.js(`(() => {
        document.querySelector('.app-root')
          .dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowDown', bubbles: true }));
        return true;
      })()`);
      await PAUSA(300);
    }

    for (const regiao of ["header", "sidebar", "tabbar", "editor"]) {
      const v = vistas[regiao];
      ctx.assert(v, `a região "${regiao}" não foi alcançada: ${Object.keys(vistas).join(", ")}`);
      ctx.assert(
        v.overlay > 0,
        `a região "${regiao}" ficou sem destaque desenhado por cima (overlay=${v.overlay})`,
      );
    }
  },
});

// ── ciclo 179: foco volta ao fechar o modal ──────────────────────────

cenarios.push({
  nome: "nav-mode: fechar modal devolve o foco e as setas voltam a andar (179)",
  async fn(bridge, ctx) {
    ctx.escrever(
      '---\ntitle: __uitest\n---\n{{ type: "actions" }}\nlayout: row\nbuttons:\n- label: Abrir\n  action: open-page\n  path: pages/__uitest.md\n{{ /actions }}\n',
    );
    await recarregar(bridge);
    await ctx.abrirPagina(bridge, ctx.nomePagina);
    await ctx.esperar(bridge, "document.querySelector('.actions-embed__add')", "o embed renderizar");

    const tecla = (k, ctrl = false) => `(() => {
      document.querySelector('.app-root').dispatchEvent(
        new KeyboardEvent('keydown', { key: ${JSON.stringify(k)}, ctrlKey: ${ctrl}, bubbles: true }));
      return true;
    })()`;

    await bridge.js(`(() => { document.querySelector('.editor__wysiwyg')?.focus(); return true; })()`);
    await bridge.js(tecla(".", true));
    await PAUSA(500);

    // Anda até o "+ ação" e abre o modal com Enter.
    for (let i = 0; i < 10; i++) {
      const atual = await bridge.js(`document.activeElement?.getAttribute('data-nav-item')`);
      if (atual === "actions-add") break;
      await bridge.js(tecla("ArrowRight"));
      await PAUSA(200);
    }
    const antes = await bridge.js(`document.activeElement?.getAttribute('data-nav-item')`);
    ctx.assertEq(antes, "actions-add", "não cheguei no botão de adicionar ação");

    await bridge.js(tecla("Enter"));
    await ctx.esperar(bridge, "document.querySelector('.modal-overlay')", "o modal abrir");

    await bridge.js(`(() => {
      document.querySelector('.modal-overlay .modal')
        .dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }));
      return true;
    })()`);
    await PAUSA(600);

    // O bug: o foco caía num <div> sem data-nav-item e o nav-mode
    // ficava mudo dali em diante.
    const depois = await bridge.js(`document.activeElement?.getAttribute('data-nav-item')`);
    ctx.assertEq(depois, "actions-add", "o foco não voltou pro botão que abriu o modal");

    await bridge.js(tecla("ArrowLeft"));
    await PAUSA(400);
    const andou = await bridge.js(`document.activeElement?.getAttribute('data-nav-item')`);
    ctx.assert(andou && andou !== "actions-add", `as setas não voltaram a andar (parou em ${andou})`);
  },
});

// ── ciclo 180: barra de título própria ───────────────────────────────

cenarios.push({
  nome: "janela: controles próprios no header e faixas de resize (180)",
  async fn(bridge, ctx) {
    ctx.escrever("---\ntitle: __uitest\n---\n\nconteudo\n");
    await recarregar(bridge);

    const chrome = await bridge.js(`({
      controles: [...document.querySelectorAll('.window-controls__btn')].map(b => b.title),
      faixas: document.querySelectorAll('.window-resize').length,
      arrasto: !!document.querySelector('header[data-tauri-drag-region]'),
    })`);
    ctx.assertEq(chrome.controles.length, 3, `esperava minimizar/maximizar/fechar: ${chrome.controles}`);
    ctx.assert(chrome.controles.includes("Fechar"), `faltou o botão de fechar: ${chrome.controles}`);
    ctx.assertEq(chrome.faixas, 8, "faltam faixas de redimensionar (8 direções)");
    ctx.assert(chrome.arrasto, "o header precisa ser área de arraste da janela");

    // A permissão é o que realmente faz o arraste funcionar: o conjunto
    // `core:default` do Tauri 2 NÃO inclui `allow-start-dragging`, então
    // o atributo existia e o arraste era negado em silêncio.
    const permissao = await bridge.js(`(async () => {
      try {
        await window.__TAURI_INTERNALS__.invoke('plugin:window|start_dragging', {});
        return { permitido: true };
      } catch (e) {
        return { permitido: false, erro: String(e) };
      }
    })()`);
    ctx.assert(
      permissao.permitido,
      `start_dragging está bloqueado — falta a permissão na capability: ${permissao.erro}`,
    );

    // O arraste tem que pegar na maior parte do header, não só num vão
    // entre dois botões.
    const cobertura = await bridge.js(`(() => {
      const h = document.querySelector('.header-bar');
      const r = h.getBoundingClientRect();
      const y = Math.round(r.top + r.height / 2);
      let arrastaveis = 0, total = 0;
      for (let frac = 0.02; frac < 1; frac += 0.04) {
        const el = document.elementFromPoint(Math.round(r.left + r.width * frac), y);
        total++;
        if (el && el.hasAttribute && el.hasAttribute('data-tauri-drag-region')) arrastaveis++;
      }
      return { arrastaveis, total };
    })()`);
    ctx.assert(
      cobertura.arrastaveis / cobertura.total > 0.5,
      `pouca área de arraste no header: ${cobertura.arrastaveis}/${cobertura.total}`,
    );

    // Maximizar e restaurar de verdade, pelo comando que o botão chama.
    // O comando volta assim que PEDE ao gerenciador de janelas; o estado
    // real só chega um quadro depois. Conferir na hora dá falso negativo
    // intermitente — daí o polling curto.
    const ciclo = await bridge.js(`(async () => {
      const eh = () => window.__TAURI_INTERNALS__.invoke('window_is_maximized', {});
      const aguardar = async (esperado) => {
        for (let i = 0; i < 30; i++) {
          if (await eh() === esperado) return esperado;
          await new Promise((r) => setTimeout(r, 50));
        }
        return await eh();
      };
      const inicio = await eh();
      const depois = await window.__TAURI_INTERNALS__.invoke('window_toggle_maximize', {});
      const conferido = await aguardar(depois);
      await window.__TAURI_INTERNALS__.invoke('window_toggle_maximize', {});
      const final = await aguardar(inicio);
      return { inicio, depois, conferido, final };
    })()`);
    ctx.assertEq(ciclo.depois, !ciclo.inicio, "alternar devia inverter o estado");
    ctx.assertEq(ciclo.conferido, ciclo.depois, "o estado devolvido tem que bater com o real");
    ctx.assertEq(ciclo.final, ciclo.inicio, "a janela devia ter voltado como estava");

    // O ícone do botão acompanha o estado.
    const icone = await bridge.js(`(() => {
      const b = [...document.querySelectorAll('.window-controls__btn')][1];
      return b.title;
    })()`);
    ctx.assert(["Maximizar", "Restaurar"].includes(icone), `título inesperado no botão: ${icone}`);
  },
});

// ── ciclo 181: novo bloco pelo teclado ───────────────────────────────

cenarios.push({
  nome: "blocos: 'n' abre bloco novo com o menu / pronto (181)",
  async fn(bridge, ctx) {
    ctx.escrever("---\ntitle: __uitest\n---\nprimeira linha\n\nsegunda linha\n");
    await recarregar(bridge);
    await ctx.abrirPagina(bridge, ctx.nomePagina);
    await ctx.esperar(bridge, "document.querySelectorAll('[data-nav-block]').length >= 2", "os blocos aparecerem");

    // Entra no modo de navegação e pede um bloco novo.
    await bridge.js(ENTRAR_EM_NAVEGACAO("primeira"));
    await ctx.esperar(bridge, "document.querySelector('.editor__modo--navegacao')", "o modo virar NAVEGAÇÃO");
    await bridge.js(`(() => {
      const alvo = document.querySelector('.nav-mode__item-active') || document.activeElement;
      alvo.dispatchEvent(new KeyboardEvent('keydown', { key: 'n', bubbles: true, cancelable: true }));
      return true;
    })()`);
    await ctx.esperar(bridge, "document.querySelector('.slash-menu')", "o menu / abrir junto");

    // Ciclo 185: idem pro bloco de texto — foi aqui que o retângulo azul
    // apareceu preso na captura do usuário.
    ctx.assertEq(
      await bridge.js(`document.querySelectorAll('.nav-mode__item-active').length`),
      0,
      "o destaque do nav-mode devia ter saído ao entrar em digitação",
    );

    // O menu traz tudo que o `/` traz — inclusive os embeds.
    const itens = await bridge.js(
      `[...document.querySelectorAll('.slash-menu__item-label')].map(e => e.textContent)`,
    );
    ctx.assert(itens.includes("Kanban"), `o menu devia ter os embeds: ${itens.join(", ")}`);
    ctx.assert(itens.includes("Título 1"), `o menu devia ter os blocos de markdown: ${itens.join(", ")}`);

    // Escolher um item insere de verdade no bloco novo, sem tocar no
    // que já existia.
    await bridge.js(`(() => {
      const item = [...document.querySelectorAll('.slash-menu__item')].find(i => i.textContent.includes('Destaque'));
      item.click();
      return true;
    })()`);
    await ctx.esperar(bridge, "document.querySelector('.callout')", "o embed escolhido aparecer");

    await bridge.js(SALVAR);
    await PAUSA(900);
    const disco = ctx.ler();
    ctx.assert(disco.includes("primeira linha"), `o texto anterior se perdeu:\n${disco}`);
    ctx.assert(disco.includes("segunda linha"), `o texto seguinte se perdeu:\n${disco}`);
    ctx.assert(disco.includes('{{ type: "callout" }}'), `o embed novo não foi pro disco:\n${disco}`);
  },
});

// ── ciclo 184: 'n' sobre embed e Escape cancelando ───────────────────

cenarios.push({
  nome: "blocos: 'n' funciona sobre embed e Escape cancela sem deixar '/' (184)",
  async fn(bridge, ctx) {
    ctx.escrever(
      '---\ntitle: __uitest\n---\ntexto antes\n\n{{ type: "callout" }}\nvariant: info\ntitle: Nota\nbody: |\n  Corpo.\n{{ /callout }}\n',
    );
    await recarregar(bridge);
    await ctx.abrirPagina(bridge, ctx.nomePagina);
    await ctx.esperar(bridge, "document.querySelector('.callout')", "o embed renderizar");

    // 1) 'n' com um controle do EMBED focado abre bloco novo depois dele.
    await bridge.js(`(() => {
      const alvo = document.querySelector('[data-nav-parent^="embed-"]');
      alvo.focus();
      alvo.dispatchEvent(new KeyboardEvent('keydown', { key: 'n', bubbles: true }));
      return true;
    })()`);
    await ctx.esperar(bridge, "document.querySelector('.slash-menu')", "o menu / abrir a partir do embed");

    // Ciclo 185: abrir o bloco novo ENCERRA a sessão de nav-mode — o
    // destaque azul não pode ficar aceso no bloco de origem.
    ctx.assertEq(
      await bridge.js(`document.querySelectorAll('.nav-mode__item-active').length`),
      0,
      "o destaque do nav-mode devia ter saído ao entrar em digitação",
    );

    // 2) Escape cancela: fecha o menu E não deixa o "/" no texto.
    await bridge.js(`(() => {
      const alvo = document.activeElement.closest('.editor__wysiwyg') || document.querySelector('.editor__wysiwyg');
      alvo.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }));
      return true;
    })()`);
    await PAUSA(700);
    ctx.assertEq(
      await bridge.js(`!!document.querySelector('.slash-menu')`),
      false,
      "o menu devia ter fechado",
    );

    await bridge.js(SALVAR);
    await PAUSA(900);
    const disco = ctx.ler();
    ctx.assert(!/^\/\s*$/m.test(disco), `sobrou um "/" solto no arquivo:\n${disco}`);
    ctx.assert(disco.includes("texto antes"), `o texto se perdeu:\n${disco}`);
    ctx.assert(disco.includes('{{ type: "callout" }}'), `o embed se perdeu:\n${disco}`);
  },
});

// ── ciclo 186: desfazer que entende de blocos ────────────────────────

cenarios.push({
  nome: "desfazer: inserir embed logo depois de digitar volta um passo só (186)",
  async fn(bridge, ctx) {
    ctx.escrever("---\ntitle: __uitest\n---\nlinha base\n");
    await recarregar(bridge);
    await ctx.abrirPagina(bridge, ctx.nomePagina);
    await ctx.esperar(bridge, "document.querySelector('.editor__wysiwyg')", "o editor abrir");

    // Digita e, SEM esperar a janela de agrupamento fechar, insere um
    // embed. Era aqui que o estado pré-inserção sumia do histórico.
    await bridge.js(`(() => {
      const seg = document.querySelector('.editor__wysiwyg');
      const ed = seg.lastElementChild || seg;
      ed.focus();
      const r = document.createRange();
      r.selectNodeContents(ed);
      r.collapse(false);
      const s = getSelection(); s.removeAllRanges(); s.addRange(r);
      document.execCommand('insertText', false, ' editado');
      return true;
    })()`);
    await PAUSA(200);

    // Parágrafo novo + "/": o menu só abre com a barra no começo de uma
    // linha, igual ao uso real.
    await bridge.js(`(() => {
      document.execCommand('insertParagraph', false);
      document.execCommand('insertText', false, '/');
      return true;
    })()`);
    await ctx.esperar(bridge, "document.querySelector('.slash-menu')", "o menu / abrir");
    await bridge.js(`(() => {
      const item = [...document.querySelectorAll('.slash-menu__item')].find(i => i.textContent.includes('Destaque'));
      item.click();
      return true;
    })()`);
    await ctx.esperar(bridge, "document.querySelector('.callout')", "o embed aparecer");

    // Ctrl+Z: tem que tirar o embed e MANTER o texto digitado.
    await bridge.js(`(() => {
      document.querySelector('.editor__bloco[contenteditable="true"]')
        .dispatchEvent(new KeyboardEvent('keydown', { key: 'z', ctrlKey: true, bubbles: true }));
      return true;
    })()`);
    await PAUSA(900);
    ctx.assertEq(
      await bridge.js(`!!document.querySelector('.callout')`),
      false,
      "o embed devia ter sumido com o desfazer",
    );

    await bridge.js(SALVAR);
    await PAUSA(900);
    const disco = ctx.ler();
    ctx.assert(!disco.includes('{{ type: "callout" }}'), `o embed voltou pro disco:\n${disco}`);
    ctx.assert(
      disco.includes("linha base editado"),
      `desfazer comeu a digitação junto — devia ter voltado UM passo só:\n${disco}`,
    );
  },
});

// ── ciclo 188: busca que enxerga dentro dos embeds ───────────────────

cenarios.push({
  nome: "busca: resultado de dentro de embed diz o tipo e leva até ele (188)",
  async fn(bridge, ctx) {
    ctx.escrever(
      '---\ntitle: __uitest\n---\ntexto solto sem o termo\n\n' +
        '{{ type: "kanban" }}\ncolumns:\n- Backlog\n- Feito\nitems:\n' +
        '- title: Zarabatana singular\n  column: Feito\n{{ /kanban }}\n',
    );
    await recarregar(bridge);
    await ctx.abrirPagina(bridge, ctx.nomePagina);
    await ctx.esperar(bridge, "document.querySelector('.embed-kanban')", "o kanban renderizar");

    // Busca pela sidebar por um termo que SÓ existe dentro do embed.
    await bridge.js(`(() => {
      const campo = document.querySelector('input[placeholder*="Buscar"]');
      const setter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value').set;
      campo.focus();
      setter.call(campo, 'Zarabatana');
      // InputEvent, e nao Event: o handler do Yew e Callback<InputEvent>
      // e um Event simples nao chega nele.
      campo.dispatchEvent(new InputEvent('input', { bubbles: true }));
      return true;
    })()`);
    await ctx.esperar(
      bridge,
      "document.querySelector('.sidebar-item__origem')",
      "o resultado mostrar de que embed veio",
    );

    const origem = await bridge.js(`document.querySelector('.sidebar-item__origem').textContent`);
    ctx.assert(
      origem.includes("Kanban") && origem.includes("Feito"),
      `a origem devia dizer o tipo e a coluna, veio: ${origem}`,
    );

    // Clicar leva até o embed e destaca.
    await bridge.js(`(() => {
      document.querySelector('.sidebar-item__origem').closest('.sidebar-item').click();
      return true;
    })()`);
    await ctx.esperar(
      bridge,
      "document.querySelector('.busca-alvo')",
      "o embed do resultado ser destacado",
    );
    ctx.assertEq(
      await bridge.js(`document.querySelector('.busca-alvo').classList.contains('embed-kanban')`),
      true,
      "o destaque devia estar no kanban",
    );
  },
});

// ── ciclo 190: aviso de mudança externa ──────────────────────────────

/// Digita no fim do editor, deixando a página com edição pendente.
const DIGITAR_NO_FIM = (texto) => `(() => {
  const seg = document.querySelector('.editor__wysiwyg');
  const ed = seg.lastElementChild || seg;
  ed.focus();
  const r = document.createRange();
  r.selectNodeContents(ed);
  r.collapse(false);
  const s = getSelection(); s.removeAllRanges(); s.addRange(r);
  document.execCommand('insertText', false, ${JSON.stringify(texto)});
  return true;
})()`;

cenarios.push({
  nome: "conflito: barra de decisão com edição pendente, e o diff mostra os dois lados (190)",
  async fn(bridge, ctx) {
    ctx.escrever("---\ntitle: __uitest\n---\nlinha um\nlinha dois\nlinha tres\n");
    await recarregar(bridge);
    await ctx.abrirPagina(bridge, ctx.nomePagina);
    await ctx.esperar(bridge, "document.querySelector('.editor__wysiwyg')", "o editor abrir");

    await bridge.js(DIGITAR_NO_FIM(" MEU TEXTO"));
    await ctx.esperar(bridge, "document.querySelector('.editor__dirty')", "a página ficar suja");

    // Alguém escreve no arquivo por fora — é o que o CLI faz.
    ctx.escrever("---\ntitle: __uitest\n---\nlinha um\nlinha dois DO DISCO\nlinha tres\n");
    await ctx.esperar(bridge, "document.querySelector('.conflito')", "a barra de conflito aparecer", 12000);

    const botoes = await bridge.js(
      `[...document.querySelectorAll('.conflito button')].map(b => b.textContent)`,
    );
    ctx.assertEq(botoes.length, 3, `a barra devia ter 3 ações, veio: ${botoes.join(" | ")}`);

    // O diff tem que mostrar OS DOIS lados — foi o bug achado na
    // validação: comparando `content_md` o texto digitado não aparecia.
    await bridge.js(`(() => { document.querySelector('.conflito button').click(); return true; })()`);
    await ctx.esperar(bridge, "document.querySelector('.conflito__l')", "o diff abrir");
    const linhas = await bridge.js(
      `[...document.querySelectorAll('.conflito__l')].map(e => e.textContent)`,
    );
    ctx.assert(
      linhas.some((l) => l.startsWith("-") && l.includes("MEU TEXTO")),
      `o diff devia mostrar o que EU escrevi: ${linhas.join(" | ")}`,
    );
    ctx.assert(
      linhas.some((l) => l.startsWith("+") && l.includes("DO DISCO")),
      `o diff devia mostrar o que veio do disco: ${linhas.join(" | ")}`,
    );

    // Recarregar traz o disco e limpa o estado de edição.
    await bridge.js(`(() => {
      [...document.querySelectorAll('.conflito button')]
        .find(b => b.textContent.includes('Recarregar')).click();
      return true;
    })()`);
    await PAUSA(1200);
    ctx.assertEq(await bridge.js(`!!document.querySelector('.conflito')`), false, "a barra devia sumir");
    ctx.assertEq(await bridge.js(`!!document.querySelector('.editor__dirty')`), false, "não devia sobrar edição pendente");
    const texto = await bridge.js(`document.querySelector('.editor__wysiwyg').innerText`);
    ctx.assert(texto.includes("DO DISCO"), `devia ter trazido o disco: ${texto}`);
    ctx.assert(!texto.includes("MEU TEXTO"), `devia ter descartado o meu: ${texto}`);
  },
});

cenarios.push({
  nome: "conflito: sem edição pendente recarrega sozinho, sem barra (190)",
  async fn(bridge, ctx) {
    ctx.escrever("---\ntitle: __uitest\n---\nantes\n");
    await recarregar(bridge);
    await ctx.abrirPagina(bridge, ctx.nomePagina);
    await ctx.esperar(bridge, "document.querySelector('.editor__wysiwyg')", "o editor abrir");

    ctx.escrever("---\ntitle: __uitest\n---\ndepois, vindo de fora\n");
    await ctx.esperar(
      bridge,
      "document.querySelector('.editor__wysiwyg').innerText.includes('vindo de fora')",
      "a página recarregar sozinha",
      12000,
    );
    ctx.assertEq(
      await bridge.js(`!!document.querySelector('.conflito')`),
      false,
      "sem edição pendente não deve pedir decisão nenhuma",
    );
  },
});

// ── ciclo 191: wikilink em código inline e clique por título ─────────

cenarios.push({
  nome: "wikilink: exemplo em código inline não vira link, e clique resolve por título do frontmatter (191)",
  async fn(bridge, ctx) {
    // `grafo.md` tem `title: Grafo do Vault` no frontmatter e nome de
    // ARQUIVO diferente — era exatamente o caso que não abria.
    ctx.escrever(
      "---\ntitle: __uitest\n---\n\n" +
        "A sintaxe `[[Página]]` cria um link.\n\n" +
        "Este é de verdade: [[Grafo do Vault]]\n",
    );
    await recarregar(bridge);
    await ctx.abrirPagina(bridge, ctx.nomePagina);
    await ctx.esperar(bridge, "document.querySelector('.editor__wysiwyg a')", "o link renderizar");

    const texto = await bridge.js(`document.querySelector('.editor__wysiwyg').innerText`);
    ctx.assert(
      texto.includes("[[Página]]"),
      `o exemplo em código inline devia continuar literal: ${texto}`,
    );
    ctx.assert(!texto.includes("anotadinho://"), `a URL interna vazou pra tela: ${texto}`);

    const links = await bridge.js(
      `[...document.querySelectorAll('.editor__wysiwyg a')].map(a => a.textContent)`,
    );
    ctx.assertEq(links.length, 1, `devia haver 1 link só, veio: ${links.join(" | ")}`);

    // Clicar abre a página cujo TÍTULO casa, mesmo com nome de arquivo
    // diferente.
    await bridge.js(`(() => { document.querySelector('.editor__wysiwyg a').click(); return true; })()`);
    await ctx.esperar(
      bridge,
      `(document.querySelector('.editor__title')||{}).textContent === 'Grafo do Vault'`,
      "a página alvo abrir",
    );
  },
});

// ── ciclo 192: alias e barra literal no wikilink ─────────────────────

cenarios.push({
  nome: "wikilink: alias mostra o texto, aponta pro alvo e sobrevive à gravação (192)",
  async fn(bridge, ctx) {
    ctx.escrever(
      "---\ntitle: __uitest\n---\n\n" +
        "com alias: [[pages/produto/grafo.md|o grafo do vault]]\n\n" +
        "sem alias: [[Missão]]\n",
    );
    await recarregar(bridge);
    await ctx.abrirPagina(bridge, ctx.nomePagina);
    await ctx.esperar(bridge, "document.querySelector('.editor__wysiwyg a')", "os links renderizarem");

    const links = await bridge.js(
      `[...document.querySelectorAll('.editor__wysiwyg a')].map(a => a.textContent)`,
    );
    ctx.assertEq(links[0], "o grafo do vault", `o texto devia ser o alias, veio: ${links[0]}`);
    ctx.assertEq(links[1], "Missão", `sem alias o texto é o alvo, veio: ${links[1]}`);

    // Gravar NÃO pode perder o alvo — era o que acontecia antes: a
    // reconstrução usava só o texto visível e sobrava [[o grafo do vault]].
    await bridge.js(SALVAR);
    await PAUSA(900);
    const disco = ctx.ler();
    ctx.assert(
      disco.includes("[[pages/produto/grafo.md|o grafo do vault]]"),
      `o alias não sobreviveu à gravação:\n${disco}`,
    );
    ctx.assert(disco.includes("[[Missão]]"), `o link sem alias mudou:\n${disco}`);

    // E clicar leva pro alvo, não pro texto.
    await bridge.js(`(() => { document.querySelector('.editor__wysiwyg a').click(); return true; })()`);
    await ctx.esperar(
      bridge,
      `(document.querySelector('.editor__title')||{}).textContent === 'Grafo do Vault'`,
      "o alvo do alias abrir",
    );
  },
});

cenarios.push({
  nome: "wikilink: arquivo com barra no nome abre escapado e também sem escape (192)",
  async fn(bridge, ctx) {
    // `|` é nome de arquivo válido no POSIX — este é o guardrail.
    const estranho = `${ctx.vault}/pages/com|barra.md`;
    const fs = await import("node:fs");
    fs.writeFileSync(estranho, "---\ntitle: Com Barra\n---\nsou o arquivo esquisito\n");
    try {
      ctx.escrever(
        "---\ntitle: __uitest\n---\n\n" +
          "escapado: [[com\\|barra]]\n\n" +
          "sem escape: [[com|barra]]\n",
      );
      await recarregar(bridge);
      await ctx.abrirPagina(bridge, ctx.nomePagina);
      await ctx.esperar(bridge, "document.querySelector('.editor__wysiwyg a')", "os links renderizarem");

      const textos = await bridge.js(
        `[...document.querySelectorAll('.editor__wysiwyg a')].map(a => a.textContent)`,
      );
      ctx.assertEq(textos[0], "com|barra", `escapado devia exibir o nome inteiro, veio: ${textos[0]}`);
      ctx.assertEq(textos[1], "barra", `sem escape a barra vira alias, veio: ${textos[1]}`);

      // O escapado abre.
      await bridge.js(`(() => { document.querySelectorAll('.editor__wysiwyg a')[0].click(); return true; })()`);
      await ctx.esperar(
        bridge,
        `(document.querySelector('.editor__title')||{}).textContent === 'Com Barra'`,
        "o alvo escapado abrir",
      );

      // E o SEM escape também, pela rede de segurança: o alvo "com" não
      // existe, então a string inteira é tentada antes de desistir.
      await ctx.abrirPagina(bridge, ctx.nomePagina);
      await ctx.esperar(bridge, "document.querySelector('.editor__wysiwyg a')", "voltar pra página de teste");
      await bridge.js(`(() => { document.querySelectorAll('.editor__wysiwyg a')[1].click(); return true; })()`);
      await ctx.esperar(
        bridge,
        `(document.querySelector('.editor__title')||{}).textContent === 'Com Barra'`,
        "a rede de segurança abrir o arquivo mesmo sem escape",
      );
    } finally {
      fs.rmSync(estranho, { force: true });
    }
  },
});

// ── ciclo 175: manipular bloco pelo teclado ──────────────────────────

/// Entra no modo de navegação de blocos com o primeiro bloco focado.
const FOCAR_PRIMEIRO_BLOCO = `(() => {
  const seg = document.querySelector('.editor__wysiwyg');
  const b = seg.children[0];
  b.focus();
  const ed = b;
  const r = document.createRange();
  r.selectNodeContents(b); r.collapse(false);
  const s = getSelection(); s.removeAllRanges(); s.addRange(r);
  ed.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }));
  return true;
})()`;

const TECLA_NO_BLOCO = (key, alt = false) => `(() => {
  const alvo = document.querySelector('.nav-mode__item-active') || document.activeElement;
  alvo.dispatchEvent(new KeyboardEvent('keydown', { key: ${JSON.stringify(key)}, altKey: ${alt}, bubbles: true }));
  return true;
})()`;

cenarios.push({
  nome: "blocos: mover, duplicar e apagar pelo teclado (175)",
  async fn(bridge, ctx) {
    ctx.escrever("---\ntitle: __uitest\n---\nalfa\n\nbeta\n\ngama\n");
    await recarregar(bridge);
    await ctx.abrirPagina(bridge, ctx.nomePagina);
    await ctx.esperar(bridge, "document.querySelector('.editor__wysiwyg')", "o editor abrir");

    const linhas = () =>
      bridge.js(`[...document.querySelectorAll('[data-nav-block]')].map(e => e.textContent.trim())`);

    await bridge.js(FOCAR_PRIMEIRO_BLOCO);
    await ctx.esperar(bridge, "document.querySelector('.nav-mode__item-active')", "o bloco ficar destacado");
    ctx.assertEq((await linhas()).join(","), "alfa,beta,gama", "estado inicial");

    // Desce o primeiro: alfa vai pro meio.
    await bridge.js(TECLA_NO_BLOCO("ArrowDown", true));
    await PAUSA(400);
    ctx.assertEq((await linhas()).join(","), "beta,alfa,gama", "Alt+↓ devia trocar com o de baixo");

    // Sobe de volta.
    await bridge.js(TECLA_NO_BLOCO("K"));
    await PAUSA(400);
    ctx.assertEq((await linhas()).join(","), "alfa,beta,gama", "K devia devolver pro topo");

    // Duplica.
    await bridge.js(TECLA_NO_BLOCO("y"));
    await PAUSA(400);
    ctx.assertEq((await linhas()).join(","), "alfa,alfa,beta,gama", "y devia duplicar logo abaixo");

    // Apaga a cópia (o foco ficou nela).
    await bridge.js(TECLA_NO_BLOCO("d"));
    await PAUSA(400);
    ctx.assertEq((await linhas()).join(","), "alfa,beta,gama", "d devia apagar o bloco focado");

    // E o foco continua num bloco, pra dar pra encadear.
    ctx.assertEq(
      await bridge.js(`!!document.querySelector('.nav-mode__item-active')`),
      true,
      "o foco não pode se perder depois de apagar",
    );

    // O disco tem que refletir a ordem final.
    await bridge.js(TECLA_NO_BLOCO("ArrowDown", true));
    await PAUSA(400);
    await bridge.js(SALVAR);
    await PAUSA(900);
    const disco = ctx.ler();
    const corpo = disco.split("---")[2].trim();
    ctx.assert(
      corpo.startsWith("beta"),
      `a reordenação não chegou no arquivo:\n${corpo}`,
    );
  },
});
