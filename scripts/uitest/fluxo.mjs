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

// ── ciclo 202: execução do agente externo ────────────────────────────
//
// Usa um agente de MENTIRA (`scripts/uitest/agente-falso.sh`) em vez do
// claude/codex de verdade: o que se testa aqui é o contrato — o prompt
// chega inteiro, a saída volta, o timeout mata, a falha é reportada — e
// não a qualidade da resposta de um modelo.

const FALSO = new URL("./agente-falso.sh", import.meta.url).pathname;

/// Chama a execução direto, sem passar pela UI, e espera terminar.
///
/// A execução deixou de ser bloqueante no ciclo 213: `iniciar_agente`
/// volta na hora e o resultado é buscado com `estado_agente`. Este
/// helper reconstitui a espera pros cenários, que continuam querendo
/// afirmar sobre o RESULTADO.
///
/// Cada chamada usa uma conversa própria: o registro é por conversa, e
/// dois cenários na mesma chave brigariam por "já existe execução em
/// andamento".
const RODAR = (args, prompt, timeout = 30) => `(async () => {
  const conversa = 'pages/__uitest-agente-' + Math.random().toString(36).slice(2) + '.md';
  try {
    await window.__TAURI_INTERNALS__.invoke('iniciar_agente', {
      adaptador: {
        nome: 'falso',
        binario: ${JSON.stringify(FALSO)},
        args: ${JSON.stringify(args)},
        cwd: '',
        timeout_s: ${timeout},
        formato: 'texto',
      },
      prompt: ${JSON.stringify(prompt)},
      vaultPath: '/home/elis/Anotadinho',
      conversaPath: conversa,
    });
  } catch (e) {
    return { ok: false, erro: String(e) };
  }
  // O limite aqui é do TESTE, não do agente: o timeout do adaptador é
  // quem decide o caso de "passou do tempo", e precisa caber dentro.
  const limite = Date.now() + (${timeout} + 10) * 1000;
  while (Date.now() < limite) {
    const e = await window.__TAURI_INTERNALS__.invoke('estado_agente', { conversaPath: conversa });
    if (!e) return { ok: false, erro: 'a execução sumiu do registro' };
    if (e.estado === 'rodando') { await new Promise(r => setTimeout(r, 150)); continue; }
    if (e.estado === 'concluido') return { ok: true, saida: e.texto };
    return { ok: false, erro: e.erro || e.estado };
  }
  return { ok: false, erro: 'o cenário desistiu de esperar' };
})()`;

fluxo.push({
  nome: "agente: o prompt chega inteiro e a saída volta (202)",
  async fn(bridge, ctx) {
    const r = await bridge.js(RODAR(["--responder", "{prompt}"], "pergunta com\nquebra de linha"));
    ctx.assertEq(r.ok, true, `a execução falhou: ${r.erro}`);
    ctx.assert(r.saida.includes("RESPOSTA para:"), `saída inesperada: ${r.saida}`);
    ctx.assert(
      r.saida.includes("quebra de linha"),
      `o prompt chegou truncado na quebra de linha: ${r.saida}`,
    );
  },
});

fluxo.push({
  nome: "agente: texto perigoso no prompt é ARGUMENTO, não comando (202)",
  async fn(bridge, ctx) {
    // Se houvesse shell no caminho, isto viraria execução. Sem shell,
    // é só texto que volta como veio.
    const veneno = "$(echo INJETADO) `whoami` && rm -rf /";
    const r = await bridge.js(RODAR(["--responder", "{prompt}"], veneno));
    ctx.assertEq(r.ok, true, `a execução falhou: ${r.erro}`);
    ctx.assert(
      r.saida.includes("$(echo INJETADO)"),
      `o texto foi interpretado em vez de passado adiante: ${r.saida}`,
    );
    ctx.assert(!r.saida.includes("INJETADO\n"), `houve substituição de shell: ${r.saida}`);
  },
});

fluxo.push({
  nome: "agente: timeout interrompe em vez de pendurar (202)",
  async fn(bridge, ctx) {
    const t0 = Date.now();
    const r = await bridge.js(RODAR(["--demorar", "{prompt}"], "oi", 2));
    const levou = Date.now() - t0;
    ctx.assertEq(r.ok, false, "devia ter falhado por timeout");
    ctx.assert(/interrompid/i.test(r.erro), `mensagem inesperada: ${r.erro}`);
    ctx.assert(levou < 20000, `demorou demais pra desistir: ${levou}ms`);
  },
});

fluxo.push({
  nome: "agente: falha do processo vira erro legível (202)",
  async fn(bridge, ctx) {
    const r = await bridge.js(RODAR(["--falhar", "{prompt}"], "oi"));
    ctx.assertEq(r.ok, false, "devia ter falhado");
    ctx.assert(/erro proposital/.test(r.erro), `o stderr não chegou: ${r.erro}`);
  },
});

fluxo.push({
  nome: "agente: saída vazia é erro, não resposta em branco (202)",
  async fn(bridge, ctx) {
    const r = await bridge.js(RODAR(["--mudo", "{prompt}"], "oi"));
    ctx.assertEq(r.ok, false, "saída vazia devia ser tratada como falha");
  },
});

fluxo.push({
  nome: "agente: configuração com shell no binário é recusada (202)",
  async fn(bridge, ctx) {
    const r = await bridge.js(`(async () => {
      try {
        await window.__TAURI_INTERNALS__.invoke('iniciar_agente', {
          conversaPath: 'pages/__uitest-agente-invalido.md',
          adaptador: { nome: 'x', binario: 'sh -c', args: ['{prompt}'], cwd: '', timeout_s: 10, formato: 'texto' },
          prompt: 'oi', vaultPath: '/home/elis/Anotadinho',
        });
        return { ok: true };
      } catch (e) { return { ok: false, erro: String(e) }; }
    })()`);
    ctx.assertEq(r.ok, false, "linha de shell no binário devia ser recusada");
    ctx.assert(/espaço|shell/i.test(r.erro), `mensagem inesperada: ${r.erro}`);
  },
});

// ── ciclo 202: a conversa como página ────────────────────────────────

