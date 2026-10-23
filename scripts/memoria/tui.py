#!/usr/bin/env python3
"""Quanto a TUI do Anotadinho ocupa de memória num dia de uso (ciclo 481).

    python3 scripts/memoria/tui.py                       # o binário de release, se houver
    python3 scripts/memoria/tui.py --binario target/debug/anotadinho-tui
    python3 scripts/memoria/tui.py --rodadas 8 --saida /tmp/x.json

Sobe a TUI num pseudoterminal (com tamanho de janela: sem `TIOCSWINSZ`
ela nasce 0x0 e não desenha nada), num vault TEMPORÁRIO criado com
`anotadinho-cli init` mais as mesmas páginas do script da janela: 20
notas, uma conversa com 30 mensagens e um projeto de 300 tarefas. Manda
as teclas de um "dia" em rodadas e amostra o PSS do processo a cada
500 ms. O vault da pessoa nunca é tocado.

Só biblioteca padrão. Linux: lê o /proc.
"""

import argparse
import fcntl
import json
import os
import pty
import select
import shutil
import struct
import subprocess
import sys
import tempfile
import termios
import time

RAIZ = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
N_PAGINAS = 20
N_TAREFAS = 300


def memoria(pid):
    """Pss, Rss e USS em KiB, ou None se o processo já morreu."""
    try:
        with open(f"/proc/{pid}/smaps_rollup") as f:
            t = f.read()
    except OSError:
        return None
    def kb(campo):
        for linha in t.splitlines():
            if linha.startswith(campo + ":"):
                return int(linha.split()[1])
        return 0
    return {"pss": kb("Pss"), "rss": kb("Rss"), "uss": kb("Private_Clean") + kb("Private_Dirty")}


def fonte_mais_nova():
    """O mtime do fonte mais recente que entra no binário da TUI."""
    mais = 0
    for crate in ("tui", "core", "ipc", "vault"):
        for raiz, _, arqs in os.walk(os.path.join(RAIZ, "crates", crate)):
            for a in arqs:
                if a.endswith((".rs", ".md", ".toml")):
                    mais = max(mais, os.path.getmtime(os.path.join(raiz, a)))
    return mais


