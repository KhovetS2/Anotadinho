---
id: "183"
titulo: "Vault de exemplo mais completo"
status: done
criado: 2026-08-21
autor: humano
prioridade: media
depende_de: []
estima_min: 90
agente_alvo: claude-opus-5
---

# Vault de exemplo mais completo

## Objetivo

Pedido do usuário: o `VaultAnotadinho` mostrava só três dos nove
embeds, e nada dos recursos que entraram depois (consulta agrupada,
transclusão, id de bloco). Quem abre o app pela primeira vez não tem
como descobrir o que ele faz sem ler o código.

## Critérios de aceite

- [x] Os NOVE tipos de embed aparecem em alguma página do vault, com
      conteúdo real (não "lorem ipsum")
- [x] Página inicial reconstruída como índice navegável dos exemplos,
      usando os próprios embeds (callout + botões + consulta viva)
- [x] Uma página por tema, em vez de uma página gigante:
      composição, consultas, referências
- [x] Transclusão demonstrada nas três formas (página inteira, seção,
      bloco) com alvos que existem de verdade no vault
- [x] Galeria com imagens de verdade em `assets/` (geradas, pequenas)
- [x] Cada exemplo mostra também o comando de terminal equivalente,
      quando existe
- [x] Tudo conferido renderizando no app, não só no arquivo

## Comandos de validação

```bash
anotadinho-cli --vault VaultAnotadinho embed list pages/exemplos-embeds.md
anotadinho-cli --vault VaultAnotadinho query --from-embed pages/exemplos/consultas.md:0
cargo test --workspace
```

## Não-objetivos

- Tutorial passo a passo (o guia do Agent OS já é o texto longo; estas
  páginas são vitrine)
- Traduzir o vault (segue em português, como o resto)

## Notas

Três imagens de exemplo foram GERADAS (gradientes de 480×300, ~1,8 KB
cada) em vez de trazer arquivo de fora — a galeria precisava de
conteúdo real, e imagem de terceiro num repositório de exemplo é
dívida de licença esperando acontecer.

Dois achados durante a escrita:

1. `![[X]]` entre crases virava transclusão — virou o ciclo 182.
2. O teste `exemplos_embeds_vault_file_parses` (que pina a estrutura da
   página de demo contra o parser) falhou assim que a página cresceu.
   Fez exatamente o que devia; a expectativa foi atualizada pros dois
   embeds novos.

Transclusão de bloco da PRÓPRIA página é barrada junto com a
auto-transclusão, então o exemplo aponta pra um bloco de outra página
(`consultas.md^neq`) — e a página de referências documenta esse limite
em vez de escondê-lo.
