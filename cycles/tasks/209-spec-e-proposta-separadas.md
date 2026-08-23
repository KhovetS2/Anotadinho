---
id: "209"
titulo: "Spec e proposta como artefatos distintos"
status: done
criado: 2026-08-22
autor: humano
prioridade: alta
depende_de: [201, 208]
estima_min: 180
agente_alvo: claude-opus
---

# Spec e proposta como artefatos distintos

## Objetivo

Correção conceitual pedida pelo usuário, e ela estava certa: as specs
que eu mesmo escrevi tinham uma seção "## Proposta" dentro, misturando o
QUÊ com o COMO.

A separação, em termos de engenharia:

| | Spec | Proposta |
|---|---|---|
| Responde | o quê e o porquê | o como |
| Contém | requisitos funcionais e não funcionais, critérios de aceite | abordagem, etapas, alternativas, riscos |
| Quando muda | quando o problema muda | quando a abordagem não serve |

O efeito prático: se a abordagem fere um padrão da casa, você descarta a
PROPOSTA e escreve outra — a spec continua valendo, porque aquilo ainda
precisa ser feito. Sem a separação, recusar uma abordagem parecia
recusar o trabalho inteiro.

## Critérios de aceite

- [x] `fluxo::corpo_padrao` com esqueletos distintos por artefato.
- [x] Teste garantindo que spec NÃO tem "abordagem" e proposta NÃO
      redefine requisito.
- [x] Botão "Planejar implementação" numa spec APROVADA, que abre a
      conversa com a spec anexada e a pergunta pronta.
- [x] A pergunta proíbe explicitamente propor requisito novo.
- [x] Spec em rascunho não oferece planejar.
- [x] Templates do vault (`spec.md` reescrito, `proposta.md` novo).
- [x] As duas specs existentes reescritas no formato certo, e o "como"
      da primeira virou uma proposta de verdade.
- [x] 2 cenários de harness.

## Comandos de validação

```bash
cargo test --workspace
cd ui && trunk build
node scripts/uitest/run.mjs
```

## Não-objetivos

- Itens 3 e 4 da spec aprovada (aviso de pendente, executar da
  proposta): ficam pro próximo ciclo.

## Dois bugs meus, dos mesmos tipos de sempre

1. **Título do arquivo em vez do frontmatter** — a conversa nascia
   "Planejar: uso-agentico-do-anotadinho". É o defeito do ciclo 196 de
   novo, agora resolvido por `scan_vault`.
2. **Indentação de código vazando pro prompt** — string de várias linhas
   com `\` preserva a indentação do FONTE. Já tinha me pegado num teste
   do ciclo 204. Agora o teste confere que nenhuma linha do prompt
   começa com espaço.
