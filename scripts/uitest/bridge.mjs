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
  let ultimo;
  while (Date.now() < ate) {
    ultimo = await bridge.js(`(() => { try { return !!(${cond}); } catch (e) { return false; } })()`);
    if (ultimo) return true;
    await new Promise((r) => setTimeout(r, 250));
  }
  throw new Error(`esperava ${descricao}, mas não aconteceu em ${limiteMs}ms`);
}

/// Abre uma página pelo nome que aparece na sidebar.
export async function abrirPagina(bridge, nome) {
  await bridge.js(`(() => {
    const alvo = [...document.querySelectorAll('*')]
      .filter(e => e.children.length === 0 && e.textContent.trim() === ${JSON.stringify(nome)})[0];
    if (alvo) alvo.click();
    return !!alvo;
  })()`);
  await esperar(
    bridge,
    `(document.querySelector('.editor__header, .editor__title')||{}).textContent?.includes(${JSON.stringify(nome)})`,
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
    const atual = await bridge.js(
      `(() => { try { return ${expr}; } catch (e) { return null; } })()`,
    );
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
