//! Renderizador Markdown → HTML usando pulldown-cmark.

use pulldown_cmark::{html, Options, Parser};

/// Converte Markdown em HTML.
///
/// Separa o frontmatter YAML (se houver) antes de renderizar — sem isso o
/// pulldown-cmark trata `---\ntitle: ...\n---` como texto solto (thematic
/// break + heading mal formado).
///
/// Embeds (`{{ type: "kanban" }}` etc) são recortados ANTES de chegar
/// aqui, por `crate::embed::segment` — o que sobra pra esta função é
/// sempre markdown "de verdade", então fences ` ```kanban ``` ` viram
/// blocos de código normais, sem tratamento especial.
///
/// Transclusão (`![[Página]]`, ciclo 170) vira um marcador vazio aqui e
/// é preenchida depois, no DOM — esta função é síncrona e não alcança o
/// vault.

/// Embrulha o `^id` do fim das linhas num `<span>` discreto.
fn marcar_ids_de_bloco(body: &str) -> String {
    body.split_inclusive('\n')
        .map(|linha| {
            let sem_quebra = linha.trim_end_matches('\n');
            if let Some(id) = anotadinho_core::links::extract_block_id(sem_quebra) {
                let limpo = anotadinho_core::links::strip_block_id(sem_quebra);
                let marcado = format!("{limpo} <span class=\"bloco-id\">^{id}</span>");
                if linha.ends_with('\n') {
                    format!("{marcado}\n")
                } else {
                    marcado
                }
            } else {
                linha.to_string()
            }
        })
        .collect()
}

/// Troca `![[Alvo]]` por um marcador de bloco que o DOM resolve
/// depois. Fora de fence de código E fora de código inline — sem isso,
/// uma página que EXPLICA a sintaxe (escrevendo `` `![[Página]]` ``)
/// via o próprio exemplo virar uma transclusão de uma página chamada
/// "Página", que obviamente não existe.
fn marcar_transclusoes(body: &str) -> String {
    let mut out = String::with_capacity(body.len());
    let mut in_fence = false;
    for linha in body.split_inclusive('\n') {
        let t = linha.trim_start();
        if t.starts_with("```") || t.starts_with("~~~") {
            in_fence = !in_fence;
            out.push_str(linha);
            continue;
        }
        if in_fence || !linha.contains("![[") {
            out.push_str(linha);
            continue;
        }
        out.push_str(&marcar_linha(linha));
    }
    out
}

