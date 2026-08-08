---
title: Nomenclatura
date: 2026-08-08
dominio: geral
tags:
- padrao
---
# Nomenclatura

## Quando se aplica

Toda função/struct/módulo novo no workspace Rust (`crates/*`, `ui/`,
`src-tauri/`).

## Regras

1. Nomes de arquivo e módulo em `snake_case`; tipos (`struct`/`enum`)
   em `PascalCase`; funções e variáveis em `snake_case` — padrão Rust
   default, sem exceção
2. Handlers IPC (`crates/ipc`) sempre prefixados com `handle_` (ex:
   `handle_list_pages`), espelhando o nome do comando Tauri
   correspondente sem o prefixo
3. Componentes Yew em `PascalCase` no nome do componente
   (`#[function_component(NomeDoComponente)]`), mas o arquivo que o
   contém em `snake_case` (ex: `properties_panel.rs` → `PropertiesPanel`)
4. Nomes em português no domínio do produto (frontmatter, docs,
   mensagens de commit), inglês só onde a convenção da linguagem/
   framework exige (nomes de trait, nomes de campo serde que espelham
   uma chave YAML em inglês, etc.)

## Exemplos

### Bom

```rust
pub fn handle_list_templates(vault_path: String) -> Result<Vec<PageMeta>, String> { ... }

#[function_component(PropertiesPanel)]
pub fn properties_panel(props: &PropertiesPanelProps) -> Html { ... }
```

### Ruim

```rust
// falta o prefixo handle_, não espelha o nome do comando Tauri
pub fn list_templates_impl(vault_path: String) -> Result<Vec<PageMeta>, String> { ... }

// componente em snake_case
#[function_component(properties_panel)]
```

## Exceções

Campos de struct que espelham uma chave YAML de frontmatter existente
(ex: `page_type` serializado como `type` via `#[serde(rename = "type")]`)
podem divergir do nome Rust idiomático pra manter compatibilidade com
arquivos `.md` já existentes no vault — sempre com o `#[serde(rename)]`
explícito, nunca silencioso.
