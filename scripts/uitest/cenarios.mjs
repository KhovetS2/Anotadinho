// Cenários do harness. Cada um é uma REGRESSÃO que já aconteceu de
// verdade — o número do ciclo entre parênteses é onde ela apareceu.
//
// Convenção: o cenário cria a página de rascunho pelo disco
// (`ctx.escrever`), abre no app, mexe pelo DOM e confere. Nunca toca em
// página real do vault.

import { esperar, recarregarEstavel, abrirPaginaEstavel } from "./bridge.mjs";

const PAUSA = (ms) => new Promise((r) => setTimeout(r, ms));

/// Recarrega o webview e espera a sidebar voltar — usado quando o teste
/// criou um arquivo novo e precisa que a listagem enxergue.
async function recarregar(bridge) {
  await recarregarEstavel(bridge);
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

export const cenarios = [
  {
    nome: "menu / lista os 9 tipos de embed e insere o escolhido (148)",
    async fn(bridge, ctx) {
      ctx.escrever("---\ntitle: __uitest\n---\n\n");
      await recarregar(bridge);
      await ctx.abrirPagina(bridge, ctx.nomePagina);
      // `abrirPagina` espera o CABEÇALHO; os blocos nascem depois, num
      // efeito. Digitar antes disso é digitar no vazio, e era o que
      // deixava este cenário vermelho de vez em quando na suíte cheia.
      await ctx.esperar(bridge, "document.querySelector('.editor__bloco')", "o bloco existir");

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
      // Datas RELATIVAS a hoje, não fixas.
      //
      // O cenário nasceu com `2026-08-10`/`14` cravados, o que
      // funcionou até a janela do cronograma (ancorada em hoje) passar
      // dessas datas — daí a barra deixou de ser desenhada e o cenário
      // ficou vermelho sozinho, sem ninguém ter mexido no código.
      const dia = (offset) => {
        const d = new Date();
        d.setDate(d.getDate() + offset);
        return d.toISOString().slice(0, 10);
      };
      ctx.escrever(
        `---\ntitle: __uitest\n---\n{{ type: "timeline" }}\nscale: month\nitems:\n- title: Etapa\n  start: ${dia(2)}\n  end: ${dia(6)}\n{{ /timeline }}\n`,
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

// ── ciclo 226: imagens persistidas e personalizáveis ───────────────

const PNG_IMAGEM_226 = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==";
const ARQUIVO_226 = (nome, tipo = "image/png") => `(() => { const bin=atob(${JSON.stringify(PNG_IMAGEM_226)}); const b=new Uint8Array(bin.length); for(let i=0;i<bin.length;i++)b[i]=bin.charCodeAt(i); return new File([b], ${JSON.stringify(nome)}, {type:${JSON.stringify(tipo)}}); })()`;

cenarios.push({
  nome: "imagens: drop múltiplo abre modal, personaliza e persiste sem blob (226)",
  async fn(bridge, ctx) {
    const fs = await import("node:fs");
    ctx.escrever("---\ntitle: __uitest\n---\ntexto\n");
    await recarregar(bridge); await ctx.abrirPagina(bridge, ctx.nomePagina);
    const antes = fs.existsSync(`${ctx.vault}/assets`) ? fs.readdirSync(`${ctx.vault}/assets`) : [];
    await bridge.js(`(() => { const alvo=document.querySelector('.editor__bloco'); alvo.focus(); const r=document.createRange();r.selectNodeContents(alvo);r.collapse(false);getSelection().removeAllRanges();getSelection().addRange(r); const dt=new DataTransfer();dt.items.add((${ARQUIVO_226("primeira.png")}));dt.items.add((${ARQUIVO_226("segunda.png")})); alvo.dispatchEvent(new DragEvent('dragover',{bubbles:true,cancelable:true,dataTransfer:dt})); alvo.dispatchEvent(new DragEvent('drop',{bubbles:true,cancelable:true,dataTransfer:dt})); return true;})()`);
    await ctx.esperar(bridge, "document.querySelectorAll('.image-modal__item').length===2", "o modal receber as duas imagens");
    // A ordem do arrasto tem que sobreviver até o modal: personalizar a
    // primeira e ver os campos irem parar na segunda seria pior do que
    // não personalizar.
    ctx.assertEq(
      await bridge.js("[...document.querySelectorAll('.image-modal__item strong')].map(e=>e.textContent).join(',')"),
      "primeira.png,segunda.png",
      "a ordem do lote não foi preservada",
    );
    for (const [indice, valor] of [
      [0, "Alt escolhido"],
      [1, "Legenda escolhida"],
      [3, "320"],
    ]) {
      await bridge.js(`(() => { const input=document.querySelectorAll('.image-modal__item')[0].querySelectorAll('input')[${indice}]; const set=Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set;set.call(input,${JSON.stringify(valor)});input.dispatchEvent(new InputEvent('input',{bubbles:true}));return true;})()`);
      await PAUSA(100);
    }
    await bridge.js(`(() => { const s=document.querySelector('.image-modal__item:first-child select');s.value='center';s.dispatchEvent(new Event('change',{bubbles:true}));return true;})()`);
    await PAUSA(100);
    await bridge.js(`([...document.querySelectorAll('.modal__actions button')].find(b=>b.textContent.includes('Inserir')).click(), true)`);
    await ctx.esperar(bridge, "document.querySelectorAll('.editor figure.inserted-image').length===2", "as imagens serem inseridas");
    await bridge.js(SALVAR); await PAUSA(900);
    let novos = [];
    try {
      const depois = fs.readdirSync(`${ctx.vault}/assets`); novos = depois.filter(f => !antes.includes(f));
      ctx.assertEq(novos.length, 2, "cada imagem deve criar um asset novo");
      const md = ctx.ler() || "";
      ctx.assert(md.includes('<figure class="inserted-image inserted-image--center">'), `alinhamento não persistiu:\n${md}`);
      ctx.assert(md.includes('alt="Alt escolhido"') && md.includes('<figcaption>Legenda escolhida</figcaption>') && md.includes('width="320"'), `campos não persistiram:\n${md}`);
      ctx.assert(!md.includes('blob:'), `blob persistido:\n${md}`);
      await recarregar(bridge); await ctx.abrirPagina(bridge, ctx.nomePagina);
      await ctx.esperar(bridge, "document.querySelectorAll('.editor figure.inserted-image img').length===2", "as imagens reabrirem");
    } finally { novos.forEach(f => fs.rmSync(`${ctx.vault}/assets/${f}`, {force:true})); }
  },
});

cenarios.push({
  nome: "imagens: drop não-imagem e paste de texto não alteram o editor (226)",
  async fn(bridge, ctx) {
    ctx.escrever("---\ntitle: __uitest\n---\ntexto\n"); await recarregar(bridge); await ctx.abrirPagina(bridge, ctx.nomePagina);
    const result = await bridge.js(`(() => { const alvo=document.querySelector('.editor__bloco'); const dt=new DataTransfer();dt.items.add(new File(['abc'],'x.txt',{type:'text/plain'}));alvo.dispatchEvent(new DragEvent('drop',{bubbles:true,cancelable:true,dataTransfer:dt})); const texto=new DataTransfer();texto.setData('text/plain','normal');const ev=new ClipboardEvent('paste',{bubbles:true,cancelable:true,clipboardData:texto});alvo.dispatchEvent(ev);return {modal:!!document.querySelector('.image-modal'),cancelado:ev.defaultPrevented};})()`);
    ctx.assertEq(result.modal, false, "arquivo não-imagem abriu o modal"); ctx.assertEq(result.cancelado, false, "paste de texto foi interceptado");
  },
});

cenarios.push({
  nome: "imagens: paste grava direto e undo remove a referência (226)",
  async fn(bridge, ctx) {
    const fs = await import("node:fs"); ctx.escrever("---\ntitle: __uitest\n---\ntexto\n"); await recarregar(bridge); await ctx.abrirPagina(bridge, ctx.nomePagina);
    const antes = fs.existsSync(`${ctx.vault}/assets`) ? fs.readdirSync(`${ctx.vault}/assets`) : [];
    await bridge.js(`(() => {const alvo=document.querySelector('.editor__bloco');alvo.focus();const r=document.createRange();r.selectNodeContents(alvo);r.collapse(false);getSelection().removeAllRanges();getSelection().addRange(r);const dt=new DataTransfer();dt.items.add((${ARQUIVO_226("colada.png")}));alvo.dispatchEvent(new ClipboardEvent('paste',{bubbles:true,cancelable:true,clipboardData:dt}));return true;})()`);
    await ctx.esperar(bridge, "document.querySelector('.editor figure.inserted-image')", "o paste inserir diretamente");
    await bridge.js(`document.querySelector('.editor__bloco').dispatchEvent(new KeyboardEvent('keydown',{key:'z',ctrlKey:true,bubbles:true,cancelable:true}));true`); await PAUSA(500);
    ctx.assertEq(await bridge.js("!!document.querySelector('.editor figure.inserted-image')"), false, "undo não removeu a inserção");
    const depois = fs.readdirSync(`${ctx.vault}/assets`); depois.filter(f=>!antes.includes(f)).forEach(f=>fs.rmSync(`${ctx.vault}/assets/${f}`,{force:true}));
  },
});

cenarios.push({
  nome: "imagens: cancelar o modal não grava asset nem toca no editor (226)",
  async fn(bridge, ctx) {
    const fs = await import("node:fs");
    ctx.escrever("---\ntitle: __uitest\n---\ntexto\n");
    await recarregar(bridge); await ctx.abrirPagina(bridge, ctx.nomePagina);
    await ctx.esperar(bridge, "document.querySelector('.editor__bloco')", "o bloco");
    const antes = fs.existsSync(`${ctx.vault}/assets`) ? fs.readdirSync(`${ctx.vault}/assets`) : [];
    await bridge.js(`(() => { const alvo=document.querySelector('.editor__bloco'); alvo.focus(); const r=document.createRange();r.selectNodeContents(alvo);r.collapse(false);getSelection().removeAllRanges();getSelection().addRange(r); const dt=new DataTransfer();dt.items.add((${ARQUIVO_226("descartada.png")})); alvo.dispatchEvent(new DragEvent('drop',{bubbles:true,cancelable:true,dataTransfer:dt})); return true;})()`);
    await ctx.esperar(bridge, "document.querySelectorAll('.image-modal__item').length===1", "o modal abrir");
    await bridge.js(`([...document.querySelectorAll('.modal__actions button')].find(b=>b.textContent.includes('Cancelar')).click(), true)`);
    await ctx.esperar(bridge, "!document.querySelector('.image-modal')", "o modal fechar");
    // O asset só nasce ao confirmar: desistir não pode deixar lixo em
    // `assets/`, senão cada arrasto arrependido vira arquivo órfão.
    const depois = fs.existsSync(`${ctx.vault}/assets`) ? fs.readdirSync(`${ctx.vault}/assets`) : [];
    ctx.assertEq(depois.filter(f => !antes.includes(f)).length, 0, "cancelar gravou asset");
    ctx.assertEq(await bridge.js("!!document.querySelector('.editor figure.inserted-image')"), false, "cancelar inseriu imagem");
  },
});

cenarios.push({
  nome: "imagens: arquivo que mente o tipo é recusado e nada é inserido (226)",
  async fn(bridge, ctx) {
    const fs = await import("node:fs");
    ctx.escrever("---\ntitle: __uitest\n---\ntexto\n");
    await recarregar(bridge); await ctx.abrirPagina(bridge, ctx.nomePagina);
    const antes = fs.existsSync(`${ctx.vault}/assets`) ? fs.readdirSync(`${ctx.vault}/assets`) : [];
    // Diz `image/png` no nome e no tipo, mas o conteúdo não é PNG. Quem
    // decide é o byte mágico no backend, não a extensão — é o que impede
    // um arquivo qualquer de entrar em `assets/` com cara de imagem.
    await bridge.js(`(() => { const alvo=document.querySelector('.editor__bloco'); alvo.focus(); const r=document.createRange();r.selectNodeContents(alvo);r.collapse(false);getSelection().removeAllRanges();getSelection().addRange(r); const dt=new DataTransfer();dt.items.add(new File(['isto nao e um png'],'mentirosa.png',{type:'image/png'})); alvo.dispatchEvent(new DragEvent('drop',{bubbles:true,cancelable:true,dataTransfer:dt})); return true;})()`);
    await ctx.esperar(bridge, "document.querySelectorAll('.image-modal__item').length===1", "o modal abrir");
    await bridge.js(`([...document.querySelectorAll('.modal__actions button')].find(b=>b.textContent.includes('Inserir')).click(), true)`);
    await ctx.esperar(bridge, "!!document.querySelector('.image-modal__error')", "o erro aparecer no modal");
    ctx.assertEq(await bridge.js("!!document.querySelector('.editor figure.inserted-image')"), false, "imagem inválida foi inserida");
    const depois = fs.existsSync(`${ctx.vault}/assets`) ? fs.readdirSync(`${ctx.vault}/assets`) : [];
    ctx.assertEq(depois.filter(f => !antes.includes(f)).length, 0, "imagem inválida virou asset");
    await bridge.js(`([...document.querySelectorAll('.modal__actions button')].find(b=>b.textContent.includes('Cancelar')).click(), true)`);
  },
});

// ── ciclo 234: barra flutuante de formatação ─────────────────────────

// Seleciona um trecho do primeiro bloco por índice de CARACTERE, sem
// supor a estrutura: depois de marcar algo, o primeiro filho do bloco
// deixa de ser um nó de texto e passa a ser o `<strong>`.
const SELECIONAR = (de, ate) => `(() => {
  const bloco = document.querySelector('.editor__bloco');
  const textos = [];
  (function andar(no) {
    if (no.nodeType === Node.TEXT_NODE) { textos.push(no); return; }
    for (const f of no.childNodes) andar(f);
  })(bloco);
  const r = document.createRange();
  let pos = 0, comecou = false;
  for (const t of textos) {
    const fim = pos + t.length;
    if (!comecou && ${de} >= pos && ${de} <= fim) { r.setStart(t, ${de} - pos); comecou = true; }
    if (comecou && ${ate} >= pos && ${ate} <= fim) { r.setEnd(t, ${ate} - pos); break; }
    pos = fim;
  }
  const s = getSelection();
  s.removeAllRanges();
  s.addRange(r);
  // Sem disparar selectionchange na mão: quem tem que acordar a barra
  // é o evento NATIVO, o único que existe quando a pessoa seleciona
  // com o mouse.
  return true;
})()`;

const CLICAR_MARCA = (titulo) => `(() => {
  const b = [...document.querySelectorAll('.selecao-barra__botao')]
    .find(x => x.title === ${JSON.stringify(titulo)});
  if (!b) return false;
  b.dispatchEvent(new MouseEvent('click', { bubbles: true, cancelable: true }));
  return true;
})()`;

cenarios.push({
  nome: "formatação: a barra só aparece com texto selecionado no editor (234)",
  async fn(bridge, ctx) {
    ctx.escrever("---\ntitle: __uitest\n---\ntexto para marcar\n");
    await recarregar(bridge);
    await ctx.abrirPagina(bridge, ctx.nomePagina);
    await ctx.esperar(bridge, "document.querySelector('.editor__bloco')", "o bloco");

    ctx.assertEq(await bridge.js("!!document.querySelector('.selecao-barra')"), false, "a barra nasce visível");
    await bridge.js(SELECIONAR(0, 5));
    await ctx.esperar(bridge, "document.querySelector('.selecao-barra')", "a barra aparecer");

    // Some quando a seleção esvazia — barra pendurada sem seleção
    // aplicaria marca em lugar nenhum.
    await bridge.js("getSelection().removeAllRanges(); document.dispatchEvent(new Event('selectionchange')); true");
    await ctx.esperar(bridge, "!document.querySelector('.selecao-barra')", "a barra sumir");
  },
});

cenarios.push({
  nome: "formatação: negrito pela barra sobrevive ao salvar (234)",
  async fn(bridge, ctx) {
    ctx.escrever("---\ntitle: __uitest\n---\ntexto para marcar\n");
    await recarregar(bridge);
    await ctx.abrirPagina(bridge, ctx.nomePagina);
    await ctx.esperar(bridge, "document.querySelector('.editor__bloco')", "o bloco");

    await bridge.js(SELECIONAR(0, 5));
    await ctx.esperar(bridge, "document.querySelector('.selecao-barra')", "a barra");
    ctx.assertEq(await bridge.js(CLICAR_MARCA("Negrito")), true, "não achei o botão de negrito");
    await PAUSA(400);
    ctx.assertEq(await bridge.js("!!document.querySelector('.editor__bloco strong')"), true, "não virou negrito no DOM");

    // O que interessa é o ARQUIVO: marca que o html_to_md não sabe
    // devolver some sozinha no autosave, três segundos depois.
    await bridge.js(SALVAR);
    await PAUSA(900);
    const md = ctx.ler() || "";
    ctx.assert(md.includes("**texto**"), `o negrito não chegou ao arquivo:\n${md}`);
    ctx.assert(md.includes("para marcar"), `o resto do texto se perdeu:\n${md}`);
  },
});

cenarios.push({
  nome: "formatação: clicar de novo na mesma marca tira ela (234)",
  async fn(bridge, ctx) {
    ctx.escrever("---\ntitle: __uitest\n---\ntexto para marcar\n");
    await recarregar(bridge);
    await ctx.abrirPagina(bridge, ctx.nomePagina);
    await ctx.esperar(bridge, "document.querySelector('.editor__bloco')", "o bloco");

    await bridge.js(SELECIONAR(0, 5));
    await ctx.esperar(bridge, "document.querySelector('.selecao-barra')", "a barra");
    await bridge.js(CLICAR_MARCA("Itálico"));
    await PAUSA(400);
    ctx.assertEq(await bridge.js("!!document.querySelector('.editor__bloco em')"), true, "não virou itálico");

    await bridge.js(CLICAR_MARCA("Itálico"));
    await PAUSA(400);
    ctx.assertEq(await bridge.js("!!document.querySelector('.editor__bloco em')"), false, "a marca não saiu");

    await bridge.js(SALVAR);
    await PAUSA(900);
    const md = ctx.ler() || "";
    ctx.assert(!md.includes("*texto*"), `o itálico ficou no arquivo:\n${md}`);
    ctx.assert(md.includes("texto para marcar"), `o texto se perdeu:\n${md}`);
  },
});

// ── ciclo 235: cor, realce e faxina do HTML ──────────────────────────

const ABRIR_CORES = `(() => {
  const b = [...document.querySelectorAll('.selecao-barra__botao')].find(x => x.title === 'Cor e realce');
  if (!b) return false;
  b.dispatchEvent(new MouseEvent('click', { bubbles: true, cancelable: true }));
  return true;
})()`;

const PINTAR = (classe) => `(() => {
  const b = [...document.querySelectorAll('.selecao-barra__amostra')]
    .find(x => x.className.includes(${JSON.stringify(classe)}));
  if (!b) return false;
  b.dispatchEvent(new MouseEvent('click', { bubbles: true, cancelable: true }));
  return true;
})()`;

cenarios.push({
  nome: "formatação: a barra aparece também em página COM embed (234)",
  async fn(bridge, ctx) {
    // Numa página com embed o editor não tem raiz única: cada segmento
    // de markdown é seu próprio `.editor__wysiwyg`. A primeira versão da
    // barra pedia a raiz, e por isso não aparecia em NENHUMA página com
    // embed — que é onde mora quase todo conteúdo do vault.
    ctx.escrever(
      "---\ntitle: __uitest\n---\ntexto antes do embed\n\n" +
        '{{ type: "fluxo" }}\nartefato: spec\netapa: rascunho\n{{ /fluxo }}\n\n' +
        "texto depois\n",
    );
    await recarregar(bridge);
    await ctx.abrirPagina(bridge, ctx.nomePagina);
    await ctx.esperar(bridge, "document.querySelector('.fluxo')", "o embed");
    await ctx.esperar(bridge, "document.querySelector('.editor__bloco')", "o bloco");

    await bridge.js(SELECIONAR(0, 5));
    await ctx.esperar(bridge, "document.querySelector('.selecao-barra')", "a barra na página com embed");
    ctx.assertEq(await bridge.js(CLICAR_MARCA("Negrito")), true, "não achei o botão");
    await PAUSA(400);
    await bridge.js(SALVAR);
    await PAUSA(900);
    const md = ctx.ler() || "";
    ctx.assert(md.includes("**texto**"), `o negrito não chegou ao arquivo:\n${md}`);
    // E o embed continua inteiro: marcar texto num segmento não pode
    // mexer no segmento vizinho.
    ctx.assert(md.includes('{{ type: "fluxo" }}') && md.includes("etapa: rascunho"),
      `o embed foi danificado:\n${md}`);
  },
});

cenarios.push({
  nome: "cor: pintar pela paleta chega ao arquivo como classe do tema (235)",
  async fn(bridge, ctx) {
    ctx.escrever("---\ntitle: __uitest\n---\ntexto para pintar\n");
    await recarregar(bridge);
    await ctx.abrirPagina(bridge, ctx.nomePagina);
    await ctx.esperar(bridge, "document.querySelector('.editor__bloco')", "o bloco");

    await bridge.js(SELECIONAR(0, 5));
    await ctx.esperar(bridge, "document.querySelector('.selecao-barra')", "a barra");
    await bridge.js(ABRIR_CORES);
    await ctx.esperar(bridge, "document.querySelector('.selecao-barra__paleta')", "a paleta");
    ctx.assertEq(await bridge.js(PINTAR("cor--ambar")), true, "não achei a amostra");
    await PAUSA(400);

    await bridge.js(SALVAR);
    await PAUSA(900);
    const md = ctx.ler() || "";
    // Classe do tema, não hex: a cor escolhida no escuro tem que
    // continuar legível no claro.
    ctx.assert(md.includes('<span class="cor--ambar">texto</span>'), `a cor não chegou ao arquivo:\n${md}`);
    ctx.assert(md.includes("para pintar"), `o resto se perdeu:\n${md}`);

    // E sobrevive a reabrir + editar, que é onde ela morria antes: o
    // `<span>` caía no braço genérico do html_to_md e virava texto puro.
    await recarregar(bridge);
    await ctx.abrirPagina(bridge, ctx.nomePagina);
    await ctx.esperar(bridge, "document.querySelector('.editor__bloco .cor--ambar')", "a cor reabrir");
    await bridge.js(SALVAR);
    await PAUSA(900);
    ctx.assert((ctx.ler() || "").includes('cor--ambar'), "a cor sumiu no segundo salvamento");
  },
});

cenarios.push({
  nome: "cor: realce é independente da cor da letra (235)",
  async fn(bridge, ctx) {
    ctx.escrever("---\ntitle: __uitest\n---\ntexto para pintar\n");
    await recarregar(bridge);
    await ctx.abrirPagina(bridge, ctx.nomePagina);
    await ctx.esperar(bridge, "document.querySelector('.editor__bloco')", "o bloco");

    await bridge.js(SELECIONAR(0, 5));
    await ctx.esperar(bridge, "document.querySelector('.selecao-barra')", "a barra");
    await bridge.js(ABRIR_CORES);
    await ctx.esperar(bridge, "document.querySelector('.selecao-barra__paleta')", "a paleta");
    // A paleta continua aberta depois de pintar — reabrir aqui a
    // fecharia, e o segundo clique cairia no vazio.
    ctx.assertEq(await bridge.js(PINTAR("cor--azul")), true, "não achei a cor de texto");
    await PAUSA(300);
    ctx.assertEq(await bridge.js(PINTAR("fundo--ambar")), true, "não achei o realce");
    await PAUSA(400);

    await bridge.js(SALVAR);
    await PAUSA(900);
    const md = ctx.ler() || "";
    // Pintar o fundo não pode apagar a cor da letra, e nem aninhar um
    // span dentro do outro a cada clique.
    ctx.assert(md.includes("cor--azul") && md.includes("fundo--ambar"), `perdeu uma das duas:\n${md}`);
    ctx.assertEq((md.match(/<span/g) || []).length, 1, `aninhou spans:\n${md}`);
  },
});

cenarios.push({
  nome: "html: script numa página não chega ao DOM, e a checklist continua (235)",
  async fn(bridge, ctx) {
    // Um `.md` do vault pode vir de git, de sincronização ou de um
    // agente: é conteúdo de terceiros.
    ctx.escrever(
      "---\ntitle: __uitest\n---\n" +
        "antes\n\n" +
        '<script>window.__invadiu = true;</script>\n\n' +
        '<img src="x.png" onerror="window.__invadiu = true;">\n\n' +
        "- [ ] uma tarefa\n- [x] outra\n",
    );
    await recarregar(bridge);
    await ctx.abrirPagina(bridge, ctx.nomePagina);
    await ctx.esperar(bridge, "document.querySelector('.editor__bloco')", "o bloco");
    await PAUSA(600);

    ctx.assertEq(await bridge.js("!!window.__invadiu"), false, "o script rodou");
    ctx.assertEq(await bridge.js("!!document.querySelector('.editor script')"), false, "o script entrou no DOM");
    ctx.assertEq(
      await bridge.js(`[...document.querySelectorAll('.editor img')].some(i => i.getAttribute('onerror'))`),
      false,
      "o onerror sobreviveu",
    );
    // E a faxina não pode comer conteúdo legítimo: a caixinha da
    // checklist é um <input type=checkbox>.
    ctx.assert(
      await bridge.js(`document.querySelectorAll('.editor input[type=checkbox]').length >= 2`),
      "a faxina comeu as caixinhas da checklist",
    );
  },
});

// ── ciclo 240: a barra continua servindo depois de cada clique ───────

cenarios.push({
  nome: "formatação: reselecionar e clicar de novo TIRA a marca, não aninha (240)",
  async fn(bridge, ctx) {
    ctx.escrever("---\ntitle: __uitest\n---\ntexto para marcar\n");
    await recarregar(bridge);
    await ctx.abrirPagina(bridge, ctx.nomePagina);
    await ctx.esperar(bridge, "document.querySelector('.editor__bloco')", "o bloco");

    await bridge.js(SELECIONAR(0, 5));
    await ctx.esperar(bridge, "document.querySelector('.selecao-barra')", "a barra");
    await bridge.js(CLICAR_MARCA("Negrito"));
    await PAUSA(400);

    // Desfaz a seleção e seleciona de novo — é o gesto real: a pessoa
    // marca, clica fora, muda de ideia e volta.
    await bridge.js("getSelection().removeAllRanges(); true");
    await ctx.esperar(bridge, "!document.querySelector('.selecao-barra')", "a barra sumir");
    await bridge.js(SELECIONAR(0, 5));
    await ctx.esperar(bridge, "document.querySelector('.selecao-barra')", "a barra voltar");
    await bridge.js(CLICAR_MARCA("Negrito"));
    await PAUSA(400);

    // Reselecionar arrastando faz o range começar FORA do <strong>, e a
    // busca pelo ancestral comum não achava a marca: o clique aninhava
    // outra, e depois não saía mais.
    ctx.assertEq(await bridge.js("document.querySelectorAll('.editor__bloco strong').length"), 0,
      "a marca não saiu na segunda passagem");
    await bridge.js(SALVAR);
    await PAUSA(900);
    const md = ctx.ler() || "";
    ctx.assert(!md.includes("**"), `sobrou marca no arquivo:\n${md}`);
    ctx.assert(md.includes("texto para marcar"), `o texto se perdeu:\n${md}`);
  },
});

cenarios.push({
  nome: "formatação: a seleção sobrevive a aplicar e a tirar (240)",
  async fn(bridge, ctx) {
    ctx.escrever("---\ntitle: __uitest\n---\ntexto para marcar\n");
    await recarregar(bridge);
    await ctx.abrirPagina(bridge, ctx.nomePagina);
    await ctx.esperar(bridge, "document.querySelector('.editor__bloco')", "o bloco");

    await bridge.js(SELECIONAR(0, 5));
    await ctx.esperar(bridge, "document.querySelector('.selecao-barra')", "a barra");

    // Quem marcou uma palavra costuma querer marcar outra coisa nela em
    // seguida. Perder a seleção obriga a selecionar de novo a cada clique.
    await bridge.js(CLICAR_MARCA("Negrito"));
    await PAUSA(400);
    ctx.assertEq(await bridge.js("getSelection().toString()"), "texto", "a seleção se perdeu ao aplicar");

    await bridge.js(CLICAR_MARCA("Itálico"));
    await PAUSA(400);
    ctx.assertEq(await bridge.js("getSelection().toString()"), "texto", "a seleção se perdeu na segunda marca");

    await bridge.js(CLICAR_MARCA("Negrito"));
    await PAUSA(400);
    ctx.assertEq(await bridge.js("getSelection().toString()"), "texto", "a seleção se perdeu ao TIRAR");
    ctx.assertEq(await bridge.js("!!document.querySelector('.selecao-barra')"), true, "a barra sumiu");

    await bridge.js(SALVAR);
    await PAUSA(900);
    const md = ctx.ler() || "";
    ctx.assert(md.includes("*texto*") && !md.includes("**texto**"),
      `devia ter sobrado só o itálico:\n${md}`);
  },
});

cenarios.push({
  nome: "formatação: a paleta de cor não reaparece aberta na seleção seguinte (240)",
  async fn(bridge, ctx) {
    ctx.escrever("---\ntitle: __uitest\n---\ntexto para marcar\n");
    await recarregar(bridge);
    await ctx.abrirPagina(bridge, ctx.nomePagina);
    await ctx.esperar(bridge, "document.querySelector('.editor__bloco')", "o bloco");

    await bridge.js(SELECIONAR(0, 5));
    await ctx.esperar(bridge, "document.querySelector('.selecao-barra')", "a barra");
    await bridge.js(ABRIR_CORES);
    await ctx.esperar(bridge, "document.querySelector('.selecao-barra__paleta')", "a paleta");

    await bridge.js("getSelection().removeAllRanges(); true");
    await ctx.esperar(bridge, "!document.querySelector('.selecao-barra')", "a barra sumir");
    await bridge.js(SELECIONAR(6, 10));
    await ctx.esperar(bridge, "document.querySelector('.selecao-barra')", "a barra voltar");

    // Uma paleta aberta por cima do texto, de uma interação que a pessoa
    // nem lembra, atrapalha em vez de ajudar.
    ctx.assertEq(await bridge.js("!!document.querySelector('.selecao-barra__paleta')"), false,
      "a paleta voltou aberta");
  },
});

cenarios.push({
  nome: "imagens: a inserida aparece na hora, sem precisar recarregar (242)",
  async fn(bridge, ctx) {
    const fs = await import("node:fs");
    ctx.escrever("---\ntitle: __uitest\n---\ntexto\n");
    await recarregar(bridge); await ctx.abrirPagina(bridge, ctx.nomePagina);
    await ctx.esperar(bridge, "document.querySelector('.editor__bloco')", "o bloco");
    const antes = fs.existsSync(`${ctx.vault}/assets`) ? fs.readdirSync(`${ctx.vault}/assets`) : [];
    let novos = [];
    try {
      await bridge.js(`(() => { const alvo=document.querySelector('.editor__bloco'); alvo.focus(); const r=document.createRange();r.selectNodeContents(alvo);r.collapse(false);getSelection().removeAllRanges();getSelection().addRange(r); const dt=new DataTransfer();dt.items.add((${ARQUIVO_226("agora.png")})); alvo.dispatchEvent(new DragEvent('drop',{bubbles:true,cancelable:true,dataTransfer:dt})); return true;})()`);
      await ctx.esperar(bridge, "document.querySelectorAll('.image-modal__item').length===1", "o modal");
      await bridge.js(`([...document.querySelectorAll('.modal__actions button')].find(b=>b.textContent.includes('Inserir')).click(), true)`);
      await ctx.esperar(bridge, "document.querySelector('.editor figure.inserted-image img')", "a imagem entrar");

      // O `src` inserido é relativo (`assets/x.png`) e o webview não
      // resolve isso: a imagem ficava em branco até alguém recarregar.
      await ctx.esperar(
        bridge,
        "document.querySelector('.editor figure.inserted-image img').src.startsWith('data:')",
        "a imagem ser resolvida na hora",
      );
      // E o caminho relativo continua guardado, senão o markdown levaria
      // a data URL inteira para o arquivo.
      ctx.assert(
        (await bridge.js("document.querySelector('.editor figure.inserted-image img').getAttribute('data-asset-src')") || "").startsWith("assets/"),
        "perdeu o caminho relativo do asset",
      );

      await bridge.js(SALVAR); await PAUSA(900);
      const md = ctx.ler() || "";
      ctx.assert(!md.includes("data:image"), `a data URL vazou pro arquivo:\n${md.slice(0, 300)}`);
      ctx.assert(md.includes('src="assets/'), `o caminho do asset não foi gravado:\n${md}`);
      novos = fs.readdirSync(`${ctx.vault}/assets`).filter(f => !antes.includes(f));
    } finally {
      novos.forEach(f => fs.rmSync(`${ctx.vault}/assets/${f}`, { force: true }));
    }
  },
});

cenarios.push({
  nome: "offline: nada de script ou estilo vem da internet (243)",
  async fn(bridge, ctx) {
    // Um app de notas locais que precisa de conexão pra desenhar um
    // diagrama não é local. E script de terceiro entrando na janela a
    // cada abertura é superfície que não se controla.
    const externos = await bridge.js(`(() => {
      const url = (e) => e.getAttribute('src') || e.getAttribute('href') || '';
      return [...document.querySelectorAll('script[src], link[href]')]
        .map(url)
        .filter(u => /^(https?:)?\\/\\//.test(u));
    })()`);
    ctx.assertEq(externos.length, 0, `veio de fora: ${externos.join(", ")}`);

    // E continuam funcionando vindo de casa.
    ctx.assertEq(await bridge.js("typeof window.mermaid"), "object", "mermaid não carregou");
    ctx.assertEq(await bridge.js("typeof window.hljs"), "object", "highlight.js não carregou");
  },
});

// ── ciclo 244: tirar a marca só do trecho selecionado ────────────────

cenarios.push({
  nome: "formatação: tirar negrito de uma palavra não desmarca a frase (244)",
  async fn(bridge, ctx) {
    ctx.escrever("---\ntitle: __uitest\n---\numa frase inteira aqui\n");
    await recarregar(bridge);
    await ctx.abrirPagina(bridge, ctx.nomePagina);
    await ctx.esperar(bridge, "document.querySelector('.editor__bloco')", "o bloco");

    // A frase toda em negrito.
    await bridge.js(SELECIONAR(0, 22));
    await ctx.esperar(bridge, "document.querySelector('.selecao-barra')", "a barra");
    await bridge.js(CLICAR_MARCA("Negrito"));
    await PAUSA(400);

    // Agora só a palavra "frase" (4..9).
    await bridge.js(SELECIONAR(4, 9));
    await ctx.esperar(bridge, "document.querySelector('.selecao-barra')", "a barra de novo");
    await bridge.js(CLICAR_MARCA("Negrito"));
    await PAUSA(400);

    // O que se pede ao clicar em negrito com uma palavra selecionada é
    // sobre AQUELA palavra: as bordas continuam em negrito.
    await bridge.js(SALVAR);
    await PAUSA(900);
    const md = ctx.ler() || "";
    // As bordas continuam em negrito, e o espaço fica FORA da marca:
    // `**uma **` não é markdown válido.
    ctx.assert(md.includes("**uma** frase **inteira aqui**"), `resultado inesperado:\n${md}`);
    // O texto continua inteiro, e "frase" ficou de fora do negrito.
    ctx.assert(md.includes("uma") && md.includes("frase") && md.includes("inteira aqui"),
      `o texto se perdeu:\n${md}`);
    const dom = await bridge.js(`document.querySelector('.editor__bloco').innerHTML`);
    ctx.assertEq(await bridge.js(`document.querySelectorAll('.editor__bloco strong').length`), 2,
      `devia sobrar negrito nas duas bordas: ${dom}`);
    ctx.assert(!(await bridge.js(`[...document.querySelectorAll('.editor__bloco strong')].some(s => s.textContent.includes('frase'))`)),
      `a palavra continuou em negrito: ${dom}`);
  },
});

cenarios.push({
  nome: "formatação: marcar por cima de marca parcial não fragmenta (244)",
  async fn(bridge, ctx) {
    ctx.escrever("---\ntitle: __uitest\n---\numa frase inteira aqui\n");
    await recarregar(bridge);
    await ctx.abrirPagina(bridge, ctx.nomePagina);
    await ctx.esperar(bridge, "document.querySelector('.editor__bloco')", "o bloco");

    await bridge.js(SELECIONAR(0, 9));
    await ctx.esperar(bridge, "document.querySelector('.selecao-barra')", "a barra");
    await bridge.js(CLICAR_MARCA("Negrito"));
    await PAUSA(400);

    // Selecionar dali até o fim e clicar de novo deve dar UM trecho, não
    // dois grudados que o markdown renderizaria torto.
    await bridge.js(SELECIONAR(4, 22));
    await ctx.esperar(bridge, "document.querySelector('.selecao-barra')", "a barra de novo");
    await bridge.js(CLICAR_MARCA("Negrito"));
    await PAUSA(400);

    ctx.assertEq(await bridge.js(`document.querySelectorAll('.editor__bloco strong').length`), 1,
      "fragmentou em vez de estender");
    await bridge.js(SALVAR);
    await PAUSA(900);
    ctx.assert((ctx.ler() || "").includes("**uma frase inteira aqui**"), `não estendeu:\n${ctx.ler()}`);
  },
});

cenarios.push({
  nome: "cor: tirar a cor de uma palavra preserva a cor das bordas (244)",
  async fn(bridge, ctx) {
    ctx.escrever("---\ntitle: __uitest\n---\numa frase inteira aqui\n");
    await recarregar(bridge);
    await ctx.abrirPagina(bridge, ctx.nomePagina);
    await ctx.esperar(bridge, "document.querySelector('.editor__bloco')", "o bloco");

    await bridge.js(SELECIONAR(0, 22));
    await ctx.esperar(bridge, "document.querySelector('.selecao-barra')", "a barra");
    await bridge.js(ABRIR_CORES);
    await ctx.esperar(bridge, "document.querySelector('.selecao-barra__paleta')", "a paleta");
    await bridge.js(PINTAR("cor--ambar"));
    await PAUSA(400);

    // Pinta só "frase" de outra cor: as bordas têm que continuar âmbar.
    await bridge.js(SELECIONAR(4, 9));
    await ctx.esperar(bridge, "document.querySelector('.selecao-barra')", "a barra de novo");
    await bridge.js(ABRIR_CORES);
    await ctx.esperar(bridge, "document.querySelector('.selecao-barra__paleta')", "a paleta de novo");
    await bridge.js(PINTAR("cor--azul"));
    await PAUSA(500);

    await bridge.js(SALVAR);
    await PAUSA(900);
    const md = ctx.ler() || "";
    ctx.assert(md.includes("cor--azul"), `a cor nova não entrou:\n${md}`);
    ctx.assertEq((md.match(/cor--ambar/g) || []).length, 2, `as bordas perderam a cor de antes:\n${md}`);
    ctx.assert(md.includes("uma") && md.includes("frase") && md.includes("inteira aqui"),
      `o texto se perdeu:\n${md}`);
  },
});

// ── ciclo 245: arrastar imagem não pode derrubar o app ───────────────

cenarios.push({
  nome: "imagens: soltar fora do editor não navega o webview pro arquivo (245)",
  async fn(bridge, ctx) {
    ctx.escrever("---\ntitle: __uitest\n---\ntexto\n");
    await recarregar(bridge);
    await ctx.abrirPagina(bridge, ctx.nomePagina);
    await ctx.esperar(bridge, "document.querySelector('.editor__bloco')", "o bloco");

    // Um `drop` que ninguém trata tem comportamento padrão: o webview
    // NAVEGA para o arquivo solto, a página do app é substituída e a
    // janela fica em branco pra sempre. Aconteceu de verdade — a janela
    // acabou em `file:///home/.../anotadinho-icon.png`.
    const r = await bridge.js(`(() => {
      const dt = new DataTransfer();
      dt.items.add((${ARQUIVO_226("solta-fora.png")}));
      const eventos = ['dragover', 'drop'].map(tipo => {
        const ev = new DragEvent(tipo, { bubbles: true, cancelable: true, dataTransfer: dt });
        document.querySelector('.sidebar-section').dispatchEvent(ev);
        return { tipo, barrado: ev.defaultPrevented };
      });
      return eventos;
    })()`);
    for (const e of r) {
      ctx.assertEq(e.barrado, true, `${e.tipo} fora do editor não foi barrado`);
    }
    // E o app continua no ar, que é o ponto.
    ctx.assert(await bridge.js("location.href.startsWith('http')"), "o webview saiu do app");
    ctx.assert(await bridge.js("!!document.querySelector('.editor__bloco')"), "o editor sumiu");
  },
});

cenarios.push({
  nome: "imagens: arquivo sem tipo MIME é aceito pela extensão (245)",
  async fn(bridge, ctx) {
    ctx.escrever("---\ntitle: __uitest\n---\ntexto\n");
    await recarregar(bridge);
    await ctx.abrirPagina(bridge, ctx.nomePagina);
    await ctx.esperar(bridge, "document.querySelector('.editor__bloco')", "o bloco");

    // No arrasto vindo do SISTEMA o WebKitGTK costuma entregar o arquivo
    // com `type` vazio. A checagem `type.startsWith('image/')` recusava a
    // imagem em silêncio: o modal nem abria, e parecia que arrastar não
    // fazia nada.
    await bridge.js(`(() => {
      const bin = atob(${JSON.stringify(PNG_IMAGEM_226)});
      const b = new Uint8Array(bin.length);
      for (let i = 0; i < bin.length; i++) b[i] = bin.charCodeAt(i);
      const semTipo = new File([b], 'sem-tipo.png', { type: '' });
      const alvo = document.querySelector('.editor__bloco');
      alvo.focus();
      const r = document.createRange();
      r.selectNodeContents(alvo); r.collapse(false);
      getSelection().removeAllRanges(); getSelection().addRange(r);
      const dt = new DataTransfer();
      dt.items.add(semTipo);
      alvo.dispatchEvent(new DragEvent('drop', { bubbles: true, cancelable: true, dataTransfer: dt }));
      return true;
    })()`);
    await ctx.esperar(bridge, "document.querySelectorAll('.image-modal__item').length===1", "o modal abrir mesmo sem tipo");
    await bridge.js(`([...document.querySelectorAll('.modal__actions button')].find(b=>b.textContent.includes('Cancelar')).click(), true)`);
  },
});

cenarios.push({
  nome: "imagens: arrasto do sistema chega como caminho, e mesmo assim insere (245)",
  async fn(bridge, ctx) {
    const fs = await import("node:fs");
    const os = await import("node:os");
    // É ASSIM que o arrasto vindo de fora chega no WebKitGTK: sem `File`
    // nenhum, só `text/uri-list` com o caminho. A sonda na janela de
    // verdade mostrou exatamente isto — `files: 0`, `types:
    // ["text/uri-list", "text/html"]` —, e era por isso que arrastar não
    // inseria nada.
    const origem = `${os.tmpdir()}/uitest imagem ${Date.now()}.png`;
    fs.writeFileSync(origem, Buffer.from(PNG_IMAGEM_226, "base64"));
    const antes = fs.existsSync(`${ctx.vault}/assets`) ? fs.readdirSync(`${ctx.vault}/assets`) : [];
    let novos = [];
    try {
      ctx.escrever("---\ntitle: __uitest\n---\ntexto\n");
      await recarregar(bridge);
      await ctx.abrirPagina(bridge, ctx.nomePagina);
      await ctx.esperar(bridge, "document.querySelector('.editor__bloco')", "o bloco");

      // O nome tem espaço de propósito: a URI vem com %20, e sem decodificar
      // o backend procuraria um arquivo que não existe.
      const uri = "file://" + origem.split("/").map(encodeURIComponent).join("/");
      await bridge.js(`(() => {
        const alvo = document.querySelector('.editor__bloco');
        alvo.focus();
        const r = document.createRange();
        r.selectNodeContents(alvo); r.collapse(false);
        getSelection().removeAllRanges(); getSelection().addRange(r);
        const dt = new DataTransfer();
        dt.setData('text/uri-list', ${JSON.stringify(uri)});
        alvo.dispatchEvent(new DragEvent('drop', { bubbles: true, cancelable: true, dataTransfer: dt }));
        return true;
      })()`);

      await ctx.esperar(bridge, "document.querySelectorAll('.image-modal__item').length===1", "o modal abrir pelo caminho");
      await bridge.js(`([...document.querySelectorAll('.modal__actions button')].find(b=>b.textContent.includes('Inserir')).click(), true)`);
      await ctx.esperar(bridge, "document.querySelector('.editor figure.inserted-image img')", "a imagem entrar");

      await bridge.js(SALVAR);
      await PAUSA(900);
      const md = ctx.ler() || "";
      ctx.assert(md.includes('src="assets/'), `não gravou o asset:\n${md}`);
      novos = fs.readdirSync(`${ctx.vault}/assets`).filter(f => !antes.includes(f));
      ctx.assertEq(novos.length, 1, "devia ter criado um asset");
    } finally {
      fs.rmSync(origem, { force: true });
      novos.forEach(f => fs.rmSync(`${ctx.vault}/assets/${f}`, { force: true }));
    }
  },
});

cenarios.push({
  nome: "imagens: caminho vindo no text/html do gerenciador de arquivos (246)",
  async fn(bridge, ctx) {
    const fs = await import("node:fs");
    const os = await import("node:os");
    // Como o gerenciador de arquivos REALMENTE entrega, medido por sonda
    // num arrasto de verdade: `text/uri-list` anunciado e VAZIO, e o
    // caminho dentro do `text/html`, como texto de uma âncora. Nenhum
    // evento sintético anterior reproduzia isso — por isso o gesto
    // falhava com a suíte inteira verde.
    const origem = `${os.tmpdir()}/uitest via html ${Date.now()}.png`;
    fs.writeFileSync(origem, Buffer.from(PNG_IMAGEM_226, "base64"));
    const antes = fs.existsSync(`${ctx.vault}/assets`) ? fs.readdirSync(`${ctx.vault}/assets`) : [];
    let novos = [];
    try {
      ctx.escrever("---\ntitle: __uitest\n---\ntexto\n");
      await recarregar(bridge);
      await ctx.abrirPagina(bridge, ctx.nomePagina);
      await ctx.esperar(bridge, "document.querySelector('.editor__bloco')", "o bloco");

      const uri = "file://" + origem.split("/").map(encodeURIComponent).join("/");
      const html = `<a style="color: rgb(255, 255, 255);">${uri}</a>`;
      await bridge.js(`(() => {
        const alvo = document.querySelector('.editor__bloco');
        alvo.focus();
        const r = document.createRange();
        r.selectNodeContents(alvo); r.collapse(false);
        getSelection().removeAllRanges(); getSelection().addRange(r);
        const dt = new DataTransfer();
        dt.setData('text/uri-list', '');
        dt.setData('text/html', ${JSON.stringify(html)});
        alvo.dispatchEvent(new DragEvent('drop', { bubbles: true, cancelable: true, dataTransfer: dt }));
        return true;
      })()`);

      await ctx.esperar(bridge, "document.querySelectorAll('.image-modal__item').length===1", "o modal abrir pelo html");
      await bridge.js(`([...document.querySelectorAll('.modal__actions button')].find(b=>b.textContent.includes('Inserir')).click(), true)`);
      await ctx.esperar(bridge, "document.querySelector('.editor figure.inserted-image img')", "a imagem entrar");

      await bridge.js(SALVAR);
      await PAUSA(900);
      ctx.assert((ctx.ler() || "").includes('src="assets/'), "não gravou o asset");
      novos = fs.readdirSync(`${ctx.vault}/assets`).filter(f => !antes.includes(f));
      ctx.assertEq(novos.length, 1, "devia ter criado um asset");
    } finally {
      fs.rmSync(origem, { force: true });
      novos.forEach(f => fs.rmSync(`${ctx.vault}/assets/${f}`, { force: true }));
    }
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

// ── ciclo 217: leitura de consultas ──────────────────────────────────

cenarios.push({
  nome: "consulta: tabela alinhada, badges e janela virtualizada (217)",
  async fn(bridge, ctx) {
    const { mkdirSync, writeFileSync, rmSync } = await import("node:fs");
    const { join } = await import("node:path");
    const pasta = join(ctx.vault, "pages/__uitest-query");
    mkdirSync(pasta, { recursive: true });
    try {
      for (let i = 0; i < 101; i++) {
        writeFileSync(join(pasta, `q-${i}.md`), `---\ntitle: Consulta ${i}\npriority: alta\ntype: spec\n---\n# ${i}\n`);
      }
      ctx.escrever(
        '---\ntitle: __uitest\n---\n{{ type: "query" }}\nfrom: pages/__uitest-query\nview: table\ncolumns:\n- priority\n- type\nmax_height: 84\n{{ /query }}\n',
      );
      await recarregar(bridge);
      await ctx.abrirPagina(bridge, ctx.nomePagina);
      await ctx.esperar(bridge, "document.querySelectorAll('.query-embed__row').length > 0", "a primeira janela da consulta renderizar");

      const inicio = await bridge.js(`(() => {
        const area = document.querySelector('.query-embed__results');
        const th = document.querySelector('.query-embed__table th:nth-child(2)');
        const td = document.querySelector('.query-embed__table td:nth-child(2)');
        const chips = [...document.querySelectorAll('.query-embed__chip')];
        return { altura: area.getBoundingClientRect().height, rola: area.scrollHeight > area.clientHeight,
          alinhado: Math.abs(th.getBoundingClientRect().left - td.getBoundingClientRect().left) < 1,
          poucos: document.querySelectorAll('.query-embed__row').length < 30,
          cor: chips.length > 1 && chips[0].className === chips[2].className };
      })()`);
      ctx.assert(inicio.altura <= 84.5, `a área passou da altura configurada: ${inicio.altura}`);
      ctx.assert(inicio.rola, "a consulta longa devia rolar internamente");
      ctx.assert(inicio.alinhado, "o valor da coluna priority não alinhou com o cabeçalho");
      ctx.assert(inicio.poucos, "a virtualização montou linhas demais no DOM");
      ctx.assert(inicio.cor, "o mesmo valor de propriedade não recebeu o mesmo badge");

      // Índices montados antes de rolar: a janela precisa MUDAR, não só
      // conter o último item — que, na ordem do índice, pode já estar no
      // primeiro recorte.
      const antes = await bridge.js(`(() => [...document.querySelectorAll('[data-query-index]')].map((e) => +e.dataset.queryIndex))()`);
      await bridge.js(`(() => { const area = document.querySelector('.query-embed__results'); area.scrollTop = area.scrollHeight; area.dispatchEvent(new Event('scroll', { bubbles: true })); return true; })()`);
      await ctx.esperar(bridge, "[...document.querySelectorAll('[data-query-index]')].some((e) => +e.dataset.queryIndex === 100)", "a janela virtualizada alcançar o último resultado");

      const fim = await bridge.js(`(() => {
        const indices = [...document.querySelectorAll('[data-query-index]')].map((e) => +e.dataset.queryIndex);
        const area = document.querySelector('.query-embed__results');
        return { primeiro: Math.min(...indices), total: indices.length,
          altura: area.getBoundingClientRect().height };
      })()`);
      ctx.assert(fim.primeiro > Math.min(...antes), `a janela não avançou: começou em ${Math.min(...antes)} e continua em ${fim.primeiro}`);
      ctx.assert(fim.total < 30, `a virtualização montou linhas demais depois de rolar: ${fim.total}`);
      ctx.assert(fim.altura <= 84.5, `a área cresceu depois de rolar: ${fim.altura}`);
    } finally {
      rmSync(pasta, { recursive: true, force: true });
    }
  },
});

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
    // O contêiner existe antes dos blocos: esperar por ele e digitar em
    // seguida é corrida, e aparecia como "a página não ficou suja".
    await ctx.esperar(bridge, "document.querySelector('.editor__bloco')", "o bloco existir");

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

cenarios.push({
  nome: "abrir uma página com conteúdo não dispara gravação vazia sozinha (248)",
  async fn(bridge, ctx) {
    // Reprodução do que apareceu de verdade: abrir QUALQUER página com
    // conteúdo real, sem tocar em nada, e esperar passar da janela do
    // autosave (3s) — nenhuma edição do usuário no meio. Se algo (troca
    // de página, reinjeção de segmento) disparar um `oninput` espúrio
    // com markdown vazio nesse intervalo, o autosave tentava gravar por
    // cima: o backend recusa (`recusar_esvaziamento`), mas o usuário via
    // o erro cru (`JsValue("...")`) sem ter feito nada.
    const conteudo = "Conteúdo real que não pode sumir sozinho.\n\nSegunda linha, só pra ter mais de um bloco.\n";
    ctx.escrever(`---\ntitle: __uitest\n---\n\n${conteudo}`);
    await recarregar(bridge);
    await ctx.abrirPagina(bridge, ctx.nomePagina);
    await ctx.esperar(bridge, "document.querySelector('.editor__bloco')", "o bloco existir");

    // 3s do debounce + folga pro round-trip com o backend.
    await PAUSA(4500);

    const erro = await bridge.js(`(() => {
      const el = document.querySelector('.editor__overlay--error');
      return el ? el.textContent : null;
    })()`);
    ctx.assert(!erro, `overlay de erro apareceu sem nenhuma edição: ${erro}`);

    const disco = ctx.ler();
    ctx.assert(
      disco.includes("Conteúdo real que não pode sumir sozinho."),
      `o conteúdo sumiu do disco sozinho:\n${disco}`,
    );
  },
});

cenarios.push({
  nome: "highlight.js não pode assar language-undefined no arquivo (249)",
  async fn(bridge, ctx) {
    // Dano real, encontrado em `pages/arquitetura.md`: uma fence sem
    // linguagem (```) voltou do disco como ```undefined. O highlight.js
    // roda EM CIMA do DOM editável e escreve `class="language-undefined"`
    // no `<code>` quando não reconhece a linguagem; o round-trip de
    // `html_to_md` lia essa classe como se fosse a linguagem do usuário
    // e a gravava. Uma vez no arquivo, nunca mais saía sozinho.
    ctx.escrever(
      `---\ntitle: __uitest\n---\n\ntexto antes\n\n\`\`\`\numa linha de código\n\`\`\`\n\ntexto depois\n`,
    );
    await recarregar(bridge);
    await ctx.abrirPagina(bridge, ctx.nomePagina);
    await ctx.esperar(bridge, "document.querySelector('pre code')", "o bloco de código existir");
    // O hljs roda depois do render; sem essa folga o teste passaria por
    // ele não ter mexido no DOM ainda, não por o conserto funcionar.
    await PAUSA(1200);

    const classe = await bridge.js(
      `(document.querySelector('pre code') || {}).className || ''`,
    );

    // Uma edição qualquer, pra forçar o round-trip pelo `html_to_md`.
    await bridge.js(`(() => {
      const alvo = [...document.querySelectorAll('.editor__bloco')]
        .find(b => b.textContent.includes('texto depois'));
      alvo.focus();
      const r = document.createRange();
      r.selectNodeContents(alvo); r.collapse(false);
      const s = getSelection(); s.removeAllRanges(); s.addRange(r);
      document.execCommand('insertText', false, '!');
      return true;
    })()`);
    await PAUSA(400);
    await bridge.js(`(() => {
      const b = [...document.querySelectorAll('button')].find(b => b.textContent.trim().startsWith('Salvar'));
      if (b) b.click();
      return !!b;
    })()`);
    await PAUSA(1200);

    const disco = ctx.ler();
    ctx.assert(
      !/```undefined/.test(disco),
      `a classe do hljs (${classe}) virou linguagem no arquivo:\n${disco}`,
    );
    ctx.assert(
      disco.includes("uma linha de código"),
      `o conteúdo do bloco de código sumiu:\n${disco}`,
    );
  },
});

// ── Ciclo 250: a pilha de navegação e a página que ela abre ─────────

/// O que está focado agora, do jeito que o nav-mode enxerga.
const FOCO_NAV = `(() => {
  const a = document.activeElement;
  if (!a) return null;
  return {
    item: a.getAttribute('data-nav-item'),
    tag: a.tagName,
    texto: (a.textContent || '').trim().slice(0, 40),
    sessao: !!document.querySelector('.nav-mode__item-active'),
  };
})()`;

const TECLA_NO_FOCO = (key) => `(() => {
  document.activeElement.dispatchEvent(
    new KeyboardEvent('keydown', { key: ${JSON.stringify(key)}, bubbles: true, cancelable: true }));
  return true;
})()`;

/// Começa uma sessão de navegação na raiz, do jeito que o usuário
/// começa: a primeira seta com a capacidade ligada.
const COMECAR_NAVEGACAO = `(() => {
  const raiz = document.querySelector('.app-root');
  raiz.focus();
  raiz.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowDown', bubbles: true, cancelable: true }));
  return true;
})()`;

/// Anda no nível atual até o item pedido (ou desiste).
const IR_ATE_O_ITEM = (item, passos = 6) => `(async () => {
  for (let i = 0; i < ${passos}; i++) {
    if (document.activeElement.getAttribute('data-nav-item') === ${JSON.stringify(item)}) return true;
    document.activeElement.dispatchEvent(
      new KeyboardEvent('keydown', { key: 'j', bubbles: true, cancelable: true }));
    await new Promise(r => setTimeout(r, 180));
  }
  return document.activeElement.getAttribute('data-nav-item') === ${JSON.stringify(item)};
})()`;

cenarios.push({
  nome: "navegação: hjkl anda onde as setas andam (250)",
  async fn(bridge, ctx) {
    ctx.escrever("---\ntitle: __uitest\n---\n\nalfa\n\nbeta\n");
    await recarregar(bridge);
    await ctx.abrirPagina(bridge, ctx.nomePagina);
    await ctx.esperar(bridge, "document.querySelector('.editor__bloco')", "o bloco existir");

    await bridge.js(COMECAR_NAVEGACAO);
    await PAUSA(400);
    const inicio = await bridge.js(FOCO_NAV);
    ctx.assert(inicio && inicio.sessao, "a sessão de navegação não começou");

    await bridge.js(TECLA_NO_FOCO("j"));
    await PAUSA(300);
    const depoisJ = await bridge.js(FOCO_NAV);
    ctx.assert(depoisJ.item !== inicio.item, `j não moveu (${inicio.item} → ${depoisJ.item})`);

    await bridge.js(TECLA_NO_FOCO("k"));
    await PAUSA(300);
    const depoisK = await bridge.js(FOCO_NAV);
    ctx.assertEq(depoisK.item, inicio.item, "k devia ter voltado pro item anterior");
  },
});

cenarios.push({
  nome: "navegação: Escape sobe UM nível por vez (250)",
  async fn(bridge, ctx) {
    ctx.escrever("---\ntitle: __uitest\n---\n\nalfa\n\nbeta\n");
    await recarregar(bridge);
    await ctx.abrirPagina(bridge, ctx.nomePagina);
    await ctx.esperar(bridge, "document.querySelector('.editor__bloco')", "o bloco existir");

    await bridge.js(COMECAR_NAVEGACAO);
    await PAUSA(400);
    await bridge.js(IR_ATE_O_ITEM("editor"));
    await PAUSA(200);
    await bridge.js(TECLA_NO_FOCO("Enter")); // desce pros blocos
    await PAUSA(500);
    const nosBlocos = await bridge.js(FOCO_NAV);
    ctx.assert(
      (nosBlocos.item || "").startsWith("bloco-"),
      `devia estar num bloco, está em ${nosBlocos.item}`,
    );

    // Primeiro Escape: sobe UM nível (blocos → raiz), sessão VIVA.
    // Antes ia direto pra raiz de qualquer profundidade, e a partir do
    // texto o handler do editor engolia a tecla — só Backspace subia.
    await bridge.js(TECLA_NO_FOCO("Escape"));
    await PAUSA(400);
    const esc1 = await bridge.js(FOCO_NAV);
    ctx.assert(esc1.sessao, "o primeiro Escape não podia encerrar a sessão");
    ctx.assert(
      !(esc1.item || "").startsWith("bloco-"),
      `o primeiro Escape devia ter saído dos blocos (ficou em ${esc1.item})`,
    );

    // Segundo Escape: na raiz, encerra.
    await bridge.js(TECLA_NO_FOCO("Escape"));
    await PAUSA(400);
    const esc2 = await bridge.js(FOCO_NAV);
    ctx.assert(!esc2.sessao, "o segundo Escape devia ter encerrado a sessão");
  },
});

cenarios.push({
  nome: "navegação: abrir página de dentro de um grupo pousa nos blocos dela (250)",
  async fn(bridge, ctx) {
    // O caminho relatado na spec: navegar até um embed de ações, entrar
    // nele, acionar um botão que abre outra página. A pilha ficava
    // apontando pro grupo do embed — que não existe mais na página nova
    // — e a próxima seta caía no resgate, cujo último recurso é a RAIZ:
    // o teclado terminava preso na barra superior.
    ctx.escrever(
      `---\ntitle: __uitest\n---\n\nalfa\n\n{{ type: "actions" }}\nbuttons:\n- label: Abrir arquitetura\n  action: open-page\n  path: pages/arquitetura.md\n{{ /actions }}\n`,
    );
    await recarregar(bridge);
    await ctx.abrirPagina(bridge, ctx.nomePagina);
    await ctx.esperar(bridge, "document.querySelector('.editor__bloco')", "o bloco existir");
    await ctx.esperar(bridge, "document.querySelector('[data-nav-group]')", "o embed existir");

    await bridge.js(COMECAR_NAVEGACAO);
    await PAUSA(400);
    await bridge.js(IR_ATE_O_ITEM("editor"));
    await bridge.js(TECLA_NO_FOCO("Enter"));
    await PAUSA(500);

    // Anda pelos blocos até o embed (o item que TAMBÉM é grupo).
    await bridge.js(`(async () => {
      for (let i = 0; i < 8; i++) {
        if (document.activeElement.hasAttribute('data-nav-group')) return true;
        document.activeElement.dispatchEvent(
          new KeyboardEvent('keydown', { key: 'j', bubbles: true, cancelable: true }));
        await new Promise(r => setTimeout(r, 180));
      }
      return false;
    })()`);
    await PAUSA(200);
    await bridge.js(TECLA_NO_FOCO("Enter")); // desce no embed
    await PAUSA(400);
    await bridge.js(TECLA_NO_FOCO("Enter")); // aciona o botão
    await PAUSA(2500);

    const titulo = await bridge.js(`(document.querySelector('.editor__title') || {}).textContent || ''`);
    ctx.assert(/arquitetura/i.test(titulo), `a página não abriu (título: "${titulo}")`);

    const foco = await bridge.js(FOCO_NAV);
    ctx.assert(
      (foco.item || "").startsWith("bloco-"),
      `o teclado devia ter pousado no conteúdo da página aberta, não em "${foco.item}"`,
    );
  },
});
