//! Conversão HTML → Markdown para o editor WYSIWYG.

use wasm_bindgen::JsCast;
use web_sys::Element;

/// Converte HTML (innerHTML de contenteditable) para Markdown.
pub fn html_to_markdown(root: &Element) -> String {
    walk(root, 0).trim().to_string()
}

fn walk(node: &Element, _depth: usize) -> String {
    let tag = node.tag_name().to_lowercase();
    let children = node.children();
    let child_count = children.length();

    match tag.as_str() {
        // Dois `\n`: heading é bloco e precisa de linha em branco depois,
        // igual ao `<p>`. Antes emitia um só e a linha em branco vinha,
        // por acidente, do espaço de formatação que o `inline_children`
        // deixava passar — quando esse espaço parou de ser conteúdo
        // (ciclo 193), o heading grudou no parágrafo seguinte.
        "h1" => format!("# {}\n\n", text_of(node)),
        "h2" => format!("## {}\n\n", text_of(node)),
        "h3" => format!("### {}\n\n", text_of(node)),
        "h4" => format!("#### {}\n\n", text_of(node)),
        "h5" => format!("##### {}\n\n", text_of(node)),
        "h6" => format!("###### {}\n\n", text_of(node)),
        "strong" | "b" => format!("**{}**", text_of(node)),
        "em" | "i" => format!("*{}*", text_of(node)),
        "u" => text_of(node),
        "s" | "del" | "strike" => format!("~~{}~~", text_of(node)),
        "code" => format!("`{}`", text_of(node)),
        "pre" => {
            // Se o <pre> tem um único filho <code class="language-X">, a
            // fence sai com a linguagem preservada (```X). Sem isso, inserir
            // um embed via slash command (ver editor.rs) perderia a
            // linguagem no primeiro save e nunca viraria um embed de verdade.
            let code_el = node.query_selector("code").ok().flatten();
            let lang = code_el.as_ref()
                .and_then(|code| code.get_attribute("class"))
                .and_then(|class| {
                    class.split_whitespace()
                        .find_map(|c| c.strip_prefix("language-").map(|s| s.to_string()))
                })
                .unwrap_or_default();
            // Pega o texto CRU do <code> direto via `text_content` (não
            // via `walk`/`text_of`) — descer pelo `walk` reentraria no
            // ramo "code" acima, que já devolveria o texto envolto em
            // ` ``` ` (fence), e este ramo envolveria de novo por cima,
            // produzindo fences aninhadas quebradas a cada save (bug
            // real: `Ctrl+Z` e reabrir a página "quebravam" todo bloco
            // de código da página).
            let text = code_el
                .map(|el| el.text_content().unwrap_or_default())
                .unwrap_or_else(|| node.text_content().unwrap_or_default());
            format!("```{}\n{}\n```\n\n", lang, text.trim_end_matches('\n'))
        }
        "blockquote" => {
            let body = text_of(node);
            body.lines().map(|l| format!("> {}\n", l)).collect::<String>() + "\n"
        }
        "li" => {
            let body = inline_children(node);
            // Item de checklist: o ramo "input" abaixo já devolveu o
            // marcador completo ("- [ ] "/"- [x] ") — prefixar outro
            // marcador aqui (o "-"/"1." de baixo) duplicava o
            // marcador ("- - [ ] texto", markdown quebrado). Combo
            // lista+checkbox pedido pelo usuário cai nesse caso.
            if let Some(rest) = body.strip_prefix("- [ ] ").or_else(|| body.strip_prefix("- [x] ")) {
                let marker = &body[..6];
                format!("{}{}\n", marker, rest.trim_start())
            } else {
                let kind = node.parent_element()
                    .map(|p| p.tag_name().to_lowercase())
                    .unwrap_or_default();
                let marker = if kind == "ol" { "1." } else { "-" };
                format!("{} {}\n", marker, body)
            }
        }
        "ul" | "ol" => {
            let mut out = String::new();
            for i in 0..child_count {
                if let Some(child) = children.item(i) {
                    if let Ok(el) = child.dyn_into::<Element>() {
                        out.push_str(&walk(&el, 0));
                    }
                }
            }
            out + "\n"
        }
        "a" => {
            let text = text_of(node);
            let href = node.get_attribute("href").unwrap_or_default();
            if let Some(encoded) = href.strip_prefix(crate::wikilink::SCHEME_PREFIX) {
                // Reconstrói o `[[...]]` original (ciclo 192).
                //
                // Antes usava só o texto visível, o que apagava o ALVO
                // de um link com alias: `[[grafo.md|Grafo do Vault]]`
                // era gravado como `[[Grafo do Vault]]` no próximo save.
                // O href guarda o miolo cru, então dá pra devolver
                // exatamente o que estava escrito — e ainda honrar uma
                // edição do texto visível, que é por que a versão antiga
                // lia o texto em primeiro lugar.
                let bruto = crate::wikilink::decode_title(encoded);
                let (alvo, alias) = anotadinho_core::links::split_wikilink(&bruto);
                match alias {
                    // Texto intocado: devolve igualzinho.
                    Some(ref a) if *a == text => format!("[[{}]]", bruto),
                    // Texto editado no editor: mantém o alvo, adota o texto novo.
                    Some(_) => format!(
                        "[[{}|{}]]",
                        anotadinho_core::links::escapar_barra(&alvo),
                        text
                    ),
                    None if text == alvo => format!("[[{}]]", bruto),
                    // Sem alias e texto trocado: comportamento de sempre.
                    None => format!("[[{}]]", text),
                }
            } else if href.is_empty() {
                text
            } else {
                format!("[{}]({})", text, href)
            }
        }
        "p" |         "div" => {
            if let Some(href) = node.get_attribute("data-pdf-href") {
                // Wrapper de embed de PDF (ver `editor.rs::
                // upgrade_embedded_assets_at`, ciclo 121) — reconstrói
                // o MESMO link markdown original (`[texto](x.pdf)`),
                // não desce pro `<iframe>` dentro. Precisa ficar
                // inline (sem `\n\n`) porque nasceu de um `<a>` que
                // estava dentro de um `<p>`.
                let text = node.get_attribute("data-pdf-text").unwrap_or_default();
                format!("[{}]({})", text, href)
            } else if node.class_name().contains("mermaid") {
                let text = text_of(node);
                format!("```mermaid\n{}\n```\n\n", text)
            } else if let Some(kind) = node.get_attribute("data-embed-insert") {
                // Marca deixada por um slash command de embed (ver
                // editor.rs) — vira o wrapper `{{ type: "X" }}` direto,
                // no mesmo formato de `EmbedData::to_fence_text()`, sem
                // passar pelos `<br>` como texto solto.
                let body = inline_children(node);
                format!("{{{{ type: \"{kind}\" }}}}\n{body}\n{{{{ /{kind} }}}}\n\n")
            } else {
                let inner = inline_children(node);
                if inner.is_empty() { "\n".to_string() } else { format!("{}\n\n", inner) }
            }
        }
        "br" => "\n".to_string(),
        "hr" => "---\n".to_string(),
        "table" => {
            // Não recursa via `walk` pros filhos (tr/td/th cairiam no
            // branch `_` genérico, concatenando o texto das células sem
            // `|` nem quebra de linha entre elas — bug real, corrigido
            // aqui montando a tabela markdown direto a partir de
            // `<thead>`/`<tbody>` via query_selector).
            let header: Vec<String> = node
                .query_selector("thead tr")
                .ok()
                .flatten()
                .map(|tr| table_cell_texts(&tr))
                .unwrap_or_default();

            let mut rows: Vec<Vec<String>> = Vec::new();
            if let Ok(trs) = node.query_selector_all("tbody tr") {
                for i in 0..trs.length() {
                    if let Some(n) = trs.item(i) {
                        if let Ok(tr) = n.dyn_into::<Element>() {
                            rows.push(table_cell_texts(&tr));
                        }
                    }
                }
            }

            let col_count = header.len().max(rows.iter().map(|r| r.len()).max().unwrap_or(0));
            if col_count == 0 {
                String::new()
            } else {
                let row_line = |cells: &[String]| -> String {
                    let mut padded = cells.to_vec();
                    padded.resize(col_count, String::new());
                    format!("| {} |\n", padded.join(" | "))
                };
                let mut out = row_line(&header);
                out.push('|');
                out.push_str(&"---|".repeat(col_count));
                out.push('\n');
                for row in &rows {
                    out.push_str(&row_line(row));
                }
                out.push('\n');
                out
            }
        }
        "img" => {
            let alt = node.get_attribute("alt").unwrap_or_default();
            let src = node.get_attribute("src").unwrap_or_default();
            format!("![{}]({})\n", alt, src)
        }
        "input" => {
            // `.checked()` (propriedade viva) — não `has_attribute("checked")`
            // (atributo estático, nunca reflete o toggle que o usuário faz
            // clicando no checkbox em runtime).
            let checked = node.dyn_ref::<web_sys::HtmlInputElement>()
                .map(|el| el.checked())
                .unwrap_or(false);
            if checked { "- [x] ".to_string() } else { "- [ ] ".to_string() }
        }
        _ => {
            if child_count > 0 {
                let mut out = String::new();
                for i in 0..child_count {
                    if let Some(child) = children.item(i) {
                        if let Ok(el) = child.dyn_into::<Element>() {
                            out.push_str(&walk(&el, 0));
                        }
                    }
                }
                out
            } else {
                text_of(node)
            }
        }
    }
}

