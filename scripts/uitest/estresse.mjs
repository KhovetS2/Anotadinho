// Bateria de ESTRESSE da interface (ciclo 259).
//
// A suíte principal responde "funciona?". Esta responde outra pergunta:
// "continua respondendo quando o conteúdo cresce?". São coisas
// diferentes, e a segunda não aparece na primeira — os cenários de lá
// usam páginas de três parágrafos, onde um `O(n²)` custa microssegundos.
//
// Fica FORA de `todos`, como a `pendentes.mjs`, por dois motivos: leva
// minutos, e mede tempo — o que a torna sensível a máquina ocupada. Uma
// suíte que às vezes falha sem ninguém ter mexido em nada deixa de ser
// sinal. Roda sob demanda:
//
//   node scripts/uitest/run.mjs --estresse
//
// ## O que os tetos significam
//
// Não são metas de desempenho. São alarmes de mudança de ORDEM,
// calibrados com folga larga pra não disparar por máquina lenta e
// apertados o bastante pra um caminho linear virando quadrático acender
// a luz. Cada cenário imprime o tempo medido junto do teto, então o
// número serve mesmo quando passa.
//
// ## Por que não medem o vault inteiro
//
// Encher o vault de quem usa com 4 mil páginas pra medir a varredura
// seria destrutivo e lento. Essa metade mora onde é barata e
// determinística: `cargo test -p anotadinho-ipc --test estresse --
// --ignored`. Aqui fica o que só existe no DOM — o editor por bloco, a
// digitação, a seleção, a rolagem virtualizada.

import { esperar, recarregarEstavel, abrirPaginaEstavel } from "./bridge.mjs";

const PAUSA = (ms) => new Promise((r) => setTimeout(r, ms));

export const estresse = [];

const SALVAR = `(() => {
  // Mesma trava do ciclo 197: só grava na página de rascunho.
  const titulo = (document.querySelector('.editor__title') || {}).textContent || '';
  if (!titulo.includes('__uitest')) {
    throw new Error('Salvar bloqueado: a página aberta é "' + titulo + '"');
  }
  const b = [...document.querySelectorAll('button')].find(b => b.textContent.trim().startsWith('Salvar'));
  if (b) b.click();
  return !!b;
})()`;

/// Mede quanto uma expressão leva DENTRO da janela, em ms.
///
/// Medir daqui incluiria a ida e volta da ponte, que varia mais que o
/// que se quer medir.
const MEDIR = (expr) => `(() => {
  const t0 = performance.now();
  const r = (() => { ${expr} })();
  return { ms: performance.now() - t0, r };
})()`;

/// Corpo markdown com `n` blocos variados.
///
/// Variados de propósito: uma página de mil parágrafos iguais não
/// exercita o mesmo caminho que uma com listas, títulos e código, e é o
/// segundo caso que as pessoas têm.
function corpoGrande(n) {
  const linhas = [];
  for (let i = 0; i < n; i++) {
    switch (i % 6) {
      case 0: linhas.push(`## Seção ${i}`); break;
      case 1: linhas.push(`Parágrafo ${i} com **negrito**, \`código\` e [[Página ${i % 40}]].`); break;
      case 2: linhas.push(`- item ${i}\n- item ${i}.2\n- item ${i}.3`); break;
      case 3: linhas.push(`> citação ${i}`); break;
      case 4: linhas.push("```rust\nfn f() -> u32 { 1 }\n```"); break;
      default: linhas.push(`Texto simples número ${i}.`); break;
    }
  }
  return linhas.join("\n\n") + "\n";
}

/// `medir` recebe (bridge, ctx) e devolve `{ ms, ... }`; `teto` é em ms.
function estressar(nome, montar, fn) {
  estresse.push({
    nome: `estresse: ${nome}`,
    async fn(bridge, ctx) {
      ctx.escrever(`---\ntitle: __uitest\n---\n${montar()}`);
      await recarregarEstavel(bridge);
      await abrirPaginaEstavel(bridge, ctx.nomePagina);
      await fn(bridge, ctx, {
        /// Falha dizendo o número, não só que passou do teto.
        dentroDoTeto(rotulo, ms, teto) {
          console.log(`      · ${rotulo}: ${Math.round(ms)}ms (teto ${teto}ms)`);
          ctx.assert(
            ms <= teto,
            `${rotulo} levou ${Math.round(ms)}ms, acima do teto de ${teto}ms — ` +
              `provavelmente a complexidade mudou de ordem`,
          );
        },
      });
    },
  });
}

