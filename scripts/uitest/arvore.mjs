// Bateria da ÁRVORE: o modelo contra a tela (ciclo 272).
//
// Passo 3 da unificação dos modelos de bloco. A `Unidade` já sabe ler
// markdown (ciclo 271), mas o editor continua derivando tudo do DOM.
// Antes de inverter — fazer a árvore virar a fonte da verdade — é
// preciso saber ONDE os dois discordam hoje.
//
// Esta bateria roda os dois em paralelo e compara: o comando
// `arvore_da_pagina` devolve o que o MODELO diz que a página tem, e o
// DOM diz o que a TELA mostra. Cada divergência é um lugar onde a
// inversão quebraria alguma coisa — e é muito mais barato descobrir
// agora do que depois.
//
// Fica fora de `todos` porque é uma medição de MIGRAÇÃO, não uma guarda
// de comportamento: enquanto o modelo não for a verdade, divergir não é
// necessariamente defeito. Roda com:
//
//   node scripts/uitest/run.mjs --arvore

import { recarregarEstavel, abrirPaginaEstavel, esperar } from "./bridge.mjs";

export const arvore = [];

const PAUSA = (ms) => new Promise((r) => setTimeout(r, ms));

/// O que o DOM mostra, no mesmo vocabulário do `Tipo::resumo` do núcleo.
///
/// A tradução mora aqui, do lado da tela, porque é a tela que tem tags
/// HTML — o modelo não sabe o que é um `<h2>`.
const DOM_RESUMO = `(() => {
  const nome = (el) => {
    if (el.getAttribute('data-nav-block') === 'embed') {
      // O tipo do embed vem do componente lá dentro, que carrega a
      // classe. Sem ele dá pra saber que É embed, não QUAL.
      const raiz = el.querySelector('[data-nav-group]');
      const cls = raiz ? [...raiz.classList] : [];
      // As classes de raiz, conferidas uma a uma no componente — três
      // dos dez eu tinha chutado errado, e o resultado foi 'embed:?'
      // aparecendo como se fosse divergência do modelo.
      const conhecidos = {
        'calendar-grid': 'calendar', 'embed-table': 'table', 'embed-kanban': 'kanban',
        'callout': 'callout', 'query-embed': 'query', 'gallery': 'gallery',
        'timeline': 'timeline', 'columns-embed': 'columns', 'fluxo': 'fluxo',
        'actions-embed': 'actions',
      };
      for (const c of cls) if (conhecidos[c]) return 'embed:' + conhecidos[c];
      return 'embed:?';
    }
    const tag = el.tagName.toLowerCase();
    if (/^h[1-6]$/.test(tag)) return 'titulo' + tag[1];
    if (tag === 'ul' || tag === 'ol') return 'lista';
    if (tag === 'li') return 'item';
    if (tag === 'blockquote') return 'citacao';
    if (tag === 'pre') return 'codigo';
    if (tag === 'hr') return 'vazia';
    return 'paragrafo';
  };
  return [...document.querySelectorAll('[data-nav-block]')].map(nome);
})()`;

/// O que o modelo diz, pelo comando do backend.
const MODELO = (caminho) => `window.__TAURI_INTERNALS__.invoke('arvore_da_pagina', {
  vaultPath: JSON.parse(localStorage.getItem('anotadinho.vault_path')),
  pagePath: ${JSON.stringify(caminho)},
})`;

/// Divergências CONHECIDAS, tiradas da comparação antes de julgar.
///
/// Cada uma é uma decisão em aberto, não um defeito escondido — e cada
/// uma tem um cenário próprio que AFIRMA que ela ainda existe, lá
/// embaixo. Quando o editor mudar, aquele cenário falha e avisa que a
/// permissão aqui pode sair.
///
/// **Estava vazia? Não.** Havia uma, e ela SAIU no ciclo 273: o modelo
/// tratava cada `- item` como unidade e o editor marcava o `<ul>`
/// inteiro. O cenário que afirmava a divergência reprovou assim que os
/// itens viraram blocos, com a mensagem dizendo o que fazer. Foi pra
/// isso que ele existia.
function semDivergenciasConhecidas(resumos) {
    return resumos;
}

/// Compara os dois e devolve a primeira divergência legível.
function primeiraDivergencia(modelo, dom) {
  const n = Math.max(modelo.length, dom.length);
  for (let i = 0; i < n; i++) {
    if (modelo[i] !== dom[i]) {
      return `posição ${i}: modelo diz ${JSON.stringify(modelo[i] ?? "(nada)")}, ` +
        `tela mostra ${JSON.stringify(dom[i] ?? "(nada)")}`;
    }
  }
  return null;
}

function comparar(nome, md) {
  arvore.push({
    nome: `árvore: ${nome} (272)`,
    async fn(bridge, ctx) {
      ctx.escrever(md);
      await recarregarEstavel(bridge);
      await abrirPaginaEstavel(bridge, ctx.nomePagina);
      await PAUSA(300);

      const dom = await bridge.js(DOM_RESUMO);
      const modelo = await bridge.js(MODELO(ctx.pagina));

      const erro = primeiraDivergencia(semDivergenciasConhecidas(modelo), dom);
      ctx.assert(
        !erro,
        `${erro}\n  modelo: ${JSON.stringify(modelo)}\n  tela:   ${JSON.stringify(dom)}`,
      );
    },
  });
}

