---
id: "216"
titulo: "Pasta escolhida pela pessoa e execução na mesma conversa"
status: done
criado: 2026-08-23
autor: humano
prioridade: alta
depende_de: ["215"]
estima_min: 150
agente_alvo: claude-opus-5
---

# Pasta escolhida pela pessoa e execução na mesma conversa

## Objetivo

Três correções no ciclo spec → proposta → execução.

## Critérios de aceite

- [x] A pasta de trabalho é ESCOLHIDA num seletor, não deduzida
- [x] Dá pra somar outras pastas (o caso de vault num lugar e vários
      repositórios noutro)
- [x] Os presets pedem permissão de escrita — executar é editar arquivo
- [x] Configuração já gravada é migrada pra ganhar a permissão
- [x] Executar CONTINUA na conversa que gerou a proposta
- [x] Proposta sem origem viva ainda ganha conversa nova
- [x] Verificado com o Codex de verdade: ele criou um arquivo no
      repositório
- [x] Cenários de harness pros três

## Comandos de validação

```bash
cargo test --workspace
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
node scripts/uitest/run.mjs
```

## Notas

### Adivinhar a pasta estava errado

O ciclo 215 deduzia a raiz pelo `.git` mais próximo. Isso só acerta
quando as notas moram dentro do repositório. Quem tem o vault num lugar
e três repositórios noutro ficava com o agente apontado pro lugar
errado — e sem saber disso.

Agora a pasta vem de um seletor. A dedução continua como PADRÃO pra
quem não escolheu nada, mas deixou de ser a única resposta. E a escolha
sendo da pessoa é o que autoriza a escrita ali: não é o app decidindo
onde o agente pode mexer.

`pastas_extras` cobre o caso de vários repositórios, via `--add-dir` —
que tanto o Claude Code quanto o Codex aceitam. O opencode fica de fora
(`arg_pasta_extra` vazio): mandar uma flag que ele não conhece
derrubaria a execução inteira, e o formato dele não foi confirmado
contra o binário.

### Executar é editar arquivo

Os presets rodavam sem permissão de escrita, então o agente lia tudo e
não mudava nada — "este ambiente está em modo somente leitura". Agora o
Codex leva `--sandbox workspace-write` e o Claude
`--permission-mode acceptEdits`.

Isso não afrouxa o fluxo de propostas: ele protege o VAULT, e a escrita
liberada é na pasta de trabalho que a pessoa escolheu, sob git.

A lista de args antigos da migração cresceu pra incluir os presets dos
ciclos 213 e 214 — sem isso, quem já usou o app ficaria preso na versão
sem escrita.

### A execução espalhava o histórico

Cada clique em "Executar" abria uma conversa NOVA: a discussão que
produziu a proposta numa página, o que o agente fez pra executá-la
noutra, sem ligação visível. Agora a proposta é lida, o `origem` do
embed de fluxo aponta pra conversa que a gerou, e a execução continua
lá. Proposta escrita à mão, ou cuja conversa foi apagada, ganha conversa
nova como antes.
