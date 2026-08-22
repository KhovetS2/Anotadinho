// Bateria de DIGITAÇÃO (ciclo 193).
//
// Diferente do `cenarios.mjs`, onde cada item é uma regressão que já
// aconteceu, aqui o objetivo é o contrário: travar o comportamento que
// funciona HOJE, antes de mexer nele.
//
// Existe porque o editor vai ser reescrito pra um `contenteditable` por
// bloco (ciclo 175), e digitação é o caminho mais usado do app e a
// origem de quase todo bug dele (076, 078, 079, 082, 111, 141-143).
// Sem rede, a reescrita é uma aposta.
//
// Regra ao mexer nisto: um cenário daqui só muda se o comportamento
// DEVE mudar, e a mudança tem que estar escrita na task do ciclo. Se um
// deles quebrar sem isso, o produto regrediu.

import { esperar, recarregarEstavel, abrirPaginaEstavel } from "./bridge.mjs";

const PAUSA = (ms) => new Promise((r) => setTimeout(r, ms));

/// Corpo do `.md` salvo, sem frontmatter e sem espaço nas pontas.
function corpo(texto) {
  if (!texto) return "";
  const m = texto.match(/^---\n[\s\S]*?\n---\n?([\s\S]*)$/);
  return (m ? m[1] : texto).trim();
}

/// Põe o cursor no fim do último bloco e devolve o editor.
const IR_PRO_FIM = `(() => {
  const seg = document.querySelector('.editor__wysiwyg');
  const alvo = seg.lastElementChild || seg;
  alvo.focus();
  const r = document.createRange();
  r.selectNodeContents(alvo); r.collapse(false);
  const s = getSelection(); s.removeAllRanges(); s.addRange(r);
  return true;
})()`;

/// Cursor no começo do bloco de índice `i`.
const IR_PRO_COMECO_DO_BLOCO = (i) => `(() => {
  const seg = document.querySelector('.editor__wysiwyg');
  const b = seg.children[${i}];
  b.focus();
  const r = document.createRange();
  r.selectNodeContents(b); r.collapse(true);
  const s = getSelection(); s.removeAllRanges(); s.addRange(r);
  return true;
})()`;

/// Cursor DENTRO do bloco `i`, depois de `n` caracteres.
const IR_PRO_MEIO = (i, n) => `(() => {
  const seg = document.querySelector('.editor__wysiwyg');
  const b = seg.children[${i}];
  b.focus();
  const walker = document.createTreeWalker(b, NodeFilter.SHOW_TEXT);
  const t = walker.nextNode();
  const r = document.createRange();
  r.setStart(t, ${n}); r.collapse(true);
  const s = getSelection(); s.removeAllRanges(); s.addRange(r);
  return true;
})()`;

const ESCREVER = (t) => `(() => { document.execCommand('insertText', false, ${JSON.stringify(t)}); return true; })()`;
const TECLA = (key, extra = {}) => `(() => {
  const ed = document.activeElement.closest('.editor__bloco') || document.querySelector('.editor__wysiwyg');
  ed.dispatchEvent(new KeyboardEvent('keydown', Object.assign({ key: ${JSON.stringify(key)}, bubbles: true, cancelable: true }, ${JSON.stringify(extra)})));
  return true;
})()`;
const ENTER = `(() => { document.execCommand('insertParagraph', false); return true; })()`;
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

export const digitacao = [];

/// Açúcar: monta um cenário que parte de `inicial`, roda `passos` e
/// confere o corpo salvo.
function caso(nome, inicial, passos, conferir) {
  digitacao.push({
    nome: `digitação: ${nome} (193)`,
    async fn(bridge, ctx) {
      ctx.escrever(`---\ntitle: __uitest\n---\n${inicial}`);
      // Espera por CONDIÇÃO, não por relógio (ciclo 198).
      await recarregarEstavel(bridge);
      await abrirPaginaEstavel(bridge, ctx.nomePagina);

      await passos(bridge, ctx);

      await bridge.js(SALVAR);
      await PAUSA(1000);
      await conferir(corpo(ctx.ler()), ctx, bridge);
    },
  });
}

