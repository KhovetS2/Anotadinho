---
title: "Planejar: Imagens arrastadas: persistir como as coladas"
type: conversa
origem: pages/specs/imagens-coladas-e-arrastadas.md
tags:
- conversa
contexto:
- pages/specs/imagens-coladas-e-arrastadas.md
- pages/padroes/escrita-no-vault.md
---
## você · 2026-08-24 19:41

Leia a spec "Imagens arrastadas: persistir como as coladas" e as páginas de padrão anexadas.

Escreva uma PROPOSTA DE IMPLEMENTAÇÃO com esta estrutura:

1. Título na primeira linha.
2. Abordagem: como atender os requisitos.
3. Etapas, na ordem de execução.
4. Padrões seguidos: cite os anexados e como a abordagem se encaixa.
5. Alternativas consideradas e por que foram descartadas.
6. Riscos.

Não proponha requisitos novos nem mude o escopo — isso é da spec. Se algum requisito estiver ambíguo, aponte a ambiguidade em vez de decidir por conta.

## agente · 2026-08-24 19:42

# Proposta de implementação — Imagens coladas e arrastadas persistidas como assets

## Abordagem

Unificar colar e arrastar imagens em um único fluxo de inserção no editor:

1. identificar arquivos de imagem no evento;
2. capturar e validar uma seleção real no bloco de destino;
3. ler os bytes de forma assíncrona;
4. solicitar ao backend a gravação segura em `assets/`, com nome único;
5. inserir no cursor um elemento de imagem pelo mecanismo próprio do editor;
6. deixar o fluxo normal do navegador intacto quando o evento não contiver imagens.

O caminho já criado no ciclo 118 deve ser corrigido e generalizado, evitando duas implementações independentes para paste e drop. A referência persistida continuará sendo markdown comum, como `![](assets/nome.ext)`, enquanto a representação no DOM será inserida por `insert_element_at_cursor`, nunca por `execCommand`.

A seleção ou posição de destino precisa ser confirmada antes de gravar o arquivo. Depois da gravação, a inserção deve integrar-se ao mecanismo existente de edição e snapshots, para marcar o documento como alterado, persistir o markdown e permitir desfazer a referência.

Para vários arquivos, o fluxo processará apenas os itens reconhecidos como imagem e preservará sua ordem na inserção. Arquivos de outros tipos serão ignorados. A leitura e a gravação serão assíncronas, mantendo a janela responsiva.

O estado visual de arraste será controlado pelos eventos de entrada, permanência, saída e soltura sobre o editor. Sua apresentação usará as classes BEM e os tokens existentes.

Duas decisões da spec permanecem ambíguas e precisam ser tomadas pela pessoa antes da execução:

- se inserir novamente a mesma imagem deve criar outro asset ou reutilizar o existente;
- se o drop deve salvar e inserir imediatamente ou abrir o fluxo do comando `/imagem`.

A segunda decisão altera diretamente a arquitetura proposta: o fluxo unificado acima pressupõe a inserção imediata exigida pelo RF1. Se for escolhido abrir `/imagem`, a proposta precisa ser revista antes da implementação.

## Etapas

1. Criar o ciclo de implementação e registrar os critérios da spec, as validações e os não-objetivos, sem avançar a etapa do fluxo da proposta.

2. Reproduzir o comportamento atual de colar e arrastar com imagens reais, incluindo o editor por bloco, para localizar a falha do ciclo 118 e confirmar por onde o `blob:` e o `execCommand` ainda entram.

3. Cobrir o armazenamento no backend com testes para bytes de imagem, geração de nomes sem colisão, preservação de arquivos existentes e propagação de falhas. A implementação concreta da duplicidade dependerá da decisão pendente da spec.

4. Extrair no frontend o fluxo comum de persistência e inserção usado por paste e drop. Esse fluxo deverá receber os arquivos, uma seleção válida e o bloco de destino; salvar cada imagem; inserir os elementos na ordem; sinalizar a edição; e comunicar erros sem inserir referências para gravações que falharam.

5. Corrigir o handler de paste nos contenteditables por bloco. Ele só deverá interceptar o evento quando houver imagem, preservando integralmente o paste nativo de texto.

6. Substituir o handler de drop baseado em `blob:` e `execCommand` pelo fluxo comum. Filtrar arquivos não-imagem, suportar múltiplas imagens e remover qualquer possibilidade de uma URL `blob:` chegar à serialização markdown.

