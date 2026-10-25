---
title: Testar no Windows
type: docs
tags: [produto, portabilidade]
---
# Testar no Windows

Roteiro pra conferir à mão, numa máquina Windows, o que os ciclos 255 e
448–450 só puderam verificar compilando e com testes no Linux. Marque
cada item ao conferir. Quando um falhar, anote a mensagem **inteira** e
o que estava na tela. O conserto começa medindo o que chegou de fato
(ver [[fronteira-do-sistema]]).

{{ type: "callout" }}
variant: info
title: O que precisa estar na máquina
body: |
  Rust pelo rustup, `cargo install tauri-cli trunk`, NSIS ou WiX pro
  instalador, e o runtime do WebView2 (já vem no Windows 11). Agentes:
  `claude` e, se quiser o passo 6, `codex`, instalados pelo npm.
{{ /callout }}

## Instalador e agente na janela

- [ ] **1.** `powershell -ExecutionPolicy Bypass -File scripts\build.ps1`
  termina sem erro. Depois de instalar, `anotadinho-cli.exe` está na
  pasta do app. *(ciclo 450 — o `build.ps1` nunca rodou)*
- [ ] **2.** Uma conversa com o Claude Code responde. *(449 — antes,
  nenhuma conversa rodava: o `.cmd` do npm recusa prompt com quebra de
  linha)*
- [ ] **3.** Pedir pro agente buscar uma página do vault funciona, e o
  progresso **não** mostra "sem as ferramentas do vault". *(450)*
- [ ] **4.** Um prompt longo, com duas ou três páginas transcluídas,
  chega inteiro: a resposta comenta o fim do contexto, não só o
  começo. *(449)*
- [ ] **5.** Interromper uma execução no meio: no Gerenciador de
  Tarefas, nenhum `node.exe` sobra. *(255, `taskkill /T`)*
- [ ] **6.** Trocar o agente pra Codex e repetir o passo 2. *(449)*

## TUI, no Windows Terminal

- [ ] **7.** Configurar o agente, fechar e abrir: a configuração
  continua lá, e o arquivo é `%APPDATA%\anotadinho\tui.json`. *(448)*
- [ ] **8.** Digitar `~\projetos` como pasta do agente vira
  `C:\Users\<você>\projetos`. *(448)*
- [ ] **9.** O carimbo de uma mensagem nova está na hora local, não em
  UTC (3 h de diferença no Brasil). *(448)*
- [ ] **10.** Abrir um anexo `.png` (`Enter` no bloco com a imagem e, se
  houver mais de um link, escolher o da imagem): o Explorer abre a imagem, e
  não a pasta Documentos. *(448 — não foi medido se o Explorer recusa
  `/`)*
- [ ] **11.** Interromper o agente pela TUI: nenhum `node.exe` sobra.
  *(448)*

## Sabidamente fora

- **opencode** continua recebendo o prompt como argumento, porque não
  foi medido se ele lê o stdin. Se estiver instalado pelo npm
  (`opencode.cmd`), espere a mensagem que manda trocar a entrega do
  prompt pra stdin. Se for o `.exe` próprio, deve funcionar.
- **Harness de UI** não roda no Windows: o agente falso é `bash`.
- **`cmd.exe` antigo**, sem VT, não é alvo da TUI.

## Onde registrar

Um item que falhar vira uma conversa ou uma spec nova, citando o número
do passo. Quando todos passarem, a callout de aviso de
[[rodar-no-windows]] pode sair.
