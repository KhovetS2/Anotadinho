#!/usr/bin/env node
// Quanto abrir uma página deixa de memória presa (ciclo 482).
//
//   node scripts/memoria/sonda-pagina.mjs --porta 9323
//   node scripts/memoria/sonda-pagina.mjs --porta 9323 --visitas 20 --so inicio,controle
//   node scripts/memoria/sonda-pagina.mjs --porta 9323 --pagina pages/minha.md
//
// O `janela.mjs` mostrou o pico vindo da página inicial e crescendo a cada
// rodada, mas uma rodada faz dez coisas: daquilo não dá pra dizer QUAL
// parte prende memória. Aqui cada experimento alterna entre duas páginas
// e nada mais, N vezes, e mede o PSS da página web (o processo do WebKit)
// depois de cada ida. A inclinação — MiB por visita — é o número.
//
// O CONTROLE alterna duas notas leves. Ele existe porque o WebKit retém
// memória mesmo sem vazamento nosso (o coletor recolhe quando decide), e
// um experimento só acusa alguma coisa se subir MAIS que o controle.
//
// As páginas de teste moram em `pages/uitest-sonda/` do vault aberto e
// são removidas no fim, inclusive com Ctrl+C. A página inicial da pessoa
// é só LIDA: o experimento "sem consultas" é uma cópia dela.

import fs from "node:fs";
import path from "node:path";
import { pidDaPorta, amostrar } from "./proc.mjs";

function arg(nome, padrao) {
  const i = process.argv.indexOf(`--${nome}`);
  return i > 0 && process.argv[i + 1] ? process.argv[i + 1] : padrao;
}
const PORTA = Number(arg("porta", 9223));
const VISITAS = Number(arg("visitas", 12));
const SO = arg("so", "").split(",").filter(Boolean);
// `--vault-grande`: acrescenta 300 tarefas e 20 notas, como o dia do
// `janela.mjs` fazia — pra ver se o tamanho do vault muda a resposta.
const GRANDE = process.argv.includes("--vault-grande");
const PASTA = "pages/uitest-sonda";
process.env.ANOTADINHO_MCP_PORT = String(PORTA);
const { Bridge, esperar, clicar } = await import("../uitest/bridge.mjs");

const PAUSA = (ms) => new Promise((r) => setTimeout(r, ms));
const MB = (kib) => kib / 1024;

const bridge = await Bridge.conectar();
const app = pidDaPorta(PORTA);
const bruto = await bridge.js("localStorage.getItem('anotadinho.vault_path')");
let vault;
try { vault = JSON.parse(bruto); } catch { vault = bruto; }
if (!vault || !fs.existsSync(vault)) {
  console.error(`✗ não achei o vault aberto (veio ${JSON.stringify(bruto)}).`);
  process.exit(1);
}
const inicial = arg("pagina", fs.existsSync(path.join(vault, "pages/inicio.md")) ? "pages/inicio.md" : "pages/incio.md");
if (fs.existsSync(path.join(vault, PASTA)) || fs.existsSync(path.join(vault, "pages/projetos/uitest-sonda"))) {
  console.error(`✗ ${PASTA} já existe em ${vault}: sobra de uma sonda interrompida? Apague e rode de novo.`);
  process.exit(1);
}

// ── as páginas dos experimentos ──────────────────────────────────────

const textoDaInicial = fs.readFileSync(path.join(vault, inicial), "utf8");
const semEmbeds = (texto, tipo) =>
  texto.replace(new RegExp(`\\{\\{ type: "${tipo}" \\}\\}[\\s\\S]*?\\{\\{ /${tipo} \\}\\}`, "g"), "");
const nota = (t) => `---\ntitle: ${t}\n---\n# ${t}\n\n` + Array.from({ length: 30 }, (_, i) => `Parágrafo ${i} de ${t}.`).join("\n\n") + "\n";
const comTitulo = (texto, t) => texto.replace(/^title: .*$/m, `title: ${t}`);
const consultas = [...textoDaInicial.matchAll(/\{\{ type: "query" \}\}[\s\S]*?\{\{ \/query \}\}/g)].map((m) => m[0]);

