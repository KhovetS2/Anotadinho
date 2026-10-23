#!/usr/bin/env node
// Quanto o Anotadinho ocupa de memória num dia de uso (ciclo 481).
//
//   node scripts/memoria/janela.mjs                 # o app de dev (9223)
//   node scripts/memoria/janela.mjs --porta 9323    # o app INSTALADO
//   node scripts/memoria/janela.mjs --rodadas 8 --saida /tmp/x.json
//
// Mede o processo do app e os processos auxiliares do WebKit (a página
// web roda num processo à parte, e a rede noutro) enquanto dirige a
// janela pelo mesmo WebSocket do harness, repetindo um "dia" em rodadas.
//
// O número que importa é o PSS: o RSS conta as bibliotecas
// compartilhadas inteiras em CADA processo, e somar o RSS do app com o
// do WebKit conta a GTK duas vezes. O PSS divide o que é compartilhado
// entre quem compartilha, e a soma dele é o que a máquina gasta de
// verdade. O RSS vai junto porque é o que o monitor do sistema mostra.
//
// Não altera nada da pessoa: tudo o que o dia usa é criado em
// `pages/uitest-memoria/` e `pages/projetos/uitest-memoria/` do vault
// aberto, e removido no fim — inclusive se o script falhar ou for
// interrompido com Ctrl+C.
//
// Linux: lê o /proc.

import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { pidDaPorta, amostrar } from "./proc.mjs";

// ── argumentos ───────────────────────────────────────────────────────

function arg(nome, padrao) {
  const i = process.argv.indexOf(`--${nome}`);
  return i > 0 && process.argv[i + 1] ? process.argv[i + 1] : padrao;
}
const PORTA = Number(arg("porta", 9223));
const RODADAS = Number(arg("rodadas", 5));
const INTERVALO_MS = 500;
const SAIDA = arg("saida", path.join(os.tmpdir(), `anotadinho-memoria-janela-${Date.now()}.json`));
if (process.platform !== "linux") {
  console.error("✗ este script lê o /proc: só roda no Linux.");
  process.exit(2);
}
// A porta vai pelo ambiente ANTES de importar a ponte: ela lê na carga.
process.env.ANOTADINHO_MCP_PORT = String(PORTA);
const { Bridge, esperar, clicar, abrirTodasAsPastas } = await import("../uitest/bridge.mjs");

// ── o vault do dia ───────────────────────────────────────────────────

const N_PAGINAS = 20;
const N_TAREFAS = 300;
const PASTA = "pages/uitest-memoria";
const PROJETO = "uitest-memoria";