fluxo.push({
  nome: "conversa: pergunta e resposta viram markdown na própria página (202)",
  async fn(bridge, ctx) {
    // Aponta o app pro agente de mentira antes de abrir a conversa.
    await bridge.js(`(() => {
      localStorage.setItem('anotadinho.adaptador_agente', JSON.stringify({
        nome: 'falso', binario: ${JSON.stringify(FALSO)},
        args: ['--responder', '{prompt}'], cwd: '', timeout_s: 20,
      }));
      return true;
    })()`);

    ctx.escrever("---\ntitle: __uitest\ntype: conversa\n---\n");
    await recarregarEstavel(bridge);
    await ctx.abrirPagina(bridge, ctx.nomePagina);
    await esperar(bridge, "document.querySelector('.conversa')", "o painel de conversa");

    await bridge.js(`(() => {
      const campo = document.querySelector('.conversa__campo');
      const set = Object.getOwnPropertyDescriptor(HTMLTextAreaElement.prototype, 'value').set;
      campo.focus();
      set.call(campo, 'qual é a capital?');
      campo.dispatchEvent(new InputEvent('input', { bubbles: true }));
      return true;
    })()`);
    await PAUSA(400);
    await bridge.js(`(() => {
      [...document.querySelectorAll('.conversa button')].find(b => b.textContent.trim() === 'Enviar').click();
      return true;
    })()`);

    await esperar(
      bridge,
      "document.querySelectorAll('.conversa__msg--agente').length > 0",
      "a resposta do agente chegar",
      15000,
    );

    const autores = await bridge.js(
      `[...document.querySelectorAll('.conversa__msg-autor')].map(e => e.textContent.trim())`,
    );
    ctx.assertEq(autores.length, 2, `devia haver 2 mensagens: ${autores.join(", ")}`);

    // O arquivo é a fonte de verdade — sem save manual.
    await PAUSA(900);
    const md = ctx.ler() || "";
    ctx.assert(/^## você · /m.test(md), `a pergunta não virou markdown:\n${md}`);
    ctx.assert(/^## agente · /m.test(md), `a resposta não virou markdown:\n${md}`);
    ctx.assert(md.includes("qual é a capital?"), `a pergunta se perdeu:\n${md}`);
    ctx.assert(md.includes("RESPOSTA para:"), `a resposta se perdeu:\n${md}`);
    ctx.assert(md.includes("type: conversa"), `o frontmatter foi destruído:\n${md}`);
  },
});

fluxo.push({
  nome: "conversa: histórico é relido do arquivo ao reabrir (202)",
  async fn(bridge, ctx) {
    ctx.escrever(
      "---\ntitle: __uitest\ntype: conversa\n---\n" +
        "## você · 2026-08-22 10:00\n\npergunta antiga\n\n" +
        "## agente · 2026-08-22 10:01\n\nresposta antiga\n",
    );
    await recarregarEstavel(bridge);
    await ctx.abrirPagina(bridge, ctx.nomePagina);
    await esperar(bridge, "document.querySelector('.conversa__msg')", "as mensagens");

    const textos = await bridge.js(
      `[...document.querySelectorAll('.conversa__msg-corpo')].map(e => e.textContent.trim())`,
    );
    ctx.assertEq(textos.length, 2, `devia reler 2 mensagens: ${textos.join(" | ")}`);
    ctx.assert(textos[0].includes("pergunta antiga"), `perdeu a pergunta: ${textos[0]}`);
    ctx.assert(textos[1].includes("resposta antiga"), `perdeu a resposta: ${textos[1]}`);
  },
});

fluxo.push({
  nome: "conversa: falha do agente não perde o que você escreveu (202)",
  async fn(bridge, ctx) {
    await bridge.js(`(() => {
      localStorage.setItem('anotadinho.adaptador_agente', JSON.stringify({
        nome: 'falso', binario: ${JSON.stringify(FALSO)},
        args: ['--falhar', '{prompt}'], cwd: '', timeout_s: 20,
      }));
      return true;
    })()`);

    ctx.escrever("---\ntitle: __uitest\ntype: conversa\n---\n");
    await recarregarEstavel(bridge);
    await ctx.abrirPagina(bridge, ctx.nomePagina);
    await esperar(bridge, "document.querySelector('.conversa')", "o painel");

    await bridge.js(`(() => {
      const campo = document.querySelector('.conversa__campo');
      const set = Object.getOwnPropertyDescriptor(HTMLTextAreaElement.prototype, 'value').set;
      set.call(campo, 'pergunta que vai falhar');
      campo.dispatchEvent(new InputEvent('input', { bubbles: true }));
      return true;
    })()`);
    await PAUSA(400);
    await bridge.js(`(() => {
      [...document.querySelectorAll('.conversa button')].find(b => b.textContent.trim() === 'Enviar').click();
      return true;
    })()`);
    await esperar(bridge, "document.querySelector('.conversa__erro')", "o erro aparecer", 15000);

    // A pergunta é gravada ANTES da chamada justamente pra isto.
    const md = ctx.ler() || "";
    ctx.assert(
      md.includes("pergunta que vai falhar"),
      `o que a pessoa escreveu se perdeu na falha:\n${md}`,
    );
  },
});

// ── ciclo 203: promover mensagem em artefato ─────────────────────────

fluxo.push({
  nome: "promover: resposta do agente vira spec com fluxo embutido (203)",
  async fn(bridge, ctx) {
    const fs = await import("node:fs");
    const criadas = [];
    try {
      ctx.escrever(
        "---\ntitle: __uitest\ntype: conversa\n---\n" +
          "## você · 2026-08-22 10:00\n\npergunta\n\n" +
          "## agente · 2026-08-22 10:01\n\nExportar nota em PDF\n\nDeve gerar um arquivo.\n",
      );
      await recarregarEstavel(bridge);
      await ctx.abrirPagina(bridge, ctx.nomePagina);
      await esperar(bridge, "document.querySelector('.conversa__msg--agente')", "a resposta");

      const clicou = await bridge.js(`(() => {
        const b = [...document.querySelectorAll('.conversa__msg-acoes button')]
          .find(x => /spec/i.test(x.textContent));
        if (!b) return false;
        b.click();
        return true;
      })()`);
      ctx.assertEq(clicou, true, "não achei o botão de virar spec");
      await PAUSA(1800);

      const esperado = `${ctx.vault}/pages/specs/exportar-nota-em-pdf.md`;
      criadas.push(esperado);
      ctx.assert(fs.existsSync(esperado), `a spec não foi criada em ${esperado}`);

      const md = fs.readFileSync(esperado, "utf8");
      ctx.assert(md.includes("type: spec"), `sem type no frontmatter:\n${md}`);
      ctx.assert(md.includes("status: rascunho"), `não nasceu em rascunho:\n${md}`);
      ctx.assert(md.includes('{{ type: "fluxo" }}'), `sem o embed de fluxo:\n${md}`);
      ctx.assert(
        md.includes("origem: pages/__uitest.md"),
        `perdeu o rastro da conversa:\n${md}`,
      );
      ctx.assert(md.includes("Deve gerar um arquivo"), `perdeu o corpo:\n${md}`);

      // E a página criada abre.
      await esperar(
        bridge,
        `/Exportar nota em PDF/.test((document.querySelector('.editor__title')||{}).textContent || '')`,
        "a spec criada abrir",
        10000,
      );
    } finally {
      for (const c of criadas) fs.rmSync(c, { force: true });
    }
  },
});

fluxo.push({
  nome: "promover: a spec criada já responde ao fluxo (203)",
  async fn(bridge, ctx) {
    const fs = await import("node:fs");
    const criadas = [];
    try {
      ctx.escrever(
        "---\ntitle: __uitest\ntype: conversa\n---\n" +
          "## agente · 2026-08-22 10:01\n\nUma proposta qualquer\n",
      );
      await recarregarEstavel(bridge);
      await ctx.abrirPagina(bridge, ctx.nomePagina);
      await esperar(bridge, "document.querySelector('.conversa__msg-acoes')", "as ações");

      await bridge.js(`(() => {
        [...document.querySelectorAll('.conversa__msg-acoes button')]
          .find(x => /proposta/i.test(x.textContent)).click();
        return true;
      })()`);
      await PAUSA(1800);
      criadas.push(`${ctx.vault}/pages/propostas/uma-proposta-qualquer.md`);

      // O embed de fluxo tem que estar vivo na página nova.
      await esperar(bridge, "document.querySelector('.fluxo')", "o embed de fluxo na página criada", 10000);
      const etapa = await bridge.js(`(document.querySelector('.fluxo__etapa')||{}).textContent`);
      ctx.assertEq(etapa, "Rascunho", "a proposta devia nascer em rascunho");
    } finally {
      for (const c of criadas) fs.rmSync(c, { force: true });
    }
  },
});

// ── ciclo 204: propostas com revisão ─────────────────────────────────
//
// A garantia central: o agente propõe, o arquivo só muda depois de
// alguém aprovar. É a defesa que continua valendo mesmo se o modelo for
// enganado — as outras reduzem a chance, esta contém o estrago.

const VAULT_ABS = "/home/elis/Anotadinho/VaultAnotadinho";

/// Abre a página de rascunho e espera a TELA de propostas.
///
/// Não dá pra usar `abrirPagina`: uma página `type: propostas` tem
/// título próprio ("Propostas do agente") e não mostra o nome do
/// arquivo, então esperar pelo nome nunca resolveria.
async function abrirTelaDePropostas(bridge, ctx) {
  await bridge.js(`(() => {
    const alvo = [...document.querySelectorAll('.sidebar-item__title')]
      .find(e => e.textContent.trim() === ${JSON.stringify("__uitest")});
    if (alvo) alvo.click();
    return !!alvo;
  })()`);
  await esperar(bridge, "document.querySelector('.propostas')", "a tela de propostas");
}



/// Cria uma proposta pelo backend, como o agente faria.
const PROPOR = (alvo, conteudo, motivo = "teste") => `(async () => {
  try {
    const id = await window.__TAURI_INTERNALS__.invoke('propor', {
      vaultPath: ${JSON.stringify(VAULT_ABS)},
      proposta: {
        id: 'uitest-proposta',
        autor: 'agente-falso',
        quando: '2026-08-22 10:00',
        motivo: ${JSON.stringify(motivo)},
        alvo: ${JSON.stringify(alvo)},
        operacao: 'criar',
        conteudo: ${JSON.stringify(conteudo)},
      },
    });
    return { ok: true, id };
  } catch (e) { return { ok: false, erro: String(e) }; }
})()`;

/// Clica um botão do item da proposta de TESTE, achando-o pelo alvo.
///
/// A fila é do usuário: pode ter proposta de verdade esperando revisão,
/// e o cenário não pode encostar nela.
const BOTAO_DA_PROPOSTA_DE_TESTE = (rotulo, alvo = "__uitest_proposta") => `(() => {
  const itens = [...document.querySelectorAll('.propostas__item')];
  const meu = itens.find(i => i.textContent.includes(${JSON.stringify(alvo)}));
  if (!meu) return false;
  const b = [...meu.querySelectorAll('.propostas__acoes button')]
    .find(x => x.textContent.trim() === ${JSON.stringify(rotulo)});
  if (!b) return false;
  b.click();
  return true;
})()`;

const APLICAR_A_DE_TESTE = BOTAO_DA_PROPOSTA_DE_TESTE("Aplicar");
const RECUSAR_A_DE_TESTE = BOTAO_DA_PROPOSTA_DE_TESTE("Recusar");

fluxo.push({
  nome: "proposta: agente propõe e o arquivo NÃO muda até aprovar (204)",
  async fn(bridge, ctx) {
    const fs = await import("node:fs");
    const alvo = `${ctx.vault}/pages/__uitest_proposta.md`;
    const pendente = `${ctx.vault}/.anotadinho/propostas/uitest-proposta.json`;
    fs.rmSync(alvo, { force: true });
    fs.rmSync(pendente, { force: true });

    try {
      const r = await bridge.js(PROPOR("pages/__uitest_proposta.md", "---\ntitle: Proposta\n---\ncorpo proposto\n"));
      ctx.assertEq(r.ok, true, `propor falhou: ${r.erro}`);

      // O ponto inteiro: a página não existe ainda.
      ctx.assertEq(fs.existsSync(alvo), false, "escreveu a página sem aprovação");
      ctx.assertEq(fs.existsSync(pendente), true, "a proposta não foi gravada");

      // A tela de revisão mostra o diff.
      ctx.escrever("---\ntitle: __uitest\ntype: propostas\n---\n");
      await recarregarEstavel(bridge);
      await abrirTelaDePropostas(bridge, ctx);
      await esperar(bridge, "document.querySelector('.propostas__item')", "a proposta na tela");

      const linhas = await bridge.js(
        `[...document.querySelectorAll('.propostas__l--entra')].map(e => e.textContent)`,
      );
      ctx.assert(
        linhas.some((l) => l.includes("corpo proposto")),
        `o diff não mostrou o conteúdo novo: ${linhas.join(" | ")}`,
      );

      // Aplicar escreve — SÓ o item da proposta de teste.
      //
      // Antes clicava no primeiro "Aplicar" da tela. Com uma proposta
      // real esperando revisão na fila, o cenário aplicou a proposta do
      // usuário sem ele ver o diff. É a mesma trava do ciclo 197, agora
      // do lado das propostas.
      const aplicou = await bridge.js(APLICAR_A_DE_TESTE);
      ctx.assertEq(aplicou, true, "não achei o item da proposta de teste na tela");
      await PAUSA(1800);
      ctx.assertEq(fs.existsSync(alvo), true, "aplicar não escreveu a página");
      ctx.assert(
        fs.readFileSync(alvo, "utf8").includes("corpo proposto"),
        "o conteúdo aplicado está errado",
      );
      ctx.assertEq(fs.existsSync(pendente), false, "a proposta ficou pendurada depois de aplicada");
    } finally {
      fs.rmSync(alvo, { force: true });
      fs.rmSync(pendente, { force: true });
    }
  },
});

fluxo.push({
  nome: "proposta: recusar descarta sem escrever nada (204)",
  async fn(bridge, ctx) {
    const fs = await import("node:fs");
    const alvo = `${ctx.vault}/pages/__uitest_proposta.md`;
    const pendente = `${ctx.vault}/.anotadinho/propostas/uitest-proposta.json`;
    fs.rmSync(alvo, { force: true });
    fs.rmSync(pendente, { force: true });

    try {
      await bridge.js(PROPOR("pages/__uitest_proposta.md", "---\ntitle: X\n---\nnão quero isto\n"));
      ctx.escrever("---\ntitle: __uitest\ntype: propostas\n---\n");
      await recarregarEstavel(bridge);
      await abrirTelaDePropostas(bridge, ctx);
      await esperar(bridge, "document.querySelector('.propostas__item')", "a proposta");

      const recusou = await bridge.js(RECUSAR_A_DE_TESTE);
      ctx.assertEq(recusou, true, "não achei o item da proposta de teste na tela");
      await PAUSA(1500);

      ctx.assertEq(fs.existsSync(alvo), false, "recusar escreveu a página");
      ctx.assertEq(fs.existsSync(pendente), false, "a proposta recusada ficou pendurada");
    } finally {
      fs.rmSync(alvo, { force: true });
      fs.rmSync(pendente, { force: true });
    }
  },
});

fluxo.push({
  nome: "proposta: caminho fora do vault é recusado no backend (204)",
  async fn(bridge, ctx) {
    for (const alvo of ["../fora.md", "/etc/passwd", "pages/../../x.md"]) {
      const r = await bridge.js(PROPOR(alvo, "conteudo"));
      ctx.assertEq(r.ok, false, `deixou propor fora do vault: ${alvo}`);
      ctx.assert(/fora do vault/i.test(r.erro), `mensagem inesperada pra ${alvo}: ${r.erro}`);
    }
  },
});

fluxo.push({
  nome: "proposta: conteúdo com embed inválido é barrado antes de virar proposta (204)",
  async fn(bridge, ctx) {
    const ruim =
      '---\ntitle: X\n---\n\n{{ type: "kanban" }}\ncolumns:\n- Backlog\nitems:\n- title: C\n  column: Fantasma\n{{ /kanban }}\n';
    const r = await bridge.js(PROPOR("pages/__uitest_ruim.md", ruim));
    ctx.assertEq(r.ok, false, "aceitou proposta com embed inválido");
    ctx.assert(/Fantasma/.test(r.erro), `a validação do 189 não rodou: ${r.erro}`);
  },
});

// ── ciclo 206: o histórico dentro do vault ───────────────────────────

fluxo.push({
  nome: "vault: as páginas de ciclo são consultáveis e têm fluxo (206)",
  async fn(bridge, ctx) {
    // O que isto protege: a migração gerou frontmatter com título
    // contendo `: `, que é YAML inválido e derruba a página inteira em
    // SILÊNCIO — ela some da consulta sem erro nenhum. Um cenário que
    // conta as páginas pega isso; olhar uma amostra, não.
    await recarregarEstavel(bridge);
    const r = await bridge.js(`(async () => {
      const paginas = await window.__TAURI_INTERNALS__.invoke('scan_vault', {
        vaultPath: '/home/elis/Anotadinho/VaultAnotadinho',
      });
      const ciclos = paginas.filter(p => p.path.startsWith('pages/ciclos/'));
      return {
        total: ciclos.length,
        comStatus: ciclos.filter(p => (p.properties || {}).status === 'concluida').length,
        semTitulo: ciclos.filter(p => !p.title || /^\\d{3}-/.test(p.title)).map(p => p.path).slice(0, 3),
      };
    })()`);

    ctx.assert(r.total > 100, `esperava o histórico migrado, veio ${r.total} páginas`);
    ctx.assertEq(
      r.comStatus,
      r.total,
      `${r.total - r.comStatus} ciclo(s) sem status legível — frontmatter quebrado`,
    );
    ctx.assertEq(
      r.semTitulo.length,
      0,
      `ciclo(s) caíram pro nome do arquivo (frontmatter não parseou): ${r.semTitulo.join(", ")}`,
    );
  },
});

fluxo.push({
  nome: "vault: a página de ciclos mostra as consultas populadas (206)",
  async fn(bridge, ctx) {
    await recarregarEstavel(bridge);
    await bridge.js(`(() => {
      const alvo = [...document.querySelectorAll('.sidebar-item__title')]
        .find(e => e.textContent.trim() === 'ciclos');
      if (alvo) alvo.click();
      return !!alvo;
    })()`);
    await esperar(bridge, "document.querySelector('.query-embed')", "as consultas renderizarem", 15000);
    await PAUSA(1500);

    const vazias = await bridge.js(
      `[...document.querySelectorAll('.query-embed')].filter(e => /0 páginas/.test(e.textContent)).length`,
    );
    const total = await bridge.js(`document.querySelectorAll('.query-embed').length`);
    ctx.assert(total >= 3, `esperava as 3 consultas do painel, veio ${total}`);
    // "Em execução" pode estar legitimamente vazia; as outras duas não.
    ctx.assert(vazias <= 1, `${vazias} consultas vazias na página de ciclos`);
  },
});

// ── ciclo 207: home e sidebar ────────────────────────────────────────

fluxo.push({
  nome: "sidebar: pastas nascem recolhidas mas o que você abre continua aberto (207)",
  async fn(bridge, ctx) {
    // Com 200+ páginas, abrir tudo por padrão enterra a estrutura. Mas
    // recolher a cada recarga da lista desfaria o que a pessoa abriu — e
    // a lista recarrega a cada gravação de página.
    await recarregarEstavel(bridge);
    const estado = () =>
      bridge.js(`(() => {
        const d = [...document.querySelectorAll('.app-sidebar details')];
        return { total: d.length, abertas: d.filter(x => x.open).length };
      })()`);

    // Espera a lista ASSENTAR: páginas e pastas chegam em chamadas
    // separadas, então logo depois do reload existe um instante com as
    // pastas já na tela e o recolhimento ainda não aplicado.
    await esperar(
      bridge,
      "document.querySelectorAll('.app-sidebar details').length >= 3",
      "as pastas aparecerem",
    );
    await PAUSA(600);

    const inicial = await estado();
    ctx.assert(inicial.total >= 3, `esperava pastas na sidebar, veio ${inicial.total}`);
    ctx.assertEq(inicial.abertas, 0, "as pastas deviam nascer recolhidas");

    await bridge.js(`(() => {
      document.querySelector('.app-sidebar details summary').click();
      return true;
    })()`);
    await PAUSA(500);
    ctx.assertEq((await estado()).abertas, 1, "clicar devia abrir a pasta");

    // Trocar de página recarrega a lista.
    await bridge.js(`(() => {
      const a = [...document.querySelectorAll('.sidebar-item__title')].find(e => e.textContent.trim() === 'sobre');
      if (a) a.click();
      return !!a;
    })()`);
    await PAUSA(2000);
    ctx.assertEq(
      (await estado()).abertas,
      1,
      "a pasta que a pessoa abriu se fechou sozinha ao recarregar a lista",
    );
  },
});

fluxo.push({
  nome: "home: mostra os dados do vault e os atalhos (207)",
  async fn(bridge, ctx) {
    await recarregarEstavel(bridge);
    await bridge.js(`(() => {
      const a = [...document.querySelectorAll('.sidebar-item__title')].find(e => e.textContent.trim() === 'incio');
      if (a) a.click();
      return !!a;
    })()`);
    await esperar(bridge, "document.querySelector('.query-embed')", "as consultas do home", 15000);
    await PAUSA(2000);

    const botoes = await bridge.js(
      `[...document.querySelectorAll('.actions-embed__btn')].map(b => b.textContent.trim())`,
    );
    ctx.assert(botoes.length >= 4, `esperava os atalhos, veio: ${botoes.join(", ")}`);

    // Nenhuma consulta pode estar vazia: seção vazia no home é o mesmo
    // problema da caixa vazia do painel (ciclo 196).
    const vazias = await bridge.js(
      `[...document.querySelectorAll('.query-embed')]
        .filter(e => /\\b0 páginas\\b/.test(e.textContent))
        .length`,
    );
    ctx.assertEq(vazias, 0, `${vazias} consulta(s) vazia(s) no home`);
  },
});

// ── ciclo 220: home fixa e conversa na família de tipos ──────────────

fluxo.push({
  nome: "abas: home fica na primeira posição, não fecha e acompanha a troca (220)",
  async fn(bridge, ctx) {
    ctx.escrever("---\ntitle: __uitest\n---\ntexto\n");
    const storage = await bridge.js(`(() => {
      const vault = JSON.parse(localStorage.getItem('anotadinho.vault_path'));
      const key = 'anotadinho.home_page::' + vault;
      const old = localStorage.getItem(key);
      localStorage.setItem(key, JSON.stringify('pages/__uitest.md'));
      return { key, old };
    })()`);
    try {
      await recarregarEstavel(bridge);
      await esperar(bridge, `document.querySelector('.tab-bar__tab')?.dataset.path === 'pages/__uitest.md'`, "a home abrir primeiro");
      const inicial = await bridge.js(`(() => {
        const t = document.querySelector('.tab-bar__tab');
        return { fixa: t.classList.contains('tab-bar__tab--fixed'), fechar: !!t.querySelector('.tab-bar__tab-close'), nav: !!t.querySelector('[data-nav-item]') };
      })()`);
      ctx.assertEq(inicial.fixa, true, "a home não tem distinção visual");
      ctx.assertEq(inicial.fechar, false, "a home oferece fechar");
      ctx.assertEq(inicial.nav, true, "a home não participa da navegação por teclado");

      const abriu = await bridge.js(`(() => {
        const alvo = [...document.querySelectorAll('.sidebar-item__title')].find(e => e.textContent.toLowerCase().includes('missao'));
        if (alvo) alvo.click();
        return !!alvo;
      })()`);
      ctx.assert(abriu, "não encontrou uma segunda página para testar as abas");
      await esperar(bridge, `document.querySelectorAll('.tab-bar__tab').length >= 2`, "a segunda aba abrir");
      const antes = await bridge.js(`document.querySelectorAll('.tab-bar__tab').length`);
      await bridge.js(`document.querySelector('button[title="Mais ações"]').click(); true`);
      await esperar(bridge, `[...document.querySelectorAll('.header-menu__item')].some(b => b.textContent.includes('Definir como início'))`, "a ação de definir início");
      await bridge.js(`(() => { [...document.querySelectorAll('.header-menu__item')].find(b => b.textContent.includes('Definir como início')).click(); return true; })()`);
      await esperar(bridge, `document.querySelector('.tab-bar__tab')?.dataset.path.includes('missao')`, "a nova home ir para o começo");
      const depois = await bridge.js(`(() => ({ n: document.querySelectorAll('.tab-bar__tab').length, antigas: [...document.querySelectorAll('.tab-bar__tab')].filter(t => t.dataset.path === 'pages/__uitest.md').length }))()`);
      ctx.assertEq(depois.n, antes, "trocar a home perdeu uma aba aberta");
      ctx.assertEq(depois.antigas, 1, "trocar a home perdeu a home anterior");
    } finally {
      await bridge.js(`(() => { const key = ${JSON.stringify(storage.key)}; const old = ${JSON.stringify(storage.old)}; if (old === null) localStorage.removeItem(key); else localStorage.setItem(key, old); return true; })()`);
    }
  },
});

// ── ciclo 208: conversa em um passo e contexto anexável ──────────────

fluxo.push({
  nome: "conversa: tipo da paleta cria uma página pronta para uso (220)",
  async fn(bridge, ctx) {
    const fs = await import("node:fs");
    const titulo = `__uitest-conversa-${Date.now()}`;
    const antes = new Set(fs.readdirSync(`${ctx.vault}/pages`));
    try {
      await recarregarEstavel(bridge);
      await bridge.js(`(() => { const raiz = document.querySelector('.app-root'); raiz.dispatchEvent(new KeyboardEvent('keydown', { key: 'k', ctrlKey: true, bubbles: true, cancelable: true })); return true; })()`);
      await esperar(bridge, "document.querySelector('[class*=palette]')", "a paleta abrir");
      const achou = await bridge.js(`(() => { const i = [...document.querySelectorAll('[class*=palette__item]')].find(x => x.textContent.includes('Nova página: Conversa')); if (i) i.click(); return !!i; })()`);
      ctx.assertEq(achou, true, "conversa não está na família de tipos");
      await esperar(bridge, "document.querySelector('.modal input')", "o pedido de título");
      await bridge.js(`(() => { const i = document.querySelector('.modal input'); const set = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value').set; set.call(i, ${JSON.stringify(titulo)}); i.dispatchEvent(new InputEvent('input', { bubbles: true })); i.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true })); return true; })()`);
      await esperar(bridge, "document.querySelector('.conversa')", "a conversa abrir pronta para uso", 12000);
      const novas = fs.readdirSync(`${ctx.vault}/pages`).filter(f => !antes.has(f));
      ctx.assertEq(novas.length, 1, `esperava uma página criada, vieram ${novas.length}`);
      const md = fs.readFileSync(`${ctx.vault}/pages/${novas[0]}`, "utf8");
      ctx.assert(md.includes("type: conversa"), `a página não nasceu como conversa:\n${md}`);
    } finally {
      for (const f of fs.readdirSync(`${ctx.vault}/pages`).filter(x => !antes.has(x))) fs.rmSync(`${ctx.vault}/pages/${f}`, { force: true });
    }
  },
});

fluxo.push({
  nome: "conversa: comando da paleta cria em um passo e anexa a página aberta (208)",
  async fn(bridge, ctx) {
    const fs = await import("node:fs");
    const pasta = `${ctx.vault}/pages/conversas`;
    const antes = fs.existsSync(pasta) ? fs.readdirSync(pasta) : [];
    try {
      ctx.escrever("---\ntitle: __uitest\n---\nconteudo de origem\n");
      await recarregarEstavel(bridge);
      await ctx.abrirPagina(bridge, ctx.nomePagina);

      await bridge.js(`(() => {
        const raiz = document.querySelector('.app-root') || document.body;
        raiz.dispatchEvent(new KeyboardEvent('keydown', { key: 'k', ctrlKey: true, bubbles: true, cancelable: true }));
        return true;
      })()`);
      await esperar(bridge, "document.querySelector('[class*=palette]')", "a paleta");
      const achou = await bridge.js(`(() => {
        const it = [...document.querySelectorAll('[class*=palette__item]')]
          .find(x => x.textContent.includes('Nova conversa com o agente'));
        if (!it) return false;
        it.click();
        return true;
      })()`);
      ctx.assertEq(achou, true, "o comando não está na paleta");

      await esperar(bridge, "document.querySelector('.conversa')", "o painel abrir", 12000);

      // A página aberta entra como anexo — é o ponto 1 da spec.
      const anexos = await bridge.js(
        `[...document.querySelectorAll('.conversa__anexo')].map(e => e.textContent.replace('×','').trim())`,
      );
      ctx.assert(anexos.includes("__uitest"), `a página aberta não foi anexada: ${anexos.join(", ")}`);

      // E o vínculo fica no ARQUIVO, não em memória — ponto 2 da spec.
      const novas = fs.readdirSync(pasta).filter((f) => !antes.includes(f));
      ctx.assertEq(novas.length, 1, `esperava 1 conversa nova, veio ${novas.length}`);
      const md = fs.readFileSync(`${pasta}/${novas[0]}`, "utf8");
      ctx.assert(md.includes("type: conversa"), `sem o tipo:\n${md}`);
      ctx.assert(md.includes("origem: pages/__uitest.md"), `sem a origem:\n${md}`);
      ctx.assert(md.includes("- pages/__uitest.md"), `sem o contexto:\n${md}`);
    } finally {
      if (fs.existsSync(pasta)) {
        for (const f of fs.readdirSync(pasta).filter((x) => !antes.includes(x))) {
          fs.rmSync(`${pasta}/${f}`, { force: true });
        }
      }
    }
  },
});

fluxo.push({
  nome: "conversa: anexar e tirar páginas grava no frontmatter (208)",
  async fn(bridge, ctx) {
    const fs = await import("node:fs");
    const alvo = `${ctx.vault}/pages/__uitest.md`;
    ctx.escrever("---\ntitle: __uitest\ntype: conversa\n---\n");
    await recarregarEstavel(bridge);
    await ctx.abrirPagina(bridge, ctx.nomePagina);
    await esperar(bridge, "document.querySelector('.conversa')", "o painel");

    await bridge.js(`(() => { document.querySelector('.conversa__anexar').click(); return true; })()`);
    await esperar(bridge, "document.querySelector('.conversa__seletor-busca')", "o seletor");

    // Sem filtro a lista é inútil com 200+ páginas.
    await bridge.js(`(() => {
      const inp = document.querySelector('.conversa__seletor-busca');
      const set = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value').set;
      inp.focus();
      set.call(inp, 'nomenclatura');
      inp.dispatchEvent(new InputEvent('input', { bubbles: true }));
      return true;
    })()`);
    await PAUSA(600);
    const itens = await bridge.js(
      `[...document.querySelectorAll('.conversa__seletor-item')].map(e => e.textContent.trim())`,
    );
    ctx.assert(itens.length > 0 && itens.length < 10, `o filtro não reduziu: ${itens.length} itens`);

    await bridge.js(`(() => { document.querySelector('.conversa__seletor-item').click(); return true; })()`);
    await PAUSA(1500);

    let md = fs.readFileSync(alvo, "utf8");
    ctx.assert(/^contexto:$/m.test(md), `o anexo não foi gravado:\n${md}`);
    ctx.assert(md.includes("nomenclatura"), `o anexo errado foi gravado:\n${md}`);
    ctx.assert(md.includes("type: conversa"), `o frontmatter foi destruído:\n${md}`);

    // Tirar também grava.
    await bridge.js(`(() => { document.querySelector('.conversa__anexo-x').click(); return true; })()`);
    await PAUSA(1500);
    md = fs.readFileSync(alvo, "utf8");
    ctx.assert(!md.includes("nomenclatura"), `tirar o anexo não gravou:\n${md}`);
    ctx.assert(md.includes("type: conversa"), `o frontmatter foi destruído ao tirar:\n${md}`);
  },
});

// ── ciclo 224: prompts padrão reutilizáveis ─────────────────────────

fluxo.push({
  nome: "conversa: prompt padrão filtra, preenche, repete, prevê e anexa contexto (224)",
  async fn(bridge, ctx) {
    const fs = await import("node:fs");
    const pasta = `${ctx.vault}/pages/prompts-default`;
    const fora = `${ctx.vault}/pages/__uitest-prompt-fora.md`;
    const conversa = `${ctx.vault}/pages/__uitest.md`;
    fs.mkdirSync(`${pasta}/sub`, { recursive: true });
    fs.writeFileSync(
      `${pasta}/sub/__uitest-prompt.md`,
      "---\ntitle: __uitest prompt completo\ntype: prompt\ncontexto:\n- pages/produto/missao.md\n---\n" +
        "Revise {{title}} para {{publico}}. Compare {{title}} com {{referencia}}.",
    );
    fs.writeFileSync(
      `${pasta}/__uitest-tipo-errado.md`,
      "---\ntitle: __uitest tipo errado\ntype: md\n---\n{{title}}",
    );
    fs.writeFileSync(
      `${pasta}/__uitest-unico.md`,
      "---\ntitle: __uitest marcador único\ntype: prompt\n---\nResuma {{title}}.",
    );
    fs.writeFileSync(
      fora,
      "---\ntitle: __uitest fora\ntype: prompt\n---\n{{title}}",
    );
    try {
      ctx.escrever("---\ntitle: __uitest\ntype: conversa\n---\n");
      await recarregarEstavel(bridge);
      await ctx.abrirPagina(bridge, ctx.nomePagina);
      await esperar(bridge, "document.querySelector('.conversa__prompt-select')", "o seletor de prompts");
      await esperar(
        bridge,
        "[...document.querySelectorAll('.conversa__prompt-select option')].some(o => o.textContent.includes('__uitest prompt completo'))",
        "o prompt ser descoberto",
      );

      const opcoes = await bridge.js(
        `[...document.querySelectorAll('.conversa__prompt-select option')].map(o => o.textContent.trim())`,
      );
      ctx.assert(opcoes.some((x) => x.includes("Nenhum")), "faltou a opção vazia");
      ctx.assert(opcoes.some((x) => x.includes("prompt completo")), "prompt da subpasta não apareceu");
      ctx.assert(!opcoes.some((x) => x.includes("tipo errado")), "página sem type: prompt apareceu");
      ctx.assert(!opcoes.some((x) => x.includes("fora")), "prompt fora da pasta apareceu");

      await bridge.js(`(() => {
        const campo = document.querySelector('.conversa__campo');
        const setArea = Object.getOwnPropertyDescriptor(HTMLTextAreaElement.prototype, 'value').set;
        setArea.call(campo, 'rascunho inicial');
        campo.dispatchEvent(new InputEvent('input', { bubbles: true }));
        return true;
      })()`);
      await PAUSA(300);
      await bridge.js(`(() => {
        const select = document.querySelector('.conversa__prompt-select');
        const option = [...select.options].find(o => o.textContent.includes('__uitest prompt completo'));
        const setSelect = Object.getOwnPropertyDescriptor(HTMLSelectElement.prototype, 'value').set;
        setSelect.call(select, option.value);
        select.dispatchEvent(new Event('change', { bubbles: true }));
        return true;
      })()`);
      await esperar(bridge, "document.querySelectorAll('.conversa__prompt-campo').length === 3", "os três campos únicos");

      const campos = await bridge.js(`(() => ({
        nomes: [...document.querySelectorAll('.conversa__prompt-campo span')].map(x => x.textContent),
        valores: [...document.querySelectorAll('.conversa__prompt-campo input')].map(x => x.value),
        enviarDesabilitado: document.querySelector('.conversa__compositor .btn--primary').disabled,
      }))()`);
      ctx.assertEq(campos.nomes.join(","), "{{title}},{{publico}},{{referencia}}", "ordem dos campos");
      ctx.assertEq(campos.valores[0], "rascunho inicial", "rascunho não preencheu a primeira variável");
      ctx.assertEq(campos.enviarDesabilitado, true, "marcador ausente não bloqueou o envio");

      for (const [indice, valor] of [[1, "leitores"], [2, "modelo"]]) {
        await bridge.js(`(() => {
          const input = document.querySelectorAll('.conversa__prompt-campo input')[${indice}];
          const set = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value').set;
          set.call(input, ${JSON.stringify(valor)});
          input.dispatchEvent(new InputEvent('input', { bubbles: true }));
          return true;
        })()`);
        await PAUSA(300);
      }
      const final = await bridge.js(`document.querySelector('.conversa__campo').value`);
      ctx.assertEq(final.split("rascunho inicial").length - 1, 2, "variável repetida não reutilizou o valor");
      ctx.assert(final.includes("leitores") && final.includes("modelo"), "valores não foram substituídos");
      ctx.assert(final.includes("<<<DADO-ANOTADINHO VALOR title>>>"), "valor não foi blindado como DADO");

      await bridge.js(`document.querySelector('.conversa__prompt-preview').click(); true`);
      await esperar(bridge, "document.querySelector('.conversa__prompt-final')", "o preview abrir");
      const preview = await bridge.js(`document.querySelector('.conversa__prompt-final').textContent`);
      ctx.assertEq(preview, final, "preview não mostra o prompt final");
      ctx.assertEq(await bridge.js(`document.querySelectorAll('.conversa__msg').length`), 0, "preview enviou a mensagem");
      await bridge.js(`document.querySelector('.modal__actions button').click(); true`);
      await PAUSA(900);
      const md = fs.readFileSync(conversa, "utf8");
      ctx.assert(md.includes("- pages/produto/missao.md"), `contexto do prompt não persistiu:\n${md}`);

      await bridge.js(`(() => {
        const select = document.querySelector('.conversa__prompt-select');
        const set = Object.getOwnPropertyDescriptor(HTMLSelectElement.prototype, 'value').set;
        set.call(select, '');
        select.dispatchEvent(new Event('change', { bubbles: true }));
        return true;
      })()`);
      await PAUSA(300);
      ctx.assertEq(
        await bridge.js(`document.querySelector('.conversa__campo').value`),
        "rascunho inicial",
        "opção vazia não restaurou a escrita livre",
      );

      await bridge.js(`(() => {
        const select = document.querySelector('.conversa__prompt-select');
        const option = [...select.options].find(o => o.textContent.includes('marcador único'));
        const set = Object.getOwnPropertyDescriptor(HTMLSelectElement.prototype, 'value').set;
        set.call(select, option.value);
        select.dispatchEvent(new Event('change', { bubbles: true }));
        return true;
      })()`);
      await esperar(bridge, "document.querySelectorAll('.conversa__prompt-campo').length === 1", "o marcador único");
      const unico = await bridge.js(`document.querySelector('.conversa__campo').value`);
      ctx.assert(unico.startsWith("Resuma O bloco abaixo"), "marcador único não foi expandido");
      ctx.assert(unico.includes("rascunho inicial"), "marcador único não recebeu o rascunho");
    } finally {
      fs.rmSync(pasta, { recursive: true, force: true });
      fs.rmSync(fora, { force: true });
    }
  },
});

// ── ciclo 209: spec e proposta são coisas diferentes ─────────────────

fluxo.push({
  nome: "spec aprovada oferece planejar, e a conversa nasce com ela anexada (209)",
  async fn(bridge, ctx) {
    const fs = await import("node:fs");
    const pasta = `${ctx.vault}/pages/conversas`;
    const antes = fs.existsSync(pasta) ? fs.readdirSync(pasta) : [];
    try {
      ctx.escrever(
        '---\ntitle: "Spec de teste: com dois-pontos"\ntype: spec\nstatus: aprovada\n---\n' +
          '{{ type: "fluxo" }}\nartefato: spec\netapa: aprovada\n{{ /fluxo }}\n\n' +
          "## Requisitos funcionais\n\n- RF1. Alguma coisa.\n",
      );
      await recarregarEstavel(bridge);
      await ctx.abrirPagina(bridge, ctx.nomePagina);
      await esperar(bridge, "document.querySelector('.fluxo')", "o embed de fluxo");

      // O botão só existe em spec APROVADA — é a ponte do "o quê" pro "como".
      ctx.assertEq(
        await bridge.js(`!!document.querySelector('.fluxo__planejar button')`),
        true,
        "spec aprovada devia oferecer planejar a implementação",
      );

      await bridge.js(`(() => { document.querySelector('.fluxo__planejar button').click(); return true; })()`);
      await esperar(bridge, "document.querySelector('.conversa')", "a conversa de planejamento", 12000);

      const anexos = await bridge.js(
        `[...document.querySelectorAll('.conversa__anexo')].map(e => e.textContent.replace('×','').trim())`,
      );
      ctx.assert(anexos.includes("__uitest"), `a spec não foi anexada: ${anexos.join(", ")}`);

      const campo = await bridge.js(`(document.querySelector('.conversa__campo')||{}).value || ''`);
      ctx.assert(campo.includes("PROPOSTA DE IMPLEMENTAÇÃO"), `a pergunta não veio pronta: ${campo}`);
      ctx.assert(
        campo.includes("Não proponha requisitos novos"),
        "faltou a trava contra o modelo reescrever o escopo",
      );
      // Título do FRONTMATTER, não o nome do arquivo (ciclo 196/209).
      ctx.assert(
        campo.includes("Spec de teste: com dois-pontos"),
        `usou o nome do arquivo em vez do título: ${campo.slice(0, 80)}`,
      );
      // A indentação do código Rust não pode vazar pro prompt.
      ctx.assertEq(
        campo.split("\n").filter((l) => l.startsWith("  ")).length,
        0,
        "indentação do código vazou pro prompt",
      );
    } finally {
      if (fs.existsSync(pasta)) {
        for (const f of fs.readdirSync(pasta).filter((x) => !antes.includes(x))) {
          fs.rmSync(`${pasta}/${f}`, { force: true });
        }
      }
    }
  },
});

fluxo.push({
  nome: "spec em rascunho NÃO oferece planejar (209)",
  async fn(bridge, ctx) {
    // Planejar antes de aprovar é planejar o que ainda pode mudar.
    ctx.escrever(
      "---\ntitle: __uitest\ntype: spec\nstatus: rascunho\n---\n" +
        '{{ type: "fluxo" }}\nartefato: spec\netapa: rascunho\n{{ /fluxo }}\n',
    );
    await recarregarEstavel(bridge);
    await ctx.abrirPagina(bridge, ctx.nomePagina);
    await esperar(bridge, "document.querySelector('.fluxo')", "o embed");
    ctx.assertEq(
      await bridge.js(`!!document.querySelector('.fluxo__planejar')`),
      false,
      "spec em rascunho não pode oferecer planejar",
    );
  },
});

// ── ciclo 210: aviso de pendente e execução da proposta ──────────────

fluxo.push({
  nome: "aviso: proposta pendente aparece no cabeçalho e some ao resolver (210)",
  async fn(bridge, ctx) {
    const fs = await import("node:fs");
    const pendente = `${ctx.vault}/.anotadinho/propostas/uitest-aviso.json`;
    const alvo = `${ctx.vault}/pages/__uitest_aviso.md`;
    fs.rmSync(pendente, { force: true });
    fs.rmSync(alvo, { force: true });
    try {
      // O agente propõe — pode ser pelo CLI ou pelo MCP, sem nenhum
      // evento de UI pra reagir. O aviso tem que aparecer assim mesmo.
      await bridge.js(`(async () => {
        await window.__TAURI_INTERNALS__.invoke('propor', {
          vaultPath: ${JSON.stringify(VAULT_ABS)},
          proposta: {
            id: 'uitest-aviso', autor: 'agente', quando: '2026-08-22 10:00',
            motivo: 'teste', alvo: 'pages/__uitest_aviso.md', operacao: 'criar',
            conteudo: '---\\ntitle: X\\n---\\ncorpo\\n',
          },
        });
        return true;
      })()`);

      await recarregarEstavel(bridge);
      await esperar(bridge, "document.querySelector('.header-bar__propostas')", "o aviso no cabeçalho");
      const texto = await bridge.js(`document.querySelector('.header-bar__propostas').textContent.trim()`);
      ctx.assert(/\d/.test(texto), `o aviso devia mostrar a contagem, veio "${texto}"`);

      // Clicar leva pra revisão — o aviso não pode levar a lugar nenhum.
      await bridge.js(`(() => { document.querySelector('.header-bar__propostas').click(); return true; })()`);
      await esperar(bridge, "document.querySelector('.propostas__item')", "a tela de revisão");

      const recusou = await bridge.js(BOTAO_DA_PROPOSTA_DE_TESTE("Recusar", "__uitest_aviso"));
      ctx.assertEq(recusou, true, "não achei o item da proposta de teste na tela");
      await PAUSA(2500);
      // O aviso some quando a fila esvazia. Se o usuário tiver proposta
      // de VERDADE esperando revisão, ela continua lá — e deve
      // continuar. O que este cenário afirma é que o aviso acompanha a
      // fila, não que a fila fica vazia.
      const restantes = fs.existsSync(`${ctx.vault}/.anotadinho/propostas`)
        ? fs.readdirSync(`${ctx.vault}/.anotadinho/propostas`).length
        : 0;
      ctx.assertEq(
        await bridge.js(`!!document.querySelector('.header-bar__propostas')`),
        restantes > 0,
        restantes > 0
          ? "o aviso sumiu com proposta ainda na fila"
          : "o aviso ficou preso depois de resolver a fila",
      );
    } finally {
      fs.rmSync(pendente, { force: true });
      fs.rmSync(alvo, { force: true });
    }
  },
});

fluxo.push({
  nome: "proposta aprovada oferece EXECUTAR, com pergunta diferente da de planejar (210)",
  async fn(bridge, ctx) {
    const fs = await import("node:fs");
    const pasta = `${ctx.vault}/pages/conversas`;
    const antes = fs.existsSync(pasta) ? fs.readdirSync(pasta) : [];
    try {
      ctx.escrever(
        '---\ntitle: "Abordagem de teste"\ntype: proposta\nstatus: aprovada\n---\n' +
          '{{ type: "fluxo" }}\nartefato: proposta\netapa: aprovada\n{{ /fluxo }}\n\n' +
          "## Abordagem\n\nFazer assim.\n",
      );
      await recarregarEstavel(bridge);
      await ctx.abrirPagina(bridge, ctx.nomePagina);
      await esperar(bridge, "document.querySelector('.fluxo__planejar button')", "o botão");

      const rotulo = await bridge.js(
        `document.querySelector('.fluxo__planejar button').textContent.trim()`,
      );
      ctx.assert(/Executar/.test(rotulo), `numa proposta o botão devia ser Executar, veio "${rotulo}"`);

      await bridge.js(`(() => { document.querySelector('.fluxo__planejar button').click(); return true; })()`);
      await esperar(bridge, "document.querySelector('.conversa')", "a conversa de execução", 12000);

      const campo = await bridge.js(`(document.querySelector('.conversa__campo')||{}).value || ''`);
      ctx.assert(campo.includes("Execute a proposta"), `pergunta errada: ${campo.slice(0, 60)}`);
      // A trava desta etapa é OUTRA: lá não muda escopo, aqui não muda
      // abordagem sem passar por proposta nova.
      ctx.assert(campo.includes("PARE e explique"), "faltou a trava contra mudar a abordagem");
      ctx.assert(
        !campo.includes("PROPOSTA DE IMPLEMENTAÇÃO"),
        "veio a pergunta de planejar numa proposta",
      );

      const anexos = await bridge.js(
        `[...document.querySelectorAll('.conversa__anexo')].map(e => e.textContent.replace('×','').trim())`,
      );
      ctx.assert(anexos.includes("__uitest"), `a proposta não foi anexada: ${anexos.join(", ")}`);

      // E dá pra promover a resposta em execução.
      const opcoes = await bridge.js(`(() => {
        const m = document.querySelector('.conversa__msg--agente');
        if (!m) return ['sem resposta ainda'];
        return [...m.querySelectorAll('.conversa__msg-acoes button')].map(b => b.textContent.trim());
      })()`);
      ctx.assert(Array.isArray(opcoes), "não consegui ler as ações");
    } finally {
      if (fs.existsSync(pasta)) {
        for (const f of fs.readdirSync(pasta).filter((x) => !antes.includes(x))) {
          fs.rmSync(`${pasta}/${f}`, { force: true });
        }
      }
    }
  },
});

// ── ciclo 213: execução assíncrona, progresso e interromper ──────────

/// Dispara sem esperar e devolve a chave da conversa, pra o cenário
/// poder observar o meio da execução.
const DISPARAR = (
  args,
  prompt,
  conversa,
  timeout = 60,
  formato = "texto",
  vault = "/home/elis/Anotadinho",
) => `(async () => {
  await window.__TAURI_INTERNALS__.invoke('iniciar_agente', {
    adaptador: {
      nome: 'falso',
      binario: ${JSON.stringify(FALSO)},
      args: ${JSON.stringify(args)},
      cwd: '',
      timeout_s: ${timeout},
      formato: ${JSON.stringify(formato)},
    },
    prompt: ${JSON.stringify(prompt)},
    vaultPath: ${JSON.stringify(vault)},
    conversaPath: ${JSON.stringify(conversa)},
  });
  return true;
})()`;

const ESTADO = (conversa) =>
  `window.__TAURI_INTERNALS__.invoke('estado_agente', { conversaPath: ${JSON.stringify(conversa)} })`;

fluxo.push({
  nome: "agente: disparar não bloqueia a chamada (213)",
  async fn(bridge, ctx) {
    const conversa = "pages/__uitest-async-rapido.md";
    const ms = await bridge.js(`(async () => {
      const t0 = performance.now();
      await ${DISPARAR(["--devagar", "{prompt}"], "x", "pages/__uitest-async-rapido.md")};
      return performance.now() - t0;
    })()`);
    // O agente falso leva 10s; se a chamada esperasse, isto não seria
    // um número pequeno. Era exatamente o problema: a tela ficava
    // presa até o modelo terminar.
    ctx.assert(ms < 1500, `a chamada demorou ${Math.round(ms)}ms — ainda está bloqueando`);
    await bridge.js(`window.__TAURI_INTERNALS__.invoke('cancelar_agente', { conversaPath: ${JSON.stringify(conversa)} })`);
  },
});

fluxo.push({
  nome: "agente: a saída parcial aparece antes de terminar (213)",
  async fn(bridge, ctx) {
    const conversa = "pages/__uitest-async-parcial.md";
    await bridge.js(DISPARAR(["--devagar", "{prompt}"], "x", conversa));
    await PAUSA(3000);
    const meio = await bridge.js(ESTADO(conversa));
    ctx.assertEq(meio && meio.estado, "rodando", `esperava rodando, veio ${JSON.stringify(meio)}`);
    ctx.assert(
      meio.parcial.includes("linha 1"),
      `sem saída parcial no meio da execução: ${JSON.stringify(meio.parcial)}`,
    );
    await bridge.js(`window.__TAURI_INTERNALS__.invoke('cancelar_agente', { conversaPath: ${JSON.stringify(conversa)} })`);
  },
});

fluxo.push({
  nome: "agente: interromper mata o processo e reporta cancelado (213)",
  async fn(bridge, ctx) {
    const conversa = "pages/__uitest-async-cancelar.md";
    await bridge.js(DISPARAR(["--devagar", "{prompt}"], "x", conversa));
    await PAUSA(1200);
    await bridge.js(
      `window.__TAURI_INTERNALS__.invoke('cancelar_agente', { conversaPath: ${JSON.stringify(conversa)} })`,
    );
    await PAUSA(1200);
    const fim = await bridge.js(ESTADO(conversa));
    ctx.assertEq(fim && fim.estado, "cancelado", `esperava cancelado, veio ${JSON.stringify(fim)}`);
  },
});

fluxo.push({
  nome: "agente: uma execução por conversa (213)",
  async fn(bridge, ctx) {
    const conversa = "pages/__uitest-async-duplo.md";
    await bridge.js(DISPARAR(["--devagar", "{prompt}"], "x", conversa));
    const segundo = await bridge.js(`(async () => {
      try {
        await ${DISPARAR(["--devagar", "{prompt}"], "y", "pages/__uitest-async-duplo.md")};
        return { ok: true };
      } catch (e) { return { ok: false, erro: String(e) }; }
    })()`);
    // Sem esta recusa, duas respostas gravariam no mesmo arquivo ao
    // mesmo tempo — dois escritores, que é a origem do bug do 209.
    ctx.assertEq(segundo.ok, false, "aceitou uma segunda execução na mesma conversa");
    ctx.assert(
      /andamento/i.test(segundo.erro),
      `erro não explica o motivo: ${segundo.erro}`,
    );
    await bridge.js(`window.__TAURI_INTERNALS__.invoke('cancelar_agente', { conversaPath: ${JSON.stringify(conversa)} })`);
  },
});

fluxo.push({
  nome: "agente: resposta é gravada mesmo sem a tela aberta (213)",
  async fn(bridge, ctx) {
    const fs = await import("node:fs");
    const rel = "pages/__uitest-async-grava.md";
    const arquivo = `${ctx.vault}/${rel}`;
    fs.writeFileSync(arquivo, "---\ntitle: __uitest async grava\ntype: conversa\n---\n");
    try {
      // Ninguém abre a conversa: o backend é quem grava, e é isso que
      // faz a resposta sobreviver a sair da página no meio.
      await bridge.js(
        DISPARAR(
          ["--responder", "{prompt}"],
          "pergunta solta",
          rel,
          30,
          "texto",
          `${process.cwd()}/${ctx.vault}`,
        ),
      );
      let disco = "";
      for (let i = 0; i < 40; i++) {
        await PAUSA(250);
        disco = fs.readFileSync(arquivo, "utf8");
        if (disco.includes("## agente")) break;
      }
      ctx.assert(
        disco.includes("## agente") && disco.includes("RESPOSTA para: pergunta solta"),
        `a resposta não chegou ao arquivo:\n${disco}`,
      );
    } finally {
      fs.rmSync(arquivo, { force: true });
    }
  },
});

fluxo.push({
  nome: "agente: stream-json vira progresso e resposta (213)",
  async fn(bridge, ctx) {
    const conversa = "pages/__uitest-async-stream.md";
    await bridge.js(DISPARAR(["--stream", "{prompt}"], "pergunta", conversa, 30, "stream_json"));
    await PAUSA(1300);
    const meio = await bridge.js(ESTADO(conversa));
    ctx.assertEq(meio && meio.estado, "rodando", `esperava rodando, veio ${JSON.stringify(meio)}`);
    // O painel mostra o que ELE está fazendo, não o JSON cru.
    ctx.assert(
      meio.parcial.includes("· Read"),
      `progresso não traduziu o evento: ${JSON.stringify(meio.parcial)}`,
    );
    ctx.assert(!meio.parcial.includes('"type"'), "o JSON cru vazou pro painel");

    let fim = null;
    for (let i = 0; i < 40; i++) {
      await PAUSA(250);
      fim = await bridge.js(ESTADO(conversa));
      if (!fim || fim.estado !== "rodando") break;
    }
    ctx.assertEq(fim && fim.estado, "concluido", `esperava concluido, veio ${JSON.stringify(fim)}`);
    ctx.assertEq(fim.texto, "RESPOSTA para: pergunta", "a resposta não saiu do evento result");
  },
});

// ── ciclo 214: dialeto do Codex e troca de agente ────────────────────

fluxo.push({
  nome: "agente: dialeto do Codex vira progresso e resposta (214)",
  async fn(bridge, ctx) {
    const conversa = "pages/__uitest-codex-dialeto.md";
    await bridge.js(DISPARAR(["--codex", "{prompt}"], "pergunta", conversa, 30, "stream_json"));
    await PAUSA(1300);
    const meio = await bridge.js(ESTADO(conversa));
    ctx.assertEq(meio && meio.estado, "rodando", `esperava rodando, veio ${JSON.stringify(meio)}`);
    ctx.assert(
      meio.parcial.includes("Vou conferir a pasta."),
      `progresso sem a narração: ${JSON.stringify(meio.parcial)}`,
    );
    ctx.assert(!meio.parcial.includes('"type"'), "o JSON cru vazou pro painel");

    let fim = null;
    for (let i = 0; i < 40; i++) {
      await PAUSA(250);
      fim = await bridge.js(ESTADO(conversa));
      if (!fim || fim.estado !== "rodando") break;
    }
    ctx.assertEq(fim && fim.estado, "concluido", `esperava concluido, veio ${JSON.stringify(fim)}`);
    // O Codex narra o que VAI fazer antes de fazer; a resposta é o
    // último recado, não a soma deles.
    ctx.assertEq(fim.texto, "RESPOSTA para: pergunta", "a resposta não é o último recado");
  },
});

fluxo.push({
  nome: "agente: o progresso mostra o texto inteiro, não só a 1ª linha (214)",
  async fn(bridge, ctx) {
    const conversa = "pages/__uitest-progresso-inteiro.md";
    await bridge.js(DISPARAR(["--devagar", "{prompt}"], "x", conversa));
    await PAUSA(4000);
    const meio = await bridge.js(ESTADO(conversa));
    ctx.assertEq(meio && meio.estado, "rodando", `esperava rodando, veio ${JSON.stringify(meio)}`);
    // Guardar só a primeira linha escondia justamente o miolo do
    // raciocínio, que é o que diz se o agente entendeu o pedido.
    ctx.assert(
      meio.parcial.split("\n").length >= 3,
      `só veio ${meio.parcial.split("\n").length} linha(s): ${JSON.stringify(meio.parcial)}`,
    );
    await bridge.js(`window.__TAURI_INTERNALS__.invoke('cancelar_agente', { conversaPath: ${JSON.stringify(conversa)} })`);
  },
});

fluxo.push({
  nome: "conversa: trocar de agente pelo chip preserva o binário ajustado (214)",
  async fn(bridge, ctx) {
    const rel = "pages/__uitest-troca.md";
    const fs = await import("node:fs");
    const arquivo = `${ctx.vault}/${rel}`;
    fs.writeFileSync(arquivo, "---\ntitle: __uitest-troca\ntype: conversa\n---\n");
    const antes = await bridge.js(`localStorage.getItem('anotadinho.adaptador_agente')`);
    try {
      await recarregarEstavel(bridge);
      await abrirPaginaEstavel(bridge, "__uitest-troca");

      // Um binário ajustado à mão, como quem aponta pro próprio
      // caminho de instalação.
      await bridge.js(`(() => {
        localStorage.setItem('anotadinho.adaptador_agente', JSON.stringify({
          nome: 'Claude Code', binario: '/caminho/meu/claude',
          args: ['-p','--output-format','stream-json','--verbose','{prompt}'],
          cwd: '', timeout_s: 1800, formato: 'stream_json' }));
        localStorage.removeItem('anotadinho.adaptadores_agente');
        return true;
      })()`);
      await recarregarEstavel(bridge);
      await abrirPaginaEstavel(bridge, "__uitest-troca");

      // Clicar e LER em chamadas separadas: o Yew só re-renderiza no
      // tick seguinte, então ler junto pega a tela de antes.
      const achou = await bridge.js(`(() => {
        const chip = document.querySelector('.conversa__agente');
        if (!chip) return false;
        chip.click();
        return true;
      })()`);
      ctx.assert(achou, "não há chip de agente na conversa");
      await PAUSA(300);
      const opcoes = await bridge.js(
        `[...document.querySelectorAll('.conversa__agente-op')].map(o => o.textContent.trim())`,
      );
      ctx.assertEq(opcoes.length, 3, `esperava 3 agentes, veio ${JSON.stringify(opcoes)}`);

      // Vai pro Codex e volta: o caminho ajustado tem que sobreviver.
      await bridge.js(`(() => {
        [...document.querySelectorAll('.conversa__agente-op')]
          .find(o => o.textContent.includes('Codex')).click();
        return true;
      })()`);
      await PAUSA(500);
      const codex = await bridge.js(`JSON.parse(localStorage.getItem('anotadinho.adaptador_agente'))`);
      ctx.assertEq(codex.nome, "Codex", "não trocou pro Codex");
      ctx.assert(codex.args.includes("--json"), `Codex sem --json: ${JSON.stringify(codex.args)}`);

      await bridge.js(`(() => { document.querySelector('.conversa__agente').click(); return true; })()`);
      await PAUSA(300);
      await bridge.js(`(() => {
        [...document.querySelectorAll('.conversa__agente-op')]
          .find(o => o.textContent.includes('Claude')).click();
        return true;
      })()`);
      await PAUSA(500);
      const volta = await bridge.js(`JSON.parse(localStorage.getItem('anotadinho.adaptador_agente'))`);
      ctx.assertEq(
        volta.binario,
        "/caminho/meu/claude",
        "trocar de agente e voltar apagou o binário ajustado à mão",
      );
    } finally {
      fs.rmSync(arquivo, { force: true });
      await bridge.js(`(() => {
        const v = ${JSON.stringify(antes)};
        if (v === null) localStorage.removeItem('anotadinho.adaptador_agente');
        else localStorage.setItem('anotadinho.adaptador_agente', v);
        localStorage.removeItem('anotadinho.adaptadores_agente');
        return true;
      })()`);
    }
  },
});

// ── ciclo 215: trava contra esvaziamento e raiz do projeto ───────────

fluxo.push({
  nome: "vault: gravar vazio por cima de página com conteúdo é recusado (215)",
  async fn(bridge, ctx) {
    const fs = await import("node:fs");
    const rel = "pages/__uitest-nao-esvaziar.md";
    const arquivo = `${ctx.vault}/${rel}`;
    fs.writeFileSync(arquivo, "---\ntitle: __uitest-nao-esvaziar\n---\ntexto que importa\n");
    try {
      // Duas propostas do vault foram zeradas sem que a causa fosse
      // reproduzida. A trava fica no ponto por onde TODO escritor passa,
      // então não depende de saber quem escreveu.
      const r = await bridge.js(`(async () => {
        try {
          await window.__TAURI_INTERNALS__.invoke('write_page', {
            vaultPath: ${JSON.stringify(`${process.cwd()}/${ctx.vault}`)},
            pagePath: ${JSON.stringify(rel)},
            content: '',
          });
          return { ok: true };
        } catch (e) { return { ok: false, erro: String(e) }; }
      })()`);
      ctx.assertEq(r.ok, false, "o app aceitou apagar o conteúdo da página");
      ctx.assert(/recusada/i.test(r.erro), `erro não explica o motivo: ${r.erro}`);
      ctx.assert(
        fs.readFileSync(arquivo, "utf8").includes("texto que importa"),
        "o conteúdo se perdeu mesmo com a recusa",
      );
    } finally {
      fs.rmSync(arquivo, { force: true });
    }
  },
});

fluxo.push({
  nome: "agente: sem cwd configurado, trabalha na raiz do projeto (215)",
  async fn(bridge, ctx) {
    const conversa = "pages/__uitest-cwd.md";
    // O agente falso ecoa o prompt; o que importa aqui é ONDE ele roda,
    // então o cenário confere o diretório pelo próprio processo.
    await bridge.js(DISPARAR(["--onde", "{prompt}"], "x", conversa, 30, "texto"));
    let fim = null;
    for (let i = 0; i < 40; i++) {
      await PAUSA(250);
      fim = await bridge.js(ESTADO(conversa));
      if (!fim || fim.estado !== "rodando") break;
    }
    ctx.assertEq(fim && fim.estado, "concluido", `esperava concluido, veio ${JSON.stringify(fim)}`);
    // Rodar dentro do vault deixava o agente sem enxergar o código que
    // a proposta manda mudar — e com escrita justo nas notas.
    ctx.assert(
      !fim.texto.trim().endsWith("VaultAnotadinho"),
      `o agente rodou dentro do vault: ${fim.texto}`,
    );
    ctx.assert(
      fim.texto.includes("Anotadinho"),
      `diretório inesperado: ${fim.texto}`,
    );
  },
});

// ── ciclo 216: execução continua na conversa de origem ───────────────

fluxo.push({
  nome: "fluxo: executar continua na conversa que gerou a proposta (216)",
  async fn(bridge, ctx) {
    const fs = await import("node:fs");
    const conversa = "pages/conversas/__uitest-origem.md";
    const proposta = "pages/propostas/__uitest-com-origem.md";
    fs.mkdirSync(`${ctx.vault}/pages/conversas`, { recursive: true });
    fs.mkdirSync(`${ctx.vault}/pages/propostas`, { recursive: true });
    fs.writeFileSync(
      `${ctx.vault}/${conversa}`,
      "---\ntitle: __uitest-origem\ntype: conversa\n---\n## você · 2026-01-01 00:00\n\noi\n",
    );
    fs.writeFileSync(
      `${ctx.vault}/${proposta}`,
      [
        "---",
        "title: __uitest-com-origem",
        "type: proposta",
        "status: aprovada",
        "---",
        "# __uitest-com-origem",
        "",
        '{{ type: "fluxo" }}',
        "artefato: proposta",
        "etapa: aprovada",
        `origem: ${conversa}`,
        "{{ /fluxo }}",
        "",
        "corpo da proposta",
        "",
      ].join("\n"),
    );
    try {
      await recarregarEstavel(bridge);
      await abrirPaginaEstavel(bridge, "__uitest-com-origem");
      const antes = fs.readdirSync(`${ctx.vault}/pages/conversas`).length;

      await bridge.js(`(() => {
        const b = [...document.querySelectorAll('[class*=fluxo] button')]
          .find(x => x.textContent.trim() === 'Executar');
        if (!b) return false;
        b.click();
        return true;
      })()`);
      await PAUSA(2500);

      // Abrir conversa nova a cada execução espalhava o histórico: a
      // discussão que produziu a proposta numa página, o que o agente
      // fez pra executá-la noutra.
      const depois = fs.readdirSync(`${ctx.vault}/pages/conversas`).length;
      ctx.assertEq(depois, antes, "criou uma conversa nova em vez de continuar na de origem");

      const aberta = await bridge.js(
        `(document.querySelector('.conversa__titulo') || {}).textContent || null`,
      );
      ctx.assertEq(aberta, "__uitest-origem", `abriu a página errada: ${aberta}`);

      const rascunho = await bridge.js(
        `(document.querySelector('.conversa__campo') || {}).value || ''`,
      );
      ctx.assert(
        rascunho.trim().length > 0,
        "a conversa abriu sem a pergunta de execução preenchida",
      );
    } finally {
      fs.rmSync(`${ctx.vault}/${conversa}`, { force: true });
      fs.rmSync(`${ctx.vault}/${proposta}`, { force: true });
    }
  },
});

fluxo.push({
  nome: "agente: pastas extras viram --add-dir na execução (216)",
  async fn(bridge, ctx) {
    const conversa = "pages/__uitest-pastas.md";
    // O agente falso ecoa os argumentos que recebeu.
    const r = await bridge.js(`(async () => {
      await window.__TAURI_INTERNALS__.invoke('iniciar_agente', {
        adaptador: {
          nome: 'falso',
          binario: ${JSON.stringify(FALSO)},
          args: ['--args', '{prompt}'],
          cwd: '',
          pastas_extras: ['/repo/um', '/repo/dois'],
          arg_pasta_extra: '--add-dir',
          timeout_s: 30,
          formato: 'texto',
        },
        prompt: 'x',
        vaultPath: '/home/elis/Anotadinho',
        conversaPath: ${JSON.stringify(conversa)},
      });
      for (let i = 0; i < 40; i++) {
        await new Promise(r => setTimeout(r, 200));
        const e = await window.__TAURI_INTERNALS__.invoke('estado_agente', { conversaPath: ${JSON.stringify(conversa)} });
        if (!e) return { ok: false, erro: 'sumiu' };
        if (e.estado !== 'rodando') return { ok: e.estado === 'concluido', saida: e.texto, erro: e.erro };
      }
      return { ok: false, erro: 'demorou' };
    })()`);
    ctx.assertEq(r.ok, true, `execução falhou: ${r.erro}`);
    // Quem tem o vault num lugar e os repositórios noutro precisa que
    // TODOS cheguem ao agente, não só um.
    ctx.assert(
      r.saida.includes("--add-dir /repo/um") && r.saida.includes("--add-dir /repo/dois"),
      `as pastas extras não chegaram: ${r.saida}`,
    );
  },
});

// ── ciclo 219: o motivo da falha não pode ser engolido pelo ruído ────

fluxo.push({
  nome: "agente: falha mostra o motivo do stream, não o ruído do stderr (219)",
  async fn(bridge, ctx) {
    const conversa = "pages/__uitest-motivo.md";
    await bridge.js(DISPARAR(["--falha-no-stream", "{prompt}"], "x", conversa, 30, "stream_json"));
    let fim = null;
    for (let i = 0; i < 40; i++) {
      await PAUSA(200);
      fim = await bridge.js(ESTADO(conversa));
      if (!fim || fim.estado !== "rodando") break;
    }
    ctx.assertEq(fim && fim.estado, "falhou", `esperava falhou, veio ${JSON.stringify(fim)}`);
    // Caso real: a conta do Codex bateu o limite e a tela mostrou
    // "Reading additional input from stdin...", que não diz nada.
    ctx.assert(
      fim.erro.includes("bateu o limite de uso"),
      `a mensagem não traz o motivo: ${fim.erro}`,
    );
    ctx.assert(
      !fim.erro.includes("stdin"),
      `o ruído do stderr venceu o motivo: ${fim.erro}`,
    );
  },
});

// ── ciclo 223: pedir alteração no que está em revisão ────────────────

fluxo.push({
  nome: "fluxo: em revisão oferece pedir alteração, e ela manda propor (223)",
  async fn(bridge, ctx) {
    const fs = await import("node:fs");
    const rel = "pages/specs/__uitest-em-revisao.md";
    const arquivo = `${ctx.vault}/${rel}`;
    fs.mkdirSync(`${ctx.vault}/pages/specs`, { recursive: true });
    fs.writeFileSync(
      arquivo,
      [
        "---",
        "title: __uitest-em-revisao",
        "type: spec",
        "status: em-revisao",
        "---",
        "# __uitest-em-revisao",
        "",
        '{{ type: "fluxo" }}',
        "artefato: spec",
        "etapa: em-revisao",
        "{{ /fluxo }}",
        "",
        "corpo",
        "",
      ].join("\n"),
    );
    const antesConversas = fs.existsSync(`${ctx.vault}/pages/conversas`)
      ? fs.readdirSync(`${ctx.vault}/pages/conversas`).length
      : 0;
    try {
      await recarregarEstavel(bridge);
      await abrirPaginaEstavel(bridge, "__uitest-em-revisao");

      const achou = await bridge.js(`(() => {
        const b = [...document.querySelectorAll('[class*=fluxo] button')]
          .find(x => x.textContent.includes('Pedir alteração'));
        if (!b) return false;
        b.click();
        return true;
      })()`);
      // Revisar só tinha duas saídas — aprovar ou mandar pra trás.
      ctx.assertEq(achou, true, "não há botão de pedir alteração em revisão");
      await PAUSA(2500);

      const rascunho = await bridge.js(
        `(document.querySelector('.conversa__campo') || {}).value || ''`,
      );
      ctx.assert(rascunho.includes("__uitest-em-revisao"), `pergunta sem o título: ${rascunho}`);
      // É isto que faz a mudança voltar como diff em vez de já aplicada.
      ctx.assert(rascunho.includes("propor"), `a pergunta não manda propor: ${rascunho}`);
      ctx.assert(rascunho.includes("NÃO grave"), `a pergunta não barra a gravação: ${rascunho}`);
      // A pergunta não pode carregar indentação do código-fonte.
      ctx.assert(!rascunho.includes("  "), `espaço duplo na pergunta: ${rascunho}`);
    } finally {
      fs.rmSync(arquivo, { force: true });
      const depois = fs.existsSync(`${ctx.vault}/pages/conversas`)
        ? fs.readdirSync(`${ctx.vault}/pages/conversas`)
        : [];
      depois
        .slice(antesConversas)
        .forEach((f) => fs.rmSync(`${ctx.vault}/pages/conversas/${f}`, { force: true }));
    }
  },
});
