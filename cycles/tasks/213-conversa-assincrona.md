---
id: "213"
titulo: "Conversa assíncrona: sobreviver à navegação, progresso e interromper"
status: done
criado: 2026-08-23
autor: humano
prioridade: alta
depende_de: ["212"]
estima_min: 180
agente_alvo: claude-opus-5
---

# Conversa assíncrona: sobreviver à navegação, progresso e interromper

## Objetivo

Duas perguntas mandadas ao agente não voltaram resposta nenhuma, e não
havia como saber por quê. Três causas somadas:

1. **Timeout de 180s.** Pedir uma proposta faz o modelo ler spec,
   padrões e código antes de escrever — passa fácil de 3 minutos. O
   processo era morto no meio e a conversa ficava muda.
2. **A requisição vivia dentro do componente.** Sair da página
   desmontava o componente e a resposta caía no vazio.
3. **Nenhum sinal de vida.** Sem saída parcial nem tempo decorrido,
   esperar era indistinguível de estar travado.

## Critérios de aceite

- [x] Mandar a pergunta devolve na hora, sem prender a tela
- [x] Timeout padrão de 30 minutos, com piso aplicado na leitura da
      configuração já gravada
- [x] Botão de interromper que mata o processo de verdade
- [x] Sair da conversa não mata a execução
- [x] A resposta é gravada mesmo com a tela fechada
- [x] Voltar pra conversa mostra o que chegou enquanto se estava fora
- [x] Progresso ao vivo: o que o agente está fazendo agora, e há quanto
      tempo
- [x] Uma execução por conversa
- [x] Cenários de harness pra cada um dos itens acima

## Comandos de validação

```bash
cargo test --workspace
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
node scripts/uitest/run.mjs
```

## Não-objetivos

- Fila de execuções. Uma por conversa basta, e é ela que garante que
  nunca há dois escritores no mesmo arquivo.
- Retomar uma execução depois de fechar o app. O processo é filho do
  app; sobreviver a ele é outro problema.

## Notas

### Quem grava a resposta

O backend, não a tela. A tela pode não estar lá — é justamente o que
este ciclo passou a permitir. Se a gravação dependesse dela, a resposta
viveria só na memória até alguém voltar, e sumiria ao fechar o app.

Isso NÃO cria dois escritores: a tela grava a PERGUNTA, o backend grava
a RESPOSTA, e `iniciar_agente` recusa uma segunda execução na mesma
conversa. As duas escritas nunca se cruzam. Foi a preocupação principal
do desenho, por causa do bug do ciclo 209.

### Progresso em tempo real, sem pedir nada ao modelo

`claude -p` segura toda a saída até terminar, então transmitir stdout
cru não mostraria nada. A saída é: `--output-format stream-json
--verbose`, que emite um evento JSON por linha conforme acontece.

`LeitorStream` (no core, testável sem app) separa duas coisas que
servem a públicos diferentes: `progresso()` traduz os eventos em algo
legível ("usando Read", "pensando alto") pra pessoa ver que há
movimento, e `resposta()` extrai o texto final do evento `result`. Sem
evento `result` — agente que fala outro dialeto — cai no texto
acumulado: entregar o que deu é melhor do que falhar por causa do
formato.

O formato é opção do adaptador (`FormatoSaida`), com `Texto` como
padrão, então quem usa codex/opencode não é afetado.

### A gravação nunca CRIA arquivo

`gravar_resposta` só acrescenta a uma conversa que já existe. Em uso
real a página sempre existe — ela é criada antes e recebe a pergunta
antes do disparo. Criar aqui só aconteceria com um path que não é
conversa nenhuma, e foi o que aconteceu na primeira rodada do harness:
os cenários do agente falso apontam pra raiz do repositório, e a
gravação encheu `pages/` de arquivo solto.

### Migração da configuração gravada

A configuração mora no navegador, então quem já usou o app tinha
`timeout_s: 180` e os args antigos — e continuaria com o agente morto
aos 3 minutos e sem progresso, sem ter como saber por quê.

`Adaptador::migrado()` roda na LEITURA e só age sobre o que RECONHECE
como padrão antigo (`nome == "Claude Code"` com args exatamente
`["-p", "{prompt}"]`). Args montados à mão não são tocados; o piso do
timeout vale pra todos.
