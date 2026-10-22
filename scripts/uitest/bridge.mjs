// Cliente do MCP Bridge que o app já expõe (porta 9223, plugin
// `tauri-plugin-mcp-bridge`). Zero dependência: o WebSocket é o global
// do Node 22+.
//
// Protocolo: {"id","command":"execute_js","args":{"script"}} →
// {"data","success","error"}.

const PORTA = Number(process.env.ANOTADINHO_MCP_PORT || 9223);
const TIMEOUT_MS = 15000;

export class Bridge {
  #ws;
  #pendentes = new Map();
  #proximoId = 1;

  static async conectar() {
    const b = new Bridge();
    await b.#abrir();
    return b;
  }

  #abrir() {
    return new Promise((resolve, reject) => {
      this.#ws = new WebSocket(`ws://127.0.0.1:${PORTA}`);
      const falhou = () =>
        reject(
          new Error(
            `não consegui falar com o app na porta ${PORTA}. ` +
              `Suba com ./scripts/dev.sh antes de rodar os testes.`,
          ),
        );
      this.#ws.onerror = falhou;
      this.#ws.onclose = () => {
        for (const { reject } of this.#pendentes.values()) {
          reject(new Error("conexão com o app caiu no meio do teste"));
        }
        this.#pendentes.clear();
      };
      this.#ws.onmessage = (ev) => {
        let msg;
        try {
          msg = JSON.parse(String(ev.data));
        } catch {
          return;
        }
        const p = this.#pendentes.get(msg.id);
        if (!p) return;
        this.#pendentes.delete(msg.id);
        if (msg.success) p.resolve(msg.data);
        else p.reject(new Error(msg.error || "erro sem mensagem"));
      };
      this.#ws.onopen = () => {
        this.#ws.onerror = () => {};
        resolve();
      };
    });
  }

  /// Roda JS no webview e devolve o valor (precisa ser serializável).
  js(script) {
    const id = String(this.#proximoId++);
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        this.#pendentes.delete(id);
        reject(new Error(`timeout de ${TIMEOUT_MS}ms rodando JS`));
      }, TIMEOUT_MS);
      this.#pendentes.set(id, {
        resolve: (v) => {
          clearTimeout(timer);
          resolve(v);
        },
        reject: (e) => {
          clearTimeout(timer);
          reject(e);
        },
      });
      this.#ws.send(JSON.stringify({ id, command: "execute_js", args: { script } }));
    });
  }

  /// Comando cru do bridge — `js()` é açúcar em cima deste.
  comando(command, args = {}) {
    const id = String(this.#proximoId++);
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        this.#pendentes.delete(id);
        reject(new Error(`timeout de ${TIMEOUT_MS}ms no comando ${command}`));
      }, TIMEOUT_MS);
      this.#pendentes.set(id, {
        resolve: (v) => {
          clearTimeout(timer);
          resolve(v);
        },
        reject: (e) => {
          clearTimeout(timer);
          reject(e);
        },
      });
      this.#ws.send(JSON.stringify({ id, command, args }));
    });
  }

  /// Fixa o tamanho da janela — a impressão digital do ciclo 187 tem
  /// geometria dentro, então a largura precisa ser a mesma sempre.
  redimensionar(width, height) {
    return this.comando("resize_window", { width, height, logical: true });
  }

  fechar() {
    this.#ws?.close();
  }
}

/// Espera `cond` (JS que devolve booleano) virar true, ou estoura.
export async function esperar(bridge, cond, descricao, limiteMs = 8000) {
  const ate = Date.now() + limiteMs;
  let ultimoErro = null;
  while (Date.now() < ate) {
    try {
      const ok = await bridge.js(
        `(() => { try { return !!(${cond}); } catch (e) { return false; } })()`,
      );
      if (ok) return true;
    } catch (e) {
      // Um eval que cai exatamente na troca do documento não é falha do
      // cenário: é a recarga acontecendo. Antes isso abortava o teste
      // com "Script execution timeout", e o cenário morria por causa
      // do relógio da máquina, não do código.
      //
      // O prazo continua valendo: se a condição nunca chegar, o erro
      // sai igual, agora dizendo qual foi o último problema.
      ultimoErro = e;
    }
    await new Promise((r) => setTimeout(r, 250));
  }
  const detalhe = ultimoErro ? ` (último erro: ${ultimoErro.message})` : "";
  throw new Error(`esperava ${descricao}, mas não aconteceu em ${limiteMs}ms${detalhe}`);
}

/// Espera o elemento existir e SÓ ENTÃO clica nele.
///
/// Clicar direto em `querySelector(...)` é corrida sempre que o cenário
/// esperou por um sinal PRÓXIMO — a aba entrou na barra, o embed
/// renderizou — em vez de esperar pelo próprio alvo. Quando o alvo ainda
/// não chegou, o erro que sai é `null is not an object`, que parece
/// defeito do app e não é. Custou meio ciclo achar isso no 276.
///
/// A mensagem de falha nomeia o seletor, então a próxima vez se lê
/// sozinha.
export async function clicar(bridge, seletor, descricao, limiteMs = 8000) {
  const alvo = descricao || `o elemento ${seletor}`;
  await esperar(bridge, `!!document.querySelector(${JSON.stringify(seletor)})`,
    `${alvo} aparecer`, limiteMs);
  return bridge.js(
    `(() => { document.querySelector(${JSON.stringify(seletor)}).click(); return true; })()`,
  );
}

