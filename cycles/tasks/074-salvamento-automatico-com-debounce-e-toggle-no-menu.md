---
id: "074"
titulo: "Salvamento automatico com debounce e toggle no menu"
status: done
criado: 2026-08-07
autor: humano
prioridade: alta
depende_de: []
estima_min: 90
agente_alvo: claude-sonnet
---

# Salvamento automático com debounce e toggle no menu

## Objetivo

Usuário pediu: salvamento automático alguns segundos depois de parar de
digitar, opção pra ativar/desativar no menu principal, e — o problema de
fundo — parar de perder o estado da página ao trocar de página sem salvar
antes.

## Critérios de aceite

- [x] Toggle "Salvamento automático" no menu ⚙ (`HeaderBar`), persistido
      em `localStorage` (`anotadinho.autosave_enabled`, padrão ativado)
- [x] Com o toggle desligado, editar não agenda mais o save automático
      de 3s — só marca "não salvo", precisa clicar em "Salvar"
- [x] Trocar de página com edições pendentes salva automaticamente antes
      da troca, **independente do toggle** — essa parte é uma rede de
      segurança contra perda de dado, não a conveniência do autosave
- [x] Testes de regressão (via MCP, ver Notas — não dá pra testar isso
      com `cargo test` puro, depende de DOM real)

## Comandos de validação

```bash
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Nenhum indicador visual de "salvando..." diferente do que já existia
- Resolver o `#[derive]`/lint de `cargo clippy` nas outras partes do
  arquivo — fora de escopo

## Notas

**Descoberta técnica central**: o autosave debounced (3s) já existia
antes deste ciclo (`trigger_debounced_save`), só faltavam o toggle e a
proteção contra perda ao trocar de página. O ponto difícil foi a
proteção: o efeito que observa `props.page` (`Effect 1` do `Editor`) só
recria seus closures quando a PÁGINA muda — lendo `*edited`/`*content_md`
(ambos `use_state`) de dentro do cleanup desse efeito sempre devolvia o
valor de QUANDO O EFEITO FOI CRIADO (`edited=false`, `content_md` vazio),
nunca o estado atual, porque `UseStateHandle` é um snapshot por
renderização — um clone capturado num efeito que só roda de novo quando
`page` muda fica congelado, mesmo que `.set()` seja chamado depois por
OUTRO clone do mesmo handle (mesma classe de bug já encontrada e corrigida
no resize do calendário, ciclo 071).

**Solução**: `edited_ref`/`pending_flush_ref`, dois `use_mut_ref`
(`Rc<RefCell<_>>` — a MESMA instância em toda renderização, ao contrário
de `use_state`). Atualizados a cada `oninput` (dentro de
`trigger_debounced_save`, que roda a cada tecla, então sempre lê valores
frescos de `content_md`/`editor_ref`/`segment_refs` — diferente do efeito
de página, essa closure É recriada a cada renderização). O cleanup do
`Effect 1` lê esses refs (sempre atuais) e, se havia edição pendente,
dispara `api::write_page` pra página que está sendo deixada, direto — sem
depender de handles `use_state` potencialmente congelados.

**Bug real encontrado de bônus**: extrair a lógica de recomputar markdown
do DOM (`do_save` → `recompute_markdown_from_dom`) revelou que o branch
de página SEM embeds nunca recolocava o frontmatter (`fm`) na frente do
corpo recomputado — só o branch COM embeds fazia isso certo. Ou seja,
**qualquer save de uma página com frontmatter e sem embeds já perdia o
frontmatter inteiro antes deste ciclo**, não é bug introduzido agora.
Corrigido no mesmo commit.

Validado ao vivo via MCP `tauri` na página `teste` (frontmatter
`title: teste` + 1 bullet): digitar texto e trocar de página IMEDIATAMENTE
(sem esperar os 3s) — reabrir a página mostrou o texto novo E o
frontmatter intacto (antes do fix de frontmatter, `title:: teste` sumia
do arquivo). Repetido com o toggle de autosave DESLIGADO: esperei 4s sem
o conteúdo ser salvo (confirmando que o timer de 3s não disparou), depois
troquei de página e confirmei que o flush de segurança salvou mesmo assim.
Nenhuma edição de teste vazou pro vault — os dois testes na página
`teste.md` foram revertidos com `git checkout` depois de confirmados.