/// Converte os `![[...]]` de UMA linha, pulando os que estão dentro de
/// crase (código inline).
fn marcar_linha(linha: &str) -> String {
    let mut out = String::with_capacity(linha.len());
    let mut em_codigo = false;
    let bytes = linha.as_bytes();
    let mut i = 0;
    while i < linha.len() {
        if bytes[i] == b'`' {
            em_codigo = !em_codigo;
            out.push('`');
            i += 1;
            continue;
        }
        if !em_codigo && linha[i..].starts_with("![[") {
            if let Some(fim) = linha[i + 3..].find("]]") {
                let alvo = linha[i + 3..i + 3 + fim].trim();
                if !alvo.is_empty() && !alvo.contains('[') && !alvo.contains('`') {
                    // HTML cru no meio do markdown: o pulldown-cmark
                    // passa adiante, e o marcador chega inteiro no DOM.
                    out.push_str(&format!(
                        "\n<div class=\"transclusao\" data-transclusao=\"{}\"></div>\n",
                        alvo.replace('"', "&quot;")
                    ));
                    i += 3 + fim + 2;
                    continue;
                }
            }
        }
        let ch = linha[i..].chars().next().unwrap_or('\0');
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

/// Converte Markdown em HTML (ver doc do módulo acima).
pub fn render(markdown: &str) -> String {
    let body = anotadinho_core::MarkdownCodec::split_frontmatter(markdown)
        .map(|(_, body)| body)
        .unwrap_or(markdown);
    // Transclusão (ciclo 170): vira um marcador ANTES do linkify, senão
    // o `[[..]]` de dentro viraria link e o `!` ficaria solto. O
    // conteúdo real é carregado depois, no DOM, por
    // `upgrade_transclusions_at` — daqui não dá pra ler o vault.
    let body = marcar_transclusoes(body);
    // Id de bloco (ciclo 176): fica no DOM, mas embrulhado num `<span>`
    // que o CSS deixa discreto. Não dá pra ESCONDER de vez: o markdown
    // é recomposto a partir do DOM na hora de salvar, então o que sai
    // da renderização some do arquivo na próxima gravação.
    let body = marcar_ids_de_bloco(&body);
    let body = crate::wikilink::linkify(&body);

    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(&body, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    habilitar_checkboxes(&html_output)
}

/// Tira o `disabled` dos checkboxes de lista de tarefas (ciclo 193).
///
/// O pulldown-cmark emite `<input disabled="" type="checkbox"`, que é o
/// certo pra renderizar markdown num site — mas aqui é um EDITOR, e um
/// input desabilitado não responde a clique. O resultado é que
/// `- [ ] tarefa` escrito em markdown era somente leitura, enquanto o
/// checkbox inserido pelo menu `/` (que nasce sem `disabled`) funcionava
/// — a mesma coisa na tela com comportamentos diferentes.
fn habilitar_checkboxes(html: &str) -> String {
    html.replace("<input disabled=\"\" type=\"checkbox\"", "<input type=\"checkbox\"")
        .replace("<input disabled type=\"checkbox\"", "<input type=\"checkbox\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transclusao_solta_vira_marcador() {
        let html = marcar_transclusoes("antes\n![[Alvo]]\ndepois\n");
        assert!(html.contains("data-transclusao=\"Alvo\""), "{html}");
        assert!(html.contains("antes"));
        assert!(html.contains("depois"));
    }

    #[test]
    fn transclusao_dentro_de_codigo_inline_nao_e_convertida() {
        // Regressão: a página que EXPLICA a sintaxe transcluía o próprio
        // exemplo e mostrava "Página não existe ainda".
        let entrada = "Use `![[Página]]` pra transcluir.\n";
        let saida = marcar_transclusoes(entrada);
        assert_eq!(saida, entrada, "código inline não pode virar transclusão");
    }

    #[test]
    fn transclusao_dentro_de_fence_nao_e_convertida() {
        let entrada = "```\n![[Alvo]]\n```\n";
        assert_eq!(marcar_transclusoes(entrada), entrada);
    }

    #[test]
    fn mistura_de_codigo_e_transclusao_na_mesma_linha() {
        let saida = marcar_transclusoes("`![[Nao]]` mas ![[Sim]] vale\n");
        assert!(saida.contains("`![[Nao]]`"), "o exemplo em código ficou intacto: {saida}");
        assert!(saida.contains("data-transclusao=\"Sim\""), "a de fora foi convertida: {saida}");
    }

    #[test]
    fn checkbox_de_tarefa_nao_sai_desabilitado() {
        // Regressão do ciclo 193: `- [ ] x` escrito em markdown vinha
        // `disabled` e não dava pra marcar no editor.
        let html = render("---\ntitle: T\n---\n- [ ] tarefa\n- [x] pronta\n");
        assert!(html.contains("type=\"checkbox\""), "{html}");
        assert!(!html.contains("disabled"), "sobrou disabled:\n{html}");
    }

    #[test]
    fn id_de_bloco_vira_marca_discreta() {
        let saida = marcar_ids_de_bloco("uma linha ^abc123\noutra linha\n");
        assert!(saida.contains("<span class=\"bloco-id\">^abc123</span>"), "{saida}");
        assert!(saida.contains("outra linha"));
        assert!(!saida.contains("linha ^abc123"), "o id cru não pode sobrar: {saida}");
    }
}
