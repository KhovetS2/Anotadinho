---
title: Proposta de implementação — Prompts padrão reutilizáveis
tags:
- proposta
- agent-os
type: proposta
date: 2026-08-24
status: aprovada
---
# Proposta de implementação — Prompts padrão reutilizáveis

{{ type: "fluxo" }}
artefato: proposta
etapa: aprovada
origem: pages/conversas/conversa-2026-08-23-21-05.md

{{ /fluxo }}

## Abordagem

Tratar prompts padrão como páginas Markdown comuns, identificadas pela combinação de `type: prompt` no frontmatter e localização em `pages/prompts-default/` ou em uma de suas subpastas. A descoberta ficará na camada de vault e será exposta à conversa pelos comandos já usados pela UI, sem criar armazenamento ou tela de configuração paralelos.

O core concentrará a interpretação do molde. Ele extrairá marcadores nomeados no formato `{{title}}`, preservará a ordem da primeira ocorrência de cada nome, pedirá um único valor por variável e reutilizará esse valor em todas as ocorrências posteriores. A expansão validará todos os campos antes de produzir o resultado; marcador sem valor será um erro explícito, nunca texto literal enviado silenciosamente.

No compositor da conversa haverá um seletor acima da ação de envio. A opção vazia conservará o fluxo atual de escrita livre. Ao escolher um prompt, o texto já digitado preencherá a primeira variável distinta encontrada no molde; se o compositor estiver vazio, essa variável também será solicitada. As demais variáveis serão pedidas na ordem da primeira ocorrência. Se o prompt não tiver marcador, o texto digitado será acrescentado ao final do conteúdo do prompt. O resultado só será enviado pela ação normal de envio. Uma ação separada abrirá um modal de visualização com o texto final já substituído, sem disparar a conversa.

A montagem da requisição manterá a blindagem do ciclo 202: valores fornecidos pela pessoa e conteúdos das páginas anexadas entrarão como DADO, sem adquirir autoridade de instrução por terem sido interpolados por um molde. O prompt expandido continuará visível antes do envio, mas a representação interna preservará a separação entre instruções do molde e dados interpolados.

As páginas de contexto serão declaradas no campo `contexto:` do frontmatter do prompt, no mesmo formato usado pelas páginas de conversa. Ao escolher o prompt, essas páginas serão incorporadas ao `contexto:` da conversa pelo mecanismo de anexos existente, com validação de existência e remoção de duplicatas, e permanecerão anexadas à conversa como ocorre no fluxo atual.

## Etapas

1. Consolidar o contrato do prompt: página com `type: prompt` dentro de `pages/prompts-default/`; variáveis `{{nome}}`; `contexto:` no frontmatter com o mesmo formato das conversas; anexos incorporados de forma persistente à conversa; rascunho preenchendo a primeira variável distinta; e, na ausência de marcador, rascunho acrescentado ao final do molde.
2. Criar no core o modelo de prompt padrão e funções puras para extrair variáveis, deduplicá-las pela primeira ocorrência, validar valores, expandir todas as ocorrências e identificar campos pendentes.
3. Cobrir o motor com testes de marcador único, três variáveis em ordem, variável repetida, valor multilinha, marcador ausente, texto sem marcador e conteúdo que tente interferir na blindagem de DADO.
4. Implementar no vault a descoberta recursiva restrita a `pages/prompts-default/`, filtrando simultaneamente por `type: prompt`, e testar exclusões por pasta e por tipo.
5. Expor por IPC e pela API da UI a listagem dos prompts e a leitura do prompt escolhido, mantendo o espelhamento de nomes e tipos entre as camadas existentes.
6. Adicionar ao compositor o seletor acima de “Enviar”, com opção vazia, estados de carregamento/erro e atributos `data-nav-item` e `data-nav-group` necessários à navegação por teclado.
7. Implementar o preenchimento das variáveis na ordem retornada pelo core: usar o rascunho existente como valor da primeira variável distinta, solicitar as restantes, pedir todas quando o compositor estiver vazio, não duplicar campos repetidos e não alterar o rascunho quando a pessoa cancelar. Para prompts sem marcador, acrescentar o rascunho ao final do conteúdo do prompt.
8. Adicionar a ação de visualização e o modal do prompt final. Impedir a visualização conclusiva e o envio enquanto houver marcador sem valor; o modal nunca envia automaticamente.
9. Ler o campo `contexto:` do frontmatter do prompt e integrar suas páginas ao `contexto:` persistente da conversa pelo fluxo atual de anexos, validando páginas ausentes e evitando duplicatas.
10. Garantir na montagem final que valores interpolados e páginas anexadas continuem delimitados como DADO, inclusive quando contiverem os próprios delimitadores usados pela blindagem.
11. Acrescentar cenários ao harness para marcador único, múltiplas variáveis, variável repetida, marcador faltando, opção vazia, filtro por pasta e tipo, preview sem envio e contexto declarado.
12. Executar `cargo test --workspace`, `cd ui && trunk build`, `cargo build --manifest-path src-tauri/Cargo.toml` e `node scripts/uitest/run.mjs`. Só concluir o ciclo após o harness rodar contra o app real e a alteração visual ser conferida.