def montar_vault(cli, vault):
    subprocess.run([cli, "--vault", vault, "init"], check=True, stdout=subprocess.DEVNULL)
    notas = os.path.join(vault, "pages", "notas")
    os.makedirs(notas, exist_ok=True)
    for i in range(1, N_PAGINAS + 1):
        blocos = []
        for b in range(40):
            blocos.append(f"## Seção {b // 10 + 1}" if b % 10 == 0 else f"Parágrafo {b} da nota {i}, com [[Nota {i % N_PAGINAS + 1}]] e **negrito** no meio.")
        blocos.append("- [ ] item um\n- [x] item dois")
        blocos.append("| a | b |\n|---|---|\n| 1 | 2 |")
        if i % 4 == 0:
            blocos.append('{{ type: "kanban" }}\ncolumns:\n- A\n- B\nitems:\n- title: um\n  column: A\n{{ /kanban }}')
        with open(os.path.join(notas, f"nota-{i}.md"), "w") as f:
            f.write(f"---\ntitle: Nota {i}\n---\n# Nota {i}\n\n" + "\n\n".join(blocos) + "\n")
    msgs = []
    for m in range(30):
        quem = "agente" if m % 2 else "você"
        msgs.append(f"## {quem} · 2026-10-23 10:{m:02d}\n\n" + "Uma resposta com várias linhas de texto. " * (12 if m % 2 else 2))
    with open(os.path.join(notas, "conversa.md"), "w") as f:
        f.write("---\ntitle: Conversa memoria\ntype: conversa\n---\n" + "\n\n".join(msgs) + "\n")
    with open(os.path.join(notas, "rascunho.md"), "w") as f:
        f.write("---\ntitle: Rascunho memoria\n---\n# Rascunho memoria\n\ncomeço\n")
    with open(os.path.join(notas, "grafo.md"), "w") as f:
        f.write("---\ntitle: Grafo memoria\ntype: graph\n---\n")
    proj = os.path.join(vault, "pages", "projetos", "memoria")
    for sub in ("tarefas", "sprints", "conversas"):
        os.makedirs(os.path.join(proj, sub), exist_ok=True)
    with open(os.path.join(proj, "projeto.md"), "w") as f:
        f.write("---\ntitle: Projeto memoria\ntype: projeto\nprojeto: memoria\n---\n# Projeto memoria\n")
    with open(os.path.join(proj, "arquitetura.md"), "w") as f:
        f.write("---\ntitle: Arquitetura\ntype: docs\nprojeto: memoria\n---\n")
    sprints = N_TAREFAS // 10
    for s in range(1, sprints + 1):
        estado = "ativa" if s == sprints else "encerrada"
        with open(os.path.join(proj, "sprints", f"{s:02d}-s{s}.md"), "w") as f:
            f.write(f"---\ntitle: Sprint {s}\ntype: sprint\nprojeto: memoria\nestado: {estado}\n---\n")
    etapas = ["ideia", "detalhando", "pronta", "bloqueada"]
    for t in range(1, N_TAREFAS + 1):
        s = (t - 1) // 10 + 1
        etapa = etapas[t % 4] if s == sprints else "feita"
        with open(os.path.join(proj, "tarefas", f"{t:03d}-t.md"), "w") as f:
            f.write(
                f"---\ntitle: Tarefa {t}\ntags:\n- tarefa\n- t{t % 7}\ntype: tarefa\nprojeto: memoria\nsprint: {s}\n"
                f"prazo: 2026-11-{t % 28 + 1:02d}\nstatus: {etapa}\n---\n# Tarefa {t}\n\n## Critérios de aceite\n\n- [x] um\n- [ ] dois\n"
            )


class Tui:
    def __init__(self, binario, vault, cfg, largura=140, altura=40):
        self.pid, self.fd = pty.fork()
        if self.pid == 0:
            # A config num diretório próprio: a da pessoa (tema, vault
            # recente) não entra na medida nem é alterada por ela.
            os.environ["XDG_CONFIG_HOME"] = cfg
            os.environ["TERM"] = "xterm-256color"
            os.execv(binario, [binario, "--vault", vault])
        fcntl.ioctl(self.fd, termios.TIOCSWINSZ, struct.pack("HHHH", altura, largura, 0, 0))

    def drenar(self, segundos):
        """Lê a saída: um pty cheio TRAVA o processo do outro lado."""
        fim = time.time() + segundos
        while time.time() < fim:
            r, _, _ = select.select([self.fd], [], [], 0.05)
            if r:
                try:
                    os.read(self.fd, 1 << 16)
                except OSError:
                    return

    def teclas(self, *ts, entre=0.15):
        mapa = {"Enter": b"\r", "Esc": b"\x1b", "Tab": b"\t", "Alt+q": b"\x1bq", "Ctrl+k": b"\x0b"}
        for t in ts:
            os.write(self.fd, mapa.get(t, t.encode()))
            self.drenar(entre)

    def abrir(self, caminho):
        """Pela paleta, pelo CAMINHO: ela lista as páginas pelo nome do
        arquivo, com o caminho ao lado. Pelo título ("Grafo memoria") não
        acha nada, e o Enter vira busca no conteúdo — foi o que a
        primeira versão media sem perceber."""
        self.teclas("Esc", ":")
        self.teclas(caminho, entre=0.3)
        self.teclas("Enter", entre=0.6)

    def sair(self):
        try:
            os.write(self.fd, b"\x11")  # Ctrl+Q
            self.drenar(0.5)
            os.kill(self.pid, 9)
        except OSError:
            pass
        try:
            os.waitpid(self.pid, 0)
        except ChildProcessError:
            pass


