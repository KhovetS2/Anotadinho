---
id: "253"
titulo: "Tema configurável"
status: done
criado: 2026-09-03
autor: humano
prioridade: baixa
depende_de: []
estima_min: 150
agente_alvo: claude-opus
---

# Tema configurável

## Objetivo

Fecha a spec [[Tema configurável]]. Até aqui, trocar qualquer coisa além
de claro/escuro exigia editar CSS e recompilar.

A spec diz que "o sistema de tokens e a disciplina de BEM já são a metade
difícil de um tema configurável; falta expor". Isso se confirmou: cada
tema novo são quatro dezenas de linhas redefinindo tokens, sem uma única
regra por componente.

## Desenho

Três escolhas independentes, porque são independentes na cabeça de quem
escolhe:

| escolha | mecanismo | RF |
|---|---|---|
| tema | `data-theme` no `<html>` | RF1, RF2 |
| cor de destaque | `--destaque` inline no `<html>` | RF3 |
| forma dos botões | `data-botoes` no `<html>` | RF4 |

Quatro temas: Escuro (padrão), Claro, Papel e Alto contraste. Cada um
mostra uma prévia de três cores na tela, que é o que permite escolher
**sem** aplicar (RF2).

`--destaque` é um gancho, não um token novo espalhado: `--accent-blue`
passou a ser `var(--destaque, #00B5FF)`. O nome do token é histórico — é
a cor de destaque do app e desde agora pode não ser azul —, mas renomear
os ~400 usos espalhados pelo CSS seria um refactor maior que o ciclo, e
com mais risco do que valor.

## Critérios de aceite

- [x] A tela lista os temas com prévia e aplica no clique
- [x] A cor de destaque muda botões, foco e seleção de forma consistente
- [x] Fechar e reabrir mantém a escolha
- [x] "Voltar ao padrão" restaura tudo
- [x] O snapshot visual dos embeds passa em pelo menos dois temas
- [x] Cenários de harness pra aplicar tema e pra persistência
- [x] RNF1: só temas com contraste de leitura são oferecidos
- [x] RNF2: nada entra no vault — cenário conferindo o diretório
- [x] RNF3: aplicar não recarrega a janela
- [x] RNF4: `data-theme` continua sendo o mecanismo

## O que fica fora, como a spec pediu

Tema escrito pelo usuário (CSS próprio), tema por página ou por vault, e
troca de fonte. O primeiro é o que sustenta o RNF1: contraste só pode ser
garantido sobre uma lista fechada.

## Comandos de validação

```bash
cargo test --workspace
cd ui && PATH="$HOME/.cargo/bin:$PATH" cargo check --target wasm32-unknown-unknown
node scripts/uitest/run.mjs
```
