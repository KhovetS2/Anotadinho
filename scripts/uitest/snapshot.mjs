#!/usr/bin/env node
// Snapshot visual dos embeds (ciclo 187).
//
// Por que NÃO é captura de pixel: o `capture_native_screenshot` do
// bridge responde
//   "Native Linux screenshot not yet implemented"
// nesta plataforma, então o harness não tem como tirar foto da janela.
// O que ele TEM é o DOM ao vivo, e daí sai uma impressão digital que
// pega a mesma classe de regressão que um diff de pixel pegaria — CSS
// de um embed vazando pro outro, cor que perdeu contraste, grade que
// quebrou, caixa que colapsou — com três vantagens práticas: o diff diz
// QUAL propriedade mudou em vez de "3.2% dos pixels diferem", não
// reprova por antialiasing, e a baseline é um JSON de poucos KB em vez
// de PNG (que já pesou 10MB no repositório uma vez, no ciclo 148).
//
// A impressão digital é, por embed: para cada combinação de classes que
// aparece na subárvore, os estilos computados que importam pro visual +
// quantos elementos têm aquela combinação. Comparar por CLASSE (e não
// por posição no DOM) é o que deixa a coisa estável: reordenar dois
// cards não é regressão visual, mudar a cor deles é.
//
// Uso:
//   node scripts/uitest/snapshot.mjs             # confere
//   node scripts/uitest/snapshot.mjs --atualizar # regrava a baseline
//
// CUIDADO: a baseline guarda pixels ABSOLUTOS, e grid se distribui pela
// largura disponível. Rodar numa janela mais estreita do que a de
// gravação faz `grid-template-columns` diferir em tudo, sem nenhuma
// mudança de estilo — as PROPORÇÕES continuam iguais, e é isso que
// denuncia o falso positivo. Se a diferença for um fator constante em
// todas as colunas, é tamanho de janela, não regressão.
//   node scripts/uitest/snapshot.mjs kanban      # só um tipo

