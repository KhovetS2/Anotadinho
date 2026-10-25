#!/usr/bin/env python3
"""Piloto de ponta a ponta do fluxo de tarefa no repositório (ciclo 520).

    python3 scripts/tui/piloto-projeto.py            # verificação passa
    python3 scripts/tui/piloto-projeto.py --falha    # verificação reprova

A TUI de verdade num tmux, o agente de teste (`agente-falso.sh
--tarefa-saas`) e um repositório git temporário com uma verificação de
verdade (`node soma.test.mjs`), branch por tarefa e commit ao aceitar.

Passa por: Pronta → Executar (tecla) → o agente roda no repositório → a
verificação → Revisão com ✓teste → Mudanças (`D`) com o commit do agente e
o arquivo novo → Feita (tecla) → o commit `tarefa 001: …` e a árvore limpa.
Com `--falha`, para em Bloqueada com ✗teste e a saída na seção Verificação.

Espera CONDIÇÃO (a etapa no disco, o texto na tela), nunca relógio: o
quadro só muda depois que a thread de fechamento termina.
"""

import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import time

AQUI = os.path.dirname(os.path.abspath(__file__))
# Sem `__pycache__` no repositório: o `tela` é importado daqui.
sys.dont_write_bytecode = True
sys.path.insert(0, AQUI)
from tela import esperar_tela, estavel, tmux  # noqa: E402

RAIZ = os.path.dirname(os.path.dirname(AQUI))
CLI = os.path.join(RAIZ, "target/debug/anotadinho-cli")
TUI = os.path.join(RAIZ, "target/debug/anotadinho-tui")
AGENTE = os.path.join(RAIZ, "scripts/uitest/agente-falso.sh")
TAREFA = "pages/projetos/meu-primeiro-projeto/tarefas/001-esqueleto-do-app.md"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--falha", action="store_true", help="a verificação reprova")
    ap.add_argument("--manter", action="store_true", help="não apaga a pasta temporária")
    a = ap.parse_args()
    for b in (CLI, TUI):
        if not os.path.exists(b):
            raise SystemExit(f"falta {b}: cargo build -p anotadinho-tui -p anotadinho-cli")

    tmp = tempfile.mkdtemp(prefix="anotadinho-piloto-")
    vault, cfg, repo = (os.path.join(tmp, x) for x in ("vault", "cfg", "repo"))
    tarefa = os.path.join(vault, TAREFA)
    s = f"piloto-{os.getpid()}"

    def git(*args):
        return subprocess.run(["git", "-C", repo, *args], capture_output=True, text=True, check=False).stdout.strip()

    def status():
        m = re.search(r"^status:\s*(\S+)", open(tarefa).read(), re.M)
        return m.group(1) if m else None

    def esperar_status(alvo, limite=30):
        ate = time.time() + limite
        while time.time() < ate:
            if status() == alvo:
                return
            time.sleep(0.2)
        raise SystemExit(f"a tarefa ficou em {status()!r}, esperava {alvo!r}\n" + tmux("capture-pane", "-p", "-t", s))

    def tecla(t):
        tmux("send-keys", "-t", s, t)
        estavel(s)

    def conferir(ok, o_que):
        print(("✓ " if ok else "✗ ") + o_que)
        if not ok:
            print(tmux("capture-pane", "-p", "-t", s))
            raise SystemExit(1)

    try:
        subprocess.run([CLI, "--vault", vault, "init"], check=True, capture_output=True)
        # O SaaS de mentira: `soma` concatena; o agente conserta e escreve o teste.
        os.makedirs(repo)
        git("init", "-q", "-b", "main")
        git("config", "user.email", "piloto@teste")
        git("config", "user.name", "piloto")
        with open(os.path.join(repo, "soma.js"), "w") as f:
            f.write("export function soma(a, b) {\n  return a + b;\n}\n")
        git("add", "soma.js")
        git("commit", "-qm", "início")
        os.makedirs(os.path.join(vault, ".anotadinho"))
        verificar = "node -e 'process.exit(2)'" if a.falha else "node soma.test.mjs"
        with open(os.path.join(vault, ".anotadinho", "projetos.json"), "w") as f:
            json.dump({"meu-primeiro-projeto": {"repositorio": repo, "pausado": False, "verificar": verificar,
                                                "branch_por_tarefa": True, "commitar_ao_aceitar": True}}, f)
        os.makedirs(os.path.join(cfg, "anotadinho"))
        with open(os.path.join(cfg, "anotadinho", "tui.json"), "w") as f:
            json.dump({"agente": {"nome": "falso", "binario": AGENTE, "args": ["--tarefa-saas", "{prompt}"]}}, f)
        subprocess.run([CLI, "--vault", vault, "set-property", TAREFA, "status", "pronta"], check=True, capture_output=True)

        tmux("new-session", "-d", "-s", s, "-x", "150", "-y", "36",
             f"env XDG_CONFIG_HOME={cfg} TERM=xterm-256color {TUI} --vault {vault}")
        esperar_tela(s, "páginas")
        tmux("send-keys", "-t", s, ":")
        esperar_tela(s, ":")
        tmux("send-keys", "-t", s, "-l", "Meu primeiro projeto")
        tmux("send-keys", "-t", s, "Enter")
        esperar_tela(s, "fila ativa")
        tecla("l")  # a coluna Pronta, onde está a 001
        tela = esperar_tela(s, "001 Esqueleto do app")
        m = re.search(r"(\d) → Executar", tela)
        conferir(m is not None, "a 001 oferece → Executar")
        tecla(m.group(1))

        if a.falha:
            esperar_status("bloqueada")
            tela = esperar_tela(s, "001-esqueleto-do-app → Bloqueada")
            conferir("✗teste" in tela, "o cartão diz ✗teste e o aviso diz pra onde a tarefa foi")
            texto = open(tarefa).read()
            conferir("## Verificação" in texto and "FALHOU" in texto, "a saída da verificação está na tarefa")
            return

        esperar_status("revisao")
        tela = esperar_tela(s, "001-esqueleto-do-app → Revisão")
        conferir("✓teste" in tela, "Revisão com ✓teste, e o aviso diz pra onde a tarefa foi")
        conferir(git("branch", "--show-current") == "tarefa/001-esqueleto-do-app", "rodou no branch da tarefa")
        conferir("soma converte a entrada" in git("log", "--oneline", "main..HEAD"), "o commit do agente está no branch")
        tecla("D")
        tela = esperar_tela(s, "soma.test.mjs")
        conferir("+  return Number(a) + Number(b);" in tela and "(novo, fora do git)" in tela,
                 "Mudanças mostra o commit do agente e o arquivo novo")
        tecla("Escape")
        tela = estavel(s)
        m = re.search(r"(\d) → Feita", tela)
        conferir(m is not None, "a Revisão oferece → Feita")
        tecla(m.group(1))
        esperar_status("feita")
        conferir(git("log", "-1", "--format=%s") == "tarefa 001: Esqueleto do app", "aceitar commitou")
        conferir(git("status", "--short") == "", "a árvore ficou limpa")
        print(estavel(s))
    finally:
        tmux("kill-session", "-t", s, check=False)
        if a.manter:
            print(f"pasta mantida em {tmp}")
        else:
            shutil.rmtree(tmp, ignore_errors=True)


if __name__ == "__main__":
    main()
