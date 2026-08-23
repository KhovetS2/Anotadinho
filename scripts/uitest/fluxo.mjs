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

/// Chama o comando de execução direto, sem passar pela UI.
const RODAR = (args, prompt, timeout = 30) => `(async () => {
  try {
    const r = await window.__TAURI_INTERNALS__.invoke('rodar_agente', {
      adaptador: {
        nome: 'falso',
        binario: ${JSON.stringify(FALSO)},
        args: ${JSON.stringify(args)},
        cwd: '',
        timeout_s: ${timeout},
      },
      prompt: ${JSON.stringify(prompt)},
      vaultPath: '/home/elis/Anotadinho',
    });
    return { ok: true, saida: r };
  } catch (e) {
    return { ok: false, erro: String(e) };
  }
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
        await window.__TAURI_INTERNALS__.invoke('rodar_agente', {
          adaptador: { nome: 'x', binario: 'sh -c', args: ['{prompt}'], cwd: '', timeout_s: 10 },
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

      // Aplicar escreve.
      await bridge.js(`(() => {
        [...document.querySelectorAll('.propostas__acoes button')]
          .find(b => b.textContent.trim() === 'Aplicar').click();
        return true;
      })()`);
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

      await bridge.js(`(() => {
        [...document.querySelectorAll('.propostas__acoes button')]
          .find(b => b.textContent.trim() === 'Recusar').click();
        return true;
      })()`);
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

// ── ciclo 208: conversa em um passo e contexto anexável ──────────────

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
