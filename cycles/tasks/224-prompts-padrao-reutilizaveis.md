# Ciclo 224 — Prompts padrão reutilizáveis

## Objetivo

Permitir que a conversa descubra páginas `type: prompt` em
`pages/prompts-default/`, preencha seus marcadores nomeados, visualize o
resultado e incorpore o contexto declarado antes do envio.

## Critérios de aceite

- [x] Somente páginas `type: prompt` dentro de `pages/prompts-default/` e
      subpastas aparecem no seletor acima de Enviar.
- [x] A opção vazia preserva o envio livre atual.
- [x] O rascunho preenche a primeira variável e as demais são solicitadas
      na ordem da primeira ocorrência.
- [x] Variáveis repetidas pedem um único valor e marcador pendente impede
      preview e envio.
- [x] Prompt sem marcador recebe o rascunho ao final.
- [x] O preview mostra o texto final e nunca envia automaticamente.
- [x] O `contexto:` do prompt é incorporado, sem duplicatas, ao contexto
      persistente da conversa.
- [x] Valores interpolados e anexos continuam tratados como DADO na
      montagem entregue ao agente.
- [x] O harness cobre marcador único, múltiplas variáveis, repetição,
      pendência, opção vazia, filtros, preview e contexto.

## Validação

```bash
cargo test --workspace
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
node scripts/uitest/run.mjs
```

## Não-objetivos

- Biblioteca compartilhada entre vaults.
- Descoberta fora de `pages/prompts-default/`.
- Condicionais, laços ou geração automática a partir do histórico.
- Tela exclusiva para cadastrar prompts.

## Status

done
