# Contexto de retry - Ciclo {ID}

Você está re-executando o ciclo {ID}. Esta task já falhou {N} vezes.

## Tentativa(s) anterior(es)

{Failure log mais recente - especialmente o que estava errado}

## Lições aprendidas

- {O que não fazer}
- {O que funcionou parcialmente}

## Restrições (não viole)

1. **Não quebre testes de ciclos done**: rode `cargo test` depois e garanta que tudo passa
2. **Não modifique arquivos de tarefas done**: se precisar, crie uma nova task
3. **Siga os critérios de aceite** do task original

## Sugestão

{Tentativa concreta de abordagem diferente}
