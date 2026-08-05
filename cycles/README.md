# 🌀 Sistema de Ciclos

O Anotadinho evolui por **ciclos**, não por commits avulsos. Cada ciclo
implementa exatamente uma feature, é isolado, validado, e commitado
depois de passar em todos os testes.

## Conceito

```
┌──────────────────────────────────────────────────────────────┐
│  cycles/orchestrator.sh run [task_id]                        │
└──────────────────────────────────────────────────────────────┘
        │
        ▼
   1. Lê próxima task pendente (ou especificada)
        │
        ▼
   2. Marca como `in_progress` (atomic write)
        │
        ▼
   3. Constrói contexto pro AI agent:
      - conteúdo da task
      - failures anteriores (se houver)
      - docs relevantes
      - estado do projeto
        │
        ▼
   4. Invoca AI agent (Claude Code CLI, etc)
        │
        ▼
   5. Agent implementa, edita arquivos
        │
        ▼
   6. Orchestrator roda validação:
      - cargo build
      - cargo test
      - cargo clippy
      - comandos custom da task
        │
        ├── PASS ──▶ status: done, git commit, status file
        │
        └── FAIL ──▶ status: failed, failure log, NÃO commita
        │
        ▼
   7. Próximo ciclo
```

## Estrutura de diretórios

```
cycles/
├── README.md                    # este arquivo
├── orchestrator.sh              # entry point principal
├── lib/
│   ├── state.sh                 # CRUD de tasks/status
│   ├── validation.sh            # roda testes
│   ├── git.sh                   # commits por ciclo
│   └── agent.sh                 # invoca AI agent
├── templates/
│   ├── task.md                  # template de task
│   ├── status.md                # template de status
│   ├── failure.md               # template de falha
│   └── retry-context.md         # contexto de retry
├── tasks/                       # .md por task (a fazer)
├── status/                      # histórico de execuções
└── failures/                    # logs de falhas pra retry
```

## Garantias

### Isolamento
- Cada task trabalha em uma área bem definida (módulo, componente, etc)
- Tasks não devem modificar arquivos de tasks anteriores
- Se uma task precisa mudar algo de task done, isso vira uma NOVA task

### Não-regressão
- `cargo test` roda TODOS os testes de TODOS os ciclos anteriores
- Se ciclo N+1 quebrar ciclo N, o ciclo N+1 falha
- Git commit por ciclo done permite `git revert` se necessário

### Histórico completo
- Arquivos em `cycles/status/` NUNCA são editados
- Cada execução gera um novo arquivo
- Falhas vão pra `cycles/failures/`

### Dependências
- Task tem campo `depende_de: ["001", "002"]`
- Orchestrator não pula dependências

## Comandos

```bash
# Roda próxima task pendente
./cycles/orchestrator.sh run

# Roda task específica
./cycles/orchestrator.sh run 005

# Lista todas as tasks
./cycles/orchestrator.sh list

# Status geral
./cycles/orchestrator.sh status

# Histórico completo
./cycles/orchestrator.sh history

# Retry de task que falhou
./cycles/orchestrator.sh retry 005

# Cria nova task a partir do template
./scripts/new-cycle.sh "Título da task"
```

## Quem executa

O orchestrator invoca um AI agent (Claude Code CLI por padrão) que:
1. Lê a task como contexto
2. Implementa o que precisa
3. Roda os comandos de validação
4. Salva status

Variável de ambiente `AGENT_CMD` permite trocar o agent:
```bash
AGENT_CMD="aider --message-file" ./cycles/orchestrator.sh run
```

## Princípio fundamental

> **Um ciclo não pode quebrar o que o outro ciclo fez.**

Se isso acontecer, o ciclo que causou a regressão é marcado como
**failed**, e o agente recebe o failure log + experiência anterior
na próxima tentativa.

**Não** tentamos "auto-corrigir" o que está funcionando. Regressões
são sinalizadas e o agente decide o que fazer.
