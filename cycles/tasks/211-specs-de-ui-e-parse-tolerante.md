---
id: "211"
titulo: "Specs de UI/QoL, bateria pendente e parse tolerante de embed"
status: done
criado: 2026-08-23
autor: humano
prioridade: alta
depende_de: ["210"]
estima_min: 120
agente_alvo: claude-opus-5
---

# Specs de UI/QoL, bateria pendente e parse tolerante de embed

## Objetivo

Transformar a lista de melhorias de UI/QoL pedida pelo usuário em specs
de verdade (requisitos + critérios de aceite, sem abordagem de
implementação — a separação fixada no ciclo 209), e escrever o harness
delas ANTES da implementação.

No caminho, um bug de perda de dado apareceu sozinho e foi corrigido.

## Critérios de aceite

- [x] Cinco specs em `pages/specs/`, uma por área coerente:
      navegação/vim, leitura de consultas, atalhos do dia a dia,
      imagens arrastadas, tema configurável
- [x] Cada spec com requisitos funcionais, não funcionais, critérios de
      aceite e fora de escopo — nenhuma diz COMO implementar
- [x] Bateria `scripts/uitest/pendentes.mjs` com um cenário por
      critério verificável, rodando por `--pendentes`
- [x] A bateria pendente fica FORA da suíte principal: ela é vermelha
      por definição e não pode estragar o sinal dos 127 cenários
- [x] Toda falha da bateria pendente é uma lacuna real de produto, não
      erro de script
- [x] Parse de embed tolerante: um campo com tipo errado descarta só
      aquele campo, não o embed inteiro
- [x] Suíte principal continua verde

## Comandos de validação

```bash
cargo test --workspace
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
node scripts/uitest/run.mjs
node scripts/uitest/run.mjs --pendentes   # vermelha de propósito
```

## Não-objetivos

- Implementar qualquer uma das cinco specs. Elas estão `em-revisao`
  esperando decisão, e o passo seguinte é uma PROPOSTA por spec.

## Notas

### Duas coisas da lista já estavam prontas

Conferido no código antes de escrever, pra a spec não pedir o que já
existe:

- O botão "Nova conversa" nas ações da home entrou no ciclo 208.
- Colar imagem entrou no ciclo 118 e grava em `assets/` de verdade. O
  que está quebrado é ARRASTAR, que insere uma URL `blob:` de sessão —
  a imagem aparece e some ao recarregar, sem nada gravado. A spec de
  imagens foi reescrita em cima disso.

### O bug que apareceu no caminho

Durante os testes de navegação o app reabriu e regravou a home, e a
consulta "O que o vault tem" voltou VAZIA do disco: `group_by`,
`aggregate` e `collapsed` sumiram.

Causa: a home tinha `collapsed: true` numa consulta, mas o campo é
`Vec<String>`. O parse de todo embed era
`serde_yaml::from_str(raw).unwrap_or_default()` — um campo com tipo
errado derruba o documento inteiro pro default, e o save seguinte grava
esse default por cima do arquivo. Perda silenciosa, sem erro em lugar
nenhum.

Correção: `parse_yaml_tolerante`, usado pelos 9 pontos de parse de
embed. Quando o documento falha, ele descarta o MENOR conjunto de
campos que faz o resto parsear — primeiro todos os singles, depois os
pares. A primeira versão tentava pares antes de esgotar os singles e
devolvia um resultado que jogava campo bom fora junto com o ruim; o
teste `campo_com_tipo_errado_nao_apaga_o_resto_da_consulta` pegou isso.

### Uma bomba-relógio no harness

O cenário "cronograma: arrastar a barra grava a data nova (155)" ficou
vermelho sozinho, sem ninguém ter mexido no código: ele cravava
`start: 2026-08-10` / `end: 2026-08-14`, e a janela do cronograma é
ancorada em HOJE. Quando a data passou de 15/08, a barra saiu da faixa
desenhada e o cenário passou a falhar por envelhecimento.

Agora as datas são relativas (`hoje+2` a `hoje+6`).