import { readFileSync, writeFileSync, existsSync, mkdirSync, unlinkSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { Bridge, esperar, abrirPagina } from "./bridge.mjs";

const AQUI = dirname(fileURLToPath(import.meta.url));
const BASELINE = join(AQUI, "baseline");
const VAULT = process.env.ANOTADINHO_VAULT || "VaultAnotadinho";
const ARQUIVO = join(VAULT, "pages/__uisnap.md");

/// Tamanho fixo de janela. Sem isso a largura entra na impressão digital
/// e a baseline vira loteria.
const LARGURA = 1280;
const ALTURA = 900;

/// Propriedades que decidem como a coisa PARECE. Deliberadamente sem
/// `width`/`height` absolutos: eles dependem de conteúdo e de data (uma
/// barra de cronograma muda de tamanho conforme o mês), e o que
/// interessa aqui é o estilo, não o calendário.
const PROPS = [
  "display",
  "flex-direction",
  "grid-template-columns",
  "gap",
  "color",
  "background-color",
  "border-top-width",
  "border-top-color",
  "border-radius",
  "font-size",
  "font-weight",
  "padding-top",
  "padding-left",
  "margin-top",
  "text-align",
  "opacity",
  "overflow-x",
];

/// Tipos cuja CONTAGEM de elementos depende da data de hoje (o mês pode
/// ter 5 ou 6 semanas). Neles a contagem é informativa; o estilo, não.
const CONTAGEM_VARIAVEL = new Set(["calendar", "timeline"]);

/// Um embed de cada tipo, com conteúdo fixo e datas relativas ao mês
/// corrente — data cravada faria a barra do cronograma sair da janela
/// visível conforme o tempo passa.
function fixture() {
  const hoje = new Date();
  const ano = hoje.getFullYear();
  const mes = String(hoje.getMonth() + 1).padStart(2, "0");
  const d = (dia) => `${ano}-${mes}-${String(dia).padStart(2, "0")}`;
  const relativa = (dias) => {
    const data = new Date(hoje.getFullYear(), hoje.getMonth(), hoje.getDate() + dias);
    return `${data.getFullYear()}-${String(data.getMonth() + 1).padStart(2, "0")}-${String(data.getDate()).padStart(2, "0")}`;
  };

  return `---
title: __uisnap
tags: [snapshot]
---

{{ type: "callout" }}
variant: info
title: Destaque
body: |
  Corpo do destaque.
{{ /callout }}

{{ type: "columns" }}
columns:
- width: 1
  body: |
    Coluna da esquerda.
- width: 2
  body: |
    Coluna da direita, mais larga.
{{ /columns }}

{{ type: "kanban" }}
columns:
- Backlog
- Fazendo
- Feito
items:
- title: Card com tudo
  column: Backlog
  description: Uma descrição.
  tags:
  - urgente
  - bug
  due: '${d(15)}'
  checklist:
  - text: Sub-item
    done: true
- title: Card simples
  column: Feito
{{ /kanban }}

{{ type: "table" }}
columns:
- name: Tarefa
- name: Status
  type: select
  options: [todo, doing, done]
- name: Tags
  type: multiselect
  options: [urgente, bug]
- name: Estimativa
  type: number
---
| Tarefa | Status | Tags         | Estimativa |
| ------ | ------ | ------------ | ---------- |
| API    | done   | urgente      | 8          |
| UI     | doing  | urgente, bug | 5          |
{{ /table }}

{{ type: "calendar" }}
entries:
- date: '${d(10)}'
  title: Evento simples
- date: '${d(12)}'
  title: Evento com hora
  start_time: '14:30'
  end_time: '15:15'
  tag: urgente
{{ /calendar }}

{{ type: "timeline" }}
scale: month
items:
- title: Primeira etapa
  start: '${relativa(-3)}'
  end: '${relativa(4)}'
  tags:
  - infra
- title: Segunda etapa
  start: '${relativa(5)}'
  end: '${relativa(12)}'
{{ /timeline }}

{{ type: "gallery" }}
columns: 3
size: md
items:
- path: assets/nao-existe-a.png
  caption: Legenda A
- path: assets/nao-existe-b.png
  caption: Legenda B
{{ /gallery }}

{{ type: "query" }}
from: pages
limit: 3
view: list
{{ /query }}

{{ type: "actions" }}
layout: row
buttons:
- label: Abrir
  icon: home
  action: open-page
  path: pages/incio.md
- label: Buscar
  icon: search
  action: run-search
  query: teste
{{ /actions }}
`;
}

/// Seletor da raiz de cada tipo de embed no DOM.
const RAIZES = {
  callout: ".callout",
  columns: ".columns-embed",
  kanban: ".embed-kanban",
  table: ".embed-table",
  calendar: ".calendar-grid",
  timeline: ".timeline-embed, [class*='timeline']",
  gallery: ".gallery",
  query: ".query-embed",
  actions: ".actions-embed, [class*='actions']",
};

/// Roda no webview: monta a impressão digital de uma subárvore.
function scriptImpressao(seletor, props) {
  return `(() => {
    const raiz = document.querySelector(${JSON.stringify(seletor)});
    if (!raiz) return null;
    const props = ${JSON.stringify(props)};
    const porClasse = {};
    const visitar = (el, prof) => {
      if (prof > 8) return;
      // Elementos sem classe não têm identidade estável — o estilo deles
      // vem do pai, que já está sendo medido.
      const chave = [...el.classList].sort().join(".");
      if (chave) {
        if (!porClasse[chave]) {
          const cs = getComputedStyle(el);
          const estilos = {};
          for (const p of props) estilos[p] = cs.getPropertyValue(p).trim();
          porClasse[chave] = { estilos, n: 0 };
        }
        porClasse[chave].n++;
      }
      for (const filho of el.children) visitar(filho, prof + 1);
    };
    visitar(raiz, 0);
    return porClasse;
  })()`;
}

function comparar(tipo, base, atual) {
  const problemas = [];
  const classes = new Set([...Object.keys(base), ...Object.keys(atual)]);
  for (const c of [...classes].sort()) {
    const b = base[c];
    const a = atual[c];
    if (!b) {
      problemas.push(`classe nova: .${c} (${a.n}x)`);
      continue;
    }
    if (!a) {
      problemas.push(`classe sumiu: .${c} (era ${b.n}x)`);
      continue;
    }
    for (const p of Object.keys(b.estilos)) {
      if (b.estilos[p] !== a.estilos[p]) {
        problemas.push(`.${c} → ${p}: "${b.estilos[p]}" virou "${a.estilos[p]}"`);
      }
    }
    if (b.n !== a.n && !CONTAGEM_VARIAVEL.has(tipo)) {
      problemas.push(`.${c} → contagem: ${b.n} virou ${a.n}`);
    }
  }
  return problemas;
}

/// Tira a impressão digital de todos os tipos. Exportado pra `run.mjs`
/// rodar isso como um cenário a mais.
export async function conferirSnapshots(bridge, { atualizar = false, filtro = null } = {}) {
  if (!existsSync(BASELINE)) mkdirSync(BASELINE, { recursive: true });
  await bridge.redimensionar(LARGURA, ALTURA);

  writeFileSync(ARQUIVO, fixture());
  try {
    await bridge.js("location.reload()");
    await new Promise((r) => setTimeout(r, 2500));
    await abrirPagina(bridge, "__uisnap");
    await esperar(bridge, "document.querySelector('.callout')", "os embeds renderizarem");
    // Consulta e galeria carregam assíncrono.
    await new Promise((r) => setTimeout(r, 1500));

    const resultados = [];
    for (const [tipo, seletor] of Object.entries(RAIZES)) {
      if (filtro && !tipo.includes(filtro)) continue;
      const atual = await bridge.js(scriptImpressao(seletor, PROPS));
      if (!atual) {
        resultados.push({ tipo, problemas: [`não achei o embed no DOM (${seletor})`] });
        continue;
      }
      const caminho = join(BASELINE, `${tipo}.json`);
      if (atualizar || !existsSync(caminho)) {
        writeFileSync(caminho, JSON.stringify(atual, null, 2) + "\n");
        resultados.push({ tipo, problemas: [], gravado: true });
        continue;
      }
      const base = JSON.parse(readFileSync(caminho, "utf8"));
      resultados.push({ tipo, problemas: comparar(tipo, base, atual) });
    }
    return resultados;
  } finally {
    if (existsSync(ARQUIVO)) unlinkSync(ARQUIVO);
  }
}

// Execução direta (não importado por `run.mjs`).
if (process.argv[1] && process.argv[1].endsWith("snapshot.mjs")) {
  const args = process.argv.slice(2);
  const atualizar = args.includes("--atualizar");
  const filtro = args.find((a) => !a.startsWith("--")) || null;

  let bridge;
  try {
    bridge = await Bridge.conectar();
  } catch (e) {
    console.error(`✗ ${e.message}`);
    process.exit(2);
  }

  const resultados = await conferirSnapshots(bridge, { atualizar, filtro });
  bridge.fechar();

  let falhas = 0;
  for (const r of resultados) {
    if (r.gravado) {
      console.log(`  ↻ ${r.tipo} (baseline gravada)`);
    } else if (r.problemas.length === 0) {
      console.log(`  ✓ ${r.tipo}`);
    } else {
      falhas++;
      console.log(`  ✗ ${r.tipo}`);
      for (const p of r.problemas) console.log(`      ${p}`);
    }
  }
  process.exit(falhas ? 1 : 0);
}
