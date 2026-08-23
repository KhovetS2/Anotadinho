# AGENTS.md — Anotadinho

Você está trabalhando no Anotadinho: um editor de notas markdown que
também é o painel do próprio desenvolvimento. As notas moram em
`VaultAnotadinho/`, DENTRO deste repositório.

Leia isto inteiro antes de mexer em qualquer coisa.

## O que este projeto é

| Camada | Tecnologia |
|---|---|
| Runtime | Tauri 2.x |
| Backend | Rust puro (workspace em `crates/`) |
| UI | Yew 0.21 + WASM, servido por `trunk` |
| Editor | um `contenteditable` por bloco |
| CSS | BEM, tokens em `:root`, claro e escuro |
| Testes | `cargo test --workspace` + harness de DOM |

```
crates/core/   modelos, markdown, embeds, query, fluxo, agente
crates/vault/  I/O e watcher
crates/ipc/    handlers dos comandos Tauri
crates/cli/    anotadinho-cli (seu canal com o vault)
ui/src/        frontend Yew: components/, api.rs, state.rs, styles/
src-tauri/     entry point e comandos
cycles/        tasks e status, um por ciclo
scripts/uitest/ harness de UI
docs/          arquitetura e design system
```

## Fale com o vault pelo `anotadinho-cli`, não pelo `cat`

O vault é markdown, então dá pra ler com `cat` e escrever com `>`. **Não
faça isso.** O frontmatter é YAML e os embeds inline
(`{{ type: "..." }} … {{ /X }}`) têm formato próprio; montar isso na mão
corrompe página, e já corrompeu (ciclo 064).

```bash
alias cli='./target/debug/anotadinho-cli --vault VaultAnotadinho'

cli list-pages --tag spec --status aprovada   # o que existe
cli read pages/specs/x.md                     # conteúdo bruto
cli read 'pages/specs/x.md^bloco'             # UM bloco só
cli search "termo"                            # full-text
cli query --from pages/specs                  # o mesmo motor do embed
cli embed list pages/x.md                     # embeds da página
cli embed get pages/x.md 0                    # YAML de um embed
cli set-property pages/x.md status aprovada   # frontmatter, corpo intocado
cli embed check pages/x.md                    # valida embeds antes de gravar
```

Se o binário não existir: `cargo build -p anotadinho-cli`.

### Escrever no vault: proponha, não grave

```bash
cli propor --alvo pages/specs/nova.md --criar --de /tmp/conteudo.md
cli propostas          # o humano revisa
```

`propor` grava um diff pra revisão humana em vez de escrever direto. É o
modo recomendado pra você. Uma nota é trabalho de alguém: sobrescrever
sem revisão não é um risco teórico, é o que já aconteceu.

**Nunca esvazie uma página.** Gravar vazio por cima de página com
conteúdo é recusado pelo app, e é sinal de bug — se você chegou nisso,
pare e explique.

## O ciclo de trabalho

Uma feature = UM ciclo. A ordem importa:

1. `cycles/tasks/{id}-{slug}.md` com objetivo, critérios de aceite,
   comandos de validação e não-objetivos
2. Implementar
3. **Validar** (seção abaixo) — não pule
4. Marcar a task `done` e os checkboxes `[x]`
5. `cycles/status/{id}-{timestamp}-done.md` com o que foi feito e como
   foi verificado
6. Commit `feat({id}): …` ou `fix({id}): …`

Não pergunte se deve seguir pro próximo ciclo. Siga.

O número do ciclo é o maior em `cycles/tasks/` mais um.

## Validação