function montar() {
  const p = path.join(vault, PASTA);
  fs.mkdirSync(p, { recursive: true });
  fs.writeFileSync(path.join(p, "leve-a.md"), nota("Leve A"));
  fs.writeFileSync(path.join(p, "leve-b.md"), nota("Leve B"));
  fs.writeFileSync(path.join(p, "sem-consultas.md"), comTitulo(semEmbeds(textoDaInicial, "query"), "Sem consultas"));
  fs.writeFileSync(path.join(p, "so-acoes.md"), `---\ntitle: So acoes\n---\n` + (textoDaInicial.match(/\{\{ type: "actions" \}\}[\s\S]*?\{\{ \/actions \}\}/) || [""])[0] + "\n");
  consultas.forEach((q, i) => fs.writeFileSync(path.join(p, `consulta-${i + 1}.md`), `---\ntitle: Consulta ${i + 1}\n---\n# Consulta ${i + 1}\n\n${q}\n`));
  for (let i = 1; i <= 10; i++) fs.writeFileSync(path.join(p, `aba-${i}.md`), nota(`Aba ${i}`));
  if (GRANDE) {
    const t = path.join(p, "grande");
    fs.mkdirSync(t, { recursive: true });
    for (let i = 1; i <= 320; i++) fs.writeFileSync(path.join(t, `g-${i}.md`), `---\ntitle: G ${i}\ntype: tarefa\nstatus: ${i % 3 ? "ideia" : "em-revisao"}\ndate: 2026-10-${String((i % 28) + 1).padStart(2, "0")}\n---\n# G ${i}\n\ntexto\n`);
  }
  // As telas pesadas do "dia" do `janela.mjs`: o quadro de 300 tarefas
  // (que relê de 3 em 3 s), a conversa longa e o grafo.
  const proj = path.join(vault, "pages/projetos", PROJETO);
  for (const sub of ["tarefas", "sprints", "conversas"]) fs.mkdirSync(path.join(proj, sub), { recursive: true });
  fs.writeFileSync(path.join(proj, "projeto.md"), `---\ntitle: Projeto sonda\ntype: projeto\nprojeto: ${PROJETO}\n---\n# Projeto sonda\n`);
  fs.writeFileSync(path.join(proj, "arquitetura.md"), `---\ntitle: Arquitetura\ntype: docs\nprojeto: ${PROJETO}\n---\n`);
  for (let s = 1; s <= 30; s++) fs.writeFileSync(path.join(proj, "sprints", `${String(s).padStart(2, "0")}-s${s}.md`), `---\ntitle: S${s}\ntype: sprint\nprojeto: ${PROJETO}\nestado: ${s === 30 ? "ativa" : "encerrada"}\n---\n`);
  for (let t = 1; t <= 300; t++) {
    const sp = Math.ceil(t / 10);
    fs.writeFileSync(path.join(proj, "tarefas", `${String(t).padStart(3, "0")}-t.md`), `---\ntitle: T${t}\ntags:\n- tarefa\ntype: tarefa\nprojeto: ${PROJETO}\nsprint: ${sp}\nstatus: ${sp === 30 ? "pronta" : "feita"}\n---\n# T${t}\n\n## Critérios de aceite\n\n- [x] um\n- [ ] dois\n`);
  }
  const msgs = Array.from({ length: 30 }, (_, m) => `## ${m % 2 ? "agente" : "você"} · 2026-10-23 10:${String(m).padStart(2, "0")}\n\n${"Uma resposta com várias linhas. ".repeat(m % 2 ? 12 : 2)}`);
  fs.writeFileSync(path.join(vault, PASTA, "conversa.md"), `---\ntitle: Conversa sonda\ntype: conversa\n---\n${msgs.join("\n\n")}\n`);
  fs.writeFileSync(path.join(vault, PASTA, "grafo.md"), "---\ntitle: Grafo sonda\ntype: graph\n---\n");
}
const PROJETO = "uitest-sonda";
const desmontar = () => {
  fs.rmSync(path.join(vault, PASTA), { recursive: true, force: true });
  fs.rmSync(path.join(vault, "pages/projetos", PROJETO), { recursive: true, force: true });
};