/// Abre uma página pelo nome que aparece na sidebar.
///
/// TENTA achar o alvo até conseguir (ciclo 270). A versão anterior
/// clicava UMA vez e, se o item ainda não estivesse na sidebar, não
/// clicava em nada — e depois esperava 8 segundos por um título que
/// nunca ia chegar. O erro dizia "esperava a página X abrir", que acusa
/// o app de não abrir quando o clique nem aconteceu.
///
/// A corrida é real: `recarregarEstavel` espera a sidebar ter ALGUM
/// item, não este item.
///
/// O clique acontece SEMPRE, mesmo que o título já bata: o app restaura
/// as abas depois de um reload, então o título da página anterior pode
/// estar na tela enquanto o conteúdo ainda é o velho.
/// Abre as pastas fechadas da sidebar, um nível por chamada, até não
/// sobrar nenhuma (ou `niveis` voltas).
///
/// Desde o ciclo 466 a página de uma pasta FECHADA não existe no DOM:
/// com o `<details>` de antes ela existia escondida, e o clique por
/// script achava o que nenhuma pessoa conseguiria clicar. Abrir a pasta é
/// o que a pessoa faz. Cada volta é uma chamada própria porque os filhos
/// só aparecem depois que o Yew renderiza.
export async function abrirTodasAsPastas(bridge, niveis = 8) {
  for (let i = 0; i < niveis; i++) {
    const abriu = await bridge.js(`(() => {
      const fechadas = [...document.querySelectorAll('.app-sidebar .sidebar-folder:not(.sidebar-folder--aberta) > .sidebar-folder__header')];
      fechadas.forEach(h => h.click());
      return fechadas.length;
    })()`);
    if (!abriu) return;
    // Condição, não relógio: espera as que foram clicadas abrirem de fato
    // (com a pasta de 168 ciclos, renderizar passa de 100 ms com a suíte
    // carregada, e o clique seguinte achava a página ainda ausente).
    const ate = Date.now() + 3000;
    while (Date.now() < ate) {
      const restam = await bridge.js(
        "document.querySelectorAll('.app-sidebar .sidebar-folder:not(.sidebar-folder--aberta)').length",
      );
      if (restam === 0 || restam < abriu) break;
      await new Promise((r) => setTimeout(r, 50));
    }
  }
}

/// Abre só as pastas de um caminho (`"produto"`, `"projetos/x/tarefas"`),
/// de fora pra dentro — o que a pessoa faz quando sabe onde a página
/// está. Abrir TUDO renderiza as 168 páginas de ciclos, e com a tela
/// pesada o grafo estourava o orçamento dele (ciclo 466).
export async function abrirPastaDe(bridge, caminho) {
  const partes = caminho.split("/").filter(Boolean);
  for (let i = 1; i <= partes.length; i++) {
    const sufixo = "/" + partes.slice(0, i).join("/");
    const sel = `[...document.querySelectorAll('.app-sidebar .sidebar-folder')].find(d => d.dataset.pasta.endsWith(${JSON.stringify(sufixo)}))`;
    await esperar(bridge, `!!${sel}`, `a pasta ${sufixo} na sidebar`);
    await bridge.js(`(() => { const d = ${sel}; if (!d.classList.contains('sidebar-folder--aberta')) d.querySelector(':scope > .sidebar-folder__header').click(); return true; })()`);
    await esperar(bridge, `${sel}.classList.contains('sidebar-folder--aberta')`, `a pasta ${sufixo} abrir`);
  }
}

export async function abrirPagina(bridge, nome, limiteMs = 10000) {
  const clicar = `(() => {
    const alvo = [...document.querySelectorAll('*')]
      .filter(e => e.children.length === 0 && e.textContent.trim() === ${JSON.stringify(nome)})[0];
    if (alvo) alvo.click();
    return !!alvo;
  })()`;

  const ate = Date.now() + limiteMs;
  let clicou = false;
  while (Date.now() < ate && !clicou) {
    clicou = await bridge.js(clicar);
    if (!clicou) {
      // A página pode estar numa pasta fechada (ciclo 466).
      await abrirTodasAsPastas(bridge);
      clicou = await bridge.js(clicar);
    }
    if (!clicou) await new Promise((r) => setTimeout(r, 250));
  }
  if (!clicou) {
    throw new Error(`"${nome}" não apareceu na sidebar em ${limiteMs}ms`);
  }
  await esperar(
    bridge,
    // `.conversa__titulo` entra aqui porque uma página `type: conversa`
    // tem cabeçalho próprio e não usa o do editor (ciclo 202).
    `(document.querySelector('.editor__header, .editor__title, .conversa__titulo')||{}).textContent?.includes(${JSON.stringify(nome)})`,
    `a página "${nome}" abrir`,
  );
}

