---
title: Tarefa rica, vault novo completo, painel inicial e medida de memória — como
tags:
- proposta
- agent-os
- projeto
type: proposta
date: 2026-10-22
spec: '[[Tarefa rica, vault novo completo, painel inicial e medida de memória]]'
status: aprovada
---
# Tarefa rica, vault novo completo, painel inicial e medida de memória — como

{{ type: "fluxo" }}
artefato: proposta
etapa: aprovada
{{ /fluxo }}

Atende `pages/specs/tarefa-rica-home-e-memoria.md`.

## Abordagem

### A. Onde cada detalhe mora (RNF2)

| Detalhe | Na página da tarefa |
|---|---|
| tags | `tags:` no frontmatter, como em qualquer página. Entra nas consultas e na página de tags sem nada novo. |
| prazo | `prazo: AAAA-MM-DD` no frontmatter |
| checklist | os `- [ ]` / `- [x]` da seção **Critérios de aceite**, que a tarefa já tem |
| comentários | seção **Comentários** no fim: `- **2026-10-22 14:30** — texto` |
| anexos | seção **Anexos**: links pros arquivos copiados pra `assets/`, pelo mesmo caminho que o editor usa ao arrastar um arquivo |

Os detalhes ficam onde a pessoa os leria no markdown. Não há um bloco
YAML escondido como o do card do kanban, porque a tarefa é uma página
inteira e o card é um item dentro de uma.

**Núcleo (`core::projeto`):**
- `Tarefa` ganha `tags`, `prazo`, `checklist: Vec<Item>`, `comentarios`
  e `anexos`, todos lidos da página.
- Funções puras que devolvem o texto novo, mexendo só no trecho
  certo:
  - `alternar_item(texto, i)`, que troca só o i-ésimo `[ ]` da seção;
  - `acrescentar_comentario(texto, quando, corpo)`;
  - `acrescentar_anexo(texto, nome, caminho)`;
  - `com_tags`;
  - `com_prazo`.
- O frontmatter sai do `serde_yaml`, como já sai hoje.
- `vencida(tarefa, hoje)`.

**ipc:** `handle_*` pra cada uma das funções acima, ditas pela pessoa,
gravando direto como `mudar_etapa_tarefa`. O anexo reaproveita a cópia
pra `assets/` que o kanban já usa.

**LLM (RF7):** `PlanoTarefa` e `propor_tarefa` ganham `tags` e `prazo`,
opcionais. O `MODELO_DE_PLANO` mostra os dois. O contrato do MCP é
atualizado, e o teste dele confere.

**Janela:**
- O cartão ganha a linha de selos (RF5).
- Um botão "detalhes" no cartão abre um painel lateral (RF6), com o
  mesmo comportamento do modal de detalhes do kanban:
  - tags e prazo;
  - a checklist clicável;
  - o campo de comentário;
  - "anexar arquivo" pelo diálogo nativo.
- O botão fica fora do arrasto, como os outros botões do cartão.

**TUI:**
- A linha vazia do cartão (que tem 4 linhas) vira a linha dos selos.
- `v` abre o modal de detalhes:
  - `j`/`k` e espaço marcam a checklist;
  - `c` comenta;
  - `t` edita as tags e `z` o prazo;
  - `a` anexa digitando o caminho do arquivo, porque a TUI não tem
    diálogo nativo.

### B. Vault novo (RF8–RF10)

- `seeds/guias/`: os dois guias, numa versão para o vault de uma pessoa
  qualquer, sem os caminhos deste repositório.
- `seeds/projetos/exemplo/`: um projeto pequeno, "Meu primeiro projeto":
  - a arquitetura;
  - duas sprints, uma ativa e uma planejada;
  - cinco tarefas espalhadas pelas etapas, com tags, prazo, checklist,
    um comentário e nenhum anexo (não há arquivo pra anexar na semente).
- O projeto de exemplo não tem repositório configurado: a fila nunca
  dispara sozinha num vault recém-criado.
- Um teste lê cada página da semente com `Tarefa::ler`, `Sprint::ler` e
  `embed check`. Se o formato mudar e a semente apodrecer, o teste
  reprova.
- O `init` já pula página que existe (RF10). Um teste confirma.