// ── a alternância ────────────────────────────────────────────────────

async function abrir(caminho) {
  const termo = caminho.replace(/\.md$/, "");
  await bridge.js(`(() => { document.querySelector('.app-root').dispatchEvent(new KeyboardEvent('keydown', { key: 'k', ctrlKey: true, bubbles: true, cancelable: true })); return true; })()`);
  await esperar(bridge, "document.querySelector('.command-palette__input')", "a paleta abrir");
  await bridge.js(`(() => { const i = document.querySelector('.command-palette__input'); Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value').set.call(i, ${JSON.stringify(termo)}); i.dispatchEvent(new InputEvent('input', { bubbles: true })); return true; })()`);
  // O item da página, pelo caminho — não o primeiro da lista, que pode
  // ser um resultado de busca em outra página.
  await esperar(bridge, `[...document.querySelectorAll('.command-palette__item')].some(e => e.textContent.includes(${JSON.stringify(termo.split("/").pop())}))`, `${termo} na paleta`);
  await bridge.js(`(() => { const it = [...document.querySelectorAll('.command-palette__item')].find(e => e.textContent.includes(${JSON.stringify(termo.split("/").pop())})); it.click(); return true; })()`);
  // Desenhou: o editor mostra a página, e as consultas (se houver) têm
  // conteúdo. Esperar o efeito, não o relógio.
  // A aba ATIVA, e não o `.editor__title`: projeto, conversa e grafo têm
  // cabeçalho próprio. A página inicial fixa mostra só o ícone na aba.
  const nome = termo.split("/").pop();
  await esperar(bridge, `(() => { const a = document.querySelector('.tab-bar__tab--active'); if (!a) return false; const t = a.querySelector('.tab-bar__tab-title'); return a.classList.contains('tab-bar__tab--fixed') ? ${JSON.stringify(caminho === inicial)} : !!t && (t.textContent.includes(${JSON.stringify(nome)}) || (t.getAttribute('title') || '').includes(${JSON.stringify(nome)})); })()`, `${termo} abrir`, 15000);
  await esperar(bridge, `!!document.querySelector('.editor__wysiwyg, .projeto, .conversa, .graph-view')`, `${termo} desenhar`, 15000);
  await esperar(bridge, `[...document.querySelectorAll('.query-embed')].every(q => !q.querySelector('.spinner'))`, `${termo} terminar de desenhar`, 15000).catch(() => {});
}

async function experimento(nome, a, b) {
  const pontos = [];
  // Aquecimento: a primeira visita carrega o que fica em cache de
  // propósito (o índice, as fontes). Não entra na inclinação.
  await abrir(a);
  await abrir(b);
  await PAUSA(2000);
  for (let v = 1; v <= VISITAS; v++) {
    await abrir(a);
    await PAUSA(800);
    await abrir(b);
    await PAUSA(1200);
    const m = amostrar(app);
    pontos.push(MB(m["webkit-pagina"]?.pss ?? 0));
    process.stdout.write(`  ${nome}: ${v}/${VISITAS}  ${pontos.at(-1).toFixed(0)} MiB   \r`);
  }
  // Mínimos quadrados: a inclinação resiste ao serrilhado do coletor,
  // que uma diferença fim−início não resiste.
  const n = pontos.length, xm = (n + 1) / 2, ym = pontos.reduce((s, y) => s + y, 0) / n;
  const incl = pontos.reduce((s, y, i) => s + (i + 1 - xm) * (y - ym), 0) / pontos.reduce((s, _, i) => s + (i + 1 - xm) ** 2, 0);
  return { nome, a, b, pontos: pontos.map((p) => Math.round(p)), por_visita_mib: Number(incl.toFixed(1)) };
}

