---
id: "{ID}"
executado_em: {DATA}
motivo: {MOTIVO}
tentativa: {N}
---

# Ciclo {ID} - failure

## Contexto para retry

A task {ID} falhou. Esta é a tentativa {N}. Ao re-executar, considere:

1. **Verifique o que mudou** desde a última tentativa
2. **Olhe os logs abaixo** pra entender onde travou
3. **Tente uma abordagem diferente** se foi a mesma coisa
4. **Se necessário, divida a task** em ciclos menores

## Erro detalhado

```
{saída do comando que falhou}
```

## O que foi tentado

{Descrição do que o agent fez}

## Próxima ação sugerida

{Sugestão do que fazer diferente}

## Logs brutos

```
{logs completos do agent e do build/test}
```
