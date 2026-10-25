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

Sem página, `--abrir` diz o que digitar na barra — o `init` já semeia um
projeto, então o quadro dele abre sem montar nada (ciclo 501):

    python3 scripts/tui/tela.py --abrir "Meu primeiro projeto" --teclas "x"
    python3 scripts/tui/tela.py --abrir "Meu primeiro" --execucoes reg.jsonl

`--execucoes` copia um registro pro `.anotadinho/execucoes.jsonl` do vault
temporário; `--preparar x.sh` roda `x.sh <vault>` antes de a TUI abrir (um
repositório git e a config do projeto, por exemplo); `--cada` mostra a tela depois de CADA tecla; `--manter` não
apaga o vault no fim e diz onde ele ficou, pra conferir o disco.
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


def estavel(sessao, limite=5.0, cores=False):
    """A tela depois de parar de mudar (duas leituras iguais seguidas).
    Com `cores`, vêm junto os códigos de cor do terminal (`capture-pane -e`)."""
    ate = time.time() + limite
    antes = None
    while time.time() < ate:
        agora = tmux("capture-pane", "-p", *(["-e"] if cores else []), "-t", sessao)
        if agora == antes:
            return agora
        antes = agora
        time.sleep(0.15)
    return antes or ""


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("pagina", nargs="?", help="arquivo .md a abrir")
    ap.add_argument("--abrir", help="o que digitar na barra de comandos (padrão: o nome da página)")
    ap.add_argument("--execucoes", help="registro de execuções (.jsonl) pro vault temporário")
    ap.add_argument("--preparar", help="executável chamado com o caminho do vault temporário, antes de abrir")
    ap.add_argument("--cada", action="store_true", help="mostra a tela depois de cada tecla")
    ap.add_argument("--manter", action="store_true", help="não apaga o vault temporário")
    ap.add_argument("--teclas", default="", help="teclas depois de abrir, separadas por espaço")
    ap.add_argument("--largura", type=int, default=100)
    ap.add_argument("--altura", type=int, default=40)
    ap.add_argument("--cores", action="store_true", help="mostra com as cores (códigos ANSI)")
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
        if a.pagina:
            nome = os.path.splitext(os.path.basename(a.pagina))[0]
            shutil.copy(a.pagina, os.path.join(vault, "pages", f"{nome}.md"))
        elif not a.abrir:
            raise SystemExit("dê uma página ou --abrir")
        nome = a.abrir or nome
        if a.execucoes:
            os.makedirs(os.path.join(vault, ".anotadinho"), exist_ok=True)
            shutil.copy(a.execucoes, os.path.join(vault, ".anotadinho", "execucoes.jsonl"))
        if a.preparar:
            # Depois do `init` e antes de a TUI subir: o que a cena precisa
            # e o `init` não semeia (um repositório git, a config do
            # projeto, a base da tarefa — ciclo 519).
            subprocess.run([a.preparar, vault], check=True)
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
            tela = estavel(sessao)
            if a.cada:
                sys.stdout.write(f"──── depois de {t} ────\n{tela}")
        sys.stdout.write(estavel(sessao, cores=a.cores))
    finally:
        tmux("kill-session", "-t", sessao, check=False)
        if a.manter:
            print(f"\nvault mantido em {vault}")
        else:
            shutil.rmtree(tmp, ignore_errors=True)


if __name__ == "__main__":
    main()
