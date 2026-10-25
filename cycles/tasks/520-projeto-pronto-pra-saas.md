---
id: "520"
titulo: "Projeto pronto pra SaaS: modelo de projeto com Ambiente e combinados, o índice do git intacto no commit recusado, o guia e o piloto de ponta a ponta"
status: done
criado: 2026-10-25
autor: agente
prioridade: alta
depende_de: ["517", "518", "519"]
estima_min: 120
---

# 520 — Fechar a série do repositório

O que falta pra o fluxo de 517–519 servir pra construir um SaaS de verdade.

## Critérios de aceite

- [x] O `projeto.md` de um projeto novo vem com a seção **Ambiente**
      (como rodar, como testar, as variáveis pelo nome) e com os
      **Combinados** mínimos de um SaaS feito com agente: testes antes de
      "concluída", segredo fora do git, commits pequenos, dependência e
      estrutura nova por proposta, o Fora de escopo respeitado.
- [x] Commit recusado (hook) devolve o índice do git como estava: o
      `add -A` não pode deixar o índice da pessoa mudado com a tarefa ainda
      em revisão (achado no 519).
- [x] O agente de teste ganha `--tarefa-saas`: muda um arquivo, commita
      uma parte e cria outro, como um agente de verdade num SaaS.
- [x] Piloto de ponta a ponta com os binários de verdade (TUI num pty, o
      agente de teste, um repositório git): Executar → o agente roda no
      repositório → a verificação passa → Revisão → Mudanças mostra o
      commit e o arquivo novo → aceitar commita.
- [x] O guia `pages/produto/como-usar-projetos.md` (por proposta) explica
      as regras do repositório, a revisão pelas Mudanças, o pedido de
      ajuste pelo comentário e um roteiro "Construir um SaaS do zero".

- [x] O `projeto montar` (plano) gera o mesmo Ambiente e, sem
      `combinados` no plano, os combinados padrão; o plano aceita
      `ambiente:`.
- [x] A TUI diz quando a thread de fechamento termina ("tarefa 001-… →
      Revisão", ou o erro) e relê o quadro na hora (achado no piloto).

## Comandos de validação

```bash
cargo test --workspace
cargo test -p anotadinho-ipc --lib repositorio
python3 scripts/tui/piloto-projeto.py
python3 scripts/tui/piloto-projeto.py --falha
```

## Não-objetivos

- Rodar um agente de verdade (Claude, um modelo local) no piloto: custa e
  não é determinístico; o contrato é o mesmo que o agente de teste exerce.