/// Abre 10 páginas, cada uma na sua aba, e fecha TODAS. É o que o "dia"
/// fazia antes do pico: o fechamento leva a janela pra página inicial.
async function abasAbreFecha() {
  const pontos = [];
  for (let v = 0; v <= VISITAS; v++) {
    for (let i = 1; i <= 10; i++) await abrir(`${PASTA}/aba-${i}.md`);
    for (let k = 0; k < 40; k++) {
      const fechou = await bridge.js(`(() => { const x = [...document.querySelectorAll('.tab-bar__tab-close')].pop(); if (x) x.click(); return !!x; })()`);
      if (!fechou) break;
      await PAUSA(80);
    }
    await PAUSA(2000);
    // A volta 0 é o aquecimento.
    if (v > 0) pontos.push(MB(amostrar(app)["webkit-pagina"]?.pss ?? 0));
    process.stdout.write(`  abas: ${v}/${VISITAS}   \r`);
  }
  const n = pontos.length, xm = (n + 1) / 2, ym = pontos.reduce((s, y) => s + y, 0) / n;
  const incl = pontos.reduce((s, y, i) => s + (i + 1 - xm) * (y - ym), 0) / pontos.reduce((s, _, i) => s + (i + 1 - xm) ** 2, 0);
  return { nome: "abas", pontos: pontos.map((p) => Math.round(p)), por_visita_mib: Number(incl.toFixed(1)) };
}

const EXPERIMENTOS = [
  ["controle", `${PASTA}/leve-a.md`, `${PASTA}/leve-b.md`],
  ["inicio", inicial, `${PASTA}/leve-a.md`],
  ["sem-consultas", `${PASTA}/sem-consultas.md`, `${PASTA}/leve-a.md`],
  ["so-acoes", `${PASTA}/so-acoes.md`, `${PASTA}/leve-a.md`],
  ...consultas.map((_, i) => [`consulta-${i + 1}`, `${PASTA}/consulta-${i + 1}.md`, `${PASTA}/leve-a.md`]),
  ["abas", null, null],
  ["projeto", `pages/projetos/${PROJETO}/projeto.md`, `${PASTA}/leve-a.md`],
  ["conversa", `${PASTA}/conversa.md`, `${PASTA}/leve-a.md`],
  ["grafo", `${PASTA}/grafo.md`, `${PASTA}/leve-a.md`],
].filter(([n]) => !SO.length || SO.includes(n));

process.on("SIGINT", () => { desmontar(); console.error("\ninterrompido: páginas da sonda removidas."); process.exit(130); });
const exe = fs.readlinkSync(`/proc/${app}/exe`);
console.log(`app ${app} (${exe}) · vault ${vault}${GRANDE ? " + 340 páginas" : ""} · página inicial ${inicial} · ${VISITAS} visitas por experimento`);
const resultados = [];
try {
  montar();
  await esperar(bridge, `document.body.innerText.includes('uitest-sonda') || true`, "o índice");
  await PAUSA(3000);
  for (const [nome, a, b] of EXPERIMENTOS) {
    resultados.push(nome === "abas" ? await abasAbreFecha() : await experimento(nome, a, b));
    console.log(`  ${nome.padEnd(14)} ${String(resultados.at(-1).por_visita_mib).padStart(7)} MiB/visita   ${resultados.at(-1).pontos.join(" ")}`);
  }
} catch (e) {
  console.error(`\n✗ a sonda parou: ${e.message}`);
  process.exitCode = 1;
} finally {
  desmontar();
  bridge.fechar?.();
}
const controle = resultados.find((r) => r.nome === "controle");
if (controle) {
  console.log(`\nAcima do controle (${controle.por_visita_mib} MiB/visita):`);
  for (const r of resultados.filter((r) => r !== controle)) {
    console.log(`  ${r.nome.padEnd(14)} ${(r.por_visita_mib - controle.por_visita_mib).toFixed(1).padStart(7)} MiB/visita`);
  }
}
consultas.forEach((q, i) => console.log(`\nconsulta-${i + 1}:\n${q.split("\n").slice(1, -1).map((l) => "    " + l).join("\n")}`));
process.exit(process.exitCode ?? 0);