const FM = "---\ntitle: __uitest\n---\n";

comparar("texto comum", FM + "# Título\n\nUm parágrafo.\n\n> citação\n");
comparar("os seis níveis de título", FM + "# a\n\n## b\n\n### c\n\n#### d\n\n##### e\n\n###### f\n");
comparar("lista com itens", FM + "- um\n- dois\n- três\n");
comparar("código não vira outra coisa", FM + "antes\n\n```rust\n# não é título\n```\n\ndepois\n");
comparar("régua horizontal", FM + "a\n\n---\n\nb\n");
comparar(
  "embed entre parágrafos",
  FM + 'antes\n\n{{ type: "callout" }}\nvariant: info\nbody: |\n  oi\n{{ /callout }}\n\ndepois\n',
);
comparar(
  "dois embeds diferentes",
  FM +
    '{{ type: "table" }}\ncolumns:\n- name: A\n---\n| A |\n| --- |\n| x |\n{{ /table }}\n\n' +
    'meio\n\n{{ type: "kanban" }}\ncolumns:\n- Fazendo\nitems:\n- title: Card\n  column: Fazendo\n{{ /kanban }}\n',
);
comparar("mistura de tudo", FM + "# t\n\np\n\n- i\n- j\n\n> c\n\n```\nx\n```\n\n---\n\nfim\n");

// ── as páginas de VERDADE ────────────────────────────────────────────
//
// Os casos acima são meus, e um fixture escrito por quem implementa
// contém só o que ele lembrou. O ciclo 271 aprendeu isso do jeito
// difícil: 4 páginas do vault reprovaram um parser que passava em 20
// testes unitários. Aqui as páginas reais entram de novo, agora
// atravessando o EDITOR.

const REAIS = ["incio", "Ciclos", "Painel", "Assets", "Tags"];

for (const nome of REAIS) {
  arvore.push({
    nome: `árvore: a página real "${nome}" (272)`,
    async fn(bridge, ctx) {
      await recarregarEstavel(bridge);
      let abriu = true;
      try {
        await abrirPaginaEstavel(bridge, nome);
      } catch {
        abriu = false;
      }
      if (!abriu) {
        // Página que não existe neste vault não é falha da árvore.
        return;
      }
      await PAUSA(400);

      // A aba ATIVA, e não a primeira. O seletor `a, b` devolve o
      // primeiro que casar com QUALQUER um dos dois — e a primeira aba
      // é a home, então a versão anterior comparava o modelo de
      // `incio.md` com o DOM de outra página. Cinco falhas que eram
      // minhas, não da árvore.
      const caminho = await bridge.js(
        `(document.querySelector('.tab-bar__tab--active')
          || document.querySelector('.tab-bar__tab'))?.dataset?.path || null`,
      );
      if (!caminho) return;

      // Página TIPADA (`type: kanban`, `tags`, `assets`, `graph`...) não
      // passa pelo editor: o app desenha uma tela própria, sem bloco
      // nenhum. O modelo diz que o arquivo tem uma lista, e tem — mas
      // ninguém a renderiza como bloco.
      //
      // É achado do ciclo 272 e vale escrever: a árvore só descreve as
      // páginas de EDITOR. Quando ela virar a fonte da verdade, as
      // tipadas continuam sendo outra coisa.
      const temEditor = await bridge.js(`!!document.querySelector('.editor__wysiwyg')`);
      if (!temEditor) return;

      const dom = await bridge.js(DOM_RESUMO);
      const modelo = await bridge.js(MODELO(caminho));
      const erro = primeiraDivergencia(semDivergenciasConhecidas(modelo), dom);
      ctx.assert(
        !erro,
        `${caminho}: ${erro}\n  modelo: ${JSON.stringify(modelo.slice(0, 12))}\n  tela:   ${JSON.stringify(dom.slice(0, 12))}`,
      );
    },
  });
}

// ── a divergência que fechou ─────────────────────────────────────────

arvore.push({
  nome: "árvore: os itens de lista são blocos nos DOIS lados (273)",
  async fn(bridge, ctx) {
    // Este cenário nasceu ao contrário: afirmava que o editor NÃO
    // marcava os itens, pra avisar no dia em que passasse a marcar.
    // Passou (ciclo 273), ele reprovou com a mensagem certa, e agora
    // afirma o acordo — que é o que protege contra a regressão.
    ctx.escrever(FM + "- um\n- dois\n- três\n");
    await recarregarEstavel(bridge);
    await abrirPaginaEstavel(bridge, ctx.nomePagina);
    await PAUSA(300);

    const dom = await bridge.js(DOM_RESUMO);
    const modelo = await bridge.js(MODELO(ctx.pagina));

    ctx.assertEq(
      dom.filter((r) => r === "item").length,
      3,
      "o editor voltou a tratar a lista como um bloco só",
    );
    ctx.assertEq(
      modelo.filter((r) => r === "item").length,
      3,
      "o modelo deixou de enxergar os itens",
    );
    ctx.assertEq(
      JSON.stringify(modelo),
      JSON.stringify(dom),
      "modelo e tela discordam sobre a lista",
    );
  },
});
