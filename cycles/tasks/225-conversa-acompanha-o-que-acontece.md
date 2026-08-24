---
id: "225"
titulo: "A conversa acompanha o que está acontecendo"
status: done
criado: 2026-08-24
autor: humano
prioridade: media
depende_de: ["224"]
estima_min: 60
agente_alvo: claude-opus-5
---

# A conversa acompanha o que está acontecendo

## Objetivo

Acompanhar uma execução longa exigia ficar arrastando a barra: o
progresso crescia embaixo, fora da vista. E abrir uma conversa comprida
mostrava o começo dela, não o que aconteceu por último.

## Critérios de aceite

- [x] Abrir uma conversa mostra o fim, não o começo
- [x] Mensagem nova e progresso do agente arrastam a vista junto
- [x] Quem subiu pra reler NÃO é puxado de volta
- [x] A caixa de progresso, que tem rolagem própria, também acompanha
- [x] Cenários de harness pros dois lados

## Comandos de validação

```bash
cargo test --workspace
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
node scripts/uitest/run.mjs
```

## Notas

### A dependência é o conteúdo, nunca a rolagem

O efeito reage à quantidade de mensagens e ao tamanho do progresso. Um
efeito que reage à própria rolagem e escreve rolagem de volta vira laço,
e laço de rolagem trava a janela — foi a hipótese que investiguei no
ciclo 222 e, mesmo não sendo a causa daquele travamento, o padrão é
real e está registrado em [[Editor e DOM]].

### Quem subiu fica onde está

`colado_no_fim` guarda se a pessoa está acompanhando o fim, com uma
folga de 40px pra rolagem por linha e arredondamento de subpixel não
desligarem o acompanhamento sozinhos. Puxar alguém de volta no meio da
leitura é pior do que não rolar.

Mora num `RefCell`, não em `use_state`: quem lê é o efeito, e handle de
`use_state` capturado em closure fica congelado — ver
[[Estado capturado em closure]].

### Um cenário que falhava por relógio, não por código

`tela: histórico da página abre o painel (200)` passou a falhar com
"Script execution timeout", inclusive isolado. Conferido à mão: o painel
abre e funciona, com backend e renderizador em 0%.

A causa era a sondagem do harness: `esperar` chamava `bridge.js` em
laço, e um eval que caía exatamente na troca do documento estourava o
limite do plugin e ABORTAVA o cenário. Com o vault maior, a recarga
passou a demorar o bastante pra isso acontecer.

Agora um eval que falha durante a recarga conta como "ainda não", e o
prazo continua valendo — se a condição nunca chegar, o erro sai igual,
dizendo qual foi o último problema.