7. Implementar o estado visual do alvo de drop usando classes BEM e tokens do design system, garantindo também a limpeza desse estado após saída, soltura ou erro.

8. Integrar a inserção ao histórico existente do editor e verificar que o primeiro undo após a operação remove a referência inserida.

9. Adicionar cenários ao harness cobrindo:

   - arraste de uma imagem real, criação do asset e persistência da referência;
   - fechamento e reabertura da página;
   - ausência de `blob:` no markdown;
   - múltiplas imagens;
   - arquivo não-imagem;
   - falha de gravação sem referência quebrada;
   - nome sem colisão;
   - undo;
   - paste de imagem real no editor por bloco;
   - paste de texto sem regressão;
   - estado visual do alvo de drop.

10. Executar `cargo test --workspace`, `trunk build`, o build do Tauri e o harness completo com o app mantido aberto pela pessoa. Conferir visualmente o estado de drop antes de qualquer eventual atualização de snapshot. Somente depois de toda a validação concluir, finalizar o ciclo, registrar o status e fazer o commit.

## Padrões seguidos

- **Editor e DOM:** elimina `execCommand`, usa `insert_element_at_cursor`, mantém um `contenteditable` por bloco e exige uma seleção real antes de gravar o asset. A serialização continuará partindo do DOM do bloco, sem criar um segundo estado concorrente.

- **Escrita no vault:** mantém um único escritor para o conteúdo da nota e delega a gravação binária ao backend. Não introduz montagem manual de YAML nem gravação direta de markdown por um caminho paralelo.

- **Validação:** inclui testes de backend, builds e harness contra o app real. Como há mudança de DOM e interação, o ciclo não poderá ser considerado validado sem o harness e a conferência visual do alvo de drop.

- **Spec, proposta e execução:** reaproveita o que já existe no ciclo 118, separa a abordagem dos requisitos e não decide as duas perguntas em aberto. Se a opção escolhida inviabilizar o fluxo comum proposto, a execução deve parar e a proposta ser revista.

- **Ciclo 118 — Colar imagem no editor vira asset:** reaproveita o comando de persistência, a leitura assíncrona de `File` e `insert_element_at_cursor`, corrigindo o comportamento de paste e generalizando-o para drop.

- **Ciclo 096 — Gestão de assets:** mantém os arquivos dentro de `assets/` e preserva as garantias de caminho seguro e de não sobrescrita usadas pela gestão do acervo.

## Alternativas consideradas e descartadas

- **Manter paste e drop como implementações separadas:** descartada porque repetiria leitura, persistência, tratamento de erros e inserção, facilitando nova divergência entre os dois caminhos.

- **Continuar usando uma URL `blob:`:** descartada porque a URL só vale durante a sessão e já foi confirmado que produz referência quebrada ao reabrir a página.

- **Usar `execCommand("insertHTML")`:** descartada porque viola o padrão do editor e pode corromper o DOM.

- **Inserir a referência antes de salvar o arquivo:** descartada porque uma falha de gravação deixaria a nota com uma referência quebrada, contrariando o RF6.

- **Interceptar todo evento de paste:** descartada porque quebraria ou substituiria o fluxo nativo de colar texto, contrariando o RNF2.

- **Ler ou gravar imagens de forma síncrona:** descartada porque imagens grandes poderiam bloquear a janela, contrariando o RNF5.

- **Escolher agora entre duplicar e reutilizar o asset, ou entre inserção imediata e `/imagem`:** descartada porque ambas são ambiguidades explicitamente reservadas pela spec à decisão humana.

## Riscos

- A seleção pode mudar enquanto a leitura e a gravação assíncronas acontecem. Ela precisará ser capturada de forma compatível com o editor, sem recorrer a estado congelado em closures.

- Em uma operação com vários arquivos, algumas gravações podem concluir e outras falhar. A referência de um arquivo que falhou não pode ser inserida; a política para assets já gravados durante uma operação parcialmente malsucedida deve seguir o comportamento existente, sem ampliar o escopo.

- Uma gravação pode concluir e a inserção no DOM falhar posteriormente, deixando um asset sem referência. Validar a seleção antes do efeito colateral reduz esse risco, mas não torna as duas operações transacionais.

- O processamento assíncrono de vários arquivos pode alterar a ordem visual se as inserções forem feitas conforme cada gravação termina.