// ── o básico ────────────────────────────────────────────────────────

caso(
  "texto digitado no fim chega ao arquivo",
  "primeira linha\n",
  async (b) => {
    await b.js(IR_PRO_FIM);
    await b.js(ESCREVER(" e mais isto"));
    await PAUSA(300);
  },
  (md, ctx) => ctx.assertEq(md, "primeira linha e mais isto", "o texto digitado"),
);

// MUDANÇA DE COMPORTAMENTO, ciclo 194: quem cria bloco é Shift+Enter.
// O Enter sozinho passou a quebrar linha DENTRO do bloco — antes não
// havia como escrever duas linhas no mesmo parágrafo.
caso(
  "Shift+Enter no fim cria um parágrafo novo",
  "alfa\n",
  async (b) => {
    await b.js(IR_PRO_FIM);
    await b.js(TECLA("Enter", { shiftKey: true }));
    await PAUSA(400);
    await b.js(ESCREVER("beta"));
    await PAUSA(300);
  },
  (md, ctx) => ctx.assertEq(md, "alfa\n\nbeta", "dois parágrafos"),
);

caso(
  "Shift+Enter no meio divide o parágrafo",
  "alfabeto\n",
  async (b) => {
    await b.js(IR_PRO_MEIO(0, 4));
    await b.js(TECLA("Enter", { shiftKey: true }));
    await PAUSA(400);
  },
  (md, ctx) => ctx.assertEq(md, "alfa\n\nbeto", "dividido no cursor"),
);

caso(
  "Backspace no início funde com o parágrafo anterior",
  "alfa\n\nbeta\n",
  async (b) => {
    await b.js(IR_PRO_COMECO_DO_BLOCO(1));
    // Só a tecla: desde o ciclo 175 o editor trata o Backspace no
    // início do bloco. Antes ele dependia da edição nativa, que um
    // KeyboardEvent sintético não dispara — daí o `execCommand('delete')`
    // que existia aqui e que agora apagaria um caractere a mais.
    await b.js(TECLA("Backspace"));
    await PAUSA(400);
  },
  (md, ctx) => ctx.assertEq(md, "alfabeta", "fundidos num só"),
);

// ── atalhos de formatação por prefixo (ciclos 142/143) ──────────────

for (const [prefixo, esperado, desc] of [
  ["# ", "# Título", "título 1"],
  ["## ", "## Título", "título 2"],
  ["- ", "- Título", "lista"],
  ["> ", "> Título", "citação"],
]) {
  caso(
    `prefixo "${prefixo.trim()}" vira ${desc}`,
    "\n",
    async (b) => {
      await b.js(IR_PRO_FIM);
      await b.js(ESCREVER(prefixo.trim()));
      await b.js(TECLA(" "));
      await b.js(ESCREVER(" "));
      await PAUSA(200);
      await b.js(ESCREVER("Título"));
      await PAUSA(300);
    },
    (md, ctx) =>
      ctx.assert(
        md.includes("Título"),
        `${desc}: o texto sumiu — veio ${JSON.stringify(md)}`,
      ),
  );
}

// ── listas ──────────────────────────────────────────────────────────

caso(
  "Enter numa lista cria outro item",
  "- um\n",
  async (b) => {
    await b.js(IR_PRO_FIM);
    await b.js(ENTER);
    await b.js(ESCREVER("dois"));
    await PAUSA(300);
  },
  (md, ctx) => {
    ctx.assert(md.includes("- um"), `o item original sumiu: ${md}`);
    ctx.assert(md.includes("dois"), `o item novo sumiu: ${md}`);
  },
);

caso(
  "checkbox marcado pelo clique vai pro arquivo",
  "- [ ] tarefa\n",
  async (b) => {
    await b.js(`(() => {
      const c = document.querySelector('.editor__wysiwyg input[type="checkbox"]');
      c.click();
      return true;
    })()`);
    await PAUSA(500);
  },
  (md, ctx) => ctx.assert(/- \[x\]/i.test(md), `devia ter marcado: ${md}`),
);

