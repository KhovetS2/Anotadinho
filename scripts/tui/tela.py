#!/usr/bin/env python3
"""A tela da TUI de verdade, num terminal de verdade (ciclos 494–497).

    python3 scripts/tui/tela.py pagina.md                 # abre e mostra
    python3 scripts/tui/tela.py pagina.md --teclas "j j"  # anda antes de mostrar
    python3 scripts/tui/tela.py pagina.md --largura 80 --altura 30

Sobe a TUI dentro de um `tmux` destacado: o tmux é o pseudoterminal E o
emulador, e o `capture-pane` devolve a tela como o terminal a desenhou —
com as larguras de caractere, os cortes e as molduras que um teste com
`TestBackend` não vê (o AGENTS.md manda conferir a TUI num pty).

O vault é TEMPORÁRIO (`anotadinho-cli init` + a página dada), e as
preferências também (`XDG_CONFIG_HOME`): a TUI grava o último vault aberto,
e sem isso o da pessoa passaria a abrir o vault de teste.

A página é aberta pela barra de comandos (`:` + o nome), e as teclas de
`--teclas` vão separadas por espaço, no formato do `tmux send-keys`
(`j`, `Enter`, `Escape`, `C-d`).
"""

import argparse
import os
import shutil
import subprocess
import sys
import tempfile
import time

RAIZ = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))


def tmux(*args, check=True):
    return subprocess.run(["tmux", *args], check=check, capture_output=True, text=True).stdout


def esperar_tela(sessao, contem, limite=10.0):
    """A tela, assim que ela tiver `contem`. Condição, não relógio."""
    ate = time.time() + limite
    tela = ""
    while time.time() < ate:
        tela = tmux("capture-pane", "-p", "-t", sessao)
        if contem in tela:
            return tela
        time.sleep(0.1)
    raise SystemExit(f"a tela não mostrou {contem!r} em {limite}s:\n{tela}")


def estavel(sessao, limite=5.0):
    """A tela depois de parar de mudar (duas leituras iguais seguidas)."""
    ate = time.time() + limite
    antes = None
    while time.time() < ate:
        agora = tmux("capture-pane", "-p", "-t", sessao)
        if agora == antes:
            return agora
        antes = agora
        time.sleep(0.15)
    return antes or ""


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("pagina", help="arquivo .md a abrir")
    ap.add_argument("--teclas", default="", help="teclas depois de abrir, separadas por espaço")
    ap.add_argument("--largura", type=int, default=100)
    ap.add_argument("--altura", type=int, default=40)
    ap.add_argument("--binario", default=os.path.join(RAIZ, "target/debug/anotadinho-tui"))
    a = ap.parse_args()

    cli = os.path.join(RAIZ, "target/debug/anotadinho-cli")
    for b in (a.binario, cli):
        if not os.path.exists(b):
            raise SystemExit(f"falta {b}: cargo build -p anotadinho-tui -p anotadinho-cli")
    if not shutil.which("tmux"):
        raise SystemExit("precisa do tmux")

    tmp = tempfile.mkdtemp(prefix="anotadinho-tela-")
    vault = os.path.join(tmp, "vault")
    cfg = os.path.join(tmp, "cfg")
    os.makedirs(cfg)
    sessao = f"anotadinho-tela-{os.getpid()}"
    try:
        subprocess.run([cli, "--vault", vault, "init"], check=True, capture_output=True)
        nome = os.path.splitext(os.path.basename(a.pagina))[0]
        shutil.copy(a.pagina, os.path.join(vault, "pages", f"{nome}.md"))
        env = f"XDG_CONFIG_HOME={cfg} TERM=xterm-256color"
        tmux("new-session", "-d", "-s", sessao, "-x", str(a.largura), "-y", str(a.altura),
             f"env {env} {a.binario} --vault {vault}")
        esperar_tela(sessao, "páginas")
        # Abre pela barra de comandos: independe de onde a página cai na
        # lista da sidebar.
        tmux("send-keys", "-t", sessao, ":")
        esperar_tela(sessao, ":")
        tmux("send-keys", "-t", sessao, "-l", nome)
        tmux("send-keys", "-t", sessao, "Enter")
        estavel(sessao)
        for t in a.teclas.split():
            tmux("send-keys", "-t", sessao, t)
            estavel(sessao)
        sys.stdout.write(estavel(sessao))
    finally:
        tmux("kill-session", "-t", sessao, check=False)
        shutil.rmtree(tmp, ignore_errors=True)


if __name__ == "__main__":
    main()
