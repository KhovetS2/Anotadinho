---
name: cycle
description: Fecha um ciclo de desenvolvimento do Anotadinho (valida, escreve status file, commita) seguindo o fluxo documentado em AGENTS.md e cycles/README.md.
---

# cycle

Empacota o "Ciclo de desenvolvimento" descrito em `AGENTS.md` pra não
precisar colar os mesmos comandos toda vez. Use depois de implementar uma
fatia de trabalho (uma task de `cycles/tasks/`, ou uma mudança equivalente
combinada com o usuário).

## Quando usar

- O usuário pediu pra "fechar o ciclo", "rodar a validação e commitar", ou
  terminou de implementar algo que corresponde a uma task numerada.
- Não use no meio de uma implementação ainda incompleta — só quando o
  trabalho já está pronto pra validar e commitar.

## Passos

1. **Identifique a task** (se houver uma correspondente em `cycles/tasks/`).
   Se não houver task file, pergunte ao usuário o ID/título antes de seguir
   (ou use `scripts/new-cycle.sh "Título"` pra criar uma, se fizer sentido).

2. **Rode a validação completa**, na raiz do repo:
   ```bash
   source "$HOME/.cargo/env" && export PATH="$HOME/.cargo/bin:$PATH"
   cargo build --workspace
   cargo test --workspace
   cargo clippy --workspace --all-targets
   (cd ui && trunk build)
   cargo build --manifest-path src-tauri/Cargo.toml
   ```
   Pare aqui e reporte se qualquer passo falhar — não force um commit com
   validação quebrada.

3. **Marque a task como done** (se houver task file): troque
   `status: pending`/`in_progress` por `status: done` em
   `cycles/tasks/{id}-*.md`, marcando os checkboxes de critério que
   realmente foram atendidos (deixe sem marcar o que não foi, com uma nota
   explicando — não marque tudo cegamente).

4. **Escreva o status file** em `cycles/status/{id}-{timestamp}-done.md`
   (formato: ver `cycles/templates/status.md` ou qualquer arquivo recente
   em `cycles/status/` como exemplo). Inclua um resumo curto do que mudou,
   arquivos tocados, testes novos, e notas úteis pro próximo ciclo.

5. **Commit**, seguindo a convenção do histórico:
   ```
   feat({id}): {título curto}
   ```
   (ou `fix({id}):` se for uma correção). Só commite os arquivos relevantes
   ao ciclo — confira `git status` antes de um `git add` amplo.

6. Reporte ao usuário: o que foi validado, o que foi commitado, e qualquer
   critério de aceite que ficou pendente pra um próximo ciclo.
