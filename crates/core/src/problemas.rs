//! A base de problemas de um projeto (ciclo 523).
//!
//! O que o revisor viu o executor errar e que tende a se repetir vira uma
//! seção de `projetos/<slug>/problemas.md` — por proposta, revisada pela
//! pessoa. A página vai anexada no detalhe, na execução e na revisão das
//! tarefas seguintes: o erro de hoje vira critério de aceite de amanhã.
//! É o `pages/padroes` do vault, feito pelo projeto.

use crate::markdown::escapar_escalar_yaml;

/// O título da seção que o revisor escreve.
pub const SECAO: &str = "## Problemas";

/// O `type:` da página.
pub const TIPO: &str = "problemas";

/// Um problema que o revisor anotou.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problema {
    /// O nome curto, que é o título da seção na base.
    pub titulo: String,
    /// O que aconteceu e como evitar.
    pub texto: String,
}

/// Os problemas da seção `## Problemas` de uma resposta.
///
/// Cada item é `- **título**: texto`, e linhas recuadas logo abaixo
/// continuam o item. Item sem título em negrito usa o começo do texto: o
/// formato errado não pode jogar fora uma lição.
pub fn ler(resposta: &str) -> Vec<Problema> {
    let linhas: Vec<&str> = resposta.lines().collect();
    let titulo = |l: &str| l.trim().starts_with('#') && l.trim().trim_start_matches('#').trim().eq_ignore_ascii_case("problemas");
    let Some(ini) = linhas.iter().position(|l| titulo(l)) else { return Vec::new() };
    let mut fora: Vec<Problema> = Vec::new();
    for l in &linhas[ini + 1..] {
        let t = l.trim_start();
        if t.starts_with("## ") || t.to_ascii_uppercase().contains(crate::revisor::MARCA_DE_VEREDITO) {
            break;
        }
        if let Some(item) = t.strip_prefix("- ").or_else(|| t.strip_prefix("* ")) {
            let item = item.trim();
            let (titulo, texto) = match item.strip_prefix("**").and_then(|r| r.split_once("**")) {
                Some((titulo, resto)) => (titulo.trim().trim_end_matches(':').to_string(), resto.trim_start_matches([':', ' ', '—', '-']).trim().to_string()),
                None => {
                    let curto: String = item.chars().take(60).collect();
                    (curto.trim().to_string(), item.to_string())
                }
            };
            if !titulo.is_empty() {
                fora.push(Problema { titulo, texto });
            }
        } else if l.starts_with("  ") && !t.is_empty() {
            if let Some(ultimo) = fora.last_mut() {
                ultimo.texto.push(' ');
                ultimo.texto.push_str(t);
            }
        }
    }
    fora
}

/// Os títulos que a base já tem (as seções `## `).
pub fn titulos(pagina: &str) -> Vec<String> {
    pagina.lines().filter_map(|l| l.strip_prefix("## ")).map(|t| t.trim().to_string()).collect()
}

/// Dois títulos dizem o mesmo? Sem caixa, sem acento e sem pontuação:
/// "Validar a entrada" e "validar a entrada." são o mesmo problema.
fn mesmo(a: &str, b: &str) -> bool {
    let simples = |s: &str| -> String {
        s.to_lowercase()
            .chars()
            .map(|c| match c {
                'á' | 'à' | 'â' | 'ã' => 'a',
                'é' | 'ê' => 'e',
                'í' => 'i',
                'ó' | 'ô' | 'õ' => 'o',
                'ú' | 'ü' => 'u',
                'ç' => 'c',
                c => c,
            })
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    };
    simples(a) == simples(b)
}

/// A página nova da base de um projeto.
pub fn pagina_nova(slug: &str, titulo_do_projeto: &str) -> String {
    format!(
        "---\ntitle: {}\ntype: {TIPO}\nprojeto: {slug}\ntags:\n- {TIPO}\n---\n# Problemas — {titulo_do_projeto}\n\n\
         O que já deu errado nas tarefas deste projeto e como evitar. O revisor propõe; você aprova. Esta página \
         vai anexada no detalhe, na execução e na revisão das tarefas: escreva critérios de aceite que cubram o \
         que se aplica.\n",
        escapar_escalar_yaml(&format!("Problemas — {titulo_do_projeto}"))
    )
}

/// A base com os problemas NOVOS acrescentados (cada um uma seção, com
/// de onde veio). `None` se nenhum é novo — aí não há o que propor.
pub fn acrescentar(base: &str, novos: &[Problema], origem: &str, quando: &str) -> Option<String> {
    let existentes = titulos(base);
    let mut texto = base.trim_end().to_string();
    let mut algum = false;
    let mut vistos: Vec<String> = Vec::new();
    for p in novos {
        if existentes.iter().chain(vistos.iter()).any(|t| mesmo(t, &p.titulo)) {
            continue;
        }
        texto.push_str(&format!("\n\n## {}\n\n{}\n\n_Visto em {origem}, {quando}._", p.titulo.trim(), p.texto.trim()));
        vistos.push(p.titulo.clone());
        algum = true;
    }
    algum.then(|| texto + "\n")
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn le_os_problemas_da_secao_e_para_no_veredito() {
        let r = "Bom.\n\n## Problemas\n\n- **Validar a entrada**: aceitou `null` — confira o tipo antes\n  de somar.\n- esqueceu o teste do caso vazio\n\nVEREDITO: aprovada\n- **fora**: x";
        let ps = ler(r);
        assert_eq!(ps.len(), 2, "{ps:?}");
        assert_eq!(ps[0], Problema { titulo: "Validar a entrada".into(), texto: "aceitou `null` — confira o tipo antes de somar.".into() });
        assert_eq!(ps[1].titulo, "esqueceu o teste do caso vazio", "sem negrito, o começo do texto");
        assert!(ler("sem seção\nVEREDITO: aprovada").is_empty());
    }

    #[test]
    fn so_entra_o_que_e_novo_e_diz_de_onde_veio() {
        let base = pagina_nova("saas", "Meu SaaS");
        assert!(base.contains("type: problemas") && base.contains("# Problemas — Meu SaaS"), "{base}");
        let p = |t: &str| Problema { titulo: t.into(), texto: format!("{t} aconteceu") };
        let um = acrescentar(&base, &[p("Validar a entrada")], "tarefas/003-login.md", "2026-10-25 10:00").unwrap();
        assert!(um.contains("## Validar a entrada\n\nValidar a entrada aconteceu\n\n_Visto em tarefas/003-login.md, 2026-10-25 10:00._"), "{um}");
        assert_eq!(acrescentar(&um, &[p("validar a ENTRADA.")], "x", "y"), None, "o mesmo título, escrito diferente");
        let dois = acrescentar(&um, &[p("Teste do vazio"), p("teste do vazio")], "x", "y").unwrap();
        assert_eq!(titulos(&dois), ["Validar a entrada", "Teste do vazio"], "repetido na mesma resposta entra uma vez");
    }
}
