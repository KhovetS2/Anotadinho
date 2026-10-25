---
title: "Como usar projetos"
type: docs
tags:
- agent-os
- guia
- projeto
---
# Como usar projetos

Um **projeto** junta num lugar só o que antes ficava espalhado: a conversa
com o agente, a arquitetura, as tarefas detalhadas e a fila que executa
cada uma no repositório do código. Funciona igual na janela e na TUI.

{{ type: "callout" }}
variant: info
title: O que continua valendo
body: |
  O agente não grava etapa nem página: tarefas novas e mudanças na
  arquitetura chegam como PROPOSTA, e você escolhe na revisão. Quem
  manda uma tarefa executar é você, mudando a etapa dela.
{{ /callout }}

## 1. Criar

Paleta → **Novo projeto** (na TUI, "Novo projeto…"). Nasce a pasta
`pages/projetos/<nome>/` com:

| Página | Para quê |
|---|---|
| `projeto.md` | Visão, objetivo, stack e os **combinados com o agente** (vale pra toda tarefa: commitar ou não, como testar, o que nunca mexer) |
| `arquitetura.md` | A arquitetura viva |
| `conversas/geral.md` | A conversa do projeto |
| `tarefas/` | Uma página por tarefa |

## 2. Apontar o repositório e as regras

No cabeçalho do projeto, **Escolher repositório** e **Configurar** (na
TUI, `o` abre tudo num formulário). O repositório é a pasta onde o agente
trabalha nas tarefas. A config fica em `.anotadinho/projetos.json`, e não
numa página: uma nota não escolhe onde um agente escreve, nem o comando
que confere o trabalho dele.

| Regra | O que faz |
|---|---|
| **Comando de verificação** (`npm test`, `cargo test`…) | Roda no repositório depois do agente. A tarefa só vai pra Revisão se ele passar; se falhar, vai pra Bloqueada, com a saída na seção **Verificação** da página — e a próxima rodada lê isso |
| Prazo da verificação | Padrão 15 min. Estourou: conta como falha |
| Branch por tarefa | Cada tarefa roda no branch dela (`tarefa/003-…`), criado de onde o repositório estiver |
| Commit ao aceitar | Revisão → Feita commita o que a tarefa deixou, como `tarefa 003: título`. Se o commit falha (um hook), a tarefa fica em Revisão com o erro, e o índice do git volta como estava |

Sem verificação, o cabeçalho avisa: **vale o que o agente disser**. Pra
um SaaS, configure: é a diferença entre "o agente disse que terminou" e
"os testes passaram".

Sem repositório, a fila não anda, e o cabeçalho diz isso.

## 3. Conversar e quebrar em tarefas

A aba **Conversa** já leva o projeto, a arquitetura e o índice das
tarefas. Use o prompt padrão **Quebrar em tarefas**: a resposta vem com
um `## Tarefa: <título>` por tarefa.

Em cada resposta do agente:

- **virar tarefas**: propõe uma tarefa por `## Tarefa:`, num lote. Na
  revisão, aplique as que quiser.
- **atualizar arquitetura**: propõe a resposta como seção nova de
  `arquitetura.md`.

## 4. As etapas de uma tarefa

| Etapa | Quem leva até ela |
|---|---|
| Ideia | você |
| Detalhando | você (o agente detalha por proposta) |
| Pronta | você: aprovou o detalhe |
| **Executar** | você: **entra na fila** |
| Executando | o app, ao começar |
| Revisão | o app, quando o agente diz que concluiu |
| Feita | você: depois da Revisão, ou direto de Pronta/Bloqueada quando foi resolvido sem execução |
| Bloqueada | o app (o agente parou e explicou) ou você |

Na janela, dá pra **arrastar** o cartão: ao pegar, só as colunas
permitidas acendem, e soltar em qualquer outra não faz nada. Executando
e Revisão nunca aceitam, porque são do app. Soltar em Executar põe na
fila, igual ao botão.

De Ideia não se vai direto pra Executar: detalhe primeiro.

Tarefa que é uma **decisão sua**, ou que você resolveu por fora, não
precisa de execução: de Pronta ou Bloqueada, leve direto pra **Feita**.
Mandar o agente executar uma decisão faz ele parar e devolver a escolha
pra você, gastando uma execução. Os botões (na
TUI, os números embaixo da tarefa) mostram só o que dá pra fazer dali.

**Detalhar com o agente** abre a conversa da tarefa com o pedido escrito,
sem enviar. É a mesma conversa onde as execuções dela ficam registradas.

## 4.1 Detalhes da tarefa

Como no card do kanban, a tarefa guarda tags, prazo, critérios,
comentários e anexos. Tudo fica na própria página, legível como
markdown:

| Detalhe | Onde fica na página |
|---|---|
| Tags | `tags:` no frontmatter |
| Prazo | `prazo: AAAA-MM-DD`. Vencido e sem estar feita, o cartão destaca |
| Critérios | os `- [ ]` de **Critérios de aceite**: são a checklist, e o cartão mostra 2/5 |
| Comentários | a seção **Comentários**, com data e hora |
| Anexos | a seção **Anexos**: os arquivos são copiados pra `assets/`. Um nome repetido vira `nome-2`, e nunca sobrescreve outro arquivo |

- **Na janela:** o `⋯` no cartão abre o painel de detalhes ao lado do
  quadro.
- **Na TUI:** `v` abre os detalhes. Enter marca o critério sem fechar a
  lista. O anexo é pelo caminho do arquivo, porque o terminal não tem
  diálogo.
- **Pelo CLI e pelo MCP:** a tarefa nova e o plano aceitam tags e prazo
  (`tarefa nova … --tag x --prazo 2026-11-05`).

## 5. Executar

Mova a tarefa pra **Executar**. Em segundos:

1. A tarefa vira **Executando** (é a reserva: ela não roda duas vezes).
2. O agente roda **no repositório do projeto**, com a tarefa, o projeto,
   a arquitetura e a proposta vinculada (se houver) anexados.
3. Ele termina com `RESULTADO: concluida` ou `RESULTADO: bloqueada`. A
   tarefa vai pra **Revisão** ou **Bloqueada**, e o relato fica na
   conversa da tarefa.

Resposta sem a linha de resultado vai pra **Bloqueada**, e não pra
Revisão: parecer pronto sem estar é o pior caso.

### Revisar o que mudou

Antes de começar, o app guarda o commit de onde a tarefa parte. Na
Revisão, **Mudanças** (o ícone de branch no cartão, ou nos detalhes; na
TUI, `D` no quadro e `Alt+m` na página da tarefa) mostra tudo o que ela
mudou desde ali: o que o agente commitou, o que deixou na árvore e os
arquivos novos. Pelo CLI: `anotadinho-cli tarefa mudancas <tarefa>`.

Da Revisão: **Feita**, ou **Executar** de novo. Pra pedir ajuste, escreva
um **comentário** na tarefa antes: a página vai anexada, e o agente é
avisado de que a tarefa já rodou e que os Comentários e a Verificação
dizem o que mudar. A segunda rodada continua da mesma base, então
Mudanças mostra a tarefa inteira, e não só a última rodada.

| Controle | Efeito |
|---|---|
| Pausar fila (`p` na TUI) | nada sai de Executar; os pedidos esperam |
| `depende:` no frontmatter | a tarefa espera as dependências estarem Feita |
| Bloquear uma tarefa em Executando | o agente dela é interrompido |
| Uma por vez | o padrão: tarefas no mesmo repositório disputam arquivos |

## 6. Sprints

Com o projeto crescendo, o quadro com todas as tarefas vira uma coluna
Feita de trezentos cartões. A sprint recorta: o quadro mostra **o que
está sendo atacado agora**, e o que já foi feito vai pro histórico.

- **Uma sprint ativa por vez.** Iniciar outra com uma já ativa é
  recusado, dizendo qual está ativa: encerre ela antes.
- **O quadro abre na sprint ativa.** O seletor do cabeçalho (`s` na TUI)
  troca pra uma planejada, pro **backlog** (tarefa sem sprint) ou pra
  **todas**. As sprints encerradas não aparecem no seletor.
- **Pôr uma tarefa numa sprint:** o seletor no cartão (`m` na TUI). Só
  a ativa e as planejadas recebem tarefa.
- **Encerrar pergunta pra onde vão as pendentes:** pra próxima sprint
  planejada (a de número menor) ou pro backlog. A opção da próxima fica
  desligada se não houver sprint planejada.
- **Histórico:** as tarefas feitas de uma sprint encerrada saem do
  quadro, inclusive de "todas". Na aba **Sprints** (`S` na TUI) ficam as
  datas, o objetivo, quantas foram feitas e quais.
- **Projeto sem sprint** funciona como antes: o quadro mostra tudo.

As dependências continuam olhando o projeto inteiro: uma tarefa da
sprint que depende de uma do backlog espera do mesmo jeito.

As sprints moram em `projetos/<nome>/sprints/`, uma página por sprint
(`type: sprint`, com `estado: planejada|ativa|encerrada`). A tarefa
guarda o número da sprint no campo `sprint:` do frontmatter.

## 7. Montar a base do projeto com a LLM

Peça no Claude Code ou no opencode: "monte a base do projeto X". O
agente tem dois caminhos, e os dois **propõem**:

- **MCP:** `ver_projeto`, `propor_plano_de_projeto`, `propor_sprint` e
  `propor_tarefa`.
- **CLI:** `projeto modelo-de-plano` imprime o formato, e `projeto
  montar --file plano.yaml` propõe tudo num lote só: projeto,
  arquitetura, sprints e tarefas.

Você revisa o lote na Revisão como qualquer proposta. Num projeto que já
existe, o plano só ACRESCENTA sprints e tarefas, numeradas depois das
que já existem. O prompt padrão "Montar a base do projeto" já traz o
pedido escrito.

## Teclas da TUI

`h`/`j`/`k`/`l` anda pelo quadro · `1`–`3` move · `Enter` abre · `v`
detalhes · `n` nova · `d` detalha · `s` filtro · `m` sprint da tarefa ·
`S` sprints · `c` conversa · `a` arquitetura · `p` pausa · `o`
repositório e regras · `D` mudanças da tarefa. Todas estão em `?`, no
grupo Projeto. Na página de uma tarefa, `Alt+m` mostra as mudanças, e
"Mover a tarefa aberta…" fica na barra de comandos.

## Pelo CLI

```
anotadinho-cli projeto listar
anotadinho-cli projeto tarefas <nome>
anotadinho-cli tarefa nova <nome> "Título"        # propõe
anotadinho-cli tarefa etapa <caminho> --para pronta   # propõe
anotadinho-cli tarefa mudancas <caminho> [--resumo]   # o diff desde a base
anotadinho-cli projeto ver <nome>                 # sprints e tarefas numa chamada
anotadinho-cli projeto modelo-de-plano            # o formato do plano
anotadinho-cli projeto montar --file plano.yaml   # propõe o projeto inteiro
anotadinho-cli sprint listar <nome>
anotadinho-cli sprint nova <nome> "Sprint 1" --objetivo "…" --fim 2026-11-05   # propõe
anotadinho-cli sprint iniciar <nome> <n|nome>     # propõe
anotadinho-cli sprint encerrar <nome> <n|nome> --pendentes proxima|backlog     # propõe
anotadinho-cli tarefa sprint <caminho> <n|nome|backlog>   # propõe
```

O que escreve pelo CLI **propõe**: o CLI é o canal do agente.

## Quando algo dá errado

| Sintoma | Causa provável |
|---|---|
| A tarefa fica em Executar | o freio do dia (por padrão, silêncio das 23h às 7h, nas guardas do agente), fila pausada, sem repositório, dependência pendente ou outra tarefa do projeto rodando — o cabeçalho diz qual |
| A tarefa fica em Executando | o app fechou no meio: mova pra Bloqueada e mande de novo |
| Foi pra Bloqueada sem motivo aparente | a resposta não terminou com `RESULTADO:` — veja a conversa da tarefa |
| Foi pra Bloqueada com ✗teste | a verificação reprovou: a saída está na seção **Verificação** da tarefa, e a próxima rodada a lê |
| Revisão → Feita não andou, com "o commit falhou" | um hook do repositório recusou o commit: a tarefa fica em Revisão e o índice do git volta como estava; resolva e aceite de novo |
| O quadro está vazio, mas há tarefas | o recorte: a sprint ativa não tem nenhuma. Troque o seletor pra **todas** ou **backlog** |
| "já há uma sprint ativa" ao iniciar | só uma por vez: encerre a ativa antes |
| Aviso "aponta pra sprint N, que não existe" | a página da sprint foi apagada: a tarefa conta como backlog até você escolher outra |

## Relacionado

- [[Como usar o modo agêntico]] — conversa, spec, proposta e execução

## Construir um SaaS do zero

1. **Novo projeto**; em `projeto.md`, escreva a visão, a stack e a seção
   **Ambiente** (como rodar, como testar, as variáveis pelo NOME). Os
   **Combinados** já vêm com o mínimo — testes antes de "concluída",
   segredo fora do git, arquitetura por proposta —; ajuste.
2. Crie a pasta do código com `git init` e um primeiro commit; aponte o
   repositório e configure a **verificação** (mesmo que comece com um
   teste só).
3. Na Conversa, peça pra **quebrar o MVP em tarefas** e aplique as
   propostas. Detalhe as primeiras com o agente até os critérios serem
   verificáveis.
4. Mova UMA pra Executar. Na Revisão, leia **Mudanças**, rode o app se
   precisar, e aceite (com o commit ao aceitar, o histórico fica limpo)
   ou comente e mande de novo.
5. Repita. Erro que volta (uma validação esquecida, um teste que não
   cobre o critério) vale um combinado novo em `projeto.md`: ele vai em
   toda execução.
