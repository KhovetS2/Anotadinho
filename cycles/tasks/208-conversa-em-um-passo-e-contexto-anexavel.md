---
id: "208"
titulo: "Conversa em um passo e contexto anexável"
status: done
criado: 2026-08-22
autor: humano
prioridade: alta
depende_de: [202, 207]
estima_min: 210
agente_alvo: claude-opus
---

# Conversa em um passo e contexto anexável

## Objetivo

Itens 1 e 2 da spec **aprovada** [[Uso agêntico do Anotadinho no dia a
dia]], mais o ponto que o usuário acrescentou na revisão: **anexar as
páginas que o modelo deve consultar**, pra ele não propor algo que já
existe nem gastar tempo procurando.

Foi a revisão funcionando: a spec foi escrita, revisada, editada por
quem decide, e só então aprovada.

## Critérios de aceite

- [x] Comando "Nova conversa com o agente" na paleta e botão no home.
- [x] Cria em `pages/conversas/`, com data no nome, `type: conversa`,
      `origem:` e a página aberta já anexada.
- [x] `contexto:` no FRONTMATTER — sobrevive a fechar o app.
- [x] Anexar e tirar páginas pela UI, gravando na hora.
- [x] Seletor com FILTRO: com 200+ páginas, lista sem busca é inútil.
- [x] Vários contextos no prompt, cada um identificado pelo caminho.
- [x] Nome de página não consegue forjar o delimitador do bloco de dado.
- [x] 2 cenários de harness + 12 testes no core.

## Comandos de validação

```bash
cargo test --workspace
cd ui && trunk build
node scripts/uitest/run.mjs
```

## Não-objetivos

- Itens 3 e 4 da spec (aviso de proposta pendente e executar a partir da
  proposta): ficam pro ciclo 209.

## Duas correções de infraestrutura, achadas doendo

**Portas iguais em dev e release.** O app instalado no sistema e o
`dev.sh` abriam a MESMA porta 9223. Quando os dois estão de pé — e
ficam, porque um é do menu e o outro do terminal — a ponte responde pelo
app errado: você edita, o dev reconstrói, e a janela na sua frente não
muda. Custou tempo três vezes nesta sessão antes de eu diagnosticar.
Release passou pra 9323.

**O harness sequestrava a configuração do agente.** Os cenários apontam
o adaptador pro agente de mentira e não devolviam: depois de rodar a
suíte, a próxima conversa de verdade falhava com "erro proposital".
Aconteceu comigo. O `run.mjs` agora guarda e restaura.