function montarDia(vault) {
  const pasta = path.join(vault, PASTA);
  fs.mkdirSync(pasta, { recursive: true });
  for (let i = 1; i <= N_PAGINAS; i++) {
    // Uma nota "de verdade": títulos, parágrafos, lista, tabela e embeds,
    // que é o que o editor desenha o dia inteiro.
    const blocos = [];
    for (let b = 0; b < 40; b++) {
      blocos.push(b % 10 === 0 ? `## Seção ${b / 10 + 1}` : `Parágrafo ${b} da nota ${i}, com [[Nota ${(i % N_PAGINAS) + 1}]] e **negrito** no meio de um texto comum.`);
    }
    blocos.push("- [ ] item um\n- [x] item dois\n- [ ] item três");
    blocos.push("| a | b |\n|---|---|\n| 1 | 2 |");
    blocos.push('{{ type: "callout" }}\nvariant: info\ntitle: Nota\nbody: um destaque\n{{ /callout }}');
    if (i % 4 === 0) blocos.push('{{ type: "kanban" }}\ncolumns:\n- A\n- B\nitems:\n- title: um\n  column: A\n- title: dois\n  column: B\n{{ /kanban }}');
    if (i % 5 === 0) blocos.push('{{ type: "query" }}\nfrom: pages/uitest-memoria\nview: list\nlimit: 10\n{{ /query }}');
    fs.writeFileSync(path.join(pasta, `nota-${i}.md`), `---\ntitle: Nota ${i}\ntags:\n- uitest-memoria\n---\n# Nota ${i}\n\n${blocos.join("\n\n")}\n`);
  }
  // A conversa com histórico: 30 mensagens.
  const msgs = [];
  for (let m = 0; m < 30; m++) {
    const quem = m % 2 ? "agente" : "você";
    msgs.push(`## ${quem} · 2026-10-23 10:${String(m).padStart(2, "0")}\n\n${"Uma resposta com várias linhas de texto pra pesar como uma conversa real. ".repeat(m % 2 ? 12 : 2)}`);
  }
  fs.writeFileSync(path.join(pasta, "conversa.md"), `---\ntitle: Conversa memoria\ntype: conversa\ntags:\n- conversa\n---\n${msgs.join("\n\n")}\n`);
  fs.writeFileSync(path.join(pasta, "rascunho.md"), "---\ntitle: Rascunho memoria\n---\n# Rascunho memoria\n\ncomeço\n");
  // O grafo por uma página `type: graph` DAQUI: a paleta só tem "Nova
  // página: Grafo", que gravaria um arquivo no vault da pessoa.
  fs.writeFileSync(path.join(pasta, "grafo.md"), "---\ntitle: Grafo memoria\ntype: graph\n---\n");

  // O projeto que deu certo: 300 tarefas, sprints, detalhes.
  const proj = path.join(vault, "pages/projetos", PROJETO);
  fs.mkdirSync(path.join(proj, "tarefas"), { recursive: true });
  fs.mkdirSync(path.join(proj, "sprints"), { recursive: true });
  fs.mkdirSync(path.join(proj, "conversas"), { recursive: true });
  fs.writeFileSync(path.join(proj, "projeto.md"), `---\ntitle: Projeto memoria\ntype: projeto\nprojeto: ${PROJETO}\n---\n# Projeto memoria\n`);
  fs.writeFileSync(path.join(proj, "arquitetura.md"), `---\ntitle: Arquitetura\ntype: docs\nprojeto: ${PROJETO}\n---\n# Arquitetura\n`);
  const sprints = N_TAREFAS / 10;
  for (let s = 1; s <= sprints; s++) {
    const estado = s === sprints ? "ativa" : "encerrada";
    fs.writeFileSync(path.join(proj, "sprints", `${String(s).padStart(2, "0")}-s${s}.md`), `---\ntitle: Sprint ${s}\ntype: sprint\nprojeto: ${PROJETO}\nestado: ${estado}\n---\n# Sprint ${s}\n`);
  }
  const etapas = ["ideia", "detalhando", "pronta", "bloqueada"];
  for (let t = 1; t <= N_TAREFAS; t++) {
    const s = Math.ceil(t / 10);
    const etapa = s === sprints ? etapas[t % etapas.length] : "feita";
    fs.writeFileSync(
      path.join(proj, "tarefas", `${String(t).padStart(3, "0")}-t.md`),
      `---\ntitle: Tarefa ${t}\ntags:\n- tarefa\n- t${t % 7}\ntype: tarefa\nprojeto: ${PROJETO}\nsprint: ${s}\nprazo: 2026-11-${String((t % 28) + 1).padStart(2, "0")}\nstatus: ${etapa}\n---\n# Tarefa ${t}\n\n## Critérios de aceite\n\n- [x] um\n- [ ] dois\n\n## Comentários\n\n- **2026-10-01 10:00** — visto\n`,
    );
  }
}

function desmontarDia(vault) {
  fs.rmSync(path.join(vault, PASTA), { recursive: true, force: true });
  fs.rmSync(path.join(vault, "pages/projetos", PROJETO), { recursive: true, force: true });
}

// ── o dia ────────────────────────────────────────────────────────────

const PAUSA = (ms) => new Promise((r) => setTimeout(r, ms));

