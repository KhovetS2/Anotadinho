#!/usr/bin/env node
// Harness de teste de UI do Anotadinho.
//
// Por que existe: quase todo bug do editor que escapou dos testes de
// unidade é comportamento de DOM — texto duplicado dentro de
// contenteditable (076), arraste que não commitava (155), Escape
// fechando a página junto com o modal (161), toolbar cobrindo controle
// (166). Nenhum deles aparece num `cargo test`. Aqui cada um vira um
// cenário roteirizado contra o app DE VERDADE.
//
// Uso:
//   ./scripts/dev.sh            # num terminal, deixa o app de pé
//   node scripts/uitest/run.mjs # noutro
//   node scripts/uitest/run.mjs callout   # só cenários que casam
//   node scripts/uitest/run.mjs --pendentes  # só a bateria das specs
//
// Sai com código != 0 se algum cenário falhar (serve pra CI local).

import { readFileSync, writeFileSync, unlinkSync, existsSync } from "node:fs";
import { join } from "node:path";
import { Bridge, esperar, abrirPagina } from "./bridge.mjs";
import { cenarios } from "./cenarios.mjs";
import { digitacao } from "./digitacao.mjs";
import { blocos } from "./blocos.mjs";
import { teclados as teclado } from "./teclado.mjs";
import { interacoes } from "./interacoes.mjs";
import { telas } from "./telas.mjs";
import { fluxo } from "./fluxo.mjs";
import { pendentes } from "./pendentes.mjs";
import { conferirSnapshots } from "./snapshot.mjs";

const VAULT = process.env.ANOTADINHO_VAULT || "VaultAnotadinho";
/// Página de rascunho dos testes — criada e apagada por eles, pra nunca
/// mexer no conteúdo real do vault.
const PAGINA = "pages/__uitest.md";

const ctx = {
  vault: VAULT,
  pagina: PAGINA,
  nomePagina: "__uitest",
  arquivo: join(VAULT, PAGINA),
  escrever(conteudo) {
    writeFileSync(this.arquivo, conteudo);
  },
  ler() {
    return existsSync(this.arquivo) ? readFileSync(this.arquivo, "utf8") : null;
  },
  apagar() {
    if (existsSync(this.arquivo)) unlinkSync(this.arquivo);
  },
  esperar,
  abrirPagina,
};

function assert(cond, msg) {
  if (!cond) throw new Error(msg);
}
function assertEq(atual, esperado, msg) {
  if (atual !== esperado) {
    throw new Error(`${msg}\n  esperado: ${JSON.stringify(esperado)}\n  veio:     ${JSON.stringify(atual)}`);
  }
}
ctx.assert = assert;
ctx.assertEq = assertEq;

// A bateria de digitação (ciclo 193) entra junto: é rede de segurança
// permanente, não um cenário pontual.
const todos = [
  ...cenarios,
  ...digitacao,
  ...blocos,
  ...teclado,
  ...interacoes,
  ...telas,
  ...fluxo,
];

// `--pendentes` roda a bateria escrita a partir das specs ainda não
// implementadas (`pendentes.mjs`). Ela fica FORA de `todos` de
// propósito: é vermelha por definição, e a suíte principal precisa
// continuar sendo o sinal confiável de "está tudo certo?".
const soPendentes = process.argv.includes("--pendentes");
const filtro = process.argv.slice(2).find((a) => !a.startsWith("--"));
const base = soPendentes ? pendentes : todos;
let selecionados = filtro
  ? base.filter((c) => c.nome.toLowerCase().includes(filtro.toLowerCase()))
  : base;

let bridge;
try {
  bridge = await Bridge.conectar();
} catch (e) {
  console.error(`✗ ${e.message}`);
  process.exit(2);
}

