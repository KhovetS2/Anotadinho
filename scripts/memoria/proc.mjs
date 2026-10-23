// Os processos do app e a memória deles, lidos do /proc (ciclos 480 e
// 482). Comum ao `janela.mjs` e à `sonda-pagina.mjs`.

import fs from "node:fs";
import { execFileSync } from "node:child_process";

/// O pid que escuta a porta da ponte: é o processo do app.
export function pidDaPorta(porta) {
  const saida = execFileSync("ss", ["-ltnpH", `sport = :${porta}`], { encoding: "utf8" });
  const m = saida.match(/pid=(\d+)/);
  if (!m) throw new Error(`ninguém escuta a porta ${porta}: o app está aberto?`);
  return Number(m[1]);
}

function lerStat(pid) {
  try {
    const s = fs.readFileSync(`/proc/${pid}/stat`, "utf8");
    // O nome vem entre parênteses e pode ter espaço: corta pelo último `)`.
    const fim = s.lastIndexOf(")");
    const resto = s.slice(fim + 2).split(" ");
    return { nome: s.slice(s.indexOf("(") + 1, fim), ppid: Number(resto[1]) };
  } catch {
    return null;
  }
}

/// O app e todos os descendentes (o WebKit é filho do app).
function arvore(raiz) {
  const todos = fs.readdirSync("/proc").filter((d) => /^\d+$/.test(d)).map(Number);
  const filhos = new Map();
  for (const pid of todos) {
    const st = lerStat(pid);
    if (!st) continue;
    if (!filhos.has(st.ppid)) filhos.set(st.ppid, []);
    filhos.get(st.ppid).push(pid);
  }
  const fora = [];
  const fila = [raiz];
  while (fila.length) {
    const p = fila.shift();
    fora.push(p);
    fila.push(...(filhos.get(p) || []));
  }
  return fora;
}

/// Pss, Rss e USS (o que só este processo usa) em KiB.
export function memoria(pid) {
  try {
    const t = fs.readFileSync(`/proc/${pid}/smaps_rollup`, "utf8");
    const kb = (campo) => Number((t.match(new RegExp(`^${campo}:\\s+(\\d+) kB`, "m")) || [0, 0])[1]);
    return { pss: kb("Pss"), rss: kb("Rss"), uss: kb("Private_Clean") + kb("Private_Dirty") };
  } catch {
    return null; // o processo morreu entre a listagem e a leitura
  }
}

/// Agrupa pelo papel, não pelo pid: o WebKit recria processos, e o
/// relatório quer "a página web", não "o pid 4182".
function papel(nome) {
  if (/WebKitWebProces|WebProcess/i.test(nome)) return "webkit-pagina";
  if (/WebKitNetworkPr|NetworkProcess/i.test(nome)) return "webkit-rede";
  if (/WebKitGPU|GPUProcess/i.test(nome)) return "webkit-gpu";
  return null;
}

export function amostrar(app) {
  const porPapel = {};
  for (const pid of arvore(app)) {
    const st = lerStat(pid);
    const m = memoria(pid);
    if (!st || !m) continue;
    const p = pid === app ? "app" : papel(st.nome);
    if (!p) continue; // `git`, o agente: não são o Anotadinho
    porPapel[p] ??= { pss: 0, rss: 0, uss: 0 };
    for (const k of ["pss", "rss", "uss"]) porPapel[p][k] += m[k];
  }
  const total = { pss: 0, rss: 0, uss: 0 };
  for (const v of Object.values(porPapel)) for (const k of Object.keys(total)) total[k] += v[k];
  porPapel.total = total;
  return porPapel;
}

