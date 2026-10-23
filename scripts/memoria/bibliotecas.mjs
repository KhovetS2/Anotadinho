#!/usr/bin/env node
// Quanto pesam as bibliotecas JS: carregar e usar (ciclo 484).
//
//   node scripts/memoria/bibliotecas.mjs --porta 9323
//   node scripts/memoria/bibliotecas.mjs --so carregar
//   node scripts/memoria/bibliotecas.mjs --porta 9323 --so usar
//
// O `index.html` carrega mermaid, 3d-force-graph (com o three) e highlight
// na abertura. Duas perguntas diferentes, e cada uma pede uma medida:
//
// CARREGAR — quanto custa só ter a biblioteca lida e compilada. No app não
// dá pra tirar uma delas sem outra build, então a medida é no MiniBrowser
// do próprio WebKitGTK: uma página com a biblioteca sozinha contra uma em
// branco, cada uma num processo novo. É o que um carregamento sob demanda
// pouparia a quem nunca usa a biblioteca.
//
// USAR — quanto o primeiro uso acrescenta e deixa preso. No app: recarrega
// a página web, mede, abre a página que usa a biblioteca, fecha as abas e
// mede de novo. O controle faz o mesmo com uma nota sem nada pesado.
//
// As páginas do USAR moram em `pages/uitest-bibliotecas/` do vault aberto e
// saem no fim, inclusive com Ctrl+C.