def dia(tui):
    """As ações de uma rodada: (nome, função)."""
    def notas():
        for i in range(1, N_PAGINAS + 1):
            tui.abrir(f"pages/notas/nota-{i}.md")
    def digitar():
        tui.abrir("pages/notas/rascunho.md")
        # Depois da paleta o foco JÁ está no conteúdo: um Tab aqui o
        # levava pra sidebar, e o texto não era digitado.
        tui.teclas("G", "o")
        texto = "texto digitado pela pessoa no meio do dia. " * 47
        for i in range(0, 2000, 50):
            tui.teclas(texto[i:i + 50], entre=0.05)
        tui.teclas("Esc")
    def buscar():
        for termo in ("Parágrafo", "negrito", "Seção 3"):
            tui.teclas("Esc", ":")
            tui.teclas(termo, entre=0.4)
            tui.teclas("Esc")
    def projeto():
        tui.abrir("pages/projetos/memoria/projeto.md")
        tui.teclas("Tab")
        for _ in range(3):
            tui.teclas("s", "j", "Enter", entre=0.3)
        tui.teclas("l", "j", "v", entre=0.3)
        tui.teclas("Esc", "S", entre=0.3)
        tui.teclas("Esc")
        tui.drenar(9)  # três voltas da releitura de 3 s
    def conversa():
        tui.abrir("pages/notas/conversa.md")
        tui.teclas("G", "g", "g")
    def grafo():
        tui.abrir("pages/notas/grafo.md")
    def fechar():
        for _ in range(30):
            tui.teclas("Alt+q", entre=0.05)
    return [("abrir 20 notas", notas), ("digitar 2000 caracteres", digitar), ("buscar", buscar),
            ("projeto com 300 tarefas", projeto), ("conversa com 30 mensagens", conversa),
            ("grafo", grafo), ("fechar abas", fechar)]


