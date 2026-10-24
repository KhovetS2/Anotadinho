---
id: "497"
titulo: "TUI: fluxograma e estados desenhados; as camadas descem pro núcleo"
status: done
criado: 2026-10-24
autor: agente
prioridade: media
depende_de: ["496"]
estima_min: 300
---

# 497 — Fluxograma e estados no terminal

Quarta e última etapa da proposta `pages/propostas/tui-codigo-e-diagramas.md`
(RF6 pra fluxograma e estados, RF8).

## Critérios de aceite

- [x] A arrumação em camadas desce pro núcleo (`diagrama::camadas`:
      `niveis` e a ordem por baricentro); o canvas do construtor passa a
      usá-la de lá.
- [x] Fluxograma e estados desenhados: caixas por forma (caixa, arredondado,
      decisão `< >`, círculo `( )`), início `●` e fim `◉`, ligações com
      ponta e rótulo, pontilhada `╌`/`╎` e grossa `━`/`┃`.
- [x] Cada ligação que anda de lado tem a sua pista no canal entre as
      camadas; volta, salto e mesma camada correm por uma pista lateral
      por fora de todas as caixas. Cruzamentos viram junções (`┼`, `├`).
- [x] Nós sozinhos na camada ficam na mesma coluna (sem degrau na seta).
- [x] TD e LR desenhados; BT e RL espelhados, com o texto posto depois.
- [x] O que não cabe é recusado (a TUI mostra o código com o motivo).
- [x] O diagrama usa a altura do painel como teto, não as 30 linhas do
      código.
- [x] Harness da janela: os cenários do canvas (490, 491) passam com as
      camadas vindas do núcleo.

## Comandos de validação

```bash
cargo test -p anotadinho-core --lib diagrama
cargo test -p anotadinho-tui
cargo test --workspace
cd ui && cargo test --lib && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
node scripts/uitest/run.mjs "(490)"
node scripts/uitest/run.mjs "(491)"
python3 scripts/tui/tela.py pages/exemplos/diagrama-mermaid.md --largura 120 --altura 60
```

## Não-objetivos

- Ligação dentro da mesma camada em LR fica legível, mas não bonita: sai
  pela pista lateral de baixo e entra pelo canal de antes do destino,
  dividindo a ponta com quem já chega ali.
