# Suíte de avaliação do agente (ciclo 413)

Todo o resto do projeto testa o Anotadinho. Isto testa o **arranjo** —
prompt, contrato de ferramentas, permissões, formato de proposta — que
faz um agente acertar ou errar. Sem a suíte, mexer no prompt padrão ou
no contrato (ciclo 407) era mudar às cegas.

Cada tarefa roda num vault **temporário e novo**: nada aqui toca vault
de ninguém, e uma tarefa não vê o que a anterior deixou.

## Os dois arquivos

- **`tarefas-falso.json`** — o arranjo, com o agente de mentira. Roda em
  segundos, não gasta token, e serve em CI (o comando sai diferente de
  zero se alguma reprovar). Verifica que a proposta chega ao vault, que
  o conteúdo é o pedido, e que a permissão por pasta barra o que está
  fora do alcance.

  ```sh
  cargo build -p anotadinho-cli
  ANOTADINHO_CLI=$PWD/target/debug/anotadinho-cli \
    ./target/debug/anotadinho-cli --vault /tmp avaliar \
      --tarefas avaliacao/tarefas-falso.json \
      --agente ./scripts/uitest/agente-falso.sh --arg=--propor --arg '{prompt}'
  ```

- **`tarefas.json`** — as mesmas perguntas para um agente **de verdade**.
  Aqui o que se mede é o agente: ele achou a nota, propôs no lugar
  certo, respeitou "não proponha nada"?

  ```sh
  ./target/debug/anotadinho-cli --vault /tmp avaliar \
    --tarefas avaliacao/tarefas.json --agente claude --arg -p --arg '{prompt}' --stream
  ```

`--so <texto>` roda só as tarefas cujo nome contém o texto.

## Escrever uma tarefa

```json
{
  "nome": "propoe-spec-de-uma-nota",
  "prompt": "Leia pages/notas/ideia.md e PROPONHA pages/specs/x.md a partir dela.",
  "vault": [["pages/notas/ideia.md", "---\ntitle: Ideia\n---\n\ntexto\n"]],
  "permissoes": { "pode": ["pages/specs"], "nunca": ["journals/"] },
  "espera": [
    { "tipo": "propoe_em", "prefixo": "pages/specs" },
    { "tipo": "conteudo_contem", "texto": "duplicar" }
  ]
}
```

`espera` aceita `propoe_em`, `nao_propoe`, `conteudo_contem`,
`resposta_contem` e `falha_contem` — **todas** precisam valer. Falha não
prevista reprova: um agente que morreu não "passou" só porque a
expectativa era não propor nada.

Quando uma tarefa reprova, o relatório mostra o que o agente respondeu e
onde ele propôs — porque quem escreve tarefa erra o enunciado tanto
quanto o agente erra a resposta.

O julgamento mora em `crates/core/src/avaliacao.rs` (puro, com testes);
montar o vault e chamar o binário é `crates/cli/src/avaliar.rs`.
