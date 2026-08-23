---
title: A escrita no vault não foi autorizada, então segue a propost
tags:
- proposta
type: proposta
date: 2026-08-23
status: aprovada
---
# A escrita no vault não foi autorizada, então segue a propost
{{ type: "fluxo" }}
artefato: proposta
etapa: aprovada
origem: pages/conversas/conversa-2026-08-23-07-04.md

{{ /fluxo }}
A escrita no vault não foi autorizada, então segue a proposta aqui — posso gravá-la em `pages/propostas/leitura-de-consultas-abordagem.md` se você liberar.

---
# Abordagem para a leitura de consultas

**Spec que atende:** [[Leitura de consultas: alinhamento, cor e altura]] — os cinco RFs e os quatro RNFs. Duas ambiguidades ficaram sem decisão, na seção do fim.

## Abordagem

Os três problemas são de apresentação e caem quase todos no mesmo lugar: `ui/src/components/embeds/inline_query.rs` e o bloco `.query-embed*` de `ui/src/styles/main.css`. O motor (`crates/core/src/query.rs`) não é tocado — ele já devolve o recorte certo; o que falha é como o recorte é desenhado.

**Alinhamento (RF1)** vira uma grade só. A tabela desalinha porque cabeçalho e linhas são caixas independentes: cada linha resolve a largura das suas células sozinha, sem nada amarrando a coluna. Uma grade única para o bloco, com `grid-template-columns` derivado de `columns`, faz o alinhamento cair por construção — e o cabeçalho de grupo e o rodapé de agregado do ciclo 169 atravessam a linha com `grid-column: 1 / -1`, sem depender de quantas colunas existem.

**Cor (RF2)** sai de uma função pura no core que mapeia valor → **índice** de paleta, não valor → cor. A cor mora em token CSS, com par claro/escuro sob `data-theme`, e a UI só escolhe a classe `.query-embed__valor--cor-N`. Isso atende os RNFs de uma vez: determinístico e sem estado (RNF2), sem tocar o `.md` (RNF3), contraste auditado uma vez por token nos dois temas (RNF4), e o texto continua onde está — a cor é fundo de badge, não substituição (RNF1).

Cor por classe, e não por `style` inline, é deliberado: a impressão digital do ciclo 187 agrupa por combinação de classes e compara estilos computados. Cor inline faria a mesma classe aparecer com computados diferentes a cada linha, e a baseline pararia de ser estável.

**Altura (RF3–RF5)** é teto e rolagem no contêiner de **resultados**, não no embed inteiro: barra de configuração, descrição da consulta e rodapé de contagem ficam fora da área que rola. O cabeçalho da tabela fica `sticky` no topo da área rolável, o que só funciona porque a grade da etapa 2 já garante o alinhamento com o que passa por baixo.  
  
**Virtualizar a lista(RF6).**Montar no scroll somente o trecho que está visível pro usuaŕio mais uns 2 para cima e para baixo se tiver e ir mudando quem está nesse intervalo visivel da lista..

## Etapas

1. **Levantar a marcação atual.** Ler `inline_query.rs` e o bloco `.query-embed*` e registrar como as três views (`list`, `table`, `cards`), o cabeçalho de grupo, o rodapé de agregado e a célula editável (ciclo 168) estão estruturados hoje. As etapas 2 e 4 mexem exatamente aí, e a suposição de que a tabela é feita de caixas por linha precisa ser confirmada antes de escrever CSS.
1. **Grade compartilhada na view de tabela (RF1).** Uma grade para o bloco, `grid-template-columns` montado a partir de `columns`, cabeçalho e células como filhos diretos. Grupo e agregado com `grid-column: 1 / -1`. `list` e `cards` não mudam — não têm coluna.
1. **Índice de cor no core (RF2, RNF1, RNF2, RNF4).** Função pura em `crates/core` — hash estável do valor, módulo N — devolvendo o índice, com testes: mesmo valor sempre no mesmo índice, índice dentro da faixa, e valores que o motor considera iguais caindo na mesma cor (`op: eq` ignora caixa, então `Alta` e `alta` não podem sair diferentes). Nasce no core pelo critério de [[Arquitetura]]: o CLI (158) e o servidor MCP (205) renderizam o mesmo recorte e, se um dia colorirem, precisam concordar com a UI — além de ser testável sem WASM, como o motor de query.
1. **N tokens de cor e aplicação na UI (RF2, RNF1).** Os N pares claro/escuro em `main.css`, a classe por índice, o valor renderizado como badge com o texto dentro. Vale nas três views; título e caminho da página não viram badge.
1. **Teto, rolagem e dica de continuação (RF3, RF5).** `max-height` + `overflow-y: auto` no contêiner de resultados, cabeçalho `sticky`, degradê curto no pé da área rolável que some ao chegar no fim. O rodapé de contagem (ciclo 154) fica fora da área que rola e vira a segunda pista de que há mais coisa.
1. **Altura por consulta (RF4).** Depende da ambiguidade A. As etapas 2–5 não dependem dela, e a 5 lê a altura de um ponto só, de propósito: trocar a origem do valor depois não refaz o resto.
1. **Validação.** Cenário de harness com consulta longa medindo que a altura não passa do teto e que o contêiner rola; conferência ao vivo pelo MCP nos dois temas; `snapshot.mjs` filtrado em `query`, lendo o relatório propriedade a propriedade **antes** de `--atualizar`, com a mudança de baseline registrada na task (é o último critério de aceite da spec).

