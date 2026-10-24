---
id: "493"
titulo: "Um Ctrl+S antigo não grava a página da aba que acabou de abrir"
status: done
criado: 2026-10-24
autor: agente
prioridade: alta
depende_de: []
estima_min: 60
---

# 493 — O Ctrl+S global repetido na troca de aba

Relato: ao aprovar a spec e a proposta `tui-codigo-e-diagramas`, mudar a
etapa, salvar e trocar de aba, apareceu "gravação recusada: isso apagaria
as 4471 letras de pages/specs/tui-codigo-e-diagramas.md". O arquivo ficou
inteiro: quem segurou foi a trava do backend (`recusar_esvaziamento`).

## Causa (reproduzida)

1. O Ctrl+S com o foco fora do texto, como logo depois de clicar num botão
   de etapa, vai pro atalho global do `App`. Ele grava
   `global_editor_action = Some((Save, nonce))` e nunca limpa.
2. O editor é remontado a cada troca de aba.
3. O efeito `use_effect_with(props.global_action)` roda na montagem, vê o
   `Save` antigo e chama `do_save`. O conteúdo ainda não carregou, então o
   markdown recalculado do DOM é vazio.

Reproduzido no app instalado com duas páginas `__uitest` em duas abas:
Ctrl+S global numa, clique na outra, e o mesmo erro aparece.

Sem a trava do backend, a página teria sido apagada. O mesmo caminho
repetia também o Desfazer e o Refazer na aba que abria.

## Critérios de aceite

- [x] O editor só reage a uma ação global NOVA: o nonce visto na montagem
      não dispara nada.
- [x] Cenário de UI: duas abas, Ctrl+S global numa, volta pra outra, e
      nada é gravado nem recusado. Visto reprovando no build antigo
      ("a aba que abriu tentou se gravar").

## Comandos de validação

```bash
cd ui && cargo test --lib && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
node scripts/uitest/run.mjs "(493)"
node scripts/uitest/run.mjs
```

## Não-objetivos

- Mudar a trava do backend: ela fez o trabalho dela.
