---
id: "228"
titulo: "Virar execução pede a implementação"
status: done
criado: 2026-08-29
autor: agente
prioridade: alta
depende_de: ["203", "216"]
estima_min: 90
---

# 228 — Virar execução pede a implementação

## Objetivo

O botão "virar execução" numa resposta criava um arquivo e ia embora. Quem
queria de fato a implementação tinha que redigitar o pedido na mão.

## Critérios de aceite

- [x] Virar execução pergunta antes, porque agora gasta tempo de modelo
- [x] Cancelar não cria página nenhuma
- [x] Confirmar cria a execução, anexa ela à conversa e pede a
      implementação, tudo na mesma conversa
- [x] A pergunta é própria da execução criada na conversa, e não promete
      uma revisão que não houve
- [x] Com execução em andamento, avisa em vez de abrir uma segunda
- [x] Virar spec e virar proposta continuam só criando e abrindo a página

## Comandos de validação

```bash
cargo test --workspace
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
node scripts/uitest/run.mjs
```

## Não-objetivos

- Avançar a etapa do fluxo em nome da pessoa (é o ciclo 229)
- Mudar o comportamento do botão "Executar" do embed de fluxo, que parte
  de uma proposta já revisada
