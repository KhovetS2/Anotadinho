---
id: "499"
titulo: "TUI: pizza, mapa mental, linha do tempo, jornada, classes e ER desenhados"
status: done
criado: 2026-10-24
autor: agente
prioridade: media
depende_de: ["498"]
estima_min: 240
---

# 499 — Os outros tipos de diagrama no terminal

A janela desenha QUALQUER mermaid (mermaid.js); a TUI só desenhava os
quatro tipos que o construtor monta (496–497), e o resto saía como código.
Pedido da pessoa: "a criação das outras partes, pra ficar coerente com a
versão GUI". Este ciclo cobre os tipos mais usados que o construtor não
monta.

## Critérios de aceite

- [x] Leitor TOLERANTE e só pra ver (`diagrama::so_ver`): nada volta pro
      arquivo, então linha que não se entende (`style`, `click`, ícone)
      fica fora do desenho em vez de recusar o diagrama.
- [x] `pie`: barras proporcionais à maior fatia, com a porcentagem do
      total e o valor no `showData`; uma cor por fatia.
- [x] `mindmap`: árvore pela indentação (`├─`/`└─`), sem id, forma nem
      `:::classe`; `::icon` fica de fora.
- [x] `timeline`: seções, período com os eventos, continuação `: evento`.
- [x] `journey`: seções e tarefas com a nota em estrelas, na cor do humor,
      e quem participa.
- [x] `classDiagram`: caixa com nome, anotação (`«abstract»`) e membros
      (dos dois jeitos: bloco `{ }` e `Classe : membro`); relações com
      herança, realização, composição, agregação, associação e
      dependência, rótulo e cardinalidade; o pai da herança e o todo da
      composição em cima.
- [x] `erDiagram`: entidade com atributos (`nome: tipo PK`), relação com o
      rótulo e as cardinalidades por extenso (`1–0..N`), pontilhada
      quando não identifica.
- [x] O grafo do 497 ganha caixa de várias linhas (nome, separador,
      corpo); em TD as caixas da camada alinham pelo topo.
- [x] Pra VER, todo mermaid perde comentário `%%`, diretiva `%%{…}%%` e
      cabeçalho `---` (o título dele vai pro topo). Antes, um comentário
      num fluxograma fazia o bloco sair como código.
- [x] Tipo sem desenho (gitGraph, quadrant…) diz qual é: "o tipo
      `gitGraph` ainda não tem desenho no terminal".
- [x] A vista de dentro do bloco (498) vale pros tipos novos.

## Comandos de validação

```bash
cargo test -p anotadinho-core --lib diagrama
cargo test -p anotadinho-tui
cargo test --workspace
cd ui && trunk build
python3 scripts/tui/tela.py tipos.md --largura 120 --altura 100
```

## Não-objetivos

- gitGraph, quadrantChart, xychart, sankey, C4, requirement, block,
  packet, architecture: continuam como código, com o motivo.
- Ponta de seta no PAI da herança (o triângulo da UML): o desenho só tem
  ponta no destino, e o pai em cima é o destino ao contrário. O rótulo
  "herda" diz o que a ponta diria.
