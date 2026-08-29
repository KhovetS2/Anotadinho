---
title: Proposta de implementação — imagens persistidas e personalizáveis
tags:
- proposta
- editor
type: proposta
date: 2026-08-28
status: aprovada
---
# Proposta de implementação — imagens persistidas e personalizáveis

{{ type: "fluxo" }}
artefato: proposta
etapa: aprovada

{{ /fluxo }}

## Origem

- [[Imagens arrastadas: persistir como as coladas]]

Esta proposta substitui a abordagem da proposta anterior para esta spec. As decisões posteriores foram: cada inserção cria um asset novo; arrastar abre o fluxo de imagem previamente preenchido; o comando `/imagem` oferece um seletor nativo; e a personalização pode ser persistida como HTML semântico dentro do Markdown.

## Abordagem

Criar um único modal de inserção de imagem, aberto pelo comando `/imagem` e pelo arraste de arquivos de imagem. O comando abre o seletor nativo do sistema e preenche o modal com os arquivos escolhidos. O arraste abre diretamente o mesmo modal, já preenchido com os arquivos soltos e com o bloco e a posição de inserção preservados.

O modal permitirá revisar cada imagem antes de confirmar e editar texto alternativo, legenda, título, largura, altura, alinhamento e proporção. Em seleções múltiplas, cada arquivo terá seus próprios campos, permanecerá na ordem recebida e poderá ser removido da operação antes da confirmação. Arquivos que não forem imagens serão ignorados e não abrirão nem alterarão a nota quando forem os únicos itens recebidos.

Texto alternativo e título serão atributos de `<img>`. Legenda será persistida em `<figcaption>`. Largura, altura, alinhamento e proporção serão convertidos para uma representação HTML semântica e estável em `<figure>` e `<img>`, usando apenas atributos e classes reconhecidos pelo renderizador do Anotadinho. O Markdown continuará legível fora do app, embora essas inserções deixem de ser sintaxe de imagem Markdown pura. Quando nenhum campo que exija `<figure>` for usado, a implementação poderá manter a forma HTML mínima, desde que a serialização e a reabertura produzam o mesmo resultado.

A confirmação será transacional do ponto de vista da nota: primeiro validar todos os campos e a posição de destino; depois gravar cada arquivo em `assets/` com nome novo e sem sobrescrita; por fim inserir todas as referências em uma única alteração do editor. Cada confirmação cria assets diferentes, mesmo quando a origem ou os bytes forem idênticos. Se uma gravação falhar, nenhuma referência quebrada será inserida e o modal permanecerá aberto com erro acionável. Assets gravados antes de uma falha parcial deverão ser removidos ou a operação deverá usar uma etapa temporária que só os publique após todas as gravações concluírem.

Colar imagem reutilizará o mesmo serviço de persistência e inserção, mas conservará o gesto direto previsto na spec: detecta somente itens de imagem, grava assets novos e insere com valores padrão. Colar texto não será interceptado. Assim, drop e `/imagem` passam pelo modal de personalização, enquanto paste continua rápido e produz a mesma referência durável. Nenhum caminho usará `execCommand`, URL `blob:` persistida ou inserção anterior à gravação.

## Etapas

1. Criar o ciclo de implementação com os critérios desta spec, registrar que esta proposta substitui a abordagem anterior e não avançar etapas de fluxo em nome da pessoa.
2. Reproduzir no harness o paste e o drop atuais com arquivos reais, localizando o caminho do `blob:`, o uso de `execCommand` e a falha do editor por bloco.
3. Consolidar no backend uma operação de asset por inserção: validar tipo e bytes, gerar nome sempre novo, impedir sobrescrita e oferecer confirmação conjunta ou limpeza segura para lote parcialmente falho.
4. Definir no core o modelo de imagem inserida e sua serialização HTML: fonte relativa ao vault, texto alternativo, título, legenda, dimensões, alinhamento e proporção; cobrir parse e round-trip sem perda ao fechar e reabrir.
5. Criar o modal de imagem e o estado de uma ou várias imagens, com preview local apenas durante a edição, campos acessíveis, validação, remoção por item, cancelamento sem efeito e navegação por teclado com `data-nav-item` e `data-nav-group`.
6. Integrar `/imagem` ao seletor nativo do sistema e abrir o modal com os arquivos selecionados. Cancelar o seletor ou o modal não grava asset nem modifica a nota.
7. Substituir o drop atual: mostrar o alvo durante o arraste, filtrar itens, capturar o bloco e a seleção reais e abrir o modal preenchido, sem inserir conteúdo no DOM antes da confirmação.
8. Corrigir paste no editor completo e por bloco usando o serviço comum. Interceptar apenas quando houver imagem; manter o paste de texto no caminho atual.
9. Inserir o HTML pelo mecanismo próprio do editor, marcar o documento como editado e agrupar uma confirmação múltipla em uma alteração que o undo remova da nota.
10. Renderizar novamente o HTML persistido com os campos de apresentação e garantir que caminhos relativos de assets sejam resolvidos sem converter a fonte persistida em `blob:`.
11. Adicionar testes do backend e do core para colisão, duplicidade intencional, falha parcial, validação, serialização e round-trip de todos os campos.
12. Adicionar cenários de harness para seletor, cancelamento, drop simples e múltiplo, arquivo não-imagem, modal preenchido, personalizações, paste real por bloco, paste de texto, undo, erro de gravação, ausência de `blob:` e reabertura da página.
13. Executar `cargo test --workspace`, `cd ui && trunk build`, `cargo build --manifest-path src-tauri/Cargo.toml` e `node scripts/uitest/run.mjs`. Conferir visualmente modal, alvo de drop e apresentação das imagens antes de qualquer atualização de snapshot.