def main():
    if sys.platform != "linux":
        sys.exit("✗ este script lê o /proc: só roda no Linux.")
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    release = os.path.join(RAIZ, "target", "release", "anotadinho-tui")
    debug = os.path.join(RAIZ, "target", "debug", "anotadinho-tui")
    ap.add_argument("--binario", default=release if os.path.exists(release) else debug)
    ap.add_argument("--cli", default=os.path.join(RAIZ, "target", "debug", "anotadinho-cli"))
    ap.add_argument("--rodadas", type=int, default=5)
    ap.add_argument("--saida", default=os.path.join(tempfile.gettempdir(), f"anotadinho-memoria-tui-{int(time.time())}.json"))
    ap.add_argument("--aceitar-velho", action="store_true", help="medir mesmo com o binário mais velho que o código")
    a = ap.parse_args()
    for b, como in ((a.binario, "cargo build --release -p anotadinho-tui"), (a.cli, "cargo build -p anotadinho-cli")):
        if not os.path.exists(b):
            sys.exit(f"✗ falta {b}: {como}")
    build = "desenvolvimento" if "/debug/" in a.binario else "release"
    # Um binário velho mede OUTRO programa: o `target/release` daqui tinha
    # mais de um mês quando o script nasceu, e a primeira medida saiu dele.
    if os.path.getmtime(a.binario) < fonte_mais_nova() and not a.aceitar_velho:
        quando = time.strftime("%Y-%m-%d %H:%M", time.localtime(os.path.getmtime(a.binario)))
        como = "cargo build --release -p anotadinho-tui" if build == "release" else "cargo build -p anotadinho-tui"
        sys.exit(f"✗ {a.binario} é de {quando}, mais velho que o código: {como} (ou --aceitar-velho)")

    base = tempfile.mkdtemp(prefix="anotadinho-memoria-")
    vault, cfg = os.path.join(base, "vault"), os.path.join(base, "cfg")
    os.makedirs(cfg)
    amostras, marcos = [], {}
    tui = None
    try:
        montar_vault(a.cli, vault)
        tui = Tui(a.binario, vault, cfg)
        t0 = time.time()
        estado = {"fase": "repouso", "rodada": 0, "acao": "repouso"}

        def amostrar():
            m = memoria(tui.pid)
            if m:
                amostras.append({"t": int((time.time() - t0) * 1000), **estado, "mem": m})
                marcos[estado["acao"]] = max(marcos.get(estado["acao"], 0), m["pss"])

        # A amostragem roda ENTRE as teclas: um laço só, sem thread, pra
        # não disputar o pty.
        proximo = [time.time()]
        drenar_original = tui.drenar
        def drenar_amostrando(segundos):
            fim = time.time() + segundos
            while time.time() < fim:
                if time.time() >= proximo[0]:
                    amostrar()
                    proximo[0] = time.time() + 0.5
                drenar_original(min(0.1, max(0, fim - time.time())))
        tui.drenar = drenar_amostrando

        tui.drenar(5)  # repouso: aberta, parada
        estado["fase"] = "dia"
        acoes = dia(tui)
        for r in range(1, a.rodadas + 1):
            estado["rodada"] = r
            for nome, fn in acoes:
                estado["acao"] = nome
                print(f"rodada {r}/{a.rodadas}: {nome}…".ljust(48), end="\r", flush=True)
                fn()
                if os.waitpid(tui.pid, os.WNOHANG) != (0, 0):
                    tui = None
                    sys.exit(f"\n✗ a TUI saiu durante \"{nome}\" (rodada {r}): a medida não vale.")
            estado["acao"] = "assentar"
            tui.drenar(3)
        estado["fase"] = "fim"
        tui.drenar(1.5)
    finally:
        if tui:
            tui.sair()
        shutil.rmtree(base, ignore_errors=True)

    if not amostras:
        sys.exit("✗ nenhuma amostra: a TUI morreu logo ao abrir?")
    mb = lambda k: f"{k / 1024:.1f}"
    rep = [x for x in amostras if x["fase"] == "repouso"]
    fim_de = lambda r: [x for x in amostras if x["rodada"] == r][-1]
    pico = max(x["mem"]["pss"] for x in amostras)
    cresce = (fim_de(a.rodadas)["mem"]["pss"] - fim_de(1)["mem"]["pss"]) / (a.rodadas - 1) if a.rodadas > 1 else 0
    resumo = {
        "repouso_kib": max(x["mem"]["pss"] for x in rep) if rep else 0,
        "pico_kib": pico,
        "fim_kib": amostras[-1]["mem"]["pss"],
        "cresce_por_rodada_kib": round(cresce),
        "repouso_rss_kib": max(x["mem"]["rss"] for x in rep) if rep else 0,
        "pico_rss_kib": max(x["mem"]["rss"] for x in amostras),
    }
    feito = time.strftime("%Y-%m-%d %H:%M", time.localtime(os.path.getmtime(a.binario)))
    print(f"\nMemória da TUI — {build} ({a.binario}, compilado {feito}) · {a.rodadas} rodada(s) · {len(amostras)} amostras")
    print("PSS em MiB (RSS entre parênteses)\n")
    print(f"{'repouso':>16}{'pico':>16}{'fim':>16}{'cresce/rodada':>16}")
    print(f"{mb(resumo['repouso_kib']) + ' (' + mb(resumo['repouso_rss_kib']) + ')':>16}"
          f"{mb(pico) + ' (' + mb(resumo['pico_rss_kib']) + ')':>16}{mb(resumo['fim_kib']):>16}{('+' if cresce >= 0 else '') + mb(cresce):>16}")
    print("\nfim de cada rodada (PSS, MiB): " + " → ".join(mb(fim_de(r)["mem"]["pss"]) for r in range(1, a.rodadas + 1)))
    print("pico por ação (PSS, MiB):")
    for acao, kib in marcos.items():
        print(f"  {acao:<28} {mb(kib)}")
    with open(a.saida, "w") as f:
        json.dump({"build": build, "binario": a.binario, "rodadas": a.rodadas, "resumo": resumo, "marcos": marcos, "amostras": amostras}, f)
    print(f"\namostras em {a.saida}")


if __name__ == "__main__":
    main()
