---
title: O app instalado liga o MCP sem depender do PATH
type: spec
date: 2026-10-06
status: rascunho
prioridade: media
tags:
- spec
- portabilidade
- agente
---
{{ type: "fluxo" }}
artefato: spec
etapa: rascunho
{{ /fluxo }}
## Por que isto existe

Desde o ciclo 426, cada execução do agente liga o servidor MCP do
Anotadinho, e é ele que dá ao agente `ler_pagina`, `buscar`, `propor` e
o resto do contrato. Quem atende o MCP é o `anotadinho-cli`, que o app
procura **ao lado do próprio executável** e, se não achar, no `PATH`
(`agente::cli_ao_lado`).

Em desenvolvimento os dois estão juntos em `target/debug`. No app
**instalado**, não: o `tauri.conf.json` não declara o CLI em lugar
nenhum, então o instalador (`.msi`/`.exe` no Windows, `.deb`/AppImage no
Linux) leva só o app. Sem o CLI ao lado e sem ele no `PATH`, o agente
sobe sem as ferramentas do vault e só dá pra perceber pela qualidade da
resposta.

No Windows isso é ainda mais provável: ninguém instala o CLI por
`cargo install`, e o instalador não mexe no `PATH`.

## Requisitos funcionais

1. **RF1** — No app instalado por qualquer instalador gerado pelo
   projeto, o agente recebe as ferramentas MCP do Anotadinho sem nenhuma
   configuração.
2. **RF2** — Em desenvolvimento (`target/debug`), tudo continua
   funcionando como hoje.
3. **RF3** — Quando não existe nenhum CLI disponível, a conversa avisa
   que o agente rodou sem as ferramentas do vault. Não pode falhar em
   silêncio.
4. **RF4** — A TUI, que é um binário separado, segue a mesma regra.

## Requisitos não funcionais

- `cargo build --manifest-path src-tauri/Cargo.toml` continua passando
  num checkout limpo, sem passo manual antes.

## Critérios de aceite

- [ ] O pacote gerado no Linux (`.deb`) contém o CLI, e uma conversa do
      app instalado lista as ferramentas MCP do Anotadinho.
- [ ] Sem CLI disponível, a conversa mostra o aviso do RF3.
- [ ] Build limpo do app continua passando.
- [ ] **Manual, numa máquina Windows (pendente):** o mesmo, com o `.msi`.

## Fora de escopo

- Pôr o CLI no `PATH` do sistema.
- Assinatura de código e atualização automática.