async function abrirPelaPaleta(bridge, termo, espera) {
  await bridge.js(`(() => { document.querySelector('.app-root').dispatchEvent(new KeyboardEvent('keydown', { key: 'k', ctrlKey: true, bubbles: true, cancelable: true })); return true; })()`);
  await esperar(bridge, "document.querySelector('.command-palette__input')", "a paleta abrir");
  await bridge.js(`(() => { const i = document.querySelector('.command-palette__input'); Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value').set.call(i, ${JSON.stringify(termo)}); i.dispatchEvent(new InputEvent('input', { bubbles: true })); return true; })()`);
  await clicar(bridge, ".command-palette__item", `${termo} na paleta`);
  await esperar(bridge, espera, `${termo} abrir`, 15000);
}

/// A página abriu: o cabeçalho traz o título OU o nome do arquivo. Aberta
/// pela paleta, a janela mostra o nome do arquivo (`nota-1`), e esperar
/// só pelo título ("Nota 1") nunca casava.
const ABRIU = (titulo, arquivo) => `[...document.querySelectorAll('.editor__title, .conversa__titulo, .projeto__titulo')].some(e => e.textContent.includes(${JSON.stringify(titulo)}) || e.textContent.includes(${JSON.stringify(arquivo)}))`;

const ACOES = [
  ["abrir 20 notas", async (b) => {
    for (let i = 1; i <= N_PAGINAS; i++) await abrirPelaPaleta(b, `${PASTA}/nota-${i}`, ABRIU(`Nota ${i}`, `nota-${i}`));
  }],
  ["digitar 2000 caracteres", async (b) => {
    await abrirPelaPaleta(b, `${PASTA}/rascunho`, ABRIU("Rascunho memoria", "rascunho"));
    await esperar(b, "!!document.querySelector('.editor__bloco [contenteditable=true], .editor__bloco[contenteditable=true]')", "o editor");
    await b.js(`(() => { const el = [...document.querySelectorAll('[contenteditable=true]')].pop(); el.focus(); const r = document.createRange(); r.selectNodeContents(el); r.collapse(false); const s = getSelection(); s.removeAllRanges(); s.addRange(r); return true; })()`);
    // De 100 em 100, como rajadas de digitação: um `insertText` por
    // caractere passaria pelo mesmo `oninput` que a tecla de verdade.
    for (let i = 0; i < 20; i++) {
      await b.js(`(() => { for (const ch of ${JSON.stringify("texto digitado pela pessoa no meio do dia. ".repeat(3).slice(0, 100))}) document.execCommand('insertText', false, ch); return true; })()`);
    }
  }],
  ["buscar", async (b) => {
    for (const termo of ["Parágrafo", "negrito", "Seção 3"]) {
      await b.js(`(() => { document.querySelector('.app-root').dispatchEvent(new KeyboardEvent('keydown', { key: 'k', ctrlKey: true, bubbles: true, cancelable: true })); return true; })()`);
      await esperar(b, "document.querySelector('.command-palette__input')", "a paleta");
      await b.js(`(() => { const i = document.querySelector('.command-palette__input'); Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value').set.call(i, ${JSON.stringify(termo)}); i.dispatchEvent(new InputEvent('input', { bubbles: true })); return true; })()`);
      await esperar(b, "document.querySelectorAll('.command-palette__item').length > 0", "resultados");
      await b.js(`(() => { document.querySelector('.command-palette__input').dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true })); return true; })()`);
    }
  }],
  ["projeto com 300 tarefas", async (b) => {
    await abrirPelaPaleta(b, `pages/projetos/${PROJETO}/projeto`, "!!document.querySelector('.projeto__cartao')");
    for (const f of ["todas", "backlog", "ativa"]) {
      await b.js(`(() => { const s = document.querySelector('.projeto__sprint-filtro'); s.value = ${JSON.stringify(f)}; s.dispatchEvent(new Event('change', { bubbles: true })); return true; })()`);
      await esperar(b, "!!document.querySelector('.projeto__quadro')", "o quadro");
    }
    await clicar(b, ".projeto__cartao .projeto__detalhes-abrir", "os detalhes");
    await esperar(b, "!!document.querySelector('.tarefa-detalhes')", "o painel");
    for (const aba of ["Sprints", "Execuções", "Quadro"]) {
      await b.js(`(() => { [...document.querySelectorAll('.projeto__aba')].find(a => a.textContent.trim() === ${JSON.stringify(aba)}).click(); return true; })()`);
    }
    // Três voltas do recarregamento de 3 s da tela do projeto.
    await PAUSA(9000);
  }],
  ["conversa com 30 mensagens", async (b) => {
    await abrirPelaPaleta(b, `${PASTA}/conversa`, ABRIU("Conversa memoria", "conversa"));
  }],
  ["grafo", async (b) => {
    await abrirPelaPaleta(b, `${PASTA}/grafo`, "!!document.querySelector('.graph-view__canvas')");
  }],
  ["fechar abas", async (b) => {
    for (let i = 0; i < 30; i++) {
      const fechou = await b.js(`(() => { const x = [...document.querySelectorAll('.tab-bar__tab-close')].pop(); if (x) x.click(); return !!x; })()`);
      if (!fechou) break;
    }
  }],
];

