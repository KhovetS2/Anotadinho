---
id: "215"
titulo: "Agente na raiz do projeto e trava contra esvaziamento"
status: done
criado: 2026-08-23
autor: humano
prioridade: alta
depende_de: ["214"]
estima_min: 120
agente_alvo: claude-opus-5
---

# Agente na raiz do projeto e trava contra esvaziamento

## Objetivo

O ciclo spec → proposta → execução travou na terceira etapa, e duas
propostas foram zeradas no vault.

## Critérios de aceite

- [x] Sem `cwd` configurado, o agente trabalha na raiz do PROJETO
- [x] Verificado com o Codex de verdade: ele responde
      `/home/elis/Anotadinho` e consegue ler `ui/src/...`
- [x] Gravar vazio por cima de página com conteúdo é recusado
- [x] Página nova ainda pode nascer vazia
- [x] A recusa vale também no caminho com versão
- [x] O diretório de trabalho aparece no chip do agente
- [x] Cenários de harness pros dois

## Comandos de validação

```bash
cargo test --workspace
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
node scripts/uitest/run.mjs
```

## Notas

### O agente rodava dentro das notas

`cwd` vazio caía no vault. Isso é o pior dos dois mundos: o agente NÃO
alcança o código que a proposta manda mudar (`ui/`, `crates/`,
`scripts/`), e alcança com escrita justamente as notas — que é o que o
fluxo de propostas do ciclo 204 existe pra proteger.

Agora `raiz_do_projeto` sobe procurando `.git`. Num projeto que guarda
as notas dentro do próprio repositório, a raiz é o lugar de onde se
enxerga o código E as notas. Sem `.git` em lugar nenhum, fica o vault,
que é o comportamento antigo.

### As propostas zeradas: causa NÃO estabelecida

Duas páginas de proposta ficaram com 0 bytes logo depois de um pedido de
execução. O que foi descartado, testando:

- **O editor não faz isso.** Abrir a proposta (inclusive a de 10 KB,
  restaurada da conversa), aprovar e clicar em Executar preserva o
  arquivo. O flush de saída de página já recusa markdown vazio.
- **O agente sandboxado também não.** Um `apply_patch` do Codex sobre um
  arquivo do vault é rejeitado ANTES de truncar: o arquivo fica
  intacto. Confirmado com o binário de verdade.

Sem reprodução, não há conserto de causa. O que dá pra fazer — e vale
independente da causa — é impedir o resultado: `recusar_esvaziamento`
recusa gravar vazio por cima de página com conteúdo, na camada de IPC,
por onde todo escritor passa (editor, conversa, backend, CLI, agente).

Apagar uma nota inteira nunca é o resultado certo de um save. Quem quer
esvaziar uma página, apaga a página.

### As propostas foram restauradas

O texto original estava na conversa que as gerou
(`conversa-2026-08-23-07-04.md`), então deu pra reconstruir a proposta
de 10 KB a partir dela. É um efeito colateral bom do desenho: a conversa
é o registro, a página promovida é uma cópia.