// ── o editor por bloco com muito bloco ──────────────────────────────

estressar("página de 1200 blocos abre e fica editável", () => corpoGrande(1200), async (b, ctx, h) => {
  await esperar(b, `document.querySelectorAll('[data-nav-block]').length > 500`,
    "os blocos renderizarem", 30000);

  const est = await b.js(`(() => {
    const blocos = document.querySelectorAll('[data-nav-block]');
    return {
      n: blocos.length,
      editaveis: [...blocos].filter(e => e.getAttribute('contenteditable') === 'true').length,
      convites: document.querySelectorAll('.editor__bloco--convite').length,
    };
  })()`);
  ctx.assert(est.n > 500, `esperava centenas de blocos, vieram ${est.n}`);
  // A regressão do ciclo 249 em escala: bloco que nasce sem
  // `contenteditable` é bloco morto, e numa página grande o sintoma é
  // "parte da página não deixa escrever".
  ctx.assertEq(est.editaveis, est.n, "há blocos não editáveis na página");
  // O convite ("Digite ou use /") pintando por cima de texto de verdade
  // foi o outro sintoma do 249.
  ctx.assertEq(est.convites, 0, "o texto de convite apareceu em bloco com conteúdo");
});

estressar("digitar no fim de uma página de 1200 blocos", () => corpoGrande(1200), async (b, ctx, h) => {
  await esperar(b, `document.querySelectorAll('[data-nav-block]').length > 500`,
    "os blocos renderizarem", 30000);

  // 60 teclas seguidas sem pausa: é o que separa "digitar é lento" de
  // "digitar trava". Cada tecla dispara o `oninput` do editor, que
  // recompõe estado.
  const r = await b.js(MEDIR(`
    const blocos = document.querySelectorAll('[data-nav-block]');
    const alvo = blocos[blocos.length - 1];
    alvo.focus();
    const sel = getSelection(); const range = document.createRange();
    range.selectNodeContents(alvo); range.collapse(false);
    sel.removeAllRanges(); sel.addRange(range);
    for (let i = 0; i < 60; i++) {
      document.execCommand('insertText', false, 'x');
    }
    return alvo.textContent.length;
  `));
  h.dentroDoTeto("60 teclas no último bloco", r.ms, 4000);
  ctx.assert(r.r > 60, `o texto não entrou no bloco (${r.r} caracteres)`);
});

estressar("salvar uma página de 1200 blocos", () => corpoGrande(1200), async (b, ctx, h) => {
  await esperar(b, `document.querySelectorAll('[data-nav-block]').length > 500`,
    "os blocos renderizarem", 30000);
  const antes = (ctx.ler() || "").length;

  const t0 = Date.now();
  await b.js(SALVAR);
  await PAUSA(2500);
  const ms = Date.now() - t0;
  h.dentroDoTeto("recompor markdown e gravar", ms, 12000);

  const depois = (ctx.ler() || "").length;
  // A guarda do ciclo 248 em escala: recomposição que perde blocos
  // numa página grande é a forma mais cara de perder trabalho.
  ctx.assert(
    depois > antes * 0.9,
    `a gravação encolheu a página de ${antes} pra ${depois} caracteres`,
  );
});

// ── seleção e navegação em escala ───────────────────────────────────

