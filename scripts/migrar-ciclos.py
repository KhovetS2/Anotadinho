#!/usr/bin/env python3
"""Traz o histórico de ciclos pra dentro do vault (ciclo 206).

Por que existe: o Anotadinho foi construído registrando cada ciclo em
`cycles/tasks/` e `cycles/status/` — arquivos que só o repositório
enxerga. Trazê-los pro vault torna o próprio produto a ferramenta de
acompanhar o produto: dá pra consultar por status, ligar uma spec ao
ciclo que a implementou, e ver o histórico no grafo.

É idempotente: rodar de novo reescreve as páginas com o mesmo conteúdo.
Não apaga nada fora de `pages/ciclos/`.

Uso:
    python3 scripts/migrar-ciclos.py [--vault VaultAnotadinho]
"""

import argparse
import pathlib
import re
import sys

RAIZ = pathlib.Path(__file__).resolve().parent.parent


def ler_frontmatter(texto):
    """Devolve (dict do frontmatter, corpo). Parser mínimo — os campos
    aqui são escalares simples, e trazer um YAML completo só pra isto
    seria dependência à toa."""
    m = re.match(r"^---\n(.*?)\n---\n?(.*)$", texto, re.S)
    if not m:
        return {}, texto
    fm = {}
    for linha in m.group(1).splitlines():
        if ":" not in linha or linha.startswith(" "):
            continue
        chave, valor = linha.split(":", 1)
        fm[chave.strip()] = valor.strip().strip('"').strip("'")
    return fm, m.group(2)


def status_por_ciclo(pasta_status):
    """Mapa id -> caminho do arquivo de status mais recente."""
    out = {}
    for f in sorted(pasta_status.glob("*.md")):
        num = f.name.split("-", 1)[0]
        out[num] = f
    return out


def escalar(valor):
    """Escapa um valor pra caber num campo de frontmatter.

    Espelha `markdown::escapar_escalar_yaml` do core. Sem isto, um título
    com `: ` no meio — e vários ciclos têm — é YAML inválido e derruba o
    frontmatter INTEIRO em silêncio: a página perde título, tipo e tags
    de uma vez, e só se percebe quando ela some de uma consulta.
    """
    v = (valor or "").strip()
    perigoso = (
        not v
        or ": " in v
        or v.endswith(":")
        or " #" in v
        or v[0] in "&*!|>%@`\"'[{-?"
        or v.lower() in {"true", "false", "null", "yes", "no", "on", "off", "~"}
    )
    if not perigoso:
        try:
            float(v)
            perigoso = True
        except ValueError:
            pass
    if not perigoso:
        return v
    return '"' + v.replace("\\", "\\\\").replace('"', '\\"') + '"'


def montar_pagina(fm, corpo, arquivo_status):
    """Página do ciclo: frontmatter consultável + embed de fluxo + o
    conteúdo original da task, mais o status se houver."""
    ciclo_id = fm.get("id", "?")
    titulo = fm.get("titulo") or f"Ciclo {ciclo_id}"
    # `status` da task usa o vocabulário do ciclo 201 (`concluida`), e não
    # o `done` original — pra a consulta do painel enxergar tudo com o
    # mesmo filtro.
    etapa = "concluida" if fm.get("status") == "done" else "em-execucao"

    depende = fm.get("depende_de", "[]")
    linhas = [
        "---",
        f"title: {escalar(f'Ciclo {ciclo_id} — {titulo}')}",
        "type: ciclo",
        f"ciclo: {escalar(ciclo_id)}",
        f"status: {etapa}",
        f"date: {escalar(fm.get('criado', ''))}",
        f"prioridade: {escalar(fm.get('prioridade', 'media'))}",
        f"depende_de: {depende}",
        "tags:",
        "- ciclo",
        "---",
        "",
        f"# Ciclo {ciclo_id} — {titulo}",
        "",
        '{{ type: "fluxo" }}',
        "artefato: execucao",
        f"etapa: {etapa}",
        "{{ /fluxo }}",
        "",
        corpo.strip(),
        "",
    ]

    if arquivo_status is not None:
        _, corpo_status = ler_frontmatter(arquivo_status.read_text(encoding="utf-8"))
        linhas += ["## Resultado", "", corpo_status.strip(), ""]

    return "\n".join(linhas)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--vault", default="VaultAnotadinho")
    args = ap.parse_args()

    vault = RAIZ / args.vault
    destino = vault / "pages" / "ciclos"
    destino.mkdir(parents=True, exist_ok=True)

    tasks = sorted((RAIZ / "cycles" / "tasks").glob("*.md"))
    if not tasks:
        print("nenhuma task encontrada", file=sys.stderr)
        return 1
    status = status_por_ciclo(RAIZ / "cycles" / "status")

    escritas = 0
    for t in tasks:
        fm, corpo = ler_frontmatter(t.read_text(encoding="utf-8"))
        if not fm.get("id"):
            continue
        pagina = montar_pagina(fm, corpo, status.get(fm["id"]))
        (destino / t.name).write_text(pagina, encoding="utf-8")
        escritas += 1

    print(f"{escritas} ciclos em {destino.relative_to(RAIZ)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