// ── relatório ────────────────────────────────────────────────────────

const MB = (kib) => (kib / 1024).toFixed(1);

function relatorio(amostras, marcos, build) {
  const papeis = [...new Set(amostras.flatMap((a) => Object.keys(a.mem)))].sort((a, b) => (a === "total") - (b === "total") || a.localeCompare(b));
  const valor = (a, p, k = "pss") => a.mem[p]?.[k] ?? 0;
  const repouso = amostras.filter((a) => a.fase === "repouso");
  const fimDe = (r) => amostras.filter((a) => a.rodada === r).at(-1);
  const linhas = [];
  linhas.push(`\nMemória do Anotadinho — ${build} · ${RODADAS} rodada(s) · ${amostras.length} amostras`);
  linhas.push("PSS em MiB (RSS entre parênteses: o que o monitor do sistema mostra)\n");
  linhas.push(["processo", "repouso", "pico", "fim", "cresce/rodada"].map((s, i) => (i ? s.padStart(16) : s.padEnd(15))).join(""));
  const resumo = {};
  for (const p of papeis) {
    const rep = repouso.length ? Math.max(...repouso.map((a) => valor(a, p))) : 0;
    const repRss = repouso.length ? Math.max(...repouso.map((a) => valor(a, p, "rss"))) : 0;
    const pico = Math.max(...amostras.map((a) => valor(a, p)));
    const picoRss = Math.max(...amostras.map((a) => valor(a, p, "rss")));
    const ultima = amostras.at(-1);
    // O crescimento por rodada é a inclinação entre o fim da 1ª e o fim
    // da última: a 1ª carrega o que fica em cache de propósito, e um
    // vazamento aparece como subida que não para.
    const f1 = fimDe(1), fN = fimDe(RODADAS);
    const cresce = RODADAS > 1 && f1 && fN ? (valor(fN, p) - valor(f1, p)) / (RODADAS - 1) : 0;
    resumo[p] = { repouso_kib: rep, pico_kib: pico, fim_kib: valor(ultima, p), cresce_por_rodada_kib: Math.round(cresce), repouso_rss_kib: repRss, pico_rss_kib: picoRss };
    linhas.push(
      [p.padEnd(15), `${MB(rep)} (${MB(repRss)})`.padStart(16), `${MB(pico)} (${MB(picoRss)})`.padStart(16), `${MB(valor(ultima, p))}`.padStart(16), `${cresce >= 0 ? "+" : ""}${MB(cresce)}`.padStart(16)].join(""),
    );
  }
  linhas.push("\nfim de cada rodada (PSS total, MiB): " + Array.from({ length: RODADAS }, (_, i) => MB(valor(fimDe(i + 1) || { mem: {} }, "total"))).join(" → "));
  // O fim da rodada cai com todas as abas fechadas, e aí a janela desenha
  // a página inicial da pessoa (consultas sobre o vault inteiro): ponto
  // de pico, não de repouso. O fim de CADA ação, por rodada, mostra onde
  // a memória sobe e se ela volta (achado na primeira medida, ciclo 480).
  linhas.push("fim de cada ação, por rodada (PSS total, MiB):");
  for (const [acao] of ACOES) {
    const porRodada = Array.from({ length: RODADAS }, (_, i) => amostras.filter((a) => a.rodada === i + 1 && a.acao === acao).at(-1));
    linhas.push(`  ${acao.padEnd(28)} ${porRodada.map((a) => (a ? MB(valor(a, "total")) : "—").padStart(7)).join("")}`);
  }
  linhas.push("pico por ação (PSS total, MiB):");
  for (const [acao, kib] of Object.entries(marcos)) linhas.push(`  ${acao.padEnd(28)} ${MB(kib)}`);
  return { texto: linhas.join("\n"), resumo };
}