## Padrões seguidos

- **Editor:** usar `insert_element_at_cursor` ou a evolução compatível desse mecanismo; nunca `execCommand`. A posição assíncrona será guardada sem capturar `use_state` congelado.
- **Vault:** assets permanecem locais em `assets/`, com caminhos relativos e nomes novos. O backend é responsável pela gravação binária segura.
- **HTML no Markdown:** `<figure>`, `<img>` e `<figcaption>` preservam a legibilidade externa e carregam os campos que Markdown puro não representa.
- **UI:** classes BEM, tokens existentes e `color-mix` quando necessário; qualquer ícone novo entra no componente central de ícones.
- **Acessibilidade:** modal, lista de imagens, campos e ações participam da navegação por teclado e expõem rótulos compreensíveis.
- **Validação:** mudanças de DOM e estilo só podem concluir o ciclo depois do harness contra o app real e da inspeção visual.

## Alternativas consideradas e descartadas

- **Salvar imediatamente no drop:** descartada porque a decisão é abrir o fluxo de imagem preenchido e só produzir efeito após confirmação.
- **Markdown puro para todas as opções:** descartada porque não representa legenda, alinhamento, dimensões e proporção de forma suficiente.
- **Reutilizar asset por hash ou origem:** descartada porque cada inserção deve criar um arquivo diferente, inclusive para conteúdo idêntico.
- **Inserir `blob:` e substituir depois:** descartada porque pode vazar para o arquivo e quebrar após encerrar a sessão.
- **Persistir antes da confirmação:** descartada porque cancelar o modal não deve deixar assets órfãos.
- **Misturar o seletor com leitura manual de caminhos na UI:** descartada porque o seletor nativo e o backend devem manter os limites de acesso e validação do Tauri.
- **Abrir o modal também para paste:** descartada nesta abordagem para preservar a rapidez do gesto e o requisito de não regredir o fluxo de colagem; paste usa valores padrão e o mesmo serviço durável.

## Riscos

- O renderizador e o serializador atuais podem normalizar ou remover atributos HTML; o round-trip precisa ser provado antes de ligar a UI aos campos.
- Largura, altura e proporção podem entrar em conflito. O modal deve definir precedência visível, preservar proporção por padrão e rejeitar combinações impossíveis, sem editar os bytes da imagem.
- Alinhamento depende de classes de apresentação que outros leitores de Markdown podem ignorar. O conteúdo e a referência continuam legíveis, mas a aparência exata fora do app não é garantida.
- Preview de arquivos grandes pode consumir memória. URLs temporárias devem existir apenas no modal, ser revogadas ao trocar ou fechar e nunca participar da serialização.
- Capturar a posição antes de abrir um modal assíncrono pode deixá-la inválida se o bloco mudar. A confirmação deve revalidar o destino antes de gravar.
- Uma falha em lote pode deixar assets órfãos se o backend não oferecer publicação atômica ou limpeza. Essa garantia deve ser resolvida antes da inserção no DOM.
- O seletor nativo e eventos sintéticos de clipboard/drop têm limitações distintas no harness; os testes devem usar arquivos reais e separar a cobertura do diálogo da cobertura do processamento.

## Ajustes necessários na spec antes da execução

A decisão aceita altera dois trechos ainda vigentes na spec: RF1 fala em gravação ao arrastar, embora agora ela ocorra somente após confirmação no modal; e RNF1 exige “markdown comum”, embora os campos completos sejam persistidos em HTML semântico dentro do Markdown. O item fora de escopo sobre redimensionar também precisa esclarecer que dimensões de apresentação são permitidas, enquanto editar os bytes da imagem continua fora de escopo. Esses ajustes devem ser propostos separadamente para manter spec e implementação coerentes.
