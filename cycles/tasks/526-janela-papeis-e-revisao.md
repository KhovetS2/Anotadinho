---
id: "526"
titulo: "Janela: os papéis e o laço de revisão na tela do projeto — escolher executor e revisor, ver a rodada, a base de problemas e os números"
status: done
criado: 2026-10-25
autor: agente
prioridade: alta
depende_de: ["521", "522", "523", "524"]
estima_min: 180
---

# 526 — A janela mostra o tech lead e o júnior

Os ciclos 521–524 puseram os papéis, o laço de revisão, a base de
problemas e os números no núcleo; a TUI ganha a tela no 525. Este é o
mesmo na janela.

## Critérios de aceite

- [x] A UI manda a lista dos agentes da pessoa (`opcoes_de_agente`) pro
      `avaliar_projetos`: o executor do projeto é procurado nela (parte da
      UI do 521, separada porque o app não estava aberto).
- [x] **Configurar** ganha Executor (o agente padrão ou um da lista),
      Revisor (nenhum ou um da lista) e o teto de rodadas (≥ 1). Um nome
      gravado que não está mais na lista aparece como opção marcada
      "(não existe)", em vez de sumir calado. Gravar manda a config inteira.
- [x] O cabeçalho diz os papéis ("executor … · revisor … · até N
      rodadas") quando não são o padrão.
- [x] Selo no cartão: em `executando`, "revisão N/T" (pendente ou
      rodando) e "ajustes" (refazendo); em `revisao`, "revisor ok",
      "teto" ou "revisor falhou".
- [x] Detalhes da tarefa: a seção **Revisão pelo agente** (estado, rodada)
      com "Conversa do revisor" quando ela existe.
- [x] Aba **Execuções**: os números por papel (`core::numeros`, calculados
      na própria UI com as execuções e as tarefas que ela já tem) e o link
      pra **base de problemas** quando ela existe.
- [x] Harness: um cenário com a config dos papéis (os seletores, o
      projetos.json), os selos do laço, a seção nos detalhes e os números.

## Comandos de validação

```bash
cargo test --workspace
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
node scripts/uitest/run.mjs "(526)"
node scripts/uitest/run.mjs
```

## Não-objetivos

- A TUI (525).