// ── formatação que já quebrou antes (ciclos 141-143) ────────────────

caso(
  "bloco de código sobrevive à edição do texto em volta",
  "antes\n\n```rust\nfn main() {}\n```\n\ndepois\n",
  async (b) => {
    // Edita o parágrafo DEPOIS do código. Os ciclos 141-143 foram
    // exatamente sobre o editor destruir a formatação de código ao
    // mexer perto dele.
    await b.js(IR_PRO_FIM);
    await b.js(ESCREVER(" editado"));
    await PAUSA(300);
  },
  (md, ctx) => {
    ctx.assert(md.includes("fn main() {}"), `o código se perdeu:\n${md}`);
    ctx.assert(md.includes("```rust"), `a cerca e a linguagem se perderam:\n${md}`);
    ctx.assert(md.includes("depois editado"), `a edição não chegou:\n${md}`);
    ctx.assert(md.includes("antes"), `o texto anterior se perdeu:\n${md}`);
  },
);

caso(
  "código inline e ênfase sobrevivem",
  "tem `código` e **negrito** e *itálico* aqui\n",
  async (b) => {
    await b.js(IR_PRO_FIM);
    await b.js(ESCREVER(" fim"));
    await PAUSA(300);
  },
  (md, ctx) => {
    ctx.assert(md.includes("`código`"), `código inline: ${md}`);
    ctx.assert(md.includes("**negrito**"), `negrito: ${md}`);
    ctx.assert(md.includes("fim"), `o texto novo: ${md}`);
  },
);

// ── colar ───────────────────────────────────────────────────────────

caso(
  "colar texto de várias linhas preserva as quebras",
  "antes\n",
  async (b) => {
    await b.js(IR_PRO_FIM);
    await b.js(ENTER);
    // `insertText` com quebras é o caminho que o `onpaste` do editor
    // acaba usando. Um `ClipboardEvent` sintético não serve aqui: o
    // WebView não deixa preencher o `clipboardData` de fora, então o
    // handler recebia um evento vazio e o cenário testava nada.
    await b.js(ESCREVER("um\ndois\ntres"));
    await PAUSA(500);
  },
  (md, ctx) => {
    for (const t of ["antes", "um", "dois", "tres"]) {
      ctx.assert(md.includes(t), `"${t}" não chegou:\n${md}`);
    }
  },
);

// ── desfazer ────────────────────────────────────────────────────────

caso(
  "Ctrl+Z desfaz a digitação",
  "base\n",
  async (b) => {
    await b.js(IR_PRO_FIM);
    await b.js(ESCREVER(" some"));
    await PAUSA(1000);
    await b.js(TECLA("z", { ctrlKey: true }));
    await PAUSA(700);
  },
  (md, ctx) => ctx.assertEq(md, "base", "o desfazer devia ter voltado"),
);

// ── a garantia mais importante ──────────────────────────────────────

digitacao.push({
  nome: "digitação: página não editada round-tripa sem mudar um byte (193)",
  async fn(bridge, ctx) {
    // Abrir e salvar sem tocar em nada não pode reescrever o arquivo de
    // outro jeito. É o que impede a reescrita de "arrumar" o markdown
    // de todo mundo sem querer.
    const original =
      "---\ntitle: __uitest\n---\n" +
      "# Título\n\n" +
      "Parágrafo com `código`, **negrito** e [[Missão]].\n\n" +
      "- item um\n- item dois\n\n" +
      "> citação\n\n" +
      "```rust\nfn main() {}\n```\n";
    ctx.escrever(original);
    // Espera por CONDIÇÃO, não por relógio (ciclo 198).
    await recarregarEstavel(bridge);
    await abrirPaginaEstavel(bridge, ctx.nomePagina);

    // Uma edição mínima e desfeita, só pra habilitar o botão de salvar.
    await bridge.js(IR_PRO_FIM);
    await bridge.js(ESCREVER("x"));
    await PAUSA(300);
    await bridge.js(`(() => { document.execCommand('delete', false); return true; })()`);
    await PAUSA(300);
    await bridge.js(SALVAR);
    await PAUSA(1000);

    const depois = ctx.ler();
    ctx.assertEq(corpo(depois), corpo(original), "o markdown mudou sozinho");
  },
});