estressar("estender a seleção por 300 blocos", () => corpoGrande(1200), async (b, ctx, h) => {
  await esperar(b, `document.querySelectorAll('[data-nav-block]').length > 500`,
    "os blocos renderizarem", 30000);

  // Shift+seta a partir do modo de navegação — o gesto de verdade
  // (ciclo 251). A primeira versão deste cenário usava Ctrl+A, que NÃO
  // é a forma de selecionar vários blocos: media zero blocos em 2ms e
  // passava com folga num teto de 3 segundos.
  await b.js(`(() => {
    const alvo = document.querySelector('.editor__bloco');
    alvo.focus();
    const r = document.createRange();
    r.selectNodeContents(alvo); r.collapse(false);
    const s = getSelection(); s.removeAllRanges(); s.addRange(r);
    alvo.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true }));
    return true;
  })()`);
  await PAUSA(600);

  // Cada Shift+seta ESTENDE a seleção e repinta o conjunto inteiro: é o
  // caminho onde um repintar de O(selecionados) por tecla vira
  // O(n²) ao longo do gesto.
  const r = await b.js(MEDIR(`
    for (let i = 0; i < 300; i++) {
      const alvo = document.querySelector('.nav-mode__item-active')
        || document.activeElement.closest('.editor__bloco')
        || document.activeElement;
      alvo.dispatchEvent(new KeyboardEvent('keydown', {
        key: 'ArrowDown', shiftKey: true, bubbles: true, cancelable: true }));
    }
    return document.querySelectorAll('.editor__bloco--selecionado').length;
  `));
  h.dentroDoTeto("300 Shift+seta estendendo a seleção", r.ms, 8000);
  // Sem esta asserção o cenário mede o nada e passa sempre: um gesto
  // errado devolve zero elementos em zero milissegundos, e o teto nunca
  // dispara. Um teste de desempenho que não confere o EFEITO é um teto
  // decorativo — foi exatamente o que aconteceu aqui.
  ctx.assert(r.r > 100, `a seleção parou em ${r.r} blocos — o gesto não estendeu`);
});

estressar("rolar uma página de 1200 blocos até o fim", () => corpoGrande(1200), async (b, ctx, h) => {
  await esperar(b, `document.querySelectorAll('[data-nav-block]').length > 500`,
    "os blocos renderizarem", 30000);

  const r = await b.js(MEDIR(`
    // Acha quem ROLA de verdade, subindo do editor: fixar um seletor de
    // classe deu 0ms na primeira versão porque acertava um elemento sem
    // rolagem nenhuma.
    let area = document.querySelector('.editor__wysiwyg');
    while (area && area.scrollHeight <= area.clientHeight + 4) area = area.parentElement;
    area = area || document.scrollingElement;
    let maximo = 0;
    for (let i = 1; i <= 40; i++) {
      area.scrollTop = area.scrollHeight * (i / 40);
      // Força o layout a cada passo: sem isto o navegador junta tudo
      // num recálculo só e a medição não diz nada.
      void area.offsetHeight;
      maximo = Math.max(maximo, area.scrollTop);
    }
    return Math.round(maximo);
  `));
  h.dentroDoTeto("40 saltos de rolagem", r.ms, 4000);
  ctx.assert(r.r > 0, "nada rolou — o cenário mediu um contêiner sem rolagem");
});

// ── embeds grandes ──────────────────────────────────────────────────

const TABELA_GRANDE = (() => {
  const linhas = [];
  for (let i = 0; i < 600; i++) {
    linhas.push(`| Item ${i} | ${["alta", "media", "baixa"][i % 3]} | ${i} |`);
  }
  // `type:` na coluna e não `kind:`, e a classe é `.task-table__table`:
  // a primeira versão deste cenário errou os dois e reprovou por 30s de
  // espera, culpando o app por um fixture inválido meu.
  return `{{ type: "table" }}
columns:
- name: Nome
- name: Prioridade
  type: select
  options: [alta, media, baixa]
- name: N
  type: number
---
| Nome | Prioridade | N |
| --- | --- | --- |
${linhas.join("\n")}
{{ /table }}
`;
})();

