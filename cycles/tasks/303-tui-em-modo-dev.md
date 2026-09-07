---
id: "303"
titulo: "A TUI em modo dev, recarregando"
status: done
criado: 2026-09-07
autor: agente
prioridade: media
depende_de: ["302"]
estima_min: 45
---

# 303 — A TUI em modo dev

## O que motivou

Pergunta de quem estava desenhando as telas: "não tem uma versão de
desenvolvimento com hot reload dela não?".

Não tinha. Ver uma mudança na TUI era `cargo build`, matar o processo e
subir de novo — três gestos pra cada ajuste de cor, e o ciclo 302
inteiro foi ajuste de cor.

## Não é hot reload, e o nome importa

Binário Rust não troca de código em voo. O que dá pra fazer é RELIGAR:
salvou, recompila, sobe de novo. O que se perde é onde o cursor estava.

Chamar isso de hot reload seria prometer o que não acontece.

## Duas decisões

**Erro de compilação não derruba a TUI que está de pé.** O build roda
antes de qualquer limpeza de tela: o erro fica legível no buffer normal
e a TUI continua no ar até passar. Recarregar pra uma tela preta com um
erro é pior do que não recarregar.

**A restauração do terminal ficou no script.** A TUI desliga o modo cru
e sai da tela alternativa na saída normal, mas a cada recarga o
processo é MORTO — e aí a limpeza dela não roda. Sem `stty sane` e sem
sair da tela alternativa no script, a primeira recompilação deixaria o
shell da pessoa quebrado.

A alternativa era ensinar o binário a tratar SIGTERM, o que custaria uma
dependência de sinais pra um problema que só existe em dev.

## Critérios de aceite

- [x] `./scripts/tui.sh` sobe a TUI e recarrega quando o código muda
- [x] `--vault`, `--tema` e `--uma-vez`
- [x] Vigia os `.rs` do workspace e o `main.css`, de onde a paleta vem
- [x] Erro de compilação não derruba a TUI que está no ar
- [x] O terminal volta ao normal depois de cada recarga
- [x] Falta do `inotifywait` dá mensagem que diz o que instalar

## Não-objetivos

- Trocar código em voo. Não dá, e prometer seria mentira.
- Preservar o cursor entre recargas.

## Comandos de validação

```bash
bash -n scripts/tui.sh
./scripts/tui.sh            # e gravar um arquivo do crate
```