import fs from "node:fs";
import path from "node:path";
import http from "node:http";
import { spawn, execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { memoria, pidDaPorta, amostrar } from "./proc.mjs";

function arg(nome, padrao) {
  const i = process.argv.indexOf(`--${nome}`);
  return i > 0 && process.argv[i + 1] ? process.argv[i + 1] : padrao;
}
const PORTA = Number(arg("porta", 9223));
const SO = arg("so", "");
const REPETICOES = Number(arg("repeticoes", 2));
const RAIZ = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const VENDOR = path.join(RAIZ, "ui/vendor");
const MINIBROWSER = "/usr/libexec/webkit2gtk-4.1/MiniBrowser";
const PASTA = "pages/uitest-bibliotecas";
const PAUSA = (ms) => new Promise((r) => setTimeout(r, ms));
const MiB = (kib) => (kib / 1024).toFixed(1).padStart(7);

// ── carregar ─────────────────────────────────────────────────────────

// Cada biblioteca e o global que prova que ela carregou.
const LIBS = {
  branco: [[], []],
  highlight: [["highlight.min.js"], ["hljs"]],
  "3d-force-graph": [["3d-force-graph.min.js"], ["ForceGraph3D"]],
  mermaid: [["mermaid.min.js"], ["mermaid"]],
  "as três": [["mermaid.min.js", "3d-force-graph.min.js", "highlight.min.js"], ["mermaid", "ForceGraph3D", "hljs"]],
  // O primeiro uso, num processo novo: no app, recarregar a página não
  // devolve memória ao sistema (o WebKit fica na marca máxima), então lá
  // não dá pra ver o primeiro uso de novo sem reabrir o app.
  "highlight, usado": [["highlight.min.js"], ["hljs"], `(() => { document.body.insertAdjacentHTML("beforeend", "<pre><code class=language-rust>" + "fn main() { let x: Vec<u32> = (0..10).collect(); }\\n".repeat(200) + "</code></pre>"); document.querySelectorAll("pre code").forEach((c) => hljs.highlightElement(c)); return document.querySelectorAll(".hljs").length; })()`],
  "3d-force-graph, usado": [["3d-force-graph.min.js"], ["ForceGraph3D"], `(() => { const d = document.createElement("div"); d.style.cssText = "width:900px;height:700px"; document.body.append(d); const nodes = Array.from({ length: 300 }, (_, i) => ({ id: i })); const links = nodes.slice(1).map((n) => ({ source: n.id, target: Math.floor(Math.random() * n.id) })); ForceGraph3D()(d).graphData({ nodes, links }); return new Promise((r) => setTimeout(() => r(d.querySelectorAll("canvas").length), 3000)); })()`],
  "mermaid, usado": [["mermaid.min.js"], ["mermaid"], `mermaid.render("m484", "graph TD\\n  A[Início] --> B{Decide}\\n  B -->|sim| C[Faz]\\n  B -->|não| D[Espera]\\n  C --> E[Fim]\\n  D --> E").then((r) => r.svg.length)`],
};

/// Serve as páginas por HTTP local. Por `file://` o mermaid não carrega
/// ("Script error.", o erro opaco de origem), e a medida dava o custo de
/// um script que falhou — parecia plausível, e era do arquivo errado.
/// A página avisa em `/carregou` quais globais existem: sem o aviso, ou
/// com um global faltando, a medida é recusada.
function servidor() {
  const paginas = new Map();
  const avisos = new Map();
  const srv = http.createServer((req, res) => {
    const url = new URL(req.url, "http://x");
    if (url.pathname === "/carregou") {
      avisos.set(url.searchParams.get("p"), JSON.parse(url.searchParams.get("g")));
      res.end("ok");
    } else if (paginas.has(url.pathname)) {
      res.setHeader("content-type", "text/html; charset=utf-8");
      res.end(paginas.get(url.pathname));
    } else if (/^\/[\w.-]+\.js$/.test(url.pathname) && fs.existsSync(path.join(VENDOR, url.pathname))) {
      res.setHeader("content-type", "text/javascript");
      res.end(fs.readFileSync(path.join(VENDOR, url.pathname)));
    } else {
      res.statusCode = 404;
      res.end();
    }
  });
  return new Promise((r) => srv.listen(0, "127.0.0.1", () => r({ srv, paginas, avisos, porta: srv.address().port })));
}

function filhos(pid) {
  try {
    return execFileSync("pgrep", ["-P", String(pid)], { encoding: "utf8" }).trim().split("\n").map(Number);
  } catch {
    return [];
  }
}

/// O processo da página do MiniBrowser: o `WebKitWebProcess` que ele
/// cria (às vezes por baixo de um `bwrap`).
function processoDaPagina(raiz) {
  const fila = [raiz];
  while (fila.length) {
    const p = fila.shift();
    try {
      if (/WebKitWebProces/.test(fs.readFileSync(`/proc/${p}/comm`, "utf8"))) return p;
    } catch {}
    fila.push(...filhos(p));
  }
  return null;
}

async function medirNoMiniBrowser([scripts, globais, uso], s) {
  const id = Math.random().toString(36).slice(2);
  const tags = scripts.map((f) => `<script src="/${f}"></script>`).join("\n");
  // O uso, quando há, entra na prova: um uso que não produziu nada (zero
  // elementos realçados, zero canvas) conta como global faltando.
  const prova = `<script>(async () => {
    const faltam = ${JSON.stringify(globais)}.filter(g => typeof window[g] === "undefined");
    ${uso ? `try { if (!(await (${uso}))) faltam.push("o uso"); } catch (e) { faltam.push("o uso: " + e); }` : ""}
    fetch("/carregou?p=${id}&g=" + encodeURIComponent(JSON.stringify(faltam)));
  })()</script>`;
  s.paginas.set(`/${id}.html`, `<!doctype html><html><head><meta charset="utf-8">${tags}</head><body>${prova}</body></html>`);
  // A mesma variável que o app põe com NVIDIA: sem ela o WebKit compõe por
  // outro caminho e a comparação com o app mente.
  const env = { ...process.env };
  if (fs.existsSync("/sys/module/nvidia/version")) env.WEBKIT_DISABLE_DMABUF_RENDERER ??= "1";
  const mb = spawn(MINIBROWSER, [`http://127.0.0.1:${s.porta}/${id}.html`], { env, stdio: "ignore" });
  try {
    for (let ate = Date.now() + 20000; !s.avisos.has(id) && Date.now() < ate; ) await PAUSA(100);
    const faltam = s.avisos.get(id);
    if (!faltam) throw new Error(`a página de ${scripts.join(", ") || "branco"} não avisou que carregou`);
    if (faltam.length) throw new Error(`${scripts.join(", ")}: ${faltam.join(", ")} falhou no MiniBrowser`);
    const pagina = processoDaPagina(mb.pid);
    if (!pagina) throw new Error("não achei o processo da página do MiniBrowser");
    // Assentar: o parse é síncrono, mas o JIT e o coletor seguem depois.
    await PAUSA(6000);
    return memoria(pagina);
  } finally {
    mb.kill();
    await PAUSA(500);
  }
}

async function carregar() {
  if (!fs.existsSync(MINIBROWSER)) {
    console.log(`(sem ${MINIBROWSER}: a medida de CARREGAR fica de fora)`);
    return;
  }
  const s = await servidor();
  try {
    console.log(`\nCARREGAR E USAR, isolado — processo da página do MiniBrowser, média de ${REPETICOES} (PSS, MiB)`);
    const base = {};
    for (const [nome, lib] of Object.entries(LIBS)) {
      let soma = 0;
      for (let i = 0; i < REPETICOES; i++) soma += (await medirNoMiniBrowser(lib, s)).pss;
      base[nome] = soma / REPETICOES;
      const delta = nome === "branco" ? "" : `   +${MiB(base[nome] - base.branco).trim()} sobre o branco`
        + (nome.endsWith(", usado") ? `, +${MiB(base[nome] - base[nome.replace(", usado", "")]).trim()} sobre só carregar` : "");
      console.log(`  ${nome.padEnd(24)}${MiB(base[nome])}${delta}`);
    }
  } finally {
    s.srv.close();
  }
}

// ── usar ─────────────────────────────────────────────────────────────

const nota = (t, corpo) => `---\ntitle: ${t}\n---\n# ${t}\n\n${corpo}\n`;
const PAGINAS = {
  controle: [nota("Bib controle", Array.from({ length: 20 }, (_, i) => `Parágrafo ${i}.`).join("\n\n")), "!!document.querySelector('.editor__title')"],
  // O editor hoje NÃO desenha um bloco ```mermaid salvo: ele vira
  // `<pre><code class="language-mermaid">` (visto no ciclo 484). O uso aqui
  // é a biblioteca chamada direto, que é o custo que um conserto teria.
  mermaid: [nota("Bib mermaid", "```mermaid\ngraph TD\n  A --> B\n```"), "!!document.querySelector('code.language-mermaid')",
    `mermaid.render("bib-484", "graph TD\\n  A[Início] --> B{Decide}\\n  B -->|sim| C[Faz]\\n  B -->|não| D[Espera]\\n  C --> E[Fim]\\n  D --> E").then(r => r.svg.length)`],
  highlight: [nota("Bib codigo", "```rust\nfn main() {\n    let x: Vec<u32> = (0..10).collect();\n    println!(\"{:?}\", x);\n}\n```\n\n```js\nconst a = [1, 2, 3].map((n) => n * 2);\n```"), "!!document.querySelector('.hljs')"],
  "3d-force-graph": ["---\ntitle: Bib grafo\ntype: graph\n---\n", "!!document.querySelector('.graph-view__canvas canvas')"],
};

async function usar() {
  process.env.ANOTADINHO_MCP_PORT = String(PORTA);
  const { Bridge, esperar, clicar, recarregarEstavel } = await import("../uitest/bridge.mjs");
  const bridge = await Bridge.conectar();
  const app = pidDaPorta(PORTA);
  const bruto = await bridge.js("localStorage.getItem('anotadinho.vault_path')");
  let vault;
  try { vault = JSON.parse(bruto); } catch { vault = bruto; }
  if (!vault || !fs.existsSync(vault)) throw new Error(`não achei o vault aberto (veio ${JSON.stringify(bruto)})`);
  const pasta = path.join(vault, PASTA);
  if (fs.existsSync(pasta)) throw new Error(`${PASTA} já existe: sobra de uma rodada interrompida? Apague e rode de novo.`);

  const desmontar = () => fs.rmSync(pasta, { recursive: true, force: true });
  process.on("SIGINT", () => { desmontar(); console.error("\ninterrompido: páginas removidas."); process.exit(130); });
  fs.mkdirSync(pasta, { recursive: true });
  for (const [nome, [texto]] of Object.entries(PAGINAS)) fs.writeFileSync(path.join(pasta, `${nome.replace(/\W/g, "")}.md`), texto);

  const pagina = () => amostrar(app)["webkit-pagina"].pss;
  const abrir = async (nome, pronto) => {
    const termo = `${PASTA}/${nome.replace(/\W/g, "")}`;
    await bridge.js(`(() => { document.querySelector('.app-root').dispatchEvent(new KeyboardEvent('keydown', { key: 'k', ctrlKey: true, bubbles: true, cancelable: true })); return true; })()`);
    await esperar(bridge, "document.querySelector('.command-palette__input')", "a paleta abrir");
    await bridge.js(`(() => { const i = document.querySelector('.command-palette__input'); Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value').set.call(i, ${JSON.stringify(termo)}); i.dispatchEvent(new InputEvent('input', { bubbles: true })); return true; })()`);
    await clicar(bridge, ".command-palette__item", `${termo} na paleta`);
    await esperar(bridge, pronto, `${nome} desenhar`, 20000);
  };
  const fecharAbas = async () => {
    for (let i = 0; i < 30; i++) {
      const fechou = await bridge.js(`(() => { const x = [...document.querySelectorAll('.tab-bar__tab-close')].pop(); if (x) x.click(); return !!x; })()`);
      if (!fechou) break;
    }
  };

  try {
    // O watcher vê as páginas novas; a paleta só as acha depois do índice.
    await esperar(bridge, `true`, "o índice");
    // Recarregar a página NÃO devolve memória ao sistema: o processo do
    // WebKit fica na marca máxima do dia (~440 MiB depois de um dia, contra
    // ~250 recém-aberto; visto no ciclo 484). Então este bloco só mede o
    // PRIMEIRO uso com o app recém-aberto; depois disso ele mostra o que
    // cada uso acrescenta a um processo que já usou tudo.
    console.log(`\nUSAR, no app — processo da página, depois de recarregar (PSS, MiB)`);
    console.log("  (o primeiro uso só aparece com o app recém-aberto: recarregar não devolve memória)");
    console.log(`  ${"".padEnd(16)}${"antes".padStart(7)}${"aberta".padStart(8)}${"fechada".padStart(8)}   fica`);
    for (const [nome, [, pronto, acao]] of Object.entries(PAGINAS)) {
      await fecharAbas();
      await recarregarEstavel(bridge);
      await PAUSA(6000);
      const antes = pagina();
      await abrir(nome, pronto);
      if (acao) {
        const r = await bridge.js(acao);
        if (!r) throw new Error(`o uso de ${nome} não produziu nada (${JSON.stringify(r)})`);
      }
      await PAUSA(4000);
      const aberta = pagina();
      await fecharAbas();
      await PAUSA(6000);
      const fechada = pagina();
      console.log(`  ${nome.padEnd(16)}${MiB(antes)} ${MiB(aberta)} ${MiB(fechada)}   ${(fechada - antes >= 0 ? "+" : "") + MiB(fechada - antes).trim()}`);
    }
  } finally {
    await fecharAbas().catch(() => {});
    desmontar();
  }
}

if (SO !== "usar") await carregar();
if (SO !== "carregar") await usar();
process.exit(0);