/// Texto (com formatação inline preservada) de cada `<th>`/`<td>` de
/// uma `<tr>`, na ordem — usado só pelo case `"table"`. Escapa `|`
/// literal na célula pra não quebrar a sintaxe da tabela markdown.
fn table_cell_texts(tr: &Element) -> Vec<String> {
    let mut out = Vec::new();
    if let Ok(cells) = tr.query_selector_all("th, td") {
        for i in 0..cells.length() {
            if let Some(node) = cells.item(i) {
                if let Ok(el) = node.dyn_into::<Element>() {
                    out.push(text_of(&el).replace('|', "\\|"));
                }
            }
        }
    }
    out
}

fn text_of(node: &Element) -> String {
    inline_children(node).trim().to_string()
}

fn inline_children(node: &Element) -> String {
    let children = node.child_nodes();
    let mut out = String::new();
    for i in 0..children.length() {
        if let Some(child) = children.item(i) {
            match child.node_type() {
                3 => {
                    let t = child.text_content().unwrap_or_default();
                    // Espaço em branco de FORMATAÇÃO (o `\n` que o
                    // `set_inner_html` deixa entre dois blocos) não é
                    // conteúdo: incluí-lo somava uma linha em branco a
                    // cada gravação, então abrir e salvar sem editar
                    // nada já mudava o arquivo (ciclo 193).
                    //
                    // Espaço entre PALAVRAS nunca tem quebra de linha,
                    // por isso o `contains('\n')` distingue os dois sem
                    // precisar saber se o pai é bloco ou inline.
                    if !(t.trim().is_empty() && t.contains('\n')) {
                        out.push_str(&t);
                    }
                }
                _ => {
                    if let Ok(el) = child.dyn_into::<Element>() {
                        out.push_str(&walk(&el, 0));
                    }
                }
            }
        }
    }
    out
}
