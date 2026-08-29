//! Faxina do HTML cru que vem do markdown (ciclo 235).
//!
//! O `pulldown-cmark` repassa HTML inline verbatim — não há como
//! desligar isso — e o resultado vai direto pro DOM por `set_inner_html`.
//! Ou seja: qualquer `.md` do vault podia injetar `<script>` ou
//! `onerror=` no webview. Um vault sincronizado, clonado por git ou
//! escrito por um agente é conteúdo de terceiros, então isso é superfície
//! de ataque, não só desleixo.
//!
//! **Não é uma allowlist.** Uma allowlist completa exigiria enumerar tudo
//! que a renderização já usa hoje (a transclusão injeta `<div>`, o
//! marcador de bloco injeta `<span>`, imagem inserida usa
//! `<figure>`/`<img>`) e qualquer esquecimento apagaria conteúdo real e
//! silenciosamente. Aqui a mira é o que faz dano: elementos que executam
//! ou carregam coisa de fora, atributos de evento, e URL executável.
//!
//! O que sobra continua passando — inclusive `<span style="color:...">`,
//! que agora é vocabulário legítimo (ciclo 235).

/// Elementos que somem inteiros, com conteúdo e tudo.
const COM_CONTEUDO: &[&str] = &["script", "style"];

/// Elementos cuja TAG some, preservando o que estiver dentro.
///
/// Só os que CARREGAM coisa de fora ou trocam o alvo do documento.
/// Controles de formulário ficaram fora de propósito: `<input>` não
/// executa nada sozinho, e a lista de tarefas do markdown é renderizada
/// exatamente como `<input type="checkbox">` — tirá-lo apagaria as
/// caixinhas de toda checklist do vault.
const SO_A_TAG: &[&str] = &[
    "iframe", "object", "applet", "link", "meta", "base", "frame", "frameset",
];

/// Limpa o HTML cru de um documento markdown.
pub fn limpar(html: &str) -> String {
    let sem_blocos = remover_blocos(html);
    remover_tags_e_atributos(&sem_blocos)
}

/// Tira `<script>…</script>` e `<style>…</style>` com o conteúdo.
fn remover_blocos(html: &str) -> String {
    let mut saida = String::with_capacity(html.len());
    let minusculo = html.to_ascii_lowercase();
    let mut i = 0;
    'fora: while i < html.len() {
        for tag in COM_CONTEUDO {
            let abre = format!("<{tag}");
            if minusculo[i..].starts_with(&abre) {
                let fecha = format!("</{tag}>");
                match minusculo[i..].find(&fecha) {
                    Some(fim) => i += fim + fecha.len(),
                    // Sem fechamento, o resto do documento é o bloco.
                    None => i = html.len(),
                }
                continue 'fora;
            }
        }
        let passo = proximo_caractere(html, i);
        saida.push_str(&html[i..passo]);
        i = passo;
    }
    saida
}

fn proximo_caractere(s: &str, i: usize) -> usize {
    let mut j = i + 1;
    while j < s.len() && !s.is_char_boundary(j) {
        j += 1;
    }
    j
}

/// Tira as tags proibidas e os atributos perigosos das que ficam.
fn remover_tags_e_atributos(html: &str) -> String {
    let mut saida = String::with_capacity(html.len());
    let bytes = html.as_bytes();
    let mut i = 0;
    while i < html.len() {
        if bytes[i] != b'<' {
            let passo = proximo_caractere(html, i);
            saida.push_str(&html[i..passo]);
            i = passo;
            continue;
        }
        let Some(fim) = html[i..].find('>').map(|p| i + p + 1) else {
            // `<` solto no texto: não é tag, é conteúdo.
            saida.push_str(&html[i..]);
            break;
        };
        let tag = &html[i..fim];
        match nome_da_tag(tag) {
            Some(nome) if SO_A_TAG.contains(&nome.as_str()) => {}
            Some(_) => saida.push_str(&limpar_atributos(tag)),
            None => saida.push_str(tag),
        }
        i = fim;
    }
    saida
}

/// `<div class=x>` → `div`; `</div>` → `div`; `<!-- -->` → `None`.
fn nome_da_tag(tag: &str) -> Option<String> {
    let interno = tag.trim_start_matches('<').trim_end_matches('>');
    let interno = interno.strip_prefix('/').unwrap_or(interno);
    let nome: String = interno
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric())
        .collect();
    (!nome.is_empty()).then(|| nome.to_ascii_lowercase())
}

