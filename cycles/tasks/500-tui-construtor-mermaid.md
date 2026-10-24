---
id: "500"
titulo: "TUI: construtor de diagramas mermaid"
status: done
criado: 2026-10-24
autor: agente
prioridade: media
depende_de: ["490", "498"]
estima_min: 360
---

# 500 — TUI: construtor de diagramas mermaid

Paridade com o construtor da janela (ciclos 487–490). A spec
`construtor-mermaid` deixou "o construtor na TUI" fora de escopo porque a
TUI não desenhava diagrama; desde 496–499 ela desenha, e o pedido veio
direto da pessoa. O modelo é o do núcleo (`diagrama::ler`/`escrever`), o
desenho da prévia é o de `diagrama::texto`.

## Critérios de aceite

- [x] Num bloco ` ```mermaid `, `c` dentro da vista do bloco (Enter, 498)
      abre o construtor; a barra de comandos oferece "Construtor do
      diagrama…" só com o cursor num bloco mermaid.
- [x] O `/` "Diagrama" abre o construtor com o seletor de tipo
      (Fluxograma, Sequência, Estados, Gantt, de `Tipo::TODOS`), com o
      exemplo de cada um já na prévia, como o "Novo diagrama" da janela.
- [x] A fonte da verdade é o texto do código: mexer no modelo reescreve o
      texto (`diagrama::escrever`); a aba Código (Tab) edita o texto direto
      com o editor de várias linhas de 390.
- [x] Abrir e aplicar sem mudar devolve o arquivo byte a byte (RF8). Uma
      mudança que não muda o modelo não reescreve o texto, e voltar ao
      modelo de quando abriu volta o texto de quando abriu.
- [x] Fora do modelo (`ForaDoModelo`): abre na aba Código com o motivo, e a
      aba Construtor recusa dizendo por quê.
- [x] Aba Construtor: listas por tipo (fluxograma: direção, nós, ligações;
      sequência: participantes, mensagens e notas; estados: estados e
      transições com início e fim; gantt: título, seções e tarefas).
      `j k g G` andam, `o` cria logo depois (`n`, nota), `dd`/`x` apagam,
      `J`/`K` movem, Enter edita no formulário (`componentes::Formulario`).
- [x] Apagar um nó leva as ligações dele (e um estado, as transições);
      participante citado, tarefa com dependente e seção com tarefa não se
      apagam, e o aviso diz por quê.
- [x] Validação como a janela: id válido e sem repetir (trocar o id leva
      as ligações/mensagens/transições junto), duração por
      `duracao_valida`, datas pelo seletor de data (364), início→fim
      recusado, id com "logo depois da anterior" recusado.
- [x] Prévia ao vivo à direita (o mesmo conteúdo do bloco na página, por
      `codigo::previa_mermaid`), rolável com as teclas da vista (`p` entra)
      e com `↕`/`↔` no título.
- [x] `Ctrl+S` aplica no bloco (caminho de `gravar_codigo`, 390) ou insere
      o bloco novo; `Esc` sai e só pergunta se mudou.
- [x] Atalhos (`?`) e `docs/paridade-gui-tui.md` atualizados.
- [x] Testes com `TestBackend` pra cada critério, dois quebrados de
      propósito, e a tela real num pty (`scripts/tui/tela.py`).

## Comandos de validação

```bash
cargo test -p anotadinho-tui --lib construtor
cargo test -p anotadinho-tui
cargo test --workspace
python3 scripts/tui/tela.py diagrama.md --largura 110 --altura 30 --teclas "j Enter c"
```

## Não-objetivos

- Canvas com posição dos nós: a posição não muda o desenho (RNF2) e o
  terminal não tem onde arrastar. As `%% anotadinho-posicoes` que a janela
  grava são preservadas (o modelo as carrega), não editadas.
- Mudar o tipo de um diagrama que já existe (a janela também só oferece o
  seletor no novo).
- Escolher várias dependências numa tarefa do gantt (`after a1 a2`): o
  formulário escolhe uma, como a janela. As que o código já tem ficam
  enquanto a primeira não muda.
