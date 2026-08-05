# Yew Decisions

Por que **Yew** pro frontend, e como ele se encaixa com Tauri.

## Stack

| Componente | Tecnologia | Versão |
|---|---|---|
| Framework UI | Yew | 0.21 |
| Build tool | Trunk | latest |
| Roteamento | yew-router | 0.18 |
| Runtime | Tauri 2 (WebView) | 2.x |

## Por que Yew

**Considerado:**
- Yew (Rust → WASM)
- Dioxus (Rust cross-platform)
- Tauri + Svelte/Solid (web puro)
- Leptos (Rust → WASM)

**Decisão: Yew**

- ✅ Maduro (0.21+ em produção há anos)
- ✅ Performance WASM (perto de nativo)
- ✅ Bundle pequeno (~1MB vs ~100MB Electron)
- ✅ React-like (familiar, components, hooks)
- ✅ Comunidade ativa
- ⚠️ WebView (não native rendering) - mas isso é verdade pra qualquer opção Tauri
- ❌ Não tem hot reload do estado (vs Dioxus Devtools)

## Pattern: Componentes

Yew usa function components com hooks (como React):

```rust
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub title: String,
}

#[function_component(Header)]
pub fn header(props: &Props) -> Html {
    html! {
        <header class="header">
            <h1>{ &props.title }</h1>
        </header>
    }
}
```

**Regras:**
- Componentes em `ui/src/components/`
- Páginas em `ui/src/pages/`
- Estado global em `ui/src/state.rs` (via `Context`)
- IPC em `ui/src/api.rs`

## Build pipeline

```
ui/src/*.rs
   │
   ▼ (Trunk + wasm-bindgen)
ui/dist/
   │  ├── *.wasm
   │  ├── *.js
   │  └── index.html
   ▼
src-tauri/tauri.conf.json::frontendDist = "../ui/dist"
   │
   ▼ (Tauri empacota no bundle final)
```

## Dev workflow

```bash
# Terminal 1: serve do Yew (hot reload)
cd ui && trunk serve --port 1420

# Terminal 2: app Tauri (conecta no Trunk)
cd src-tauri && cargo tauri dev
```

Ou use `scripts/dev.sh` que faz os dois.