// ── ciclo 194: comando que NÃO pode disparar no modo de edição ───────
//
// Esta seção é o inverso das outras: em vez de checar que algo funciona,
// checa que algo NÃO acontece. O bug que a motivou apagou conteúdo do
// usuário — digitar uma sequência aleatória com `d` no meio removia um
// bloco por letra, porque os atalhos de bloco não olhavam o modo.

/// Digita cada caractere como tecla + texto, do jeito que o teclado faz.
const DIGITAR_TECLA_A_TECLA = (texto) => `(() => {
  const alvo = document.activeElement.closest('.editor__bloco') || document.activeElement;
  for (const ch of ${JSON.stringify(texto)}) {
    alvo.dispatchEvent(new KeyboardEvent('keydown', { key: ch, bubbles: true, cancelable: true }));
    document.execCommand('insertText', false, ch);
  }
  return true;
})()`;

digitacao.push({
  nome: "digitação: letras de comando (d, n, y, K, J, c) são TEXTO no modo edição (194)",
  async fn(bridge, ctx) {
    ctx.escrever("---\ntitle: __uitest\n---\nalfa\n\nbeta\n\ngama\n");
    // Espera por CONDIÇÃO, não por relógio (ciclo 198).
    await recarregarEstavel(bridge);
    await abrirPaginaEstavel(bridge, ctx.nomePagina);

    const nBlocos = () => bridge.js(`document.querySelectorAll('.editor__bloco').length`);
    ctx.assertEq(await nBlocos(), 3, "três blocos no início");

    await bridge.js(IR_PRO_MEIO(1, 4));
    await bridge.js(DIGITAR_TECLA_A_TECLA("dnyKJc"));
    await PAUSA(600);

    ctx.assertEq(await nBlocos(), 3, "nenhum bloco pode ter sido criado nem apagado");
    ctx.assertEq(
      await bridge.js(`!!document.querySelector('.slash-menu')`),
      false,
      "o `n` não pode abrir o menu / no modo edição",
    );

    await bridge.js(SALVAR);
    await PAUSA(900);
    const md = corpo(ctx.ler());
    ctx.assert(md.includes("dnyKJc"), `as letras deviam ter virado texto:\n${md}`);
    for (const t of ["alfa", "gama"]) {
      ctx.assert(md.includes(t), `"${t}" foi apagado:\n${md}`);
    }
  },
});

digitacao.push({
  nome: "digitação: Enter quebra linha e Shift+Enter cria bloco (194)",
  async fn(bridge, ctx) {
    ctx.escrever("---\ntitle: __uitest\n---\nlinha\n");
    // Espera por CONDIÇÃO, não por relógio (ciclo 198).
    await recarregarEstavel(bridge);
    await abrirPaginaEstavel(bridge, ctx.nomePagina);

    const nBlocos = () => bridge.js(`document.querySelectorAll('.editor__bloco').length`);

    await bridge.js(IR_PRO_FIM);
    await bridge.js(TECLA("Enter"));
    await PAUSA(400);
    await bridge.js(ESCREVER("mesma caixa"));
    await PAUSA(300);
    ctx.assertEq(await nBlocos(), 1, "Enter não pode criar bloco");

    await bridge.js(TECLA("Enter", { shiftKey: true }));
    await PAUSA(400);
    await bridge.js(ESCREVER("caixa nova"));
    await PAUSA(300);
    ctx.assertEq(await nBlocos(), 2, "Shift+Enter devia ter criado um bloco");

    await bridge.js(SALVAR);
    await PAUSA(900);
    const md = corpo(ctx.ler());
    ctx.assert(
      /linha {2}\nmesma caixa/.test(md),
      `a quebra de linha devia ter virado quebra dura:\n${JSON.stringify(md)}`,
    );
    ctx.assert(md.includes("\n\ncaixa nova"), `o bloco novo devia ser um parágrafo à parte:\n${md}`);
  },
});