/// Espera até `expr` devolver o MESMO valor em duas leituras seguidas.
///
/// Serve pra "a tela parou de mudar": o app recarrega a página aberta
/// quando o watcher vê o arquivo que o cenário acabou de escrever, e
/// mexer no DOM no meio dessa recarga desfaz o que o cenário fez. Antes
/// isso era resolvido com `PAUSA(2200)` — tempo suficiente no pior caso,
/// e desperdiçado em todos os outros (ciclo 198).
export async function esperarEstavel(bridge, expr, descricao, limiteMs = 10000) {
  const ate = Date.now() + limiteMs;
  let anterior = null;
  let iguais = 0;
  while (Date.now() < ate) {
    let atual = null;
    try {
      atual = await bridge.js(
        `(() => { try { return ${expr}; } catch (e) { return null; } })()`,
      );
    } catch {
      // Recarga em curso: conta como leitura instável, não como falha.
      atual = null;
    }
    if (atual !== null && atual !== "" && atual === anterior) {
      iguais++;
      // Duas leituras iguais seguidas com 120ms entre elas: o suficiente
      // pra pegar uma recarga em curso, que troca o DOM inteiro.
      if (iguais >= 2) return atual;
    } else {
      iguais = 0;
    }
    anterior = atual;
    await new Promise((r) => setTimeout(r, 120));
  }
  throw new Error(`esperava ${descricao} estabilizar, mas não aconteceu em ${limiteMs}ms`);
}

/// Recarrega e espera a UI voltar — sem tempo fixo.
///
/// O marcador em `window` é o que evita a corrida: logo depois de pedir
/// o reload, o DOM ANTIGO ainda está lá, e uma condição do tipo "a
/// sidebar tem itens" passa na hora contra a tela velha. O cenário então
/// clicava num nó que estava prestes a ser destruído. O marcador só
/// desaparece quando o documento é REALMENTE trocado (ciclo 198).
/// Foco simulado quando a janela do app NÃO tem o foco do sistema (ciclo 455).
///
/// Medido no WebKitGTK: sem foco do sistema (`document.hasFocus()` falso,
/// a pessoa trabalhando noutra janela), `el.focus()` troca o
/// `activeElement`, mas NÃO dispara `focus` nem `blur`. Três cenários
/// gravam no blur (célula de tabela, corpo do callout, campo da consulta)
/// e reprovavam inteiros, conforme quem estivesse com a janela em foco
/// — inclusive isolados, o que parecia regressão e não era.
///
/// O calço só entra nessa condição e faz `focus()`/`blur()` dispararem o
/// que uma janela focada dispararia. Ele não prova a fronteira com o
/// sistema (ver o padrão "fronteira-do-sistema"): o que estes cenários
/// medem é a lógica DEPOIS do foco, e é essa que continua exercida.
/// Devolve se o calço está ativo, pra o relatório dizer.
export async function garantirFoco(bridge) {
  return await bridge.js(`(() => {
    if (window.__uitest_foco_simulado) return true;
    if (document.hasFocus()) return false;
    window.__uitest_foco_simulado = true;
    const focar = HTMLElement.prototype.focus;
    const desfocar = HTMLElement.prototype.blur;
    const sair = (el, para) => {
      el.dispatchEvent(new FocusEvent('blur', { relatedTarget: para }));
      el.dispatchEvent(new FocusEvent('focusout', { bubbles: true, relatedTarget: para }));
    };
    HTMLElement.prototype.focus = function (...args) {
      const antes = document.activeElement;
      focar.apply(this, args);
      if (document.activeElement !== this || antes === this) return;
      if (antes && antes !== document.body) sair(antes, this);
      this.dispatchEvent(new FocusEvent('focus', { relatedTarget: antes }));
      this.dispatchEvent(new FocusEvent('focusin', { bubbles: true, relatedTarget: antes }));
    };
    HTMLElement.prototype.blur = function () {
      const estava = document.activeElement === this;
      desfocar.call(this);
      if (estava) sair(this, null);
    };
    return true;
  })()`);
}

export async function recarregarEstavel(bridge) {
  await bridge.js("window.__uitest_reload = 1; location.reload(); true");
  await esperar(
    bridge,
    "typeof window.__uitest_reload === 'undefined'",
    "o reload trocar o documento",
    15000,
  );
  await esperar(
    bridge,
    "document.querySelectorAll('.sidebar-item__title').length > 0",
    "a sidebar voltar depois do reload",
    15000,
  );
  // O reload troca o documento, e o calço de foco vai junto.
  await garantirFoco(bridge);
}

/// Abre a página de rascunho e espera o conteúdo PARAR de mudar.
export async function abrirPaginaEstavel(bridge, nome) {
  await abrirPagina(bridge, nome);
  await esperarEstavel(
    bridge,
    "(document.querySelector('.editor__wysiwyg')||{}).innerText || 'vazio'",
    "o conteúdo do editor",
  );
}