- A correção do paste pode afetar o caminho de texto mais utilizado do editor. O handler deve permanecer condicionado à presença de imagem e o harness precisa cobrir explicitamente essa regressão.

- O histórico por snapshots pode agrupar ou separar incorretamente as inserções múltiplas, afetando o comportamento do undo.

- Eventos de drag podem atravessar elementos filhos e causar oscilação no indicador visual. O estado precisa considerar a entrada e a saída efetivas da área do editor.

- A escolha pendente de abrir `/imagem` em vez de inserir imediatamente é incompatível com parte central desta abordagem e exige revisão da proposta antes da execução.

## você · 2026-08-24 19:43

Execute a proposta "Proposta de implementação — Imagens coladas e arrastadas per", que já foi revisada e aprovada.

Ao terminar, relate:

1. O que foi feito, etapa por etapa.
2. Como foi validado.
3. O que ficou de fora, e por quê.

Siga a abordagem aprovada. Se durante a execução ela se mostrar inviável, PARE e explique o problema em vez de mudar de rumo por conta — mudar a abordagem exige uma proposta nova.

## agente · 2026-08-24 19:43

A execução está bloqueada antes da etapa 1: a proposta aprovada ainda mantém duas decisões explicitamente pendentes, e determina que não sejam escolhidas pelo agente.

Preciso que você defina:

1. Ao inserir a mesma imagem novamente: criar um novo asset ou reutilizar o existente?
2. Ao soltar uma imagem: salvar e inserir imediatamente, ou abrir o fluxo de `/imagem`?

A segunda opção (`/imagem`) exige revisar a proposta, pois conflita com a abordagem aprovada de inserção imediata. Nenhum arquivo foi alterado.

## você · 2026-08-24 19:46

E como funciona o fluxo de /imagem?

## agente · 2026-08-24 19:46

Hoje, o fluxo de `/imagem` funciona assim:

1. A pessoa digita `/imagem` no editor e escolhe **Imagem — URL ou arquivo de imagem**.
2. O texto do comando é removido.
3. Abre-se um campo pedindo **“Caminho da imagem ou URL”**.
4. Se for uma URL iniciada por `http`, o editor insere diretamente um `<img>` apontando para ela. A imagem não é copiada para o vault.
5. Se for um caminho local, o backend copia o arquivo para `assets/` preservando o nome original.
6. Após a cópia, o frontend insere a imagem no cursor com `insert_element_at_cursor`.
7. O DOM é convertido novamente em markdown, o documento é marcado como editado e depois segue o salvamento normal da nota.

Portanto, `/imagem` é um fluxo com ação posterior e confirmação manual: o drop abriria o diálogo e a pessoa ainda teria que fornecer um caminho ou URL. Ele não recebe automaticamente o arquivo que está sendo arrastado.

Há ainda diferenças importantes em relação ao comportamento exigido pela spec:

- O drop imediato já possui o próprio arquivo e seus bytes; pedir novamente o caminho seria redundante.
- O caminho local de `/imagem` usa `copy_to_assets`, que atualmente pode sobrescrever um asset de mesmo nome. Isso não atende ao RF7.
- Erros na cópia local são atualmente ignorados silenciosamente, enquanto o RF6 exige comunicar o erro.
- URLs externas são aceitas por `/imagem`, mas baixar imagens de URL está explicitamente fora do escopo desta spec.
- O fluxo aceita uma entrada por vez, enquanto o drop precisa suportar vários arquivos.
- A gravação ocorre assincronamente e a inserção usa a seleção existente quando termina, o que merece atenção caso o cursor mude nesse intervalo.

Assim, relacionar o drop ao fluxo de `/imagem` pode significar duas coisas distintas:

- **Reutilizar a infraestrutura interna:** salvar em `assets/`, inserir pelo mecanismo seguro do editor, recomputar markdown e marcar como editado. Isso é compatível com a proposta aprovada.
- **Abrir literalmente o diálogo de `/imagem`:** isso muda o drop para uma operação em duas etapas e não aproveita diretamente o arquivo arrastado. Também entra em tensão com o RF1, que descreve salvar e inserir ao arrastar.

Minha leitura técnica é que “relacionar” pode ser atendido compartilhando a infraestrutura, sem abrir o diálogo. Porém, como a spec deixou essa experiência explicitamente em aberto, essa escolha ainda precisa ser feita pela pessoa antes da execução. Nenhum arquivo foi alterado.
