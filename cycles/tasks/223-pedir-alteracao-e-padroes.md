---
id: "223"
titulo: "Pedir alteração em revisão, e os padrões dos ciclos no vault"
status: done
criado: 2026-08-24
autor: humano
prioridade: alta
depende_de: ["222"]
estima_min: 120
agente_alvo: claude-opus-5
---

# Pedir alteração em revisão, e os padrões dos ciclos no vault

## Objetivo

Revisar só tinha duas saídas: aprovar, ou mandar pra trás. Faltava a
terceira, que é a mais comum — "está quase, muda estes pontos".

E os padrões que 222 ciclos produziram viviam só no histórico, sem
forma de anexar numa conversa.

## Critérios de aceite

- [x] Spec ou proposta em revisão oferece "Pedir alteração"
- [x] O botão abre a conversa com a página anexada e a pergunta pronta
- [x] A pergunta manda PROPOR, não gravar — a mudança volta como diff
- [x] Nenhuma pergunta carrega indentação do código-fonte
- [x] Sete páginas de padrão anexáveis em `pages/padroes/`
- [x] Cenário de harness

## Comandos de validação

```bash
cargo test --workspace
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
node scripts/uitest/run.mjs
```

## Notas

### O diff já funcionava, e havia prova disso

A pergunta era se o agente consegue alterar uma spec com revisão por
diff. Consegue, e não em teoria: ao investigar, havia uma proposta REAL
do Codex esperando na fila, editando
`pages/specs/prompts-padrao-reutilizaveis.md`. Ele usou
`anotadinho-cli propor` por conta, seguindo o `AGENTS.md`.

O que faltava era o caminho de ida: não havia botão que abrisse essa
conversa a partir da página em revisão.

### O prompt manda propor

`pergunta_de_alteracao` diz explicitamente pra NÃO gravar o arquivo e
usar `propor`. Sem isso o agente pode gravar direto — ele tem escrita na
pasta de trabalho desde o ciclo 216, e o vault mora dentro dela. A trava
do diff é o pedido, não o sandbox.

### Indentação vazando no prompt, de novo

`pergunta_de_planejamento` chegava com "Se algum          requisito" —
visível na tela do usuário. Era continuação de linha de código
preservando a indentação do FONTE, o mesmo defeito dos ciclos 204 e 209.
Agora há um teste que varre TODAS as perguntas atrás de espaço duplo e
de linha começando com espaço.

### O harness aplicou uma proposta de verdade

Rodando a suíte com a proposta do Codex na fila, o cenário do ciclo 204
clicou no primeiro "Aplicar" da tela — que era o dela. A spec do usuário
foi alterada sem ele ver o diff.

Revertido e devolvido à fila. O cenário agora acha o item pelo alvo
(`__uitest_proposta`) antes de clicar, e o do aviso (210) deixou de
afirmar que a fila esvazia: ele afirma que o aviso ACOMPANHA a fila,
que é o que importa e continua verdade com proposta real esperando.

É a mesma lição do ciclo 197, do outro lado: o vault é do usuário, e o
harness não encosta no que não é dele.
