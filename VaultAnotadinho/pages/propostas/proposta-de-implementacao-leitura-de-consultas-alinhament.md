---
title: 'Proposta de implementação — leitura de consultas: alinhament'
tags:
- proposta
type: proposta
date: 2026-08-23
status: aprovada
---
# Proposta de implementação — leitura de consultas: alinhament

{{ type: "fluxo" }}
artefato: proposta
etapa: aprovada
origem: pages/conversas/conversa-2026-08-23-07-04.md

{{ /fluxo }}

# Proposta de implementação — leitura de consultas: alinhamento, cor, altura e virtualização

## Abordagem

A implementação permanece concentrada na apresentação das consultas, sem alterar o conteúdo dos arquivos `.md` consultados nem o motor que seleciona resultados.

O alinhamento da tabela será corrigido preservando o elemento `<table>` existente. Embora cabeçalho e células usem elementos de tabela, o CSS atual aplica `display: flex` às linhas do corpo (`<tr>`), retirando-as do algoritmo de layout tabular. A correção separa os estilos de linhas de lista das linhas de tabela, para que `<th>` e `<td>` compartilhem as mesmas larguras de coluna novamente.

As propriedades serão exibidas como badges de texto com cor derivada de modo determinístico. A chave será formada por `coluna + campo + valor`, normalizada para que valores considerados iguais pela consulta também recebam a mesma cor. Em listas e cartões, onde não existe uma coluna de tabela, a chave usa `campo + valor`. A paleta ficará em tokens CSS, com variantes para os temas claro e escuro; o texto continua visível e legível.

Cada consulta terá uma área de resultados com altura máxima e rolagem interna. O cabeçalho da tabela ficará fixo durante essa rolagem, e um degradê no limite inferior indicará visualmente quando houver mais conteúdo.

A altura será configurável na declaração YAML do próprio embed, junto de opções como `view`, `limit` e `columns`. Isso altera a declaração da consulta, mas não os dados ou páginas retornados por ela.

A virtualização será aplicada somente quando a consulta tiver muitos resultados, não estiver agrupada e estiver nas visualizações Lista ou Tabela. Para torná-la previsível, essas linhas terão altura fixa e conteúdo truncado em uma linha; o valor integral continuará disponível por tooltip e durante a edição. A janela renderizada será calculada pela altura útil da área de resultados, pela altura fixa da linha e por uma margem antes/depois da área visível.

## Etapas

1. Levantar a estrutura atual de `inline_query.rs`, incluindo Lista, Tabela, Cartões, agrupamento, agregados, edição em linha e navegação por teclado.

2. Corrigir o alinhamento da Tabela:
   - separar a classe/estilo da linha de lista da linha `<tr>`;
   - remover o `display: flex` aplicado às linhas de tabela;
   - confirmar que cabeçalho e corpo voltam a compartilhar as larguras das colunas pelo layout nativo do navegador.

3. Criar no core a função pura que calcula o índice de cor:
   - normalizar coluna, campo e valor;
   - gerar um hash estável;
   - reduzir o resultado ao número de cores disponível;
   - testar determinismo, faixa válida e equivalência entre valores normalizados.

4. Criar tokens e modificadores BEM para a paleta de badges nos temas claro e escuro.

5. Aplicar badges de propriedade às três visualizações:
   - Tabela: valores das células de propriedade;
   - Lista e Cartões: propriedades exibidas no resultado;
   - título e caminho da página permanecem texto normal.

6. Criar o contêiner rolável de resultados:
   - `max-height` e `overflow-y: auto`;
   - cabeçalho de tabela `sticky`;
   - rodapé de contagem fora da área rolável;
   - degradê inferior que desaparece ao alcançar o fim do conteúdo.

7. Adicionar a opção YAML de altura máxima por consulta e validar seu parsing e valor padrão.

8. Implementar virtualização por quantidade de resultados:
   - ativar apenas acima do limiar definido, inicialmente 100 resultados;
   - limitar a Lista e a Tabela sem agrupamento;
   - usar linhas de altura fixa, uma linha visual e truncamento;
   - calcular itens visíveis por `ceil(altura_da_area / altura_da_linha)`;
   - renderizar itens visíveis mais uma margem de cinco itens antes e depois;
   - usar espaçadores superior e inferior para preservar a altura total da rolagem;
   - ajustar a janela renderizada antes de focar um item fora dela pela navegação de teclado.

9. Validar visualmente, funcionalmente e pelos builds/testes. Atualizar snapshots apenas quando a mudança esperada estiver identificada e registrada na task.

## Padrões seguidos

- **Arquitetura**: o índice determinístico de cor fica no core, por ser lógica testável e potencialmente compartilhável; tokens e apresentação permanecem na UI.
- **Nomenclatura**: funções e arquivos em `snake_case`; componentes Yew em `PascalCase`; nomes de domínio em português quando já for a convenção do módulo.
- **Tema configurável / Tema e Design**: cores exclusivamente por tokens CSS e variantes de `data-theme`, sem hexadecimais no componente.
- **BEM**: modificadores como `.query-embed__valor--cor-N`, mantendo a convenção já usada pelo embed.
- **Ciclo 187 / snapshots**: cores por classes, e não por `style` inline, para que o snapshot de estilos computados continue estável.

## Alternativas consideradas e descartadas

- **Transformar a tabela em CSS Grid**: descartado. A tabela já tem semântica e mecanismo nativo para manter colunas alinhadas; o problema real é o `display:flex` indevido no `<tr>`.
- **Cor por HSL calculado inline**: descartado por dificultar a garantia de contraste em ambos os temas e por tornar snapshots menos estáveis.
- **Mapa persistido de valor para cor**: descartado, pois introduz estado e contraria o requisito de determinismo sem armazenamento.
- **Paginação ou redução de `limit`**: descartadas porque substituem a rolagem interna pedida pela spec.
- **Virtualização com linhas de altura variável**: descartada nesta proposta. Exigiria medição dinâmica, ancoragem de scroll e tratamento específico para agrupamento e edição.
- **Virtualizar cartões e resultados agrupados**: descartado. Esses formatos não têm linhas uniformes, portanto não são compatíveis com o cálculo de espaçadores por altura fixa.

## Riscos

- **Truncamento na virtualização**: consultas virtualizadas deixam de mostrar valores longos em múltiplas linhas. O texto completo depende do tooltip ou da edição.
- **Edição em linha**: o campo editável precisa respeitar a altura fixa e não ser recortado pelo contêiner rolável.
- **Navegação por teclado**: o foco deve atualizar a janela virtualizada antes de tentar alcançar uma linha ainda não montada no DOM.
- **Contraste de badges**: cada token precisa ser conferido nos temas claro e escuro.
- **Colisões de hash**: valores distintos podem receber a mesma cor; isso não compromete a leitura porque o texto permanece visível.
- **Snapshots visuais**: a mudança esperadamente altera estilos das consultas; qualquer alteração fora delas indicará possível vazamento de CSS.
- **Altura por consulta**: a configuração no YAML do embed precisa ter sintaxe e valor padrão definidos durante a implementação; caso o formato atual não comporte essa opção, a execução deve parar para nova proposta.
