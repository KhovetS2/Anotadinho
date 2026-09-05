// Bateria PENDENTE: cenários escritos a partir de specs ainda não
// implementadas.
//
// Por que fica separada de `todos`: ela é VERMELHA de propósito. A
// suíte principal é o sinal de "está tudo certo?" e precisa ficar verde
// — misturar aqui destruiria esse sinal. Roda só com:
//
//   node scripts/uitest/run.mjs --pendentes
//
// À medida que cada spec for implementada, o cenário correspondente
// migra pra bateria permanente (`interacoes.mjs`, `telas.mjs`, etc) e
// sai daqui.
//
// ─────────────────────────────────────────────────────────────────────
//
// ESTÁ VAZIA (ciclo 257), e é a primeira vez.
//
// Os últimos sete cenários saíram assim:
//
// - `consultas` (3): o RF2 estava implementado com a chave errada; a
//   correção foi de uma linha e eles migraram pra `interacoes.mjs`
//   (ciclo 256).
// - `imagens` (4): o produto já fazia o que a spec pedia, por um
//   caminho que a spec não previa — o arraste abre o modal de
//   personalização, e a referência gravada é `<figure>` e não `![](…)`.
//   Os cenários é que estavam velhos, não o app. Reescritos contra o
//   critério real ("referência válida, nenhum `blob:`") e migrados
//   (ciclo 257).
//
// A lição do segundo grupo vale pra quem escrever aqui depois: um
// cenário pendente envelhece. Ele nasce de uma spec e passa a valer
// como se fosse a spec, mas o produto pode ter respondido a mesma
// pergunta de outro jeito no caminho — inclusive respondendo a uma
// "pergunta em aberto" que a própria spec deixou. Antes de tratar um
// vermelho daqui como bug, confira se ele não é só desatualização.
//
// O helper abaixo fica de pé pro próximo uso.

import { recarregarEstavel, abrirPaginaEstavel } from "./bridge.mjs";

export const pendentes = [];

/// `setup` pode ser o markdown da página de rascunho, ou
/// `{ md, vim }`.
///
/// O markdown é escrito e o vim é ligado ANTES do reload: a sidebar é
/// montada na carga (escrever depois deixa a página invisível pro
/// `abrirPaginaEstavel`) e o vim só entra em vigor no mount.
///
/// Cada cenário nomeia a spec que o originou, pra a ligação sobreviver.
export function pendente(spec, nome, setup, fn) {
  if (typeof setup === "function") {
    fn = setup;
    setup = null;
  }
  const { md = null, vim = false } =
    typeof setup === "string" ? { md: setup } : setup || {};
  pendentes.push({
    nome: `[${spec}] ${nome}`,
    async fn(bridge, ctx) {
      if (md) ctx.escrever(md);
      if (vim) {
        await bridge.js(`localStorage.setItem('anotadinho.vim_mode_enabled', 'true'); true`);
      }
      try {
        await recarregarEstavel(bridge);
        if (md) await abrirPaginaEstavel(bridge, ctx.nomePagina);
        await fn(bridge, ctx);
      } finally {
        // `run.mjs` normaliza o vim pra desligado só no INÍCIO da
        // suíte — sem desligar aqui, um cenário de vim contamina todos
        // os seguintes.
        if (vim) {
          await bridge.js(
            `localStorage.setItem('anotadinho.vim_mode_enabled', 'false'); true`,
          );
        }
      }
    },
  });
}