estressar("tabela de 600 linhas renderiza e responde", () => TABELA_GRANDE, async (b, ctx, h) => {
  await esperar(b, `!!document.querySelector('.task-table__table tbody tr')`,
    "a tabela renderizar", 30000);

  const r = await b.js(MEDIR(`
    const linhas = document.querySelectorAll('.task-table__table tbody tr');
    // Layout forçado depois de contar: é o custo real de ter isso na
    // tela, não só no DOM.
    void document.body.offsetHeight;
    return linhas.length;
  `));
  h.dentroDoTeto("medir a tabela renderizada", r.ms, 2000);
  ctx.assert(r.r > 100, `esperava centenas de linhas na tabela, vieram ${r.r}`);
});

const CONSULTA_AMPLA = `{{ type: "query" }}
from: pages
where:
- field: type
  op: exists
view: table
columns:
- type
- tags
- status
- prioridade
- title
{{ /query }}
`;

estressar("consulta sobre o vault inteiro, 5 colunas", () => CONSULTA_AMPLA, async (b, ctx, h) => {
  await esperar(b, `!!document.querySelector('.query-embed__table td')`,
    "a consulta renderizar", 30000);

  // As sugestões de célula (`<datalist>`) percorrem o índice INTEIRO uma
  // vez por coluna, a cada render. Este é o cenário que expõe isso: 5
  // colunas, uma delas (`title`) com um valor distinto por página.
  const r = await b.js(MEDIR(`
    const el = document.querySelector('.query-embed');
    // Um clique numa célula entra em edição e força o re-render que
    // recalcula as sugestões.
    const td = document.querySelector('.query-embed__table tbody td');
    if (td) td.click();
    void el.offsetHeight;
    return document.querySelectorAll('.query-embed__table tbody tr').length;
  `));
  h.dentroDoTeto("entrar em edição numa célula", r.ms, 3000);
  await b.js(`document.activeElement?.blur(); true`);
});

// ── troca de contexto sob pressão ───────────────────────────────────

estressar("trocar de aba repetidamente com página grande aberta", () => corpoGrande(800), async (b, ctx, h) => {
  await esperar(b, `document.querySelectorAll('[data-nav-block]').length > 300`,
    "os blocos renderizarem", 30000);

  // O ciclo 249 mostrou que ir e voltar CONSERTAVA um bloco morto —
  // o que quer dizer que a ida e a volta refazem trabalho. Fazer isso
  // dez vezes seguidas é o que mostra se sobra estado pendurado.
  const t0 = Date.now();
  for (let i = 0; i < 5; i++) {
    await b.js(`(() => {
      const abas = document.querySelectorAll('.tab-bar__tab');
      if (abas.length > 1) abas[(${i} + 1) % abas.length].click();
      return abas.length;
    })()`);
    await PAUSA(300);
    await abrirPaginaEstavel(b, ctx.nomePagina);
  }
  h.dentroDoTeto("5 idas e voltas", Date.now() - t0, 30000);

  const est = await b.js(`(() => {
    const blocos = document.querySelectorAll('[data-nav-block]');
    return {
      n: blocos.length,
      editaveis: [...blocos].filter(e => e.getAttribute('contenteditable') === 'true').length,
    };
  })()`);
  ctx.assertEq(est.editaveis, est.n, "depois das idas e voltas há bloco não editável");
});

// ── o caso patológico: uma linha muito longa ────────────────────────

estressar("parágrafo único de 200 mil caracteres", () => "a".repeat(200_000) + "\n", async (b, ctx, h) => {
  await esperar(b, `!!document.querySelector('[data-nav-block]')`, "o bloco renderizar", 30000);

  // Uma linha enorme é diferente de muitas linhas: quebra de linha,
  // medição de texto e seleção passam a dominar, e é o formato que um
  // arquivo colado de fora costuma ter.
  const r = await b.js(MEDIR(`
    const bloco = document.querySelector('[data-nav-block]');
    bloco.focus();
    const sel = getSelection(); const range = document.createRange();
    range.selectNodeContents(bloco); range.collapse(false);
    sel.removeAllRanges(); sel.addRange(range);
    document.execCommand('insertText', false, 'FIM');
    return bloco.textContent.length;
  `));
  h.dentroDoTeto("digitar no fim de uma linha de 200 mil", r.ms, 5000);
  ctx.assert(r.r >= 200_000, `o bloco perdeu texto: ${r.r} caracteres`);
});
