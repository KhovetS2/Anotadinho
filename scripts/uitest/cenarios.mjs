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
  const el = document.querySelector('.editor__wysiwyg[contenteditable="true"]');
  if (!el) return false;
  el.focus();
  const r = document.createRange();
  r.selectNodeContents(el); r.collapse(false);
  const s = getSelection(); s.removeAllRanges(); s.addRange(r);
  document.execCommand('insertText', false, ${JSON.stringify(texto)});
  return true;
})()`;

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
      const el = document.querySelector('.editor__wysiwyg[contenteditable="true"]');
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