```bash
cargo test --workspace
cd ui && cargo test --lib && trunk build
node scripts/uitest/snapshot.mjs query
node scripts/uitest/run.mjs
```

## Padrões seguidos

- **[[Nomenclatura]]** — a única página marcada como padrão no vault. Arquivos e funções em `snake_case`, componentes Yew em `PascalCase` com arquivo em `snake_case`, nomes em português no domínio (`indice_de_cor`, `altura_maxima`). A regra do prefixo `handle_` não é acionada: nada aqui atravessa o IPC — a menos que a ambiguidade A se resolva por preferência de app, e aí o handler nasce com o prefixo, espelhando o comando Tauri.
- **Tokens e BEM**, registrados em [[Tema configurável]] e [[Tema e Design]] — nada de hex cru no componente, `color-mix` para translúcidos, par claro/escuro por `data-theme`. As classes novas seguem o BEM que o bloco já usa (`.query-embed__grupo`, `.query-embed__editavel`), com o índice como modificador.
- **[[Arquitetura]]** — "lógica que a UI, o CLI e o servidor MCP precisam concordar desce pro core": o índice de cor desce; paleta e CSS ficam na UI.
- **Ciclo 187** — snapshot por classe, não por pixel. É o que obriga a cor a entrar por classe e não por `style`.

## Alternativas consideradas

- **HSL por hash aplicada inline** (ângulo áureo no matiz). Descartada: contraste vira imprevisível e teria que ser verificado matiz a matiz nos dois temas (RNF4), e a cor no `style` desestabiliza a baseline do snapshot.
- **Mapa valor → cor guardado** (config do app ou frontmatter). Descartada: guardar estado contraria RNF2 diretamente e, no frontmatter, também RNF3. É, na prática, a escolha manual de cor que a spec põe fora de escopo.
- **`<table>` nativa em vez de grade CSS.** Daria o alinhamento de graça e chegou perto. Descartada porque o bloco não é só linhas: grupo e agregado precisariam de `colspan` recalculado a cada mudança de `columns`, enquanto `grid-column: 1 / -1` não depende da contagem.
- **Paginar em vez de rolar.** Descartada: RF3 pede rolagem interna; paginar seria trocar o requisito.
- **Baixar `limit` nas consultas grandes.** Não resolve — esconde resultado em vez de emoldurar, e já dá pra fazer hoje sem mudança nenhuma.

## Ambiguidades a resolver

**A. RF4 contra RNF3 — onde mora a altura por consulta.** RF4 pede altura ajustável por consulta; RNF3 diz que o `.md` não muda. Chave no YAML do embed: fica ao lado de `limit`, `view` e `columns`, versiona no git, viaja com a página — mas o `.md` muda. Preferência do app: o `.md` fica intacto — mas precisa de identidade estável do embed, e a que existe hoje é posicional (`--from-embed pages/...:0`), que quebra ao mover o bloco. Minha leitura é que RNF3 fala do conteúdo das páginas **consultadas**, não da declaração da consulta, o que tornaria a chave no YAML a resposta — mas isso é interpretação, a spec não diz. (a altura por consulta era relacionado a altura da div da consulta quando ela retornava muitos itens limitar o tamanho da div para caso a pessoa nao queira utilizar ela não atrapalhe ela)

**B. RF2 — o que exatamente compartilha cor.** Duas indefinições. O escopo: o texto exige consistência "na mesma página", enquanto RNF2 entrega consistência global de brinde; se global não for desejado, precisa ser dito, porque sai de graça. E a chave do hash: só o valor (`alta`) ou o par campo+valor (`prioridade=alta`)? Muda o resultado visível — com o par, `type: alta` e `prioridade: alta` teriam cores distintas. Assumi o par campo+valor na etapa 3, porque evita colisão acidental entre escalas independentes, mas é suposição minha. (Acho que deve ser  campo+valor (`prioridade=alta`) devido a melhorar distinção, a principio deixa global a configuração para facilitar no uso do dia a dia quie vc defina  prioridade:alta = cor de destaque, vc consiga reaproveitar em varios locais).

## Riscos

- **Baseline do snapshot carimbando regressão.** A mudança reprova de propósito, e `--atualizar` é fácil demais de rodar. Se outro tipo de embed também reprovar, é vazamento de CSS — a classe de defeito que o 187 existe pra pegar.
- **Colisão de hash.** Com N cores e muitos valores distintos, dois valores diferentes recebem a mesma cor. Não quebra RNF1 (o texto continua), mas engana quem lê só pela cor. Sem estado guardado não dá pra eliminar; dá pra reduzir com N folgado.
- **Contraste no tema claro.** [[Tema e Design]] só documenta a paleta escura. Os N pares precisam ser conferidos nos dois temas antes de fixar — badge ilegível em um tema só passa despercebido.
- **Nav-mode contra o cabeçalho sticky.** Os ciclos 133/135/165 levam o foco de teclado pra dentro dos embeds; com área rolável e cabeçalho fixo, rolar até o item focado precisa considerar o contêiner, não a página. O risco concreto é o item focado parar atrás do cabeçalho.
- **Célula editável perto da borda que rola.** A edição em linha do 168 abre um campo dentro da célula; com `overflow`, pode ser cortado na primeira ou última linha visível. Precisa de conferência ao vivo.
- **Coluna espremida por valor longo.** Uma grade compartilhada faz um `path` ou lista de `tags` grande apertar as outras. Truncar resolve, mas é decisão de apresentação que a spec não pediu — se aparecer, vira pergunta, não escolha silenciosa.