```bash
cargo test --workspace
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

Mexeu em UI? O harness também. Ele roda contra o app DE VERDADE, porque
bug de DOM não aparece em `cargo test`:

```bash
node scripts/uitest/run.mjs      # sai != 0 se quebrou
node scripts/uitest/run.mjs foo  # só cenários que casam com "foo"
```

**Não tente subir o app você mesmo.** `./scripts/dev.sh` abre uma janela
e fica rodando pra sempre; num comando não-interativo isso trava, e num
sandbox costuma nem começar. Quem deixa o app de pé é a pessoa, num
terminal dela.

O harness fala com o app por WebSocket em `127.0.0.1:9223`. Se ele
responder

```
✗ não consegui falar com o app na porta 9223.
```

o app não está aberto: **peça pra pessoa abrir** e diga que a validação
de UI ficou pendente. Não marque o ciclo como validado, não escreva o
status `done` e não commite. Um ciclo de UI sem harness é meio ciclo, e
dizer o contrário é pior do que não ter rodado.

Se a falha for `Operation not permitted (os error 1)` ao abrir socket, é
o sandbox sem rede: relate exatamente isso, porque é configuração do
agente e não do repositório.

O app instalado usa a porta 9323, então ele e o dev convivem — mas
**nunca deixe dois apps abertos no mesmo vault**: os dois escrevem nos
mesmos arquivos.

### Snapshot visual

`node scripts/uitest/run.mjs` confere o estilo computado dos embeds
contra `scripts/uitest/baseline/`. Mudou estilo de propósito?

```bash
node scripts/uitest/snapshot.mjs --atualizar query
```

Só regrave a baseline depois de OLHAR a tela e confirmar que ficou certo.
Regravar pra calar o teste desfaz o motivo dele existir (ciclo 187).

## Regras que não se negociam

- **Nada de `execCommand`** pra inserir no editor. Use
  `insert_embed_marker_at_cursor` / `insert_element_at_cursor`.
- **Serialize YAML com `serde_yaml`**, nunca montando string.
- **CSS só com tokens** de `ui/src/styles/main.css`, classes em BEM,
  `color-mix` pra translúcidos. Nada de hex ou px cru.
- **Ícone novo entra em `ui/src/components/icon.rs`**. Nada de emoji.
- **Marque `data-nav-item` / `data-nav-group`** no DOM novo, senão ele
  nasce inalcançável por teclado.
- **`use_state` capturado em closure fica CONGELADO.** Se o valor muda
  entre a criação do closure e a execução dele, use `use_mut_ref`. Esse
  bug já voltou nos ciclos 155, 157, 201 e 213 — se você está lendo
  estado de dentro de um efeito ou timer, é quase certo que é ele.
- **Comentário explica POR QUÊ**, não o quê. Registre o bug que a linha
  evita, não a sintaxe.
- Escreva em português, como o resto do repositório.

## Fluxo de spec e proposta

O vault guarda o trabalho em três artefatos, e a diferença entre eles é
o que mantém o processo honesto:

- **Spec** (`pages/specs/`) — requisitos funcionais e não funcionais,
  critérios de aceite, fora de escopo. **O QUE** precisa existir. Nunca
  diz como implementar.
- **Proposta** (`pages/propostas/`) — abordagem, etapas, alternativas
  descartadas, riscos. **COMO** atender aquela spec. Recusar uma
  proposta não recusa a spec.
- **Execução** — o trabalho, a partir de uma proposta aprovada.

Cada um tem um embed `{{ type: "fluxo" }}` com a etapa. Só o humano
avança etapa.

**Se um requisito estiver ambíguo, aponte a ambiguidade — não decida por
conta.** E se a abordagem aprovada se mostrar inviável no meio da
execução, PARE e explique: mudar de abordagem exige proposta nova.

## Quando você não conseguir

Relate o que faltou, com precisão, em vez de contornar:

- Sem permissão de escrita? Diga qual caminho e qual operação falhou.
- Sem acesso a um diretório? Diga qual. A pasta de trabalho é escolhida
  pela pessoa nas configurações do agente, e `--add-dir` acrescenta
  outras.
- Não deu pra validar? Diga o que não rodou e por quê.

Um relatório honesto do que faltou vale mais que um ciclo marcado como
pronto sem ter sido.

## Onde ler mais

- `docs/architecture.md` — como o código está organizado
- `docs/design-system.md` — tokens, BEM, componentes
- `VaultAnotadinho/pages/produto/como-usar-modo-agentico.md` — o fluxo
  visto pelo humano
- `cycles/tasks/` — 216 ciclos de histórico. Antes de mudar algo que
  parece estranho, procure o ciclo que o criou: quase todo detalhe
  esquisito é cicatriz de um bug real.
