#!/usr/bin/env node
// De que é feita a memória do processo do app (ciclo 483).
//
//   node scripts/memoria/app.mjs --porta 9323
//   node scripts/memoria/app.mjs --porta 9323 --referencia
//
// O `janela.mjs` mede o processo do app como um número só (~145 MiB de PSS
// parado). Esse número junta coisas que não se cortam do mesmo jeito: os
// buffers da janela, que o app DIVIDE com o WebKit e crescem com a área;
// as bibliotecas do GTK e do WebKit, divididas com todo processo que as
// carrega; o driver de vídeo; e o heap, onde o GTK, o WebKit e o nosso
// código alocam pelo mesmo `malloc`.
//
// A parte do Rust vem do próprio app (`memoria_do_rust`, o alocador
// contado): pelo `smaps` ela é indistinguível do resto do heap.
//
// `--referencia` abre por alguns segundos o MiniBrowser do próprio
// WebKitGTK numa página em branco e decompõe do mesmo jeito: é o piso de
// uma janela com WebView nesta máquina, sem nada do Anotadinho.

import fs from "node:fs";
import { spawn, execFileSync } from "node:child_process";
import { pidDaPorta } from "./proc.mjs";

function arg(nome, padrao) {
  const i = process.argv.indexOf(`--${nome}`);
  return i > 0 && process.argv[i + 1] ? process.argv[i + 1] : padrao;
}
const PORTA = Number(arg("porta", 9223));
const REFERENCIA = process.argv.includes("--referencia");
const MINIBROWSER = "/usr/libexec/webkit2gtk-4.1/MiniBrowser";
const PAUSA = (ms) => new Promise((r) => setTimeout(r, ms));
const MiB = (kib) => (kib / 1024).toFixed(1).padStart(7);

/// O papel de cada mapeamento. A ordem importa: o binário do app também
/// é um arquivo, e o buffer da janela também é memória compartilhada.
function categoria(nome, exe) {
  if (nome === exe) return "binário";
  if (/memfd:(WebKitSharedMemory|gdk-wayland|wayland-)/.test(nome)) return "buffers da janela";
  if (/nvidia|libgallium|libLLVM|\/dev\/dri|libGLX|libEGL|libGLdispatch|mesa|radeon|intel|iris/i.test(nome)) return "driver de vídeo";
  if (/^\[(heap|anon|stack)/.test(nome) || nome === "") return "heap e anônima";
  if (/\.so/.test(nome)) return "bibliotecas";
  return "outros";
}

export function decompor(pid) {
  const exe = fs.readlinkSync(`/proc/${pid}/exe`);
  const soma = {};
  let atual = null;
  for (const linha of fs.readFileSync(`/proc/${pid}/smaps`, "utf8").split("\n")) {
    const cab = linha.match(/^[0-9a-f]+-[0-9a-f]+ \S+ \S+ \S+ \S+\s*(.*)$/);
    if (cab) {
      const nome = cab[1].trim() || "[anon]";
      atual = soma[categoria(nome, exe)] ??= { pss: 0, uss: 0, rss: 0 };
      continue;
    }
    const m = linha.match(/^(Pss|Rss|Private_Clean|Private_Dirty):\s+(\d+) kB/);
    if (!m || !atual) continue;
    if (m[1] === "Pss") atual.pss += Number(m[2]);
    else if (m[1] === "Rss") atual.rss += Number(m[2]);
    else atual.uss += Number(m[2]);
  }
  return soma;
}

const ORDEM = ["heap e anônima", "buffers da janela", "binário", "driver de vídeo", "bibliotecas", "outros"];

function imprimir(titulo, soma, rust) {
  console.log(`\n${titulo}`);
  console.log(`  ${"".padEnd(26)}${"PSS".padStart(7)}${"USS".padStart(8)}   MiB`);
  const total = { pss: 0, uss: 0 };
  for (const c of ORDEM) {
    const v = soma[c];
    if (!v) continue;
    total.pss += v.pss;
    total.uss += v.uss;
    console.log(`  ${c.padEnd(26)}${MiB(v.pss)} ${MiB(v.uss)}`);
    if (c === "heap e anônima" && rust) {
      const vivos = rust.vivos / 1024;
      console.log(`    ${"└ dele, do Rust (vivos)".padEnd(24)}${"".padStart(7)} ${MiB(vivos)}   pico ${MiB(rust.pico / 1024).trim()}`);
      console.log(`    ${"└ GTK, WebKit, driver".padEnd(24)}${"".padStart(7)} ${MiB(v.uss - vivos)}`);
    }
  }
  console.log(`  ${"total".padEnd(26)}${MiB(total.pss)} ${MiB(total.uss)}`);
}

async function referencia() {
  if (!fs.existsSync(MINIBROWSER)) {
    console.log(`\n(sem ${MINIBROWSER}: a referência fica de fora)`);
    return;
  }
  // A mesma variável que o app põe com NVIDIA (`contornar_travamento_nvidia`):
  // sem ela o WebKit toma outro caminho de composição e a comparação mente.
  const env = { ...process.env };
  if (fs.existsSync("/sys/module/nvidia/version")) env.WEBKIT_DISABLE_DMABUF_RENDERER ??= "1";
  const filho = spawn(MINIBROWSER, ["about:blank"], { env, stdio: "ignore" });
  try {
    // Pronto = o processo da página nasceu; depois, o mesmo assentamento
    // que o app teve (ele está parado há muito mais que isto).
    const limite = Date.now() + 10000;
    while (Date.now() < limite) {
      try {
        execFileSync("pgrep", ["-P", String(filho.pid)]);
        break;
      } catch {
        // pgrep sai != 0 enquanto não há filho
      }
      await PAUSA(200);
    }
    await PAUSA(5000);
    imprimir("MiniBrowser em branco (piso do WebKitGTK nesta máquina)", decompor(filho.pid), null);
  } finally {
    filho.kill();
  }
}

process.env.ANOTADINHO_MCP_PORT = String(PORTA);
const { Bridge } = await import("../uitest/bridge.mjs");
const bridge = await Bridge.conectar();
const app = pidDaPorta(PORTA);
// Build anterior ao 483 não tem o comando: decompõe mesmo assim, sem a
// linha do Rust, em vez de não dizer nada.
const rust = await bridge.js(
  "window.__TAURI_INTERNALS__.invoke('memoria_do_rust').catch(() => null)",
);
if (!rust) console.error("! o app não tem `memoria_do_rust` (build anterior ao ciclo 483): sem a parte do Rust.");
imprimir(`Anotadinho (pid ${app})`, decompor(app), rust);
if (REFERENCIA) await referencia();
console.log(
  "\nPSS divide o que é compartilhado entre quem usa; USS é só deste processo." +
    "\nOs buffers da janela são divididos com o WebKitWebProcess e crescem com a área da janela.",
);
process.exit(0);