// Normaliza as configurações persistidas antes de rodar (ciclo 197).
//
// Elas moram no `localStorage` e sobrevivem entre sessões, então um
// clique perdido ou uma tecla de atalho apertada durante uma depuração
// desliga o nav-mode e a suíte inteira passa a falhar por um motivo que
// não está no código. Aconteceu de verdade: `nav_mode_enabled` ficou
// `false` e três cenários quebraram sem nenhuma mudança relacionada.
try {
  const mudou = await bridge.js(`(() => {
    const antes = {
      nav: localStorage.getItem('anotadinho.nav_mode_enabled'),
      vim: localStorage.getItem('anotadinho.vim_mode_enabled'),
    };
    localStorage.setItem('anotadinho.nav_mode_enabled', 'true');
    localStorage.setItem('anotadinho.vim_mode_enabled', 'false');
    return antes.nav !== 'true' || antes.vim === 'true';
  })()`);
  if (mudou) {
    console.log("  ↻ configurações normalizadas (nav-mode ligado, vim desligado)");
    await bridge.js("location.reload(); true");
    await new Promise((r) => setTimeout(r, 2500));
  }
} catch (e) {
  console.log(`  ! não consegui normalizar as configurações: ${e.message}`);
}

// Guarda a configuração do AGENTE antes de rodar (ciclo 208).
//
// Os cenários apontam o adaptador pro agente de mentira. Sem restaurar,
// a suíte deixa o app do usuário configurado pra um script de teste — e
// a próxima conversa dele falha com "erro proposital". Aconteceu.
let adaptadorOriginal = null;
try {
  adaptadorOriginal = await bridge.js(
    `localStorage.getItem('anotadinho.adaptador_agente')`,
  );
} catch {
  /* app sem storage acessível: segue sem restaurar */
}

let passaram = 0;
const falharam = [];
const inicio = Date.now();

for (const cenario of selecionados) {
  const t0 = Date.now();
  try {
    ctx.apagar();
    await cenario.fn(bridge, ctx);
    passaram++;
    console.log(`  ✓ ${cenario.nome} (${Date.now() - t0}ms)`);
  } catch (e) {
    falharam.push({ nome: cenario.nome, erro: e.message });
    console.log(`  ✗ ${cenario.nome} (${Date.now() - t0}ms)`);
    console.log(`      ${e.message.replace(/\n/g, "\n      ")}`);
  } finally {
    ctx.apagar();
  }
}

// Snapshot visual dos embeds (ciclo 187) — entra como um cenário a mais
// pra `run.mjs` continuar sendo o comando único de "está tudo certo?".
// `--sem-snapshot` pula (útil quando você está no meio de um redesenho e
// ainda não quer regravar a baseline).
if (!filtro && !soPendentes && !process.argv.includes("--sem-snapshot")) {
  const t0 = Date.now();
  try {
    const resultados = await conferirSnapshots(bridge);
    const ruins = resultados.filter((r) => r.problemas.length);
    if (ruins.length) {
      throw new Error(
        ruins
          .map((r) => `${r.tipo}:\n    ${r.problemas.join("\n    ")}`)
          .join("\n  "),
      );
    }
    passaram++;
    console.log(`  ✓ snapshot visual dos ${resultados.length} embeds (187) (${Date.now() - t0}ms)`);
  } catch (e) {
    falharam.push({ nome: "snapshot visual dos embeds (187)", erro: e.message });
    console.log(`  ✗ snapshot visual dos embeds (187) (${Date.now() - t0}ms)`);
    console.log(`      ${e.message.replace(/\n/g, "\n      ")}`);
  }
  selecionados.push({ nome: "snapshot visual dos embeds (187)" });
}

// Devolve o adaptador do usuário.
if (adaptadorOriginal !== null) {
  try {
    await bridge.js(`(() => {
      const v = ${JSON.stringify(adaptadorOriginal)};
      if (v === null) localStorage.removeItem('anotadinho.adaptador_agente');
      else localStorage.setItem('anotadinho.adaptador_agente', v);
      return true;
    })()`);
  } catch {
    console.log("  ! não consegui restaurar a configuração do agente");
  }
}

bridge.fechar();

console.log(
  `\n${passaram}/${selecionados.length} cenários passaram em ${((Date.now() - inicio) / 1000).toFixed(1)}s`,
);
if (falharam.length) {
  console.log(`\nfalhas:\n${falharam.map((f) => `  - ${f.nome}: ${f.erro.split("\n")[0]}`).join("\n")}`);
  process.exit(1);
}
