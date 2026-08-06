# Design System do Anotadinho

## Tokens de design (CSS custom properties)

Todas as cores, espaçamentos, tipografia e bordas são definidos como
variáveis CSS em `ui/src/styles/main.css`. **Nunca** use valores
hexadecimais ou pixels diretamente nos componentes — sempre referencie
as variáveis.

### Cores

| Token | Valor | Uso |
|---|---|---|
| `--bg-base` | `#272930` | Fundo principal |
| `--bg-surface` | `#32343F` | Cards, sidebar, header |
| `--bg-elevated` | `#3D404F` | Hover, selecionado, dialogs |
| `--text-primary` | `#E4E8F5` | Texto principal |
| `--text-muted` | `#989EB7` | Texto secundário, placeholders |
| `--accent-blue` | `#00B5FF` | Links, destaque, selecionado |
| `--accent-purple` | `#9327FF` | Gradientes, blockquotes |
| `--border` | `#3D404F` | Bordas, divisores |
| `--success` | `#248569` | Status salvo |
| `--warning` | `#DB7E21` | Dirty state |
| `--error` | `#E71D32` | Erros, botão excluir hover |

Pra variações translúcidas de uma cor (ex: fundo de badge, anel de foco),
use `color-mix(in srgb, var(--token) X%, transparent)` em vez de cravar um
`rgba(...)` — assim a variação segue o token automaticamente se ele mudar
de novo no futuro (isso já aconteceu uma vez: a paleta foi trocada e vários
`rgba()` cravados no CSS ficaram desatualizados até serem encontrados numa
revisão).

### Espaçamento

| Token | Valor |
|---|---|
| `--sp-1` | 0.25rem (4px) |
| `--sp-2` | 0.5rem (8px) |
| `--sp-3` | 0.75rem (12px) |
| `--sp-4` | 1rem (16px) |
| `--sp-6` | 1.5rem (24px) |
| `--sp-8` | 2rem (32px) |

### Tipografia

| Token | Stack |
|---|---|
| `--font-sans` | Inter, system-ui, -apple-system, sans-serif |
| `--font-mono` | JetBrains Mono, Fira Code, monospace |

### Bordas

| Token | Valor |
|---|---|
| `--r-sm` | 4px |
| `--r-md` | 8px |
| `--r-lg` | 12px |

## Convenções de CSS

### Nomenclatura BEM

Use BEM (Block Element Modifier) para todas as classes:

```css
.component-name { }          /* Bloco */
.component-name__element { } /* Elemento */
.component-name--modifier { } /* Modificador */
```

Exemplos válidos:
- `.sidebar-item` (bloco)
- `.sidebar-item__title` (elemento)
- `.sidebar-item--selected` (modificador)
- `.editor__header` (elemento do bloco editor)

### Estrutura do arquivo

Dois arquivos, cada um carregado via `data-trunk` no `ui/index.html`:

- `ui/src/styles/main.css` — tokens (`:root`/`.theme-light`), reset global,
  chrome do app (header bar, tab bar, sidebar, editor/toolbar/statusbar) e
  a tipografia do conteúdo renderizado (`.editor__wysiwyg …`).
- `ui/src/styles/components.css` — componentes reutilizáveis e
  independentes de contexto: `.btn`, `.badge`, `.input`, `.card`, `.switch`,
  `.tooltip`, spinner.

Separe por seções com comentários dentro de cada arquivo:

```css
/* Nome do componente */
.component { ... }
.component__element { ... }
```

Não crie um terceiro arquivo CSS por componente — se algo não é claramente
"chrome do app" nem um componente `.btn`-like reutilizável, o padrão é
`main.css`.

## Componentes Yew

### Regras

1. **Um componente por arquivo** em `ui/src/components/`
2. **Props dedicadas**: cada componente define seu próprio `#[derive(Properties)]`
3. **Estado local com `use_state`**: estado interno do componente
4. **Comunicação via Callbacks**: pai → filho via props, filho → pai via `Callback<T>`
5. **Sem estado global**: evite `Context` ou estado compartilhado. Se necessário,
   passe via props a partir do `App` raiz.
6. **Componentes puros**: mesma prop → mesma renderização. Use `PartialEq` nas Props.

### Template de componente

```rust
//! Descrição do componente.

use yew::prelude::*;

/// Props do MeuComponente.
#[derive(Properties, PartialEq, Clone)]
pub struct MeuComponenteProps {
    /// Descrição da prop.
    pub titulo: String,
    /// Callback opcional.
    #[prop_or_default]
    pub on_click: Callback<()>,
}

/// Componente que faz X.
#[function_component(MeuComponente)]
pub fn meu_componente(props: &MeuComponenteProps) -> Html {
    html! {
        <div class="meu-componente">
            <h3>{ &props.titulo }</h3>
        </div>
    }
}
```

### Registro

Todo componente novo deve ser:
1. Criado em `ui/src/components/nome.rs`
2. Exportado em `ui/src/components/mod.rs` via `pub mod nome;`

## IPC / API

Chamadas ao backend Tauri são centralizadas em `ui/src/api.rs`.
Cada função é async e retorna `Result<T, String>`.

```rust
pub async fn minha_funcao(param: &str) -> Result<MeuTipo, String> {
    let args = js_sys::Object::new();
    js_sys::Reflect::set(&args, &JsValue::from_str("param"), &JsValue::from_str(param))
        .map_err(|e| format!("{:?}", e))?;
    let args = JsValue::from(args);
    let result = tauri_invoke("meu_comando", &args).await.map_err(|e| format!("{:?}", e))?;
    serde_wasm_bindgen::from_value(result).map_err(|e| format!("deserialize: {}", e))
}
```

## Backend (crates)

- **core**: modelos de dados puros (Block, Page, Property)
- **vault**: I/O de arquivos, watcher
- **ipc**: handlers de comandos Tauri
- **search**: busca full-text (futuro)

Cada crate tem seus próprios testes em `#[cfg(test)] mod tests`.