// ── principal ────────────────────────────────────────────────────────

const app = pidDaPorta(PORTA);
const exe = fs.readlinkSync(`/proc/${app}/exe`);
const build = /target\/debug\//.test(exe) ? `desenvolvimento (${exe})` : `instalado (${exe})`;
const bridge = await Bridge.conectar();
const bruto = await bridge.js("localStorage.getItem('anotadinho.vault_path')");
let vault;
try { vault = JSON.parse(bruto); } catch { vault = bruto; }
if (!vault || !fs.existsSync(vault)) {
  console.error(`✗ não achei o vault aberto no app (veio ${JSON.stringify(bruto)}).`);
  process.exit(1);
}
if (fs.existsSync(path.join(vault, PASTA)) || fs.existsSync(path.join(vault, "pages/projetos", PROJETO))) {
  console.error(`✗ ${PASTA} ou o projeto ${PROJETO} já existem em ${vault}: sobra de uma rodada interrompida? Apague e rode de novo.`);
  process.exit(1);
}
console.log(`app ${app} · ${build}\nvault ${vault}`);

const amostras = [];
let fase = "repouso", rodada = 0, acaoAtual = "repouso";
const marcos = {};
const t0 = Date.now();
const amostrador = setInterval(() => {
  const mem = amostrar(app);
  amostras.push({ t: Date.now() - t0, fase, rodada, acao: acaoAtual, mem });
  const k = acaoAtual;
  marcos[k] = Math.max(marcos[k] ?? 0, mem.total.pss);
}, INTERVALO_MS);

let limpo = false;
const limpar = () => {
  if (limpo) return;
  limpo = true;
  clearInterval(amostrador);
  desmontarDia(vault);
};
process.on("SIGINT", () => { limpar(); console.error("\ninterrompido: páginas temporárias removidas."); process.exit(130); });

try {
  // Repouso: o app aberto, parado, antes de qualquer coisa.
  await PAUSA(5000);
  montarDia(vault);
  // O watcher vê as páginas novas e o app reindexa; espera a sidebar.
  await esperar(bridge, `document.body.innerText.includes('uitest-memoria')`, "as páginas do dia no índice", 30000).catch(() => {});
  await abrirTodasAsPastas(bridge);
  fase = "dia";
  for (rodada = 1; rodada <= RODADAS; rodada++) {
    for (const [nome, fn] of ACOES) {
      acaoAtual = nome;
      process.stdout.write(`rodada ${rodada}/${RODADAS}: ${nome}…`.padEnd(48) + "\r");
      await fn(bridge);
    }
    acaoAtual = "assentar";
    // Deixa o coletor de lixo do JS e o WebKit assentarem antes da
    // amostra de fim de rodada — senão a "subida" é só lixo ainda não
    // recolhido.
    await PAUSA(3000);
  }
  rodada = RODADAS;
  fase = "fim";
  await PAUSA(1500);
} catch (e) {
  limpar();
  console.error(`\n✗ o dia parou: ${e.message}`);
  process.exit(1);
} finally {
  limpar();
  bridge.fechar?.();
}

const { texto, resumo } = relatorio(amostras, marcos, build);
console.log(texto);
fs.writeFileSync(SAIDA, JSON.stringify({ build, app, vault, rodadas: RODADAS, intervalo_ms: INTERVALO_MS, resumo, marcos, amostras }, null, 1));
console.log(`\namostras em ${SAIDA}`);
process.exit(0);
