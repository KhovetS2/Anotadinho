---
id: "506"
titulo: "TUI: editor de código de verdade; editar diagrama abre formulário e código juntos"
status: done
criado: 2026-10-24
autor: agente
prioridade: alta
depende_de: ["500", "505"]
estima_min: 180
---

# 506 — Um editor de código e um que facilita preencher

Pedido: "faça um editar tanto de código como um que facilita para o
usuário preencher".

O de código era o campo de várias linhas do ciclo 390:
- sem cor;
- sem indentação, e o Tab virava quatro espaços onde estivesse;
- linha larga saía cortada, com o cursor fora da tela;
- sem desfazer.

O de preencher já existia: é o construtor do 500. Mas só se chegava a ele
pelo `c` dentro do bloco, e o `i` num diagrama abria o código cru.

## Critérios de aceite

- [x] Editor de código com:
  - [x] número de linha e cor da linguagem, a mesma do bloco na página;
  - [x] a linha do cursor marcada;
  - [x] Enter no mesmo recuo, um nível a mais depois de `{ ( [ :`, e
        `{|}` mais Enter desce o `}`; `}` numa linha só de espaços volta
        um nível;
  - [x] Tab até o próximo múltiplo de 4, Shift+Tab desindenta, e
        Backspace no recuo apaga um nível;
  - [x] Home esperto, Ctrl+Home/End, PgUp/PgDn;
  - [x] Ctrl+Z/Y, com letras seguidas valendo um passo, e Ctrl+K apaga
        a linha;
  - [x] a vista segue o cursor nas duas direções;
  - [x] o topo diz linha, coluna e linguagem.
- [x] Ctrl+L escolhe a linguagem numa lista, e a cerca muda ao gravar.
      Sem trocar, a cerca fica como estava, com os atributos.
- [x] Bloco novo pelo `/` → Código abre o editor com a lista de
      linguagens. Antes, a palavra "código" de mentira entrava na página.
- [x] Editar um diagrama (`i`/`A`, ou Enter dentro do bloco) abre o
      construtor: o formulário de preencher e a aba Código. Um bloco que
      vira mermaid pelo seletor também vai pro construtor.
- [x] A aba Código do construtor usa o mesmo editor.
- [x] Shift+Tab e Ctrl+Home/End chegam do terminal (`main.rs`).

## Comandos de validação

```bash
cargo test -p anotadinho-tui
cargo check -p anotadinho-tui --target x86_64-pc-windows-gnu
python3 scripts/tui/tela.py editor-novo.md --teclas 'j o / … Enter r Enter …'
```

## Não-objetivos

- Fechar par de chaves e aspas sozinho: atrapalha mais do que ajuda
  quando se cola código.
- Seleção com Shift e copiar/colar dentro do editor.
- Busca dentro do editor.
