# Anotadinho scripts

## new-cycle.sh

Cria uma nova task a partir do template.

```bash
./scripts/new-cycle.sh "Vault picker"          # cria 002-vault-picker.md
./scripts/new-cycle.sh --id 005 "Editor básico"  # cria 005-editor-basico.md
```

## tui.sh

Sobe o Anotadinho num terminal, em modo dev. Fica de olho nos `.rs` do
workspace e no `main.css` (de onde a paleta é lida em tempo de
compilação): salvou, ele recompila e religa a TUI.

```bash
./scripts/tui.sh                       # vault do repo, tema escuro, recarregando
./scripts/tui.sh --tema papel
./scripts/tui.sh --vault ~/MeuVault
./scripts/tui.sh --uma-vez             # compila, roda, e só
```

Não é hot reload de verdade — binário Rust não troca de código em voo.
É religar, e o que se perde é onde o cursor estava.

Erro de compilação **não derruba** a TUI que está de pé: o erro fica
legível na tela normal e ela continua no ar até o build passar.

Requer `inotifywait` (pacote `inotify-tools`).
