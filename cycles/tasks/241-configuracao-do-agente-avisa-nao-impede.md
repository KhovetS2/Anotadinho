---
id: "241"
titulo: "A configuração do agente avisa, não impede"
status: done
criado: 2026-08-30
autor: agente
prioridade: media
depende_de: ["239"]
estima_min: 45
---

# 241 — A configuração do agente avisa, não impede

## Objetivo

A validação recusava executável que parecesse linha de comando. O
argumento era impedir "execução de shell pela porta dos fundos" — mas não
existe shell no caminho, então ela não impedia nada; só tirava de quem
configura a chance de apontar o que quisesse na própria máquina.

## Critérios de aceite

- [x] `validar` recusa só o que torna a execução impossível: sem
      executável, sem `{prompt}`, marcador repetido
- [x] `aviso` é separado e não desabilita salvar
- [x] A interface mostra o aviso em amarelo e deixa gravar
- [x] O cenário passa a provar a garantia real: nenhum shell é executado

## Comandos de validação

```bash
cargo test --workspace
cd ui && trunk build
node scripts/uitest/run.mjs
```
