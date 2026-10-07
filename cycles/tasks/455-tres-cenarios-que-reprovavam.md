---
id: "455"
titulo: "Harness: os três cenários que reprovavam, e o resultado 'não se aplica aqui'"
status: done
criado: 2026-10-07
autor: agente
prioridade: media
depende_de: ["451"]
estima_min: 90
---

# 455 — Os três cenários que reprovavam

Pedido direto da pessoa. Cada um reprovava por um motivo diferente, e
nenhum era defeito do app.

## Critérios de aceite

- [x] "aparência: aplicar… (253)": esperava `escuro` como padrão, e o
      padrão é `mocha` desde o 387. Agora parte do padrão MEDIDO (sem
      preferência gravada) e devolve a preferência da pessoa num
      `finally`. Reprova quando "voltar ao padrão" diverge do padrão de
      quem não tem preferência (sabotado: `Default` = `claro`)
- [x] "prompt padrão… (224)": desde o 423 o preview é o prompt montado
      (peso, "como agir", contexto). Confere que o texto final está
      INTEIRO lá dentro, e que o contexto do prompt aparece junto
- [x] "janela: controles… (180)": no Hyprland, a janela encaixada é
      reportada como maximizada e "restaurar" é ignorado. Novo resultado
      `ctx.naoSeAplica(motivo)`: aparece no relatório (`○`), não conta
      como verde nem reprova. O cenário só cai nele com evidência
      (estado E tamanho iguais depois do pedido); o resto das asserções
      continua valendo
- [x] Foco simulado quando a janela do app não tem o foco do sistema
      (`garantirFoco`): sem ele, `focus()`/`blur()` não disparam eventos
      no WebKitGTK, e callout (151), consulta (168) e tabela (196)
      reprovavam inteiros, inclusive isolados. O relatório avisa quando o
      calço está ativo
- [x] A suíte devolve as preferências da pessoa no fim: o agente (o
      caso "não havia agente gravado" deixava o de teste no lugar) e a
      aparência (o cenário do vault gravava `escuro`)
- [x] Suíte completa sem `✗`

## Comandos de validação

```bash
node scripts/uitest/run.mjs aparência
node scripts/uitest/run.mjs "prompt padrão filtra"
node scripts/uitest/run.mjs "controles próprios"
node scripts/uitest/run.mjs
```

## Não-objetivos

- O botão de maximizar não faz nada numa janela encaixada no Hyprland.
  Isso é comportamento visível pra pessoa, mas é decisão do gerenciador.
  Fica registrado
- O cenário "o tema não entra no vault" ainda devolve a aparência
  gravando `escuro`, e os snapshots do fim da suíte dependem disso
  (a baseline foi gravada nesse tema). Mexer ali exige olhar a baseline
