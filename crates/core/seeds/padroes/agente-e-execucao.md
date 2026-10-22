---
title: Agente e execução
date: 2026-08-24
dominio: agente
tags:
- padrao
---
# Agente e execução

## Quando se aplica

Qualquer mudança no caminho que executa um agente externo: adaptador,
prompt, acompanhamento, ou o que ele pode alcançar.

## As regras

1. **Sem shell.** `Command::new(binario).args(...)`, e o prompt vai
   pelo stdin ou como UM argumento. Aspas, quebras e `$(...)` no prompt
   são texto.
2. **O prompt prefere o stdin.** Pela linha de comando ele bate em dois
   limites: no Windows, a std recusa quebra de linha em argumento pra
   `.cmd`/`.bat` (os shims do npm), e no Linux um argumento não passa de
   128 KiB. Agente novo só vai pelo argumento se não souber ler o
   stdin.
3. **A configuração vem das preferências**, nunca do conteúdo de uma
   página. Uma nota que chegue de terceiro não escolhe o que roda.
4. **Conteúdo do vault entra no prompt BLINDADO**, entre marcadores e
   precedido do aviso de que é dado, não instrução.
5. **A execução não bloqueia a tela.** O processo é do backend; a tela
   pergunta como está. Sair da página não mata nem perde a resposta.
6. **Quem grava a resposta é o backend**, porque a tela pode não estar
   lá.
7. **Uma execução por conversa.** É o que garante que nunca há dois
   escritores no mesmo arquivo.
8. **O motivo da falha vem do STREAM**, não do stderr — o stderr quase
   sempre é ruído de inicialização.
9. **A pasta de trabalho é escolha da pessoa.** É a escolha dela que
   autoriza a escrita ali.
10. **Timeout generoso (30 min) com botão de interromper.** Limite curto
   mata trabalho legítimo no meio.
11. **A fila de tarefas é por nível, com reserva antes de iniciar.** O
   que está em `executar` É a fila. A tarefa vira `executando` ANTES do
   processo subir, e só se reserva o que cabe iniciar agora: tarefa
   reservada sem processo segura o projeto inteiro. O app só grava etapa
   que registra o que uma decisão humana causou (`executando`, `revisao`,
   `bloqueada`). O agente não grava etapa nenhuma.
12. **O repositório de um projeto é config, não página.** Ele fica em
   `.anotadinho/projetos.json` e entra pelo `cwd` do adaptador, como
   parâmetro. Uma nota que decidisse onde um agente com `acceptEdits`
   escreve seria uma porta aberta.

## Por que existe

- **202** — o desenho sem shell é o que fecha injeção pelo prompt.
- **213** — timeout de 3 min matava o planejamento no meio; a
  requisição vivia no componente e sumia ao trocar de página.
- **214** — `claude -p` segura a saída até o fim; sem `stream-json` não
  há sinal de vida.
- **216** — rodar no vault deixava o agente sem ver o código E com
  escrita nas notas: o pior dos dois mundos.
- **449** — no Windows nenhuma conversa rodava: todo prompt tem quebra
  de linha, e o shim `.cmd` do npm não a aceita como argumento.
- **219** — a conta bateu o limite e a tela mostrou "Reading additional
  input from stdin...".
- **457** — a fila por borda ("o que já vi") perdia o pedido feito com a
  fila pausada. Por nível com reserva, o pedido espera e roda ao
  retomar.
- **459** — reservar sem vaga deixaria a tarefa em `executando` sem
  agente nenhum. Por isso o frontend diz quantas vagas tem.
- **462** — conferir as dependências comparando cada uma com todas as
  tarefas era `O(n²)`: 76 → 143 µs por tarefa de 250 pra 2000.
