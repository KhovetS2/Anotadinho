---
id: "519"
titulo: "TUI: o repositório no fluxo da tarefa — configurar a verificação, ver o resultado e revisar o diff"
status: done
criado: 2026-10-25
autor: agente
prioridade: alta
depende_de: ["517", "514", "503", "502"]
estima_min: 180
---

# 519 — A TUI revisa o código da tarefa

Pedido: a TUI como lugar completo pra construir um SaaS com agentes. O
517 pôs o repositório no fluxo da tarefa (verificar, base, branch, commit
ao aceitar), mas a TUI só deixava escolher a pasta: o comando de
verificação não se configurava, o resultado não aparecia, e o código
que a tarefa mudou não se via.

## Critérios de aceite

- [x] `o` no quadro abre um formulário com repositório, comando de
      verificação (vazio = nenhum), prazo em segundos, branch por tarefa e
      commitar ao aceitar. Salvar manda a `ConfigProjeto` inteira pelo
      `Pedido::GravarConfigDoProjeto`, com a pausa e a concorrência como
      estavam. Prazo que não é número (ou zero) é recusado com o formulário
      aberto; a pasta que não existe é recusada pelo `main` (com `~`
      expandido) e o formulário volta com o que foi escrito.
- [x] O cabeçalho do projeto diz "verificação `npm test` · prazo 15 min",
      ou "sem verificação — vale o que o agente disser" em cor de atenção,
      e "branch por tarefa" / "commit ao aceitar" quando ligados. Cabe em
      80 e 150 colunas (corte com `…`, pelo mesmo ajuste do 514).
- [x] Selo no cartão: ✓teste (sucesso) / ✗teste (erro). Também na faixa
      da página da tarefa.
- [x] `v` mostra o branch, "Ver mudanças no repositório" (com a base) e
      "Ver a última verificação" (passou/falhou/não rodou); esta abre a
      seção **Verificação** só pra ler, e Esc volta aos detalhes.
- [x] `D` no quadro, `Alt+m` na página da tarefa e "Tarefa: ver mudanças
      no repositório" na barra de comandos abrem o diff em tela inteira:
      base curta, branch, aviso, a lista de arquivos com o estado e o diff
      com cor (+ verde, − vermelho, `@@` em destaque, arquivo em negrito).
      `j k`, `Ctrl+D/U`, `g G`, `n N` entre arquivos; Esc fecha. O IO é um
      `Pedido` do `main`, e o erro do `ipc` vai pro aviso.
- [x] Mover pra feita quando o commit falha mostra o erro no aviso, numa
      linha, e a tarefa fica onde estava.
- [x] Testes com `TestBackend` pra cada critério; um quebrado de propósito
      reprovou.
- [x] Conferido num terminal de verdade (`scripts/tui/tela.py`) em 80 e
      150 colunas.

## Comandos de validação

```bash
cargo test -p anotadinho-tui
cargo test --workspace
cargo check -p anotadinho-tui --target x86_64-pc-windows-gnu
python3 scripts/tui/tela.py --abrir meu-primeiro-projeto/projeto --preparar x.sh --teclas "l D n"
```

## Não-objetivos

- A janela: é o ciclo 518.
- Rolagem horizontal do diff: a linha longa corta com `…`, como o código.
- Merge do branch da tarefa: fica com a pessoa (como no 517).