## Padrões seguidos

- **[[Spec, proposta e execução]]** — a proposta descreve o caminho técnico sem alterar os requisitos e registra as decisões confirmadas para contexto, rascunho e prompts sem marcador antes da execução.
- **[[Nomenclatura]]** — tipos e componentes Yew usarão `PascalCase`; arquivos, módulos, funções e variáveis usarão `snake_case`; handlers IPC seguirão o prefixo já adotado pelo projeto.
- **Templates de página, ciclos 100 e 112** — os marcadores mantêm a convenção legível `{{nome}}`, sem acrescentar condicionais, laços ou outra linguagem de templates.
- **Conversa com agente, ciclo 202** — a montagem final preserva a distinção entre instruções e DADO para o texto interpolado e para páginas de contexto.
- **Páginas normais do vault** — prompts permanecem Markdown editável pelo editor existente, sem cadastro duplicado em preferências ou banco separado.
- **Navegação do app** — controles novos participam do sistema existente por `data-nav-item` e `data-nav-group`.
- **Estado assíncrono da UI** — seleções e valores lidos por callbacks ou efeitos usarão referência atualizada quando puderem mudar depois da criação da closure, evitando captura congelada de `use_state`.

## Alternativas consideradas e por que foram descartadas

- **Tela própria de cadastro de prompts:** descartada porque prompts devem ser páginas normais e editáveis no fluxo existente.
- **Descobrir qualquer página com `type: prompt`:** descartada porque a spec exige também a localização em `pages/prompts-default/` ou subpastas.
- **Descobrir qualquer arquivo da pasta reservada:** descartada porque localização e `type: prompt` são critérios simultâneos.
- **Substituição direta por `String::replace`:** descartada porque não modela ordem, deduplicação e campos ausentes de forma segura, além de misturar instruções e dados.
- **Solicitar um valor por ocorrência:** descartada porque ocorrências da mesma variável devem reutilizar um único valor.
- **Enviar ao concluir o preenchimento:** descartada porque o resultado precisa permanecer visível e o preview não pode enviar automaticamente.
- **Deixar marcadores pendentes no texto:** descartada porque a ausência precisa ser apontada antes do envio.
- **Criar uma linguagem de template completa:** descartada porque condicionais e laços estão fora de escopo.
- **Criar uma declaração própria de contexto para prompts:** descartada porque o formato `contexto:` das páginas de conversa já foi definido como contrato para essas páginas.
- **Anexar contexto somente no envio atual:** descartada porque os anexos devem seguir o comportamento normal das conversas e permanecer no `contexto:` delas.
- **Solicitar novamente a primeira variável quando já existe rascunho:** descartada porque o texto do compositor deve preencher a primeira variável distinta encontrada.
- **Ignorar o rascunho em prompt sem marcador:** descartada porque, nesse caso, ele deve ser acrescentado ao final do conteúdo do prompt.

## Riscos

- **Colisão com sintaxe existente:** `{{title}}` também aparece em templates de criação de páginas. O parser precisa operar apenas no corpo de páginas já reconhecidas como prompts padrão.
- **Separação entre instrução e DADO:** exibir um texto expandido único sem perder a origem de cada trecho exige uma representação interna que mantenha os valores interpolados identificáveis até a montagem final.
- **Rascunho preexistente com várias variáveis:** preencher automaticamente a primeira variável altera o papel aparente do texto conforme a ordem do molde; o preview deve deixar essa associação visível antes do envio.
- **Prompt sem marcador:** acrescentar o rascunho ao final precisa preservar uma separação Markdown legível, sem fundir a última linha do molde com o conteúdo digitado.
- **Contexto ausente ou renomeado:** uma referência pode deixar de existir entre a seleção e o envio; o erro precisa surgir antes do disparo.
- **Persistência dos anexos:** como o contexto do prompt passa ao `contexto:` da conversa, ele afetará mensagens futuras; a UI deve refletir imediatamente os anexos incorporados e evitar duplicatas.
- **Estado congelado na UI:** callbacks encadeados podem ler rascunho, prompt ou valores antigos; o fluxo precisa usar estado atual durante seleção, preview e envio.
- **Regressão visual e de teclado:** o seletor e o modal alteram o compositor e precisam de harness e inspeção visual contra o app real.
