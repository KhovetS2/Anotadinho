# Design System do Anotadinho

## Tokens de design (CSS custom properties)

Todas as cores, espaçamentos, tipografia e bordas são definidos como
variáveis CSS em `ui/src/styles/main.css`. **Nunca** use valores
hexadecimais ou pixels diretamente nos componentes — sempre referencie
as variáveis.

### Cores

| Token | Valor | Uso |
|---|---|---|
| `--bg-base` | `#0F172A` | Fundo principal |
| `--bg-surface` | `#1E293B` | Cards, sidebar, header |
| `--bg-elevated` | `#334155` | Hover, selecionado, dialogs |
| `--text-primary` | `#F1F5F9` | Texto principal |
| `--text-muted` | `#94A3B8` | Texto secundário, placeholders |
| `--accent-blue` | `#3B82F6` | Links, destaque, selecionado |
| `--accent-purple` | `#8B5CF6` | Gradientes, blockquotes |
| `--border` | `#334155` | Bordas, divisores |
| `--success` | `#10B981` | Status salvo |
| `--warning` | `#F59E0B` | Dirty state |
| `--error` | `#EF4444` | Erros, botão excluir hover |

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

`ui/src/styles/main.css` é o único arquivo de estilo. Todas as regras
de todos os componentes ficam aqui. Separe por seções com comentários:

```css
/* Nome do componente */
.component { ... }
.component__element { ... }
```

Não crie arquivos CSS separados por componente.

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