/// Remove atributos de evento e URL executável de uma tag de abertura.
fn limpar_atributos(tag: &str) -> String {
    let minusculo = tag.to_ascii_lowercase();
    if !minusculo.contains("on") && !minusculo.contains("javascript:") {
        return tag.to_string();
    }

    let mut saida = String::with_capacity(tag.len());
    let mut resto = tag;
    // Copia o nome da tag e vai varrendo atributo a atributo.
    let corte = resto
        .find(|c: char| c.is_whitespace())
        .unwrap_or(resto.len());
    saida.push_str(&resto[..corte]);
    resto = &resto[corte..];

    for atributo in separar_atributos(resto) {
        let nome = atributo
            .split(['=', ' '])
            .next()
            .unwrap_or("")
            .trim()
            .to_ascii_lowercase();
        // `on*` cobre onclick, onerror, onload e os outros — e nenhum
        // atributo legítimo de conteúdo começa com "on".
        if nome.starts_with("on") {
            continue;
        }
        if atributo.to_ascii_lowercase().replace(char::is_whitespace, "").contains("javascript:") {
            continue;
        }
        saida.push(' ');
        saida.push_str(atributo.trim());
    }
    if !saida.ends_with('>') {
        saida.push('>');
    }
    saida
}

/// Quebra o miolo de uma tag em atributos, respeitando aspas.
fn separar_atributos(resto: &str) -> Vec<String> {
    let mut itens = Vec::new();
    let mut atual = String::new();
    let mut aspas: Option<char> = None;
    for c in resto.chars() {
        match c {
            '"' | '\'' if aspas.is_none() => {
                aspas = Some(c);
                atual.push(c);
            }
            c if Some(c) == aspas => {
                aspas = None;
                atual.push(c);
            }
            '>' | '/' if aspas.is_none() => {
                if !atual.trim().is_empty() {
                    itens.push(std::mem::take(&mut atual));
                }
                atual.clear();
            }
            c if c.is_whitespace() && aspas.is_none() => {
                if !atual.trim().is_empty() {
                    itens.push(std::mem::take(&mut atual));
                }
                atual.clear();
            }
            c => atual.push(c),
        }
    }
    if !atual.trim().is_empty() {
        itens.push(atual);
    }
    itens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn script_some_com_conteudo() {
        let sujo = "antes<script>alert(1)</script>depois";
        assert_eq!(limpar(sujo), "antesdepois");
    }

    #[test]
    fn script_sem_fechamento_nao_deixa_resto_passar() {
        assert_eq!(limpar("ok<script>roubar()"), "ok");
    }

    #[test]
    fn atributo_de_evento_sai_e_a_tag_fica() {
        let limpo = limpar("<img src=\"x.png\" onerror=\"alert(1)\">");
        assert!(limpo.contains("src=\"x.png\""), "perdeu o src: {limpo}");
        assert!(!limpo.to_lowercase().contains("onerror"), "manteve o onerror: {limpo}");
    }

    #[test]
    fn link_com_javascript_perde_o_href() {
        let limpo = limpar("<a href=\"javascript:alert(1)\">clica</a>");
        assert!(!limpo.to_lowercase().contains("javascript:"), "{limpo}");
        assert!(limpo.contains("clica"), "apagou o texto do link: {limpo}");
    }

    #[test]
    fn iframe_perde_a_tag_mas_nao_o_texto() {
        let limpo = limpar("<iframe src=\"http://x\">fallback</iframe>");
        assert!(!limpo.contains("iframe"), "{limpo}");
        assert!(limpo.contains("fallback"), "{limpo}");
    }

    #[test]
    fn o_que_a_renderizacao_ja_usa_continua_passando() {
        // Estes três são injetados pelo PRÓPRIO código antes de
        // renderizar. Uma faxina que os comesse apagaria transclusão,
        // âncora de bloco e imagem — em silêncio.
        for html in [
            "<div class=\"transclusao\" data-alvo=\"pages/x.md\"></div>",
            "<span class=\"bloco-id\">^abc</span>",
            "<figure class=\"inserted-image\"><img src=\"assets/a.png\" alt=\"a\"><figcaption>oi</figcaption></figure>",
        ] {
            assert_eq!(limpar(html), html, "a faxina comeu conteúdo legítimo");
        }
    }

    #[test]
    fn cor_continua_passando() {
        // Cor é vocabulário legítimo desde o ciclo 235 — some junto com
        // o resto seria consertar uma coisa quebrando outra.
        for html in [
            "<span class=\"cor--ambar\">oi</span>",
            "<span style=\"color:#ff0000\">oi</span>",
            "<span style=\"background-color:#ff0\">oi</span>",
        ] {
            assert_eq!(limpar(html), html, "a faxina comeu a cor");
        }
    }

    #[test]
    fn texto_com_menor_que_nao_vira_tag() {
        assert_eq!(limpar("2 < 3 e 4 > 1"), "2 < 3 e 4 > 1");
    }

    #[test]
    fn a_caixinha_da_checklist_sobrevive() {
        // A lista de tarefas do markdown vira `<input type="checkbox">`.
        // Uma faxina que levasse controles de formulário junto apagaria
        // a caixinha de toda checklist do vault.
        let html = "<li><input disabled=\"\" type=\"checkbox\"/>tarefa</li>";
        assert_eq!(limpar(html), html);
    }

    #[test]
    fn html_normal_atravessa_intacto() {
        let html = "<p>oi <strong>tudo</strong> bem</p>";
        assert_eq!(limpar(html), html);
    }
}
