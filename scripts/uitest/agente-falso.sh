#!/usr/bin/env bash
# Agente de MENTIRA pros cenários do ciclo 202.
#
# O que se testa com ele é o CONTRATO da execução — prompt chega inteiro,
# saída volta, timeout mata, falha é reportada — e não a qualidade da
# resposta de um modelo. Usar claude/codex de verdade tornaria a suíte
# lenta, cara e não determinística.
#
# Modos: --responder (padrão), --demorar, --falhar, --mudo.
case "$1" in
  --demorar) sleep 60 ;;
  --falhar) echo "erro proposital" >&2; exit 3 ;;
  --mudo) exit 0 ;;
esac
printf 'RESPOSTA para: %s' "$2"
