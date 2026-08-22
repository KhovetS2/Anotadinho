// Fluxo de trabalho: spec → proposta → execução (ciclo 201+).
//
// A garantia que estes cenários protegem é a que dá segurança ao
// acoplamento com agentes: **nenhuma etapa avança sozinha**, e não
// existe caminho da UI que pule a revisão.

import { esperar, recarregarEstavel, abrirPaginaEstavel } from "./bridge.mjs";

const PAUSA = (ms) => new Promise((r) => setTimeout(r, ms));

export const fluxo = [];

const SALVAR = `(() => {
  const titulo = (document.querySelector('.editor__title') || {}).textContent || '';
  if (!titulo.includes('__uitest')) {
    throw new Error('Salvar bloqueado: a página aberta é "' + titulo + '"');
  }
  const b = [...document.querySelectorAll('button')].find(b => b.textContent.trim().startsWith('Salvar'));
  if (b) b.click();
  return !!b;
})()`;

const ETAPA = `(() => ({
  atual: (document.querySelector('.fluxo__etapa') || {}).textContent || null,
  botoes: [...document.querySelectorAll('.fluxo__acoes button')].map(b => b.textContent.trim()),
  passoAtual: (document.querySelector('.fluxo__passo--atual') || {}).textContent || null,
}))()`;

const CLICAR_ETAPA = (rotulo) => `(() => {
  const b = [...document.querySelectorAll('.fluxo__acoes button')]
    .find(x => x.textContent.trim() === ${JSON.stringify(rotulo)});
  if (!b) return false;
  b.click();
  return true;
})()`;

function caso(nome, corpo, fn) {
  fluxo.push({
    nome: `fluxo: ${nome} (201)`,
    async fn(bridge, ctx) {
      ctx.escrever(`---\ntitle: __uitest\nstatus: rascunho\n---\n${corpo}`);
      await recarregarEstavel(bridge);
      await abrirPaginaEstavel(bridge, ctx.nomePagina);
      await esperar(bridge, "document.querySelector('.fluxo')", "o embed de fluxo");
      await fn(bridge, ctx, {
        etapa: () => bridge.js(ETAPA),
        salvarELer: async () => {
          await bridge.js(SALVAR);
          await PAUSA(1000);
          return ctx.ler() || "";
        },
      });
    },
  });
}

const RASCUNHO = '{{ type: "fluxo" }}\nartefato: spec\netapa: rascunho\n{{ /fluxo }}\n';

caso("mostra a etapa atual e a trilha", RASCUNHO, async (b, ctx, h) => {
  const e = await h.etapa();
  ctx.assertEq(e.atual, "Rascunho", "a etapa atual");
  ctx.assertEq(e.passoAtual, "Rascunho", "o passo destacado na trilha");
});

caso("da revisão NÃO existe botão pra pular pra execução", RASCUNHO, async (b, ctx, h) => {
  // É a regra inteira do ciclo: de rascunho só dá pra ir pra revisão.
  const e = await h.etapa();
  ctx.assertEq(e.botoes.join(","), "Em revisão", `botões oferecidos: ${e.botoes.join(", ")}`);
  ctx.assert(
    !e.botoes.some((t) => /execução|aprovada|concluída/i.test(t)),
    `a UI ofereceu pular a revisão: ${e.botoes.join(", ")}`,
  );
});

caso("avançar grava a etapa no embed e o status no frontmatter", RASCUNHO, async (b, ctx, h) => {
  await b.js(CLICAR_ETAPA("Em revisão"));
  await PAUSA(900);
  ctx.assertEq((await h.etapa()).atual, "Em revisão", "a etapa devia ter avançado");

  const md = await h.salvarELer();
  ctx.assert(md.includes("etapa: em-revisao"), `o embed não gravou a etapa:\n${md}`);
  ctx.assert(
    /^status: em-revisao$/m.test(md),
    `o frontmatter não foi espelhado — as consultas não enxergam:\n${md}`,
  );
});

caso("caminho completo até concluída", RASCUNHO, async (b, ctx, h) => {
  for (const destino of ["Em revisão", "Aprovada", "Em execução", "Concluída"]) {
    const ok = await b.js(CLICAR_ETAPA(destino));
    ctx.assertEq(ok, true, `não havia botão pra "${destino}"`);
    await PAUSA(800);
    ctx.assertEq((await h.etapa()).atual, destino, `parou em "${destino}"`);
  }
  const md = await h.salvarELer();
  ctx.assert(md.includes("etapa: concluida"), `não chegou em concluída:\n${md}`);
});

caso(
  "etapa depois da aprovação avisa que é fechada pra agente",
  '{{ type: "fluxo" }}\nartefato: spec\netapa: aprovada\n{{ /fluxo }}\n',
  async (b, ctx, h) => {
    ctx.assertEq(
      await b.js(`!!document.querySelector('.fluxo__aviso')`),
      true,
      "faltou o aviso de etapa fechada pra edição automática",
    );
  },
);

caso(
  "proposta mostra o link pra origem",
  '{{ type: "fluxo" }}\nartefato: proposta\netapa: rascunho\norigem: pages/produto/missao.md\n{{ /fluxo }}\n',
  async (b, ctx, h) => {
    ctx.assertEq(
      await b.js(`!!document.querySelector('.fluxo__origem')`),
      true,
      "proposta sem botão de origem",
    );
    await b.js(`(() => { document.querySelector('.fluxo__origem').click(); return true; })()`);
    await esperar(
      b,
      `/miss/i.test((document.querySelector('.editor__title')||{}).textContent || '')`,
      "a página de origem abrir",
      10000,
    );
  },
);