### C. Painel inicial (RF11–RF13)

- `seeds/inicio.md` vira o painel (`type: landing`), feito de embeds
  `query` que já existem e se atualizam sozinhos (RF12):
  - projetos (`type: projeto`);
  - sprints ativas (`type: sprint`, `estado: ativa`, colunas `projeto`
    e `fim`);
  - tarefas em `revisao`;
  - specs por `status`;
  - propostas por `status`;
  - as últimas execuções.
- O texto de hoje vai pra `seeds/guias/guia-do-vault.md`, com link no
  topo do painel (RF13).
- A fila de Revisão das propostas fica em `.anotadinho/`, fora das
  páginas, e a consulta não a vê. O painel tem um link pra Revisão em
  vez disso.
- Separado, pela ambiguidade apontada na spec: uma proposta com o
  mesmo painel pro `VaultAnotadinho`.

### D. Medida de memória (RF14–RF18)

`scripts/memoria/janela.mjs`:
- conecta pelo mesmo WebSocket do harness, na porta 9223 (dev) ou 9323
  (o app instalado, com `--porta 9323`);
- acha a árvore de processos: o app, o `WebKitWebProcess` e o
  `WebKitNetworkProcess`;
- amostra a memória de cada um a cada 500 ms, lendo o
  `/proc/<pid>/smaps_rollup`:
  - **PSS**: as bibliotecas compartilhadas divididas de forma justa, e
    o número a comparar;
  - **RSS**: o que o monitor do sistema mostra;
- roda o "dia" em rodadas (padrão 5). Cada rodada:
  1. abre 20 páginas;
  2. abre e fecha abas;
  3. digita 2000 caracteres numa página `uitest-memoria-*`;
  4. busca;
  5. abre a paleta;
  6. abre um projeto `uitest-memoria-*` com 300 tarefas e troca de
     filtro e de aba;
  7. abre uma conversa e o grafo;
  8. fecha tudo;
- remove as páginas temporárias num `finally` (RNF4);
- imprime a tabela (repouso, pico, fim, crescimento por rodada, por
  processo) e grava o JSON junto, com a build medida (RF18).

`scripts/memoria/tui.py`:
- só biblioteca padrão do Python;
- sobe a TUI num pty (`TIOCSWINSZ`), num vault temporário criado com
  `init` mais o projeto de 300 tarefas;
- manda as teclas do mesmo "dia" e amostra o PSS do processo;
- mesmo relatório.

O primeiro relatório medido vai pro status do ciclo e pra uma página
`pages/produto/memoria.md`, proposta.

## Etapas

1. **Núcleo e ipc:** campos, funções puras, handlers, plano e MCP.
2. **Janela:** selos e painel de detalhes, mais o harness.
3. **TUI:** selos, `v`, foto de tela e pty.
4. **Vault novo e painel:** semente, guias, exemplo e testes, mais a
   proposta do painel pro `VaultAnotadinho`.
5. **Memória:** os dois scripts e a primeira medida.

## Alternativas descartadas

- **Guardar os detalhes num bloco YAML como o do card do kanban.** Só o
  app leria. Fere o RNF2, e a checklist ficaria duplicada com os
  critérios.
- **Comentários na conversa da tarefa.** A conversa é do agente, e o
  contexto que vai pro modelo cresceria com recado entre pessoas.
- **Medir só RSS.** Somar o RSS dos processos do WebKit conta as
  bibliotecas compartilhadas várias vezes e exagera o total.
- **Medir com o `ps` de fora, sem dirigir o app.** Mostra o repouso,
  mas não o uso, e não acha vazamento.

## Riscos

- **O caminho do anexo** depende de onde a página da tarefa está
  (quatro níveis abaixo de `pages/`). O link tem que abrir no app e no
  markdown puro. O caminho certo é medido antes de escrever, como no
  padrão "fronteira do sistema".
- **A build de desenvolvimento** (Rust e WASM sem otimização) mede bem
  mais que a instalada. O script diz qual mediu, e o relatório
  recomenda medir na 9323.
- **Medir no app instalado** cria páginas `uitest-memoria-*` no vault
  aberto, e as remove no fim. Num vault sincronizado, elas podem
  aparecer por um instante noutra máquina.
